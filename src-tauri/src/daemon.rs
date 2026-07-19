use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use crate::config::AppConfig;

pub struct DaemonState {
    pub running: Arc<Mutex<bool>>,
    pub log_history: Arc<Mutex<Vec<String>>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub captured_image: Arc<std::sync::Mutex<Option<image::RgbaImage>>>,
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
    }
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

pub fn upload_via_scp(local_path: &Path, ssh: &crate::config::SshConfig) -> Result<String, String> {
    let filename = local_path.file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| "Invalid local file name".to_string())?;

    let remote_dest = format!("{}/{}", ssh.remote_dir, filename);

    let dest_spec = if let Some(ref username) = ssh.username {
        if username.is_empty() {
            format!("{}:{}", ssh.host, remote_dest)
        } else {
            format!("{}@{}:{}", username, ssh.host, remote_dest)
        }
    } else {
        format!("{}:{}", ssh.host, remote_dest)
    };

    // Ensure the remote directory exists so scp doesn't fail on a missing
    // folder (first run, or a user-supplied path that doesn't exist yet).
    // Best-effort: ignore errors here and let scp surface any real failure.
    let ssh_target = if let Some(ref username) = ssh.username {
        if username.is_empty() {
            ssh.host.clone()
        } else {
            format!("{}@{}", username, ssh.host)
        }
    } else {
        ssh.host.clone()
    };
    let mut mkdir_args = Vec::new();
    if let Some(port) = ssh.port {
        mkdir_args.push("-p".to_string());
        mkdir_args.push(port.to_string());
    }
    mkdir_args.push("-o".to_string());
    mkdir_args.push("ConnectTimeout=5".to_string());
    mkdir_args.push("-o".to_string());
    mkdir_args.push("BatchMode=yes".to_string());
    mkdir_args.push("--".to_string());
    mkdir_args.push(ssh_target);
    mkdir_args.push(format!("mkdir -p '{}'", ssh.remote_dir));
    let _ = std::process::Command::new("ssh").args(&mkdir_args).output();

    let local_path_str = local_path.to_string_lossy().to_string();

    let mut args = Vec::new();
    if let Some(port) = ssh.port {
        args.push("-P".to_string());
        args.push(port.to_string());
    }
    
    // Use -- to separate options from positional file arguments
    args.push("--".to_string());
    
    args.push(local_path_str);
    args.push(dest_spec);

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

