//! Bounded job queue + single sequential worker for the capture→inject pipeline.
//!
//! Replaces the old per-hotkey `thread::spawn` + `PIPELINE_LOCK` pattern. The
//! hotkey handler now just snapshots the config, wraps it in a `TransferJob`,
//! and enqueues it; a single long-running worker drains the queue.
//!
//! Why a single worker (no concurrent uploads)?
//!   * Ordering matters more than throughput for a human-paced tool — two
//!     back-to-back captures must inject in the order they were taken, and
//!     `inject_swap`'s clipboard backup/restore must never interleave.
//!   * A bounded backlog (capacity `QUEUE_CAPACITY`) absorbs a quick double-fire
//!     instead of dropping the second capture the way the mutex did.
//!
//! The worker is a plain OS thread, not a tokio task, because the pipeline body
//! is synchronous. SFTP uploads reuse the shared tokio runtime via
//! `ssh::get_runtime().block_on(..)`. Processing is panic-guarded so one bad
//! job can't permanently stall the worker.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use tauri::{AppHandle, Emitter};

use crate::config::AppConfig;
use crate::daemon::{log_message, CapturedArtifact};
use crate::transport::ArtifactTransport;

pub type JobId = u64;

/// Bounded queue capacity. Most users never approach this; if they do, the
/// newest captures are dropped with a log line rather than growing unbounded.
const QUEUE_CAPACITY: usize = 8;

static JOB_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_job_id() -> JobId {
    JOB_COUNTER.fetch_add(1, Ordering::Relaxed)
}

// ── Structured error ──────────────────────────────────────────────────────

/// Classified pipeline error. Underlying functions still return `String`
/// errors; we wrap them at the job boundary so `JobEvent::Failed` can carry a
/// category instead of an opaque blob. (Step 7 — minimal, no crate-wide
/// `Result<_, AppError>` rewrite yet.)
#[derive(Debug, Clone)]
pub enum AppError {
    Capture(String),
    Upload(String),
    Injection(String),
    QueueFull,
    WorkerGone,
    Panic,
}

impl AppError {
    pub fn message(&self) -> String {
        match self {
            AppError::Capture(m) => format!("Capture error: {}", m),
            AppError::Upload(m) => format!("Upload error: {}", m),
            AppError::Injection(m) => format!("Injection error: {}", m),
            AppError::QueueFull => "Job queue is full".to_string(),
            AppError::WorkerGone => "Job worker is no longer running".to_string(),
            AppError::Panic => "Job panicked — worker recovered".to_string(),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

// ── Job model (Step 4) ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Processing,
    Resolving,
    Uploading,
    Injecting,
    Completed,
    Failed,
    Cancelled,
}

/// A single capture→compress→route→upload→inject unit of work.
///
/// `artifact` is `None` for the clipboard hotkey path (Alt+V); `Some` for
/// region/fullscreen captures that bypass the clipboard round-trip.
pub struct TransferJob {
    pub id: JobId,
    pub state: JobState,
    pub artifact: Option<CapturedArtifact>,
    /// Config snapshot taken at trigger time — a capture always uses the
    /// settings that were active when it was taken, not when the worker
    /// eventually picks it up.
    pub config: AppConfig,
    pub app_handle: AppHandle,
    pub log_history: Arc<Mutex<Vec<String>>>,
    pub created_at: std::time::SystemTime,
}

impl TransferJob {
    pub fn new(
        artifact: Option<CapturedArtifact>,
        config: AppConfig,
        app_handle: AppHandle,
        log_history: Arc<Mutex<Vec<String>>>,
    ) -> Self {
        Self {
            id: next_job_id(),
            state: JobState::Queued,
            artifact,
            config,
            app_handle,
            log_history,
            created_at: std::time::SystemTime::now(),
        }
    }

    fn log(&self, message: &str) {
        log_message(&self.app_handle, &self.log_history, message);
    }

    /// Update the job's state and notify any frontend listener on `job_event`.
    fn set_state(&mut self, state: JobState) {
        self.state = state;
        let _ = self.app_handle.emit(
            "job_event",
            JobEvent::StateChanged {
                id: self.id,
                state,
            },
        );
    }
}

// ── Structured events (Step 7) ────────────────────────────────────────────
//
// The frontend still primarily reads `log_append` for now; these events are
// emitted so a future job-progress UI can subscribe to `job_event` without any
// backend changes. Logs are for human diagnosis, events are for UI state.

#[derive(Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JobEvent {
    Created { id: JobId },
    StateChanged { id: JobId, state: JobState },
    Completed { id: JobId, paste_text: String },
    Failed { id: JobId, error: String },
}

// ── JobManager (Step 5) ───────────────────────────────────────────────────

pub struct JobManager {
    // `SyncSender` is `Send` but `!Sync`, so wrapping it in a Mutex is what
    // makes `JobManager` (and thus the `static OnceLock` below) `Sync`.
    sender: Mutex<SyncSender<TransferJob>>,
}

static JOB_MANAGER: OnceLock<JobManager> = OnceLock::new();

/// Lazily start the single job worker on first use. The worker lives for the
/// lifetime of the process (killed on exit alongside other daemon threads).
pub fn job_manager() -> &'static JobManager {
    JOB_MANAGER.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel(QUEUE_CAPACITY);
        thread::Builder::new()
            .name("img2cli-job-worker".into())
            .spawn(move || worker_loop(rx))
            .expect("Failed to spawn job worker thread");
        JobManager { sender: Mutex::new(tx) }
    })
}

