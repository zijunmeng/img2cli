use crate::config::Config;
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::path::Path;
use std::thread;
use std::time::Duration;

#[cfg(unix)]
use daemonize::Daemonize;
#[cfg(unix)]
use nix::sys::signal::{kill, Signal};
#[cfg(unix)]
use nix::unistd::Pid;
#[cfg(unix)]
use std::fs::File;

#[cfg(unix)]
pub fn start_daemon(config: &Config) -> Result<(), String> {
    let pid_file = Path::new("/tmp/img2cli.pid");
    if pid_file.exists() {
        if let Ok(pid_str) = fs::read_to_string(pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                if is_process_running(pid) {
                    return Err(format!("img2cli is already running with PID {}", pid));
                }
            }
        }
        let _ = fs::remove_file(pid_file);
    }

    let stdout = File::create("/tmp/img2cli.out")
        .map_err(|e| format!("Failed to create stdout log file: {}", e))?;
    let stderr = File::create("/tmp/img2cli.err")
        .map_err(|e| format!("Failed to create stderr log file: {}", e))?;

    let daemonize = Daemonize::new()
        .pid_file("/tmp/img2cli.pid")
        .working_directory("/tmp")
        .stdout(stdout)
        .stderr(stderr);

    println!("Starting img2cli daemon in background...");
    
    match daemonize.start() {
        Ok(_) => {
            if let Err(e) = run_service(config, None, true) {
                eprintln!("Daemon service failed: {}", e);
                std::process::exit(1);
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to daemonize: {}", e)),
    }
}

#[cfg(windows)]
pub fn start_daemon(_config: &Config) -> Result<(), String> {
    Err("Daemon mode ('start' command) is not supported on Windows. Please use 'run' command to run the service in the foreground.".to_string())
}

#[cfg(unix)]
pub fn stop_daemon() -> Result<(), String> {
    let pid_file = Path::new("/tmp/img2cli.pid");
    if !pid_file.exists() {
        return Err("img2cli is not running (PID file not found)".to_string());
    }

    let pid_str = fs::read_to_string(pid_file)
        .map_err(|e| format!("Failed to read PID file: {}", e))?;
    let pid: i32 = pid_str.trim().parse()
        .map_err(|e| format!("Failed to parse PID: {}", e))?;

    let pid_struct = Pid::from_raw(pid);

    match kill(pid_struct, Signal::SIGTERM) {
        Ok(_) => {
            println!("Sent STOP signal to process {}", pid);
            let mut attempts = 0;
            while is_process_running(pid) && attempts < 10 {
                thread::sleep(Duration::from_millis(100));
                attempts += 1;
            }
            if pid_file.exists() {
                let _ = fs::remove_file(pid_file);
            }
            println!("img2cli daemon stopped successfully.");
            Ok(())
        }
        Err(e) => {
            if e == nix::errno::Errno::ESRCH {
                let _ = fs::remove_file(pid_file);
                return Err("img2cli was not running, but PID file existed. Cleaned up PID file.".to_string());
            }
            Err(format!("Failed to stop process {}: {}", pid, e))
        }
    }
}

#[cfg(windows)]
pub fn stop_daemon() -> Result<(), String> {
    use std::process::Command;
    let output = Command::new("taskkill")
        .args(&["/F", "/IM", "img2cli.exe"])
        .output()
        .map_err(|e| format!("Failed to execute taskkill: {}", e))?;

    if output.status.success() {
        println!("img2cli daemon stopped successfully (terminated background processes).");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("找不到") {
            return Err("img2cli is not running (no background processes found).".to_string());
        }
        Err(format!("Failed to stop daemon: {}", stderr))
    }
}

#[cfg(unix)]
pub fn check_status() -> Result<(), String> {
    let pid_file = Path::new("/tmp/img2cli.pid");
    if !pid_file.exists() {
        println!("img2cli status: Stopped");
        return Ok(());
    }

    let pid_str = fs::read_to_string(pid_file)
        .map_err(|e| format!("Failed to read PID file: {}", e))?;
    let pid: i32 = pid_str.trim().parse()
        .map_err(|e| format!("Failed to parse PID: {}", e))?;

    if is_process_running(pid) {
        println!("img2cli status: Running (PID: {})", pid);
    } else {
        println!("img2cli status: Dead (PID file exists but process is not running)");
        let _ = fs::remove_file(pid_file);
    }
    Ok(())
}

#[cfg(windows)]
pub fn check_status() -> Result<(), String> {
    use std::process::Command;
    let output = Command::new("tasklist")
        .args(&["/FI", "IMAGENAME eq img2cli.exe"])
        .output()
        .map_err(|e| format!("Failed to execute tasklist: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("img2cli.exe") {
        // Parse tasklist output to make sure it's not just the header
        let running_instances = stdout.lines()
            .filter(|line| line.starts_with("img2cli.exe"))
            .count();
        if running_instances > 0 {
            println!("img2cli status: Running ({} background process(es) found)", running_instances);
            return Ok(());
        }
    }
    println!("img2cli status: Stopped");
    Ok(())
}

#[cfg(unix)]
fn is_process_running(pid: i32) -> bool {
    let proc_path = format!("/proc/{}", pid);
    Path::new(&proc_path).exists()
}

pub fn run_service(config: &Config, override_ssh: Option<crate::config::SshConfig>, auto_route: bool) -> Result<(), String> {
    println!("Clipboard monitoring service started. Initializing...");

    // 1. Initial cleanup of temporary files
    let save_dir = config.get_save_dir();
    if let Err(e) = crate::utils::clean_old_files(&save_dir, 24) {
        eprintln!("Initial cleanup warning: {}", e);
    }

    // 2. Start periodic cleanup thread (runs every hour)
    let config_clone = config.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(3600));
            let s_dir = config_clone.get_save_dir();
            let _ = crate::utils::clean_old_files(&s_dir, 24);
        }
    });

    // 3. Clipboard polling loop
    let mut last_image_sig = None;
    let override_ssh_clone = override_ssh.clone();

    loop {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            if let Ok(img_data) = cb.get_image() {
                let len = img_data.bytes.len();
                let sig = (
                    img_data.width,
                    img_data.height,
                    len,
                    img_data.bytes.get(0..100.min(len)).map(|s| s.to_vec()),
                );

                if Some(sig.clone()) != last_image_sig {
                    last_image_sig = Some(sig);
                    println!("New image in clipboard detected! Processing...");
                    
                    // Trigger the upload and write the path back to the clipboard
                    handle_clipboard_image(config, &override_ssh_clone, auto_route);
                }
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
}

fn handle_clipboard_image(config: &Config, override_ssh: &Option<crate::config::SshConfig>, auto_route: bool) {
    let filename = crate::utils::generate_unique_filename();
    let local_dest = config.get_save_dir().join(&filename);

    // Capture, compress, and save image locally
    match crate::clipboard::capture_and_save_image(config, &local_dest) {
        Ok(output_str) => {
            let mut save_path_opt = None;

            if config.workspace_aware && !config.ssh.as_ref().map_or(false, |s| s.enabled) {
                if let Some(cwd) = crate::utils::get_active_terminal_cwd() {
                    let images_dir = cwd.join("images");
                    let assets_dir = cwd.join("assets");
                    let target_dir = if assets_dir.exists() {
                        assets_dir
                    } else {
                        images_dir
                    };
                    save_path_opt = Some((target_dir.join(&filename), true));
                }
            }

            // Determine active window title and find matching target
            let mut active_ssh = None;

            if let Some(ssh) = override_ssh {
                active_ssh = Some(ssh.clone());
            } else if auto_route {
                if let Some(title) = crate::utils::get_active_window_title() {
                    println!("Active window title: {:?}", title);
                    if let Some(targets) = &config.ssh_targets {
                        for target in targets {
                            if let Some(pattern) = &target.match_pattern {
                                if title.to_lowercase().contains(&pattern.to_lowercase()) {
                                    println!("Match found: target config {:?} matches title {:?}", target.host, title);
                                    active_ssh = Some(target.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // If no match was found, fall back to the default SSH config (if enabled)
            if active_ssh.is_none() {
                if let Some(default_ssh) = &config.ssh {
                    if default_ssh.enabled {
                        println!("No matching target found. Falling back to default SSH configuration.");
                        active_ssh = Some(default_ssh.clone());
                    }
                }
            }

            let final_output = if let Some(ssh) = &active_ssh {
                match crate::utils::upload_via_scp(&local_dest, ssh) {
                    Ok(remote_path) => {
                        match config.output_format.to_lowercase().as_str() {
                            "markdown" => format!("![image]({})", remote_path),
                            "html" => format!("<img src=\"{}\" />", remote_path),
                            _ => remote_path,
                        }
                    }
                    Err(e) => {
                        let err_msg = format!("SSH upload failed: {}", e);
                        eprintln!("{}", err_msg);
                        crate::utils::send_notification("img2cli Error", &err_msg);
                        return;
                    }
                }
            } else {
                if let Some((target_path, true)) = save_path_opt {
                    if let Some(parent) = target_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if std::fs::rename(&local_dest, &target_path).is_ok() {
                        let parent_name = target_path.parent()
                            .and_then(|p| p.file_name())
                            .and_then(|s| s.to_str())
                            .unwrap_or("images");
                        let rel_file_path = format!("./{}/{}", parent_name, filename);
                        match config.output_format.to_lowercase().as_str() {
                            "markdown" => format!("![image]({})", rel_file_path),
                            "html" => format!("<img src=\"{}\" />", rel_file_path),
                            _ => rel_file_path,
                        }
                    } else {
                        output_str
                    }
                } else {
                    output_str
                }
            };

            // Write the formatted output path text back to the clipboard
            if let Ok(mut cb) = arboard::Clipboard::new() {
                if cb.set_text(final_output).is_ok() {
                    println!("Screenshot uploaded and clipboard updated with path!");
                    crate::utils::send_notification("img2cli", "Screenshot uploaded! Ready to paste (Ctrl+V).");
                } else {
                    eprintln!("Failed to set clipboard text.");
                }
            }
        }
        Err(e) => {
            let err_msg = format!("Capture failed: {}", e);
            eprintln!("{}", err_msg);
            crate::utils::send_notification("img2cli Error", &err_msg);
        }
    }
}
