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

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use tauri::{AppHandle, Emitter};

use crate::config::AppConfig;
use crate::daemon::{log_message, CapturedArtifact};

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
    sender: SyncSender<TransferJob>,
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
        JobManager { sender: tx }
    })
}

impl JobManager {
    /// Enqueue a job without blocking the caller. Returns `QueueFull` when the
    /// backlog is full (the worker is behind) — the caller logs and drops it.
    pub fn submit(&self, job: TransferJob) -> Result<(), AppError> {
        match self.sender.try_send(job) {
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

    // 4. Route, in priority order:
    //    (a) manual Dynamic Router Targets (by match_pattern)
    //    (b) ssh-config auto-detect (active window title vs ~/.ssh/config hosts)
    //    (c) default SSH host, then (d) local path (resolved in step 5)
    job.set_state(JobState::Resolving);
    let mut active_target = None;
    let mut auto_detected_ssh: Option<crate::config::SshConfig> = None;

    if let Some(title) = crate::daemon::get_active_window_title() {
        let title_lower = title.to_lowercase();
        job.log(&format!("Active window title: {:?}", title));

        // (a) manual targets — explicit user intent, highest priority
        if let Some(ref targets) = job.config.targets {
            for target in targets {
                if target.enabled
                    && !target.match_pattern.is_empty()
                    && title_lower.contains(&target.match_pattern.to_lowercase())
                {
                    job.log(&format!("Matched target pattern {:?}", target.match_pattern));
                    active_target = Some(target.clone());
                    break;
                }
            }
        }

        // (b) ssh-config auto-detect: works for any terminal whose title
        //     contains the host's alias or hostname (most do).
        if active_target.is_none() {
            let default_remote = job
                .config
                .ssh
                .as_ref()
                .map(|s| s.remote_dir.clone())
                .filter(|d| !d.is_empty())
                .unwrap_or_else(|| "/tmp/img2cli".to_string());
            if let Some(cfg_path) = crate::ssh_config::ssh_config_path() {
                if let Ok(content) = std::fs::read_to_string(&cfg_path) {
                    let hosts = crate::ssh_config::parse_ssh_config(&content);
                    // pick the most specific match (longest alias/host in title)
                    let best = hosts
                        .into_iter()
                        .filter(|h| {
                            (!h.alias.is_empty() && title_lower.contains(&h.alias.to_lowercase()))
                                || (!h.host.is_empty()
                                    && title_lower.contains(&h.host.to_lowercase()))
                        })
                        .max_by_key(|h| h.alias.len().max(h.host.len()));
                    if let Some(h) = best {
                        job.log(&format!("Auto-detected SSH host from title: {:?}", h.alias));
                        auto_detected_ssh = Some(crate::config::SshConfig {
                            enabled: true,
                            host: h.host,
                            port: Some(h.port),
                            username: Some(h.username),
                            remote_dir: default_remote,
                            match_pattern: Some(h.alias),
                            remember_password: true,
                        });
                    }
                }
            }
        }
    }

    // 5. Resolve to an upload target or a local copy path
    let mut scp_upload_ssh = None;
    let mut local_dest_dir: Option<PathBuf> = None;

    if let Some(target) = active_target {
        match target.r#type.as_str() {
            "ssh" => {
                scp_upload_ssh = Some(crate::config::SshConfig {
                    enabled: true,
                    host: target.host.unwrap_or_default(),
                    port: target.port,
                    username: target.username,
                    remote_dir: target
                        .remote_dir
                        .unwrap_or_else(|| "/tmp/img2cli".to_string()),
                    match_pattern: Some(target.match_pattern),
                    remember_password: target.remember_password.unwrap_or(true),
                });
            }
            "local" => {
                local_dest_dir = target.local_dir.map(PathBuf::from);
            }
            _ => {}
        }
    } else if let Some(ssh) = auto_detected_ssh {
        job.log(&format!("Auto-routing via ssh-config to {}", ssh.host));
        scp_upload_ssh = Some(ssh);
    } else if let Some(ref default_ssh) = job.config.ssh {
        if default_ssh.enabled {
            job.log("No match found. Falling back to default SSH.");
            scp_upload_ssh = Some(default_ssh.clone());
        }
    }

    // 6. Upload (or local copy) → build the paste text
    job.set_state(JobState::Uploading);
    let paste_text = if let Some(ssh) = scp_upload_ssh {
        let user = ssh.username.clone().unwrap_or_default();
        let identity = crate::ssh::identity_key(&user, &ssh.host, ssh.port);
        let port = ssh.port.unwrap_or(22);
        let remote_result = if let Some(pw) = crate::ssh::get_stored_password(&identity) {
            job.log(&format!("Uploading via SFTP (password) to {}...", ssh.host));
            crate::ssh::upload_via_sftp(&ssh.host, port, &user, &pw, &ssh.remote_dir, &local_dest)
        } else {
            job.log(&format!("Uploading via SCP (key) to {}...", ssh.host));
            crate::daemon::upload_via_scp(&local_dest, &ssh)
        };
        match remote_result {
            Ok(remote_path) => {
                let base_format = match job.config.output_format.to_lowercase().as_str() {
                    "markdown" => format!("![image]({})", remote_path),
                    "html" => format!("<img src=\"{}\" />", remote_path),
                    _ => remote_path,
                };
                wrap_quotes(base_format, job.config.wrap_single_quotes)
            }
            Err(e) => {
                let err_msg = format!("Upload failed: {}", e);
                job.log(&err_msg);
                return Err(AppError::Upload(err_msg));
            }
        }
    } else {
        let local_path = if let Some(dest_dir) = local_dest_dir {
            let _ = std::fs::create_dir_all(&dest_dir);
            let final_local_path = dest_dir.join(&filename);
            if std::fs::copy(&local_dest, &final_local_path).is_ok() {
                final_local_path
            } else {
                local_dest
            }
        } else {
            local_dest
        };

        let path_str = local_path.to_string_lossy().to_string();
        let base_format = match job.config.output_format.to_lowercase().as_str() {
            "markdown" => format!("![image]({})", path_str),
            "html" => format!("<img src=\"{}\" />", path_str),
            _ => path_str,
        };
        wrap_quotes(base_format, job.config.wrap_single_quotes)
    };

    // 7. Inject paste link into focused terminal (serialized by the worker)
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