impl JobManager {
    /// Enqueue a job without blocking the caller. Returns `QueueFull` when the
    /// backlog is full (the worker is behind) — the caller logs and drops it.
    pub fn submit(&self, job: TransferJob) -> Result<(), AppError> {
        match self.sender.lock().unwrap().try_send(job) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => Err(AppError::QueueFull),
            Err(TrySendError::Disconnected(_)) => Err(AppError::WorkerGone),
        }
    }
}

fn worker_loop(rx: Receiver<TransferJob>) {
    while let Ok(mut job) = rx.recv() {
        // Panic-guarded so a panicking job can't kill the worker permanently.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            process_job(&mut job)
        }));
        match result {
            Ok(Ok(paste_text)) => {
                job.state = JobState::Completed;
                let _ = job.app_handle.emit(
                    "job_event",
                    JobEvent::Completed {
                        id: job.id,
                        paste_text,
                    },
                );
            }
            Ok(Err(e)) => {
                job.state = JobState::Failed;
                // The failing step already wrote a diagnostic line via job.log().
                let _ = job.app_handle.emit(
                    "job_event",
                    JobEvent::Failed {
                        id: job.id,
                        error: e.message(),
                    },
                );
            }
            Err(_) => {
                job.state = JobState::Failed;
                job.log("Job panicked — worker recovered, continuing.");
                let _ = job.app_handle.emit(
                    "job_event",
                    JobEvent::Failed {
                        id: job.id,
                        error: AppError::Panic.message(),
                    },
                );
            }
        }
    }
}

