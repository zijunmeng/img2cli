#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod clipboard;
mod daemon;
mod injector;
mod ssh_config;

use config::AppConfig;
use tauri::Manager;
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::tray::TrayIconBuilder;

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

    // Dynamic shortcut unregistration & registration updates with error recovery/rollback
    if old_hotkey != config.global_hotkey {
        let manager = app_handle.global_shortcut();
        
        let old_shortcut_parsed = tauri_plugin_global_shortcut::Shortcut::from_str(&old_hotkey).ok();
        let new_shortcut_parsed = tauri_plugin_global_shortcut::Shortcut::from_str(&config.global_hotkey).ok();
        
        if let Some(old_s) = old_shortcut_parsed {
            let _ = manager.unregister(old_s);
        }
        
        if let Some(new_s) = new_shortcut_parsed {
            if let Err(e) = manager.register(new_s) {
                // Rollback: try to restore the old shortcut if the new one failed
                if let Some(old_s) = old_shortcut_parsed {
                    let _ = manager.register(old_s);
                }
                daemon::log_message(
                    &app_handle,
                    &state.log_history,
                    &format!("Error: Failed to register hotkey '{}': {:?}. Restored old hotkey.", config.global_hotkey, e),
                );
                return Err(format!("Failed to register hotkey: {:?}", e));
            } else {
                daemon::log_message(&app_handle, &state.log_history, &format!("Registered new global shortcut: {}", config.global_hotkey));
            }
        } else {
            // Rollback: try to restore the old shortcut if the new one format is invalid
            if let Some(old_s) = old_shortcut_parsed {
                let _ = manager.register(old_s);
            }
            daemon::log_message(&app_handle, &state.log_history, &format!("Warning: Invalid global hotkey: {}", config.global_hotkey));
            return Err("Invalid global hotkey format".to_string());
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

#[tauri::command]
fn load_ssh_config() -> Result<Vec<ssh_config::SshHostEntry>, String> {
    let path = ssh_config::ssh_config_path()
        .ok_or_else(|| "Could not determine home directory".to_string())?;
    if !path.exists() {
        return Err(format!("OpenSSH config not found: {:?}", path));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
    Ok(ssh_config::parse_ssh_config(&content))
}

/// Re-launch the app elevated (Windows only) so SendInput/Enigo can reach
/// terminals that run as Administrator. UIPI otherwise blocks synthetic input
/// into a higher-integrity window, producing
/// "not all input events were sent ... blocked by UIPI".
#[cfg(windows)]
fn restart_as_admin(app: &tauri::AppHandle) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::core::PCWSTR;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };
    let file: Vec<u16> = std::ffi::OsStr::new(exe.as_os_str())
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let verb: Vec<u16> = "runas".encode_utf16().chain(std::iter::once(0)).collect();

    // ShellExecuteW(NULL, "runas", <exe>, NULL, NULL, SW_SHOWNORMAL).
    // A return value <= 32 means failure (e.g. the user declined the UAC prompt).
    let hinst = unsafe {
        ShellExecuteW(
            0,
            PCWSTR(verb.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR(std::ptr::null::<u16>()),
            PCWSTR(std::ptr::null::<u16>()),
            SW_SHOWNORMAL,
        )
    };
    if hinst as isize <= 32 {
        // User declined UAC or it otherwise failed — keep the current instance.
        return;
    }
    app.exit(0);
}

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app_handle, shortcut, event| {
                if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    if let Some(state) = app_handle.try_state::<daemon::DaemonState>() {
                        // Check if the triggered shortcut matches our configured shortcut
                        let configured_hotkey = if let Ok(cfg) = state.config.read() {
                            cfg.global_hotkey.clone()
                        } else {
                            "".to_string()
                        };
                        use std::str::FromStr;
                        if let Ok(configured_shortcut) = tauri_plugin_global_shortcut::Shortcut::from_str(&configured_hotkey) {
                            if shortcut == &configured_shortcut {
                                daemon::trigger_capture_and_paste(app_handle, &state);
                            }
                        }
                    }
                }
            })
            .build())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Intercept close events to hide the Settings window instead of exiting
                api.prevent_close();
                let _ = window.hide();
            }
        })
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
            
            // Build the system tray and context menu
            let show_i = MenuItem::with_id(app, "show", "Show Settings", true, None::<&str>)?;
            let exit_i = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
            #[cfg(windows)]
            let admin_i = MenuItem::with_id(
                app,
                "restart_admin",
                "Restart as Administrator",
                true,
                None::<&str>,
            )?;
            let mut builder = MenuBuilder::new(app).item(&show_i);
            #[cfg(windows)]
            {
                builder = builder.item(&admin_i);
            }
            let menu = builder.item(&exit_i).build()?;
            
            let icon = app.default_window_icon().cloned().unwrap();
            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "restart_admin" => {
                            #[cfg(windows)]
                            restart_as_admin(app);
                        }
                        "exit" => app.exit(0),
                        _ => {}
                    }
                })
                .build(app)?;

            let window = app.get_webview_window("main").unwrap();
            let _ = window.hide();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_log_history,
            test_connection,
            load_ssh_config
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
