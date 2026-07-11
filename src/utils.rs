use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(unix)]
use std::collections::HashSet;

pub fn generate_unique_filename() -> String {
    let now = Local::now();
    // Use high precision timestamp to ensure uniqueness
    format!("img_{}.jpg", now.format("%Y%m%d_%H%M%S_%f"))
}

pub fn send_notification(summary: &str, body: &str) {
    if notify_rust::Notification::new()
        .summary(summary)
        .body(body)
        .timeout(3000)
        .show()
        .is_err()
        {
            #[cfg(unix)]
            {
                // Fallback to notify-send CLI if dbus notification fails on Linux
                Command::new("notify-send")
                    .args(&[summary, body])
                    .status()
                    .ok();
            }
        }
}

#[cfg(windows)]
pub fn get_active_window_title() -> Option<String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowTextLengthW};
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == 0 {
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

#[cfg(not(windows))]
pub fn get_active_window_title() -> Option<String> {
    None
}

pub fn clean_old_files(save_dir: &Path, max_age_hours: u32) -> std::io::Result<u32> {
    let mut deleted_count = 0;
    if !save_dir.exists() {
        return Ok(0);
    }

    let now = Local::now();
    let limit_duration = chrono::Duration::hours(max_age_hours as i64);

    for entry in fs::read_dir(save_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                // Clean up files starting with "img_" and ending with ".jpg"
                if filename.starts_with("img_") && filename.ends_with(".jpg") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let modified_chrono: chrono::DateTime<Local> = modified.into();
                            if now.signed_duration_since(modified_chrono) > limit_duration {
                                if fs::remove_file(path).is_ok() {
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

#[cfg(unix)]
fn get_process_ppid_and_name(pid: u32) -> Option<(u32, String)> {
    let stat_content = fs::read_to_string(format!("/proc/{}/stat", pid)).ok()?;
    let last_paren = stat_content.rfind(')')?;
    let (comm_part, rest) = stat_content.split_at(last_paren);
    let comm = comm_part.split('(').last()?.to_string();

    let mut parts = rest.split_whitespace();
    parts.next(); // Skip state
    let ppid_str = parts.next()?;
    let ppid: u32 = ppid_str.parse().ok()?;

    Some((ppid, comm))
}

#[cfg(unix)]
fn find_active_shell_child(parent_pid: u32) -> Option<u32> {
    let mut processes = Vec::new();
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries {
            if let Ok(entry) = entry {
                let name = entry.file_name();
                if let Some(name_str) = name.to_str() {
                    if let Ok(pid) = name_str.parse::<u32>() {
                        if let Some((ppid, comm)) = get_process_ppid_and_name(pid) {
                            processes.push((pid, ppid, comm));
                        }
                    }
                }
            }
        }
    }

    let mut queue = vec![parent_pid];
    let mut visited = HashSet::new();
    let shell_names = ["bash", "zsh", "fish", "sh", "dash"];

    while let Some(current_pid) = queue.pop() {
        if !visited.insert(current_pid) {
            continue;
        }

        for &(pid, ppid, ref comm) in &processes {
            if ppid == current_pid {
                if shell_names.contains(&comm.as_str()) {
                    return Some(pid);
                }
                queue.push(pid);
            }
        }
    }

    // Fallback to the terminal process PID itself
    Some(parent_pid)
}

#[cfg(unix)]
pub fn get_active_terminal_cwd() -> Option<PathBuf> {
    // 1. Get active window ID using xprop
    let output = Command::new("xprop")
        .args(&["-root", "_NET_ACTIVE_WINDOW"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse: _NET_ACTIVE_WINDOW(WINDOW): window id # 0x2e0000a
    let win_id = stdout.split("window id #").nth(1)?.trim();
    if win_id.is_empty() || win_id == "0x0" {
        return None;
    }

    // 2. Get PID of the active window
    let output = Command::new("xprop")
        .args(&["-id", win_id, "_NET_WM_PID"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse: _NET_WM_PID(CARDINAL) = 12345
    let pid_str = stdout.split('=').nth(1)?.trim();
    let term_pid: u32 = pid_str.parse().ok()?;

    // 3. Find descendants to locate shell CWD
    let shell_pid = find_active_shell_child(term_pid)?;

    // 4. Resolve symlink /proc/<pid>/cwd
    let cwd_path = PathBuf::from(format!("/proc/{}/cwd", shell_pid));
    if cwd_path.exists() {
        fs::read_link(cwd_path).ok()
    } else {
        None
    }
}

#[cfg(windows)]
pub fn get_active_terminal_cwd() -> Option<PathBuf> {
    None
}

pub fn upload_via_scp(local_path: &Path, ssh: &crate::config::SshConfig) -> Result<String, String> {
    if !ssh.enabled {
        return Err("SSH upload is not enabled".to_string());
    }

    let filename = local_path.file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| "Invalid local file name".to_string())?;

    let remote_dest = format!("{}/{}", ssh.remote_dir, filename);

    // Format destination for scp command
    let dest_spec = if let Some(username) = &ssh.username {
        if username.is_empty() {
            format!("{}:{}", ssh.host, remote_dest)
        } else {
            format!("{}@{}:{}", username, ssh.host, remote_dest)
        }
    } else {
        format!("{}:{}", ssh.host, remote_dest)
    };

    let local_path_str = local_path.to_string_lossy().to_string();

    println!("Uploading {} to remote {}...", local_path_str, dest_spec);

    let mut args = Vec::new();
    if let Some(port) = ssh.port {
        args.push("-P".to_string());
        args.push(port.to_string());
    }
    args.push(local_path_str);
    args.push(dest_spec);

    let output = Command::new("scp")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute scp command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("scp failed: {}", stderr));
    }

    Ok(remote_dest)
}