pub fn trigger_capture_and_paste(app_handle: &AppHandle, state: &DaemonState) {
    let app_handle_clone = app_handle.clone();
    let log_history_clone = state.log_history.clone();
    let config_clone = if let Ok(config) = state.config.read() {
        config.clone()
    } else {
        log_message(app_handle, &state.log_history, "Error: Failed to read configuration lock.");
        return;
    };
    
    // Spawn asynchronously to prevent locking the global shortcut handler
    thread::spawn(move || {
        log_message(&app_handle_clone, &log_history_clone, "Hotkey triggered. Capturing clipboard...");
        
        // 1. Generate temp filename
        let filename = format!("img_{}.jpg", chrono::Local::now().format("%Y%m%d_%H%M%S_%f"));
        
        // 2. Get local temporary directory
        let local_dir = if let Some(ref dir) = config_clone.save_dir {
            dir.clone()
        } else {
            std::env::temp_dir().join("img2cli")
        };
        let local_dest = local_dir.join(&filename);
        
        // 3. Capture & compress clipboard image
        match crate::clipboard::capture_and_save_image(&config_clone, &local_dest) {
            Ok(capture_result) => {
                // If Base64 format is selected, the result contains the complete data URI string!
                if config_clone.output_format.to_lowercase() == "base64" {
                    let paste_text = if config_clone.wrap_single_quotes {
                        format!("'{}'", capture_result)
                    } else {
                        capture_result
                    };
                    
                    log_message(&app_handle_clone, &log_history_clone, "Base64 image generated. Injecting data URI...");
                    match crate::injector::inject_text(&paste_text, &config_clone.injection_mode) {
                        Ok(_) => log_message(&app_handle_clone, &log_history_clone, "Injection completed successfully."),
                        Err(e) => log_message(&app_handle_clone, &log_history_clone, &format!("Injection failed: {}", e)),
                    }
                    return;
                }
                
                log_message(&app_handle_clone, &log_history_clone, &format!("Image saved locally to {:?}", local_dest));
                
                // 4. Route, in priority order:
                //    (a) manual Dynamic Router Targets (by match_pattern)
                //    (b) ssh-config auto-detect (title vs ~/.ssh/config hosts)
                //    (c) default SSH host, then (d) local path (resolved in step 5)
                let mut active_target = None;
                let mut auto_detected_ssh: Option<crate::config::SshConfig> = None;

                if let Some(title) = get_active_window_title() {
                    let title_lower = title.to_lowercase();
                    log_message(&app_handle_clone, &log_history_clone, &format!("Active window title: {:?}", title));

                    // (a) manual targets — explicit user intent, highest priority
                    if let Some(ref targets) = config_clone.targets {
                        for target in targets {
                            if target.enabled
                                && !target.match_pattern.is_empty()
                                && title_lower.contains(&target.match_pattern.to_lowercase())
                            {
                                log_message(&app_handle_clone, &log_history_clone, &format!("Matched target pattern {:?}", target.match_pattern));
                                active_target = Some(target.clone());
                                break;
                            }
                        }
                    }

                    // (b) ssh-config auto-detect: works for any terminal whose
                    //     title contains the host's alias or hostname (most do).
                    if active_target.is_none() {
                        let default_remote = config_clone
                            .ssh
                            .as_ref()
                            .map(|s| s.remote_dir.clone())
                            .filter(|d| !d.is_empty())
                            .unwrap_or_else(|| "/tmp/img2cli".to_string());
                        if let Some(cfg_path) = crate::ssh_config::ssh_config_path() {
                            if let Ok(content) = std::fs::read_to_string(&cfg_path) {
                                let hosts = crate::ssh_config::parse_ssh_config(&content);
                                // pick the most specific match (longest alias/host in title)
                                let best = hosts.into_iter().filter(|h| {
                                    (!h.alias.is_empty() && title_lower.contains(&h.alias.to_lowercase()))
                                        || (!h.host.is_empty() && title_lower.contains(&h.host.to_lowercase()))
                                }).max_by_key(|h| h.alias.len().max(h.host.len()));
                                if let Some(h) = best {
                                    log_message(&app_handle_clone, &log_history_clone, &format!("Auto-detected SSH host from title: {:?}", h.alias));
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
                
                // 5. Build scp upload configuration or local copy path
                let mut scp_upload_ssh = None;
                let mut local_dest_dir = None;
                
                if let Some(target) = active_target {
                    match target.r#type.as_str() {
                        "ssh" => {
                            scp_upload_ssh = Some(crate::config::SshConfig {
                                enabled: true,
                                host: target.host.unwrap_or_default(),
                                port: target.port,
                                username: target.username,
                                remote_dir: target.remote_dir.unwrap_or_else(|| "/tmp/img2cli".to_string()),
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
                    log_message(&app_handle_clone, &log_history_clone, &format!("Auto-routing via ssh-config to {}", ssh.host));
                    scp_upload_ssh = Some(ssh);
                } else if let Some(ref default_ssh) = config_clone.ssh {
                    if default_ssh.enabled {
                        log_message(&app_handle_clone, &log_history_clone, "No match found. Falling back to default SSH.");
                        scp_upload_ssh = Some(default_ssh.clone());
                    }
                }
                
                // 6. Perform copy/upload operations
                let paste_text = if let Some(ssh) = scp_upload_ssh {
                    let user = ssh.username.clone().unwrap_or_default();
                    let identity = crate::ssh::identity_key(&user, &ssh.host, ssh.port);
                    let port = ssh.port.unwrap_or(22);
                    let remote_result = if let Some(pw) = crate::ssh::get_stored_password(&identity) {
                        log_message(&app_handle_clone, &log_history_clone, &format!("Uploading via SFTP (password) to {}...", ssh.host));
                        crate::ssh::upload_via_sftp(&ssh.host, port, &user, &pw, &ssh.remote_dir, &local_dest)
                    } else {
                        log_message(&app_handle_clone, &log_history_clone, &format!("Uploading via SCP (key) to {}...", ssh.host));
                        upload_via_scp(&local_dest, &ssh)
                    };
                    match remote_result {
                        Ok(remote_path) => {
                            let base_format = match config_clone.output_format.to_lowercase().as_str() {
                                "markdown" => format!("![image]({})", remote_path),
                                "html" => format!("<img src=\"{}\" />", remote_path),
                                _ => remote_path,
                            };
                            if config_clone.wrap_single_quotes {
                                format!("'{}'", base_format)
                            } else {
                                base_format
                            }
                        }
                        Err(e) => {
                            let err_msg = format!("Upload failed: {}", e);
                            log_message(&app_handle_clone, &log_history_clone, &err_msg);
                            return;
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
                    let base_format = match config_clone.output_format.to_lowercase().as_str() {
                        "markdown" => format!("![image]({})", path_str),
                        "html" => format!("<img src=\"{}\" />", path_str),
                        _ => path_str,
                    };
                    if config_clone.wrap_single_quotes {
                        format!("'{}'", base_format)
                    } else {
                        base_format
                    }
                };
                
                // 7. Inject paste link into focused terminal CWD
                log_message(&app_handle_clone, &log_history_clone, &format!("Injecting paste link: {}", paste_text));
                match crate::injector::inject_text(&paste_text, &config_clone.injection_mode) {
                    Ok(_) => log_message(&app_handle_clone, &log_history_clone, "Injection completed successfully."),
                    Err(e) => log_message(&app_handle_clone, &log_history_clone, &format!("Injection failed: {}", e)),
                }
            }
            Err(e) => {
                log_message(&app_handle_clone, &log_history_clone, &format!("Clipboard image capture failed: {}", e));
            }
        }
    });
}
