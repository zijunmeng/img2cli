use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::path::Path;
use tauri::{AppHandle, Emitter};
use crate::config::AppConfig;

// ── CapturedArtifact: unified representation of a captured image ──────────

pub type ArtifactId = u64;

/// A captured image with metadata about its source. Replaces the old
/// clipboard-round-trip pattern (region capture → write clipboard → read
/// clipboard back). Now region capture creates a CapturedArtifact directly.
#[derive(Debug, Clone)]
pub struct CapturedArtifact {
    #[allow(dead_code)] // future queue-age diagnostics
    pub id: ArtifactId,
    pub image: Arc<image::RgbaImage>,
    pub source: CaptureSource,
    #[allow(dead_code)] // future queue-age diagnostics
    pub created_at: std::time::SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)] // Clipboard/Fullscreen/File mark future capture paths
pub enum CaptureSource {
    Clipboard,
    Region,
    Fullscreen,
    File,
}

impl CapturedArtifact {
    // Caller is the win/mac-gated capture pipeline.
    #[cfg_attr(not(any(target_os = "windows", target_os = "macos")), allow(dead_code))]
    pub fn new(image: image::RgbaImage, source: CaptureSource) -> Self {
        let created_at = std::time::SystemTime::now();
        let id = created_at
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        Self { id, image: Arc::new(image), source, created_at }
    }
}

pub struct DaemonState {
    pub running: Arc<Mutex<bool>>,
    pub log_history: Arc<Mutex<Vec<String>>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub captured_image: Arc<std::sync::Mutex<Option<image::RgbaImage>>>,
    /// On-screen window rects (CSS px) captured alongside the frozen frame —
    /// feeds the overlay's auto window detection (Roadmap 6-J).
    pub window_rects: Arc<std::sync::Mutex<Vec<WindowRect>>>,
    /// Most recent background upload (v0.3.15 capture-then-upload): lets the
    /// inject hotkey skip re-uploading when the clipboard still holds the
    /// exact image that was already uploaded.
    pub last_upload: Arc<std::sync::Mutex<Option<LastUpload>>>,
    /// Pin-to-screen windows (v0.4.2): pin id → composited image data URL.
    pub pins: Arc<std::sync::Mutex<std::collections::HashMap<u32, String>>>,
}

/// A completed background upload, keyed by the fingerprint of the raw image.
#[derive(Debug, Clone)]
pub struct LastUpload {
    pub fingerprint: u64,
    pub delivered_path: String,
}

/// Stable fingerprint of raw RGBA pixels — the same buffer hashes identically
/// on the upload side (capture artifact) and the inject side (clipboard peek).
pub fn image_fingerprint(width: u32, height: u32, bytes: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (width, height).hash(&mut h);
    bytes.hash(&mut h);
    h.finish()
}

/// A detected on-screen window rectangle in CSS pixels.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WindowRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub title: String,
}

pub fn log_message(app_handle: &AppHandle, log_history: &Arc<Mutex<Vec<String>>>, message: &str) {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let formatted = format!("[{}] {}", timestamp, message);

    if let Ok(mut history) = log_history.lock() {
        history.push(formatted.clone());
        if history.len() > 100 {
            history.remove(0); // Keep last 100 log lines
        }
    }

    let _ = app_handle.emit("log_append", formatted);
}