/// The capture→compress→route→upload→inject pipeline. Runs on the single job
/// worker, so every step (including injection) is strictly serialized —
/// Step 6 falls out of the single-worker design for free.
///
/// Routing itself lives in `resolver::resolve_target` (Step 8); this function
/// is now just orchestration: capture → resolve → deliver → inject.
fn process_job(job: &mut TransferJob) -> Result<String, AppError> {
    job.set_state(JobState::Processing);

    // 1. Temp filename + local dir
    let filename = format!("img_{}.jpg", chrono::Local::now().format("%Y%m%d_%H%M%S_%f"));
    let local_dir = match job.config.save_dir.clone() {
        Some(d) => d,
        None => std::env::temp_dir().join("img2cli"),
    };
    let local_dest = local_dir.join(&filename);

    // 2. Capture & compress — from artifact (region capture) or clipboard
    let capture_result = if let Some(ref art) = job.artifact {
        job.log(&format!("Processing artifact (source: {:?})...", art.source));
        crate::clipboard::process_and_save_image(&art.image, &job.config, &local_dest)
    } else {
        job.log("Hotkey triggered. Capturing clipboard...");
        crate::clipboard::capture_and_save_image(&job.config, &local_dest)
    };
    let capture_result = capture_result.map_err(AppError::Capture)?;

    // 3. Base64 short-circuit: the result already is the complete data URI.
    if job.config.output_format.to_lowercase() == "base64" {
        let paste_text = wrap_quotes(capture_result, job.config.wrap_single_quotes);
        job.log("Base64 image generated. Injecting data URI...");
        job.set_state(JobState::Injecting);
        crate::injector::inject_text(&paste_text, &job.config.injection_mode)
            .map_err(AppError::Injection)?;
        return Ok(paste_text);
    }

    job.log(&format!("Image saved locally to {:?}", local_dest));

    // 4. Resolve destination via the routing chain
    //    (manual rules → ssh-config auto-detect → default SSH → local).
    job.set_state(JobState::Resolving);
    let foreground = crate::routing::ForegroundContext {
        window_title: crate::daemon::get_active_window_title(),
    };
    if let Some(ref title) = foreground.window_title {
        job.log(&format!("Active window title: {:?}", title));
    }
    // Parse ~/.ssh/config once up front; resolvers stay pure / testable.
    let ssh_hosts = crate::ssh_config::ssh_config_path()
        .and_then(|p| std::fs::read_to_string(&p).ok())
        .map(|c| crate::ssh_config::parse_ssh_config(&c))
        .unwrap_or_default();
    let route = {
        let request = crate::routing::RouteRequest {
            foreground: &foreground,
            config: &job.config,
            ssh_hosts: &ssh_hosts,
        };
        match crate::routing::default_chain().resolve(&request) {
            Ok(c) => c,
            // LocalFallback always matches in the standard chain, so this is
            // unreachable — handled defensively by reusing the temp file.
            Err(crate::routing::RouteError::NoRoute) => crate::routing::RouteCandidate {
                target: crate::routing::DeliveryTarget::Local(crate::routing::LocalTarget {
                    dir: local_dest.clone(),
                }),
                source: crate::routing::RouteSource::LocalFallback,
                reason: "no route (defensive fallback)".to_string(),
                confidence: 0,
            },
        }
    };
    job.log(&format!(
        "Routed via {:?} (confidence {}): {}",
        route.source, route.confidence, route.reason
    ));

    // 5. Deliver via transport (upload / local copy), then format the link
    job.set_state(JobState::Uploading);
    let processed = crate::transport::ProcessedArtifact {
        local_path: local_dest.clone(),
        filename: filename.clone(),
    };
    let delivered = match crate::transport::default_transport().deliver(&processed, &route.target) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("Delivery failed: {}", e);
            job.log(&msg);
            return Err(AppError::Upload(msg));
        }
    };
    job.log(&format!("Delivered: {}", delivered.delivered_path));
    let paste_text = wrap_quotes(
        crate::cli_adapter::adapter_for(&job.config.output_format)
            .render(&delivered.delivered_path),
        job.config.wrap_single_quotes,
    );

    // 6. Inject paste link into focused terminal (serialized by the worker)
    job.set_state(JobState::Injecting);
    job.log(&format!("Injecting paste link: {}", paste_text));
    crate::injector::inject_text(&paste_text, &job.config.injection_mode)
        .map_err(AppError::Injection)?;

    Ok(paste_text)
}

fn wrap_quotes(s: String, wrap: bool) -> String {
    if wrap {
        format!("'{}'", s)
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // JobManager's ordering + bounding come from a bounded `mpsc::sync_channel`
    // drained by a single worker. `TransferJob` carries an `AppHandle` that
    // can't be constructed without a running Tauri app, so these tests pin the
    // CHANNEL + COUNTER guarantees the manager relies on, rather than driving
    // the real worker end-to-end.

    #[test]
    fn sync_channel_preserves_submission_order() {
        // same primitive JobManager uses (mpsc::sync_channel, single receiver)
        let (tx, rx) = std::sync::mpsc::sync_channel::<u32>(8);
        for v in [10, 20, 30, 40] {
            tx.send(v).unwrap();
        }
        let received: Vec<u32> = (0..4).map(|_| rx.recv().unwrap()).collect();
        assert_eq!(received, vec![10, 20, 30, 40], "jobs must dequeue in submission order");
    }

    #[test]
    fn sync_channel_rejects_overflow() {
        let (tx, _rx) = std::sync::mpsc::sync_channel::<u32>(3);
        for v in 1..=3 {
            tx.try_send(v).unwrap();
        }
        // 4th submit when full → Err(Full); JobManager.submit maps this to
        // AppError::QueueFull (drops the capture instead of blocking the hotkey).
        assert!(matches!(tx.try_send(4), Err(TrySendError::Full(_))));
    }

    #[test]
    fn queue_capacity_is_a_sane_bound() {
        // Guard against accidentally making the queue unbounded or absurdly
        // large/small. The exact value is tunable; the bound must exist.
        assert!(
            QUEUE_CAPACITY >= 1 && QUEUE_CAPACITY <= 64,
            "QUEUE_CAPACITY out of sane range: {}",
            QUEUE_CAPACITY
        );
    }

    #[test]
    fn job_ids_are_monotonic() {
        // fetch_add only increases, so consecutive ids always increase — even
        // under concurrent callers (logs/events stay orderable by job id).
        let a = next_job_id();
        let b = next_job_id();
        assert!(b > a, "job ids must increase ({} > {})", b, a);
    }
}
