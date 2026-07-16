#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod daemon;

use config::AppConfig;
use tauri::Manager;

#[tauri::command]
fn get_config(state: tauri::State<'_, daemon::DaemonState>) -> Result<AppConfig, String> {
    if let Ok(config) = state.config.read() {
        Ok(config.clone())
    } else {
        Err("Failed to acquire read lock for daemon configuration".to_string())
    }
}

#[tauri::command]
fn save_config(config: AppConfig, state: tauri::State<'_, daemon::DaemonState>) -> Result<(), String> {
    config.save()?;
    if let Ok(mut daemon_config) = state.config.write() {
        *daemon_config = config;
        Ok(())
    } else {
        Err("Failed to acquire write lock for daemon configuration".to_string())
    }
}

#[tauri::command]
fn get_log_history(state: tauri::State<'_, daemon::DaemonState>) -> Result<Vec<String>, String> {
    if let Ok(history) = state.log_history.lock() {
        Ok(history.clone())
    } else {
        Err("Failed to acquire log history lock".to_string())
    }
}

#[tauri::command]
async fn test_connection(
    host: String,
    port: Option<u16>,
    username: Option<String>,
) -> Result<String, String> {
    // 1. Sanitize inputs to prevent SSH option injection vulnerabilities
    let host_trimmed = host.trim();
    if host_trimmed.starts_with('-') {
        return Err("Invalid host: host name cannot start with a hyphen".to_string());
    }
    
    if let Some(ref user) = username {
        let user_trimmed = user.trim();
        if user_trimmed.starts_with('-') {
            return Err("Invalid username: username cannot start with a hyphen".to_string());
        }
    }

    use std::process::Command;
    let mut args = Vec::new();
    if let Some(p) = port {
        args.push("-p".to_string());
        args.push(p.to_string());
    }
    
    args.push("-o".to_string());
    args.push("ConnectTimeout=5".to_string());
    args.push("-o".to_string());
    args.push("BatchMode=yes".to_string());
    
    // Use -- to separate options from host argument
    args.push("--".to_string());
    
    let dest = if let Some(ref user) = username {
        let user_trimmed = user.trim();
        if user_trimmed.is_empty() {
            host_trimmed.to_string()
        } else {
            format!("{}@{}", user_trimmed, host_trimmed)
        }
    } else {
        host_trimmed.to_string()
    };
    args.push(dest);
    args.push("echo 'SUCCESS'".to_string());
    
    let output = Command::new("ssh")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute ssh command: {}", e))?;
        
    if output.status.success() {
        Ok("Connection Successful!".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if stderr.is_empty() {
            Err("SSH Connection Failed".to_string())
        } else {
            Err(stderr)
        }
    }
}

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Ensure configuration exists safely without destructive overwriting on syntax errors
            let path = AppConfig::config_file_path();
            if !path.exists() {
                let config = AppConfig::default();
                let _ = config.save();
            }
            
            let initial_config = match AppConfig::load() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Configuration load error: {}", e);
                    // App will fallback to default config in memory, but we keep the corrupt file intact
                    AppConfig::default()
                }
            };
            
            // Start the daemon thread and manage its lifecycle state
            let daemon_state = daemon::start_daemon(app.handle().clone(), initial_config);
            
            // Route startup error logs through the daemon logging system if loading failed
            if path.exists() {
                if let Err(e) = AppConfig::load() {
                    daemon::log_message(
                        &app.handle(),
                        &daemon_state.log_history,
                        &format!("Configuration load error: {}", e),
                    );
                }
            }
            
            app.manage(daemon_state);
            
            let window = app.get_webview_window("main").unwrap();
            let _ = window.hide();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_log_history,
            test_connection
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            // Signal daemon background loop to terminate on application exit
            if let Some(state) = app_handle.try_state::<daemon::DaemonState>() {
                if let Ok(mut running) = state.running.lock() {
                    *running = false;
                }
            }
        }
    });
}