pub fn clean_old_files(save_dir: &Path, max_age_days: u32) -> std::io::Result<u32> {
    let mut deleted_count = 0;
    if !save_dir.exists() {
        return Ok(0);
    }

    let now = chrono::Local::now();
    let limit_duration = chrono::Duration::days(max_age_days as i64);

    for entry in std::fs::read_dir(save_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                // Clean up files starting with "img_" and ending with ".jpg"
                if filename.starts_with("img_") && filename.ends_with(".jpg") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let modified_chrono: chrono::DateTime<chrono::Local> = modified.into();
                            if now.signed_duration_since(modified_chrono) > limit_duration {
                                if std::fs::remove_file(path).is_ok() {
                                    deleted_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(deleted_count)
}

pub fn start_daemon(app_handle: AppHandle, config: AppConfig) -> DaemonState {
    let running = Arc::new(Mutex::new(true));
    let running_clone = running.clone();
    let log_history = Arc::new(Mutex::new(Vec::new()));
    let log_history_clone = log_history.clone();
    let config_lock = Arc::new(RwLock::new(config));
    let config_lock_clone = config_lock.clone();
    let app_clone = app_handle.clone();

    thread::spawn(move || {
        log_message(&app_clone, &log_history_clone, "Daemon background thread started.");

        // Perform initial cleanup of expired temporary image files on startup
        let (save_dir, keep_days) = if let Ok(cfg) = config_lock_clone.read() {
            let dir = cfg.save_dir.clone().unwrap_or_else(|| std::env::temp_dir().join("img2cli"));
            (dir, cfg.clean_keep_days)
        } else {
            (std::env::temp_dir().join("img2cli"), 1)
        };
        match clean_old_files(&save_dir, keep_days) {
            Ok(count) => {
                if count > 0 {
                    log_message(&app_clone, &log_history_clone, &format!("Cleaned up {} expired temporary image files.", count));
                }
            }
            Err(e) => {
                log_message(&app_clone, &log_history_clone, &format!("Warning: Failed to clean old temporary files: {}", e));
            }
        }

        loop {
            if let Ok(running) = running_clone.lock() {
                if !*running {
                    break;
                }
            } else {
                break; // Exit if mutex is poisoned
            }

            thread::sleep(std::time::Duration::from_millis(500));
        }

        log_message(&app_clone, &log_history_clone, "Daemon background thread stopped.");
    });

    DaemonState {
        running,
        log_history,
        config: config_lock,
        captured_image: Arc::new(std::sync::Mutex::new(None)),
        window_rects: Arc::new(std::sync::Mutex::new(Vec::new())),
        last_upload: Arc::new(std::sync::Mutex::new(None)),
        pins: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
    }
}

/// Foreground window's process executable name, lowercase ("orca.exe") — a
/// stabler host signal than window titles (v0.4.1 host_policy).
#[cfg(windows)]
pub fn get_foreground_process_name() -> Option<String> {
    use windows_sys::Win32::Foundation::{CloseHandle, HWND};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return None;
        }
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut buf = [0u16; 512];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len);
        CloseHandle(handle);
        if ok == 0 {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        path.rsplit(['\\', '/']).next().map(str::to_lowercase)
    }
}

#[cfg(not(windows))]
pub fn get_foreground_process_name() -> Option<String> {
    None
}

#[cfg(windows)]
pub fn get_active_window_title() -> Option<String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowTextLengthW};
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return None;
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        let read = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        if read == 0 {
            return None;
        }
        buf.truncate(read as usize);
        String::from_utf16(&buf).ok()
    }
}

#[cfg(target_os = "macos")]
pub fn get_active_window_title() -> Option<String> {
    // Requires Accessibility permission for System Events control.
    let script = r#"tell application "System Events" to get title of front window of (first process whose frontmost is true)"#;
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(target_os = "linux")]
pub fn get_active_window_title() -> Option<String> {
    // Works on X11 (requires the `xdotool` binary). Wayland compositors
    // generally don't expose other apps' window titles, so this returns None.
    let wid_out = std::process::Command::new("xdotool")
        .arg("getactivewindow")
        .output()
        .ok()?;
    if !wid_out.status.success() {
        return None;
    }
    let wid = String::from_utf8_lossy(&wid_out.stdout).trim().to_string();
    if wid.is_empty() {
        return None;
    }
    let name_out = std::process::Command::new("xdotool")
        .arg("getwindowname")
        .arg(&wid)
        .output()
        .ok()?;
    if !name_out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&name_out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn get_active_window_title() -> Option<String> {
    None
}

/// SCP upload via the system `ssh`/`scp` binaries (key-based auth).
/// Takes resolved connection params (mirroring `routing::SshTarget`) so the
/// transport layer doesn't have to reconstruct an `SshConfig`. Returns the
/// remote path on success.
pub fn upload_via_scp(
    local_path: &Path,
    host: &str,
    port: u16,
    username: &str,
    remote_dir: &str,
) -> Result<String, String> {
    let filename = local_path.file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| "Invalid local file name".to_string())?;

    let remote_dest = format!("{}/{}", remote_dir, filename);

    let dest_spec = if username.is_empty() {
        format!("{}:{}", host, remote_dest)
    } else {
        format!("{}@{}:{}", username, host, remote_dest)
    };
    let ssh_target = if username.is_empty() {
        host.to_string()
    } else {
        format!("{}@{}", username, host)
    };

    // Best-effort mkdir -p on the remote so scp doesn't fail on a missing
    // folder (first run, or a user-supplied path that doesn't exist yet).
    let mkdir_args = vec![
        "-p".to_string(), port.to_string(),
        "-o".to_string(), "ConnectTimeout=5".to_string(),
        "-o".to_string(), "BatchMode=yes".to_string(),
        "--".to_string(), ssh_target,
        format!("mkdir -p '{}'", remote_dir),
    ];
    let _ = std::process::Command::new("ssh").args(&mkdir_args).output();

    let args = vec![
        "-P".to_string(), port.to_string(),
        "--".to_string(),
        local_path.to_string_lossy().to_string(),
        dest_spec,
    ];

    let output = std::process::Command::new("scp")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute scp command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("scp failed: {}", stderr));
    }

    Ok(remote_dest)
}

