#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod clipboard;
mod daemon;
mod injector;

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
fn save_config(
    config: AppConfig,
    state: tauri::State<'_, daemon::DaemonState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    use std::str::FromStr;
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // Read old hotkey value to check for changes
    let old_hotkey = if let Ok(c) = state.config.read() {
        c.global_hotkey.clone()
    } else {
        "".to_string()
    };

    config.save()?;

    // Update managed daemon configuration
    if let Ok(mut daemon_config) = state.config.write() {
        *daemon_config = config.clone();
    } else {
        return Err("Failed to acquire write lock for daemon configuration".to_string());
    }

    // Dynamic shortcut unregistration & registration updates
    if old_hotkey != config.global_hotkey {
        let manager = app_handle.global_shortcut();
        
        if let Ok(old_shortcut) = tauri_plugin_global_shortcut::Shortcut::from_str(&old_hotkey) {
            let _ = manager.unregister(old_shortcut);
        }
        
        if let Ok(new_shortcut) = tauri_plugin_global_shortcut::Shortcut::from_str(&config.global_hotkey) {
            if manager.register(new_shortcut).is_ok() {
                daemon::log_message(&app_handle, &state.log_history, &format!("Registered new global shortcut: {}", config.global_hotkey));
            }
        } else {
            daemon::log_message(&app_handle, &state.log_history, &format!("Warning: Invalid global hotkey: {}", config.global_hotkey));
        }
    }

    Ok(())
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
    let host_trimmed = host.trim();
    if host_trimmed.is_empty() {
        return Err("Invalid host: host name cannot be empty".to_string());
    }
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
        .plugin(tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app_handle, _shortcut, event| {
                if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    if let Some(state) = app_handle.try_state::<daemon::DaemonState>() {
                        daemon::trigger_capture_and_paste(app_handle, &state);
                    }
                }
            })
            .build())
        .setup(|app| {
            // Ensure configuration exists safely
            let path = AppConfig::config_file_path();
            if !path.exists() {
                let config = AppConfig::default();
                let _ = config.save();
            }
            
            let mut load_error = None;
            let initial_config = match AppConfig::load() {
                Ok(c) => c,
                Err(e) => {
                    let err_msg = format!("Configuration load error: {}", e);
                    eprintln!("{}", err_msg);
                    load_error = Some(err_msg);
                    AppConfig::default()
                }
            };
            
            // Start the daemon thread
            let daemon_state = daemon::start_daemon(app.handle().clone(), initial_config.clone());
            
            // Register initial global shortcut
            use std::str::FromStr;
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            if let Ok(shortcut) = tauri_plugin_global_shortcut::Shortcut::from_str(&initial_config.global_hotkey) {
                let _ = app.handle().global_shortcut().register(shortcut);
            } else {
                daemon::log_message(
                    &app.handle(),
                    &daemon_state.log_history,
                    &format!("Warning: Invalid initial global hotkey: {}", initial_config.global_hotkey),
                );
            }

            // Route startup load error log if present
            if let Some(err_msg) = load_error {
                daemon::log_message(
                    &app.handle(),
                    &daemon_state.log_history,
                    &err_msg,
                );
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
            if let Some(state) = app_handle.try_state::<daemon::DaemonState>() {
                if let Ok(mut running) = state.running.lock() {
                    *running = false;
                }
            }
        }
    });
}