/// Hotkey entry point (Alt+V): read the clipboard. Thin wrapper around
/// `trigger_with_artifact`.
/// Enqueue a background upload-only job (v0.3.15 "capture = upload"): called
/// right after a confirmed region lands on the clipboard, so the SFTP upload
/// overlaps with the user switching windows. The inject hotkey then pastes
/// the already-uploaded path instantly (fingerprint-matched fast path).
// Caller is the win/mac-gated capture_region.
#[cfg_attr(not(any(target_os = "windows", target_os = "macos")), allow(dead_code))]
pub fn trigger_upload_only(app_handle: &AppHandle, state: &DaemonState, image: image::RgbaImage) {
    let config = if let Ok(config) = state.config.read() {
        config.clone()
    } else {
        return;
    };
    // Inline base64 never uploads; nothing to do in the background.
    if config.output_format.to_lowercase() == "base64" {
        return;
    }
    log_message(app_handle, &state.log_history, "Uploading in the background…");
    let artifact = CapturedArtifact::new(image, CaptureSource::Region);
    let job = crate::job::TransferJob::new(
        Some(artifact),
        config,
        app_handle.clone(),
        state.log_history.clone(),
        false,
    );
    let id = job.id;
    let _ = app_handle.emit("job_event", crate::job::JobEvent::Created { id });
    if let Err(e) = crate::job::job_manager().submit(job) {
        log_message(
            app_handle,
            &state.log_history,
            &format!(
                "Background upload not queued ({}). It will upload when you inject instead.",
                e
            ),
        );
    }
}

pub fn trigger_capture_and_paste(app_handle: &AppHandle, state: &DaemonState) {
    trigger_with_artifact(app_handle, state, None);
}

/// Snapshot the config, wrap it (and an optional pre-captured artifact) in a
/// `TransferJob`, and hand it to the single job worker. Returns immediately so
/// the global-shortcut handler never blocks.
///
/// * `artifact = None` → Alt+V clipboard path (worker reads the clipboard).
/// * `artifact = Some` → region/fullscreen capture, flows straight through
///   with no clipboard round-trip.
///
/// See `job::JobManager` for execution and ordering guarantees.
pub fn trigger_with_artifact(
    app_handle: &AppHandle,
    state: &DaemonState,
    artifact: Option<CapturedArtifact>,
) {
    let config = if let Ok(config) = state.config.read() {
        config.clone()
    } else {
        log_message(app_handle, &state.log_history, "Error: Failed to read configuration lock.");
        return;
    };

    let job = crate::job::TransferJob::new(
        artifact,
        config,
        app_handle.clone(),
        state.log_history.clone(),
        true,
    );
    let id = job.id;

    let _ = app_handle.emit("job_event", crate::job::JobEvent::Created { id });

    match crate::job::job_manager().submit(job) {
        Ok(()) => log_message(
            app_handle,
            &state.log_history,
            &format!("Job #{} queued.", id),
        ),
        Err(crate::job::AppError::QueueFull) => log_message(
            app_handle,
            &state.log_history,
            "Job queue full — capture dropped. Wait for the current upload to finish.",
        ),
        Err(e) => log_message(
            app_handle,
            &state.log_history,
            &format!("Failed to enqueue job #{}: {}", id, e),
        ),
    }
}
