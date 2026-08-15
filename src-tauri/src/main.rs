#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod clipboard;
mod capture;
mod cli_adapter;
mod daemon;
mod injector;
mod job;
mod host_policy;
mod routing;
mod ssh;
mod ssh_config;
mod transport;

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

/// Reject hotkey strings that would hijack system-wide shortcuts (Roadmap
/// Milestone 6-C). Pure string-token validation so it is unit-testable without
/// an AppHandle; `Shortcut::from_str` still gates final parseability later at
/// OS registration. Registering e.g. Ctrl+V globally would break pasting in
/// every application AND self-sabotage the Copy-mode flow (every manual paste
/// would spawn a failed "no image in clipboard" job — observed 2026-08-14).
fn hotkey_rejection(hotkey: &str) -> Option<String> {
    let s = hotkey.trim();
    if s.is_empty() {
        return Some("hotkey is empty".to_string());
    }
    let parts: Vec<&str> = s.split('+').map(str::trim).collect();
    let (mods, last) = parts.split_at(parts.len() - 1);
    let key = last[0];

    let has = |names: &[&str]| {
        mods.iter()
            .any(|m| names.iter().any(|n| m.eq_ignore_ascii_case(n)))
    };
    let has_ctrl = has(&["Ctrl", "Control"]);
    let has_alt = has(&["Alt"]);
    let has_shift = has(&["Shift"]);
    let has_meta = has(&["Meta", "Win", "Super", "Cmd", "Command"]);

    for m in mods {
        let known = ["Ctrl", "Control", "Alt", "Shift", "Meta", "Win", "Super", "Cmd", "Command"]
            .iter()
            .any(|n| m.eq_ignore_ascii_case(n));
        if !known {
            return Some(format!("unknown modifier '{}'", m));
        }
    }

    // OS-level combinations (Win/Cmd + key open system features) are never
    // safe for an app to own globally.
    if has_meta {
        return Some("Win/Meta combinations are reserved by the OS".to_string());
    }

    // No modifier: only function keys are safe to own globally (bare letters
    // would be swallowed while typing anywhere; F8 is a supported default).
    if mods.is_empty() {
        let mut chars = key.chars();
        let is_function_key = matches!(chars.next(), Some('F' | 'f'))
            && {
                let rest: String = chars.collect();
                !rest.is_empty()
                    && rest.chars().all(|c| c.is_ascii_digit())
                    && rest.parse::<u8>().map(|n| (1..=24).contains(&n)).unwrap_or(false)
            };
        if !is_function_key {
            return Some(
                "a hotkey without modifiers must be a function key (F1-F24)".to_string(),
            );
        }
        return None;
    }

    // Standard edit/system shortcuts that must keep working inside every app.
    let ctrl_blacklist = ["C", "V", "X", "Z", "S", "A", "F4"];
    let ctrl_standard = has_ctrl
        && !has_alt
        && !has_shift
        && ctrl_blacklist.iter().any(|k| key.eq_ignore_ascii_case(k));
    let alt_standard =
        has_alt && !has_ctrl && (key.eq_ignore_ascii_case("F4") || key.eq_ignore_ascii_case("Tab"));
    if ctrl_standard || alt_standard {
        return Some(format!("'{}' is a standard system shortcut", s));
    }
    None
}

#[tauri::command]
fn save_config(
    config: AppConfig,
    state: tauri::State<'_, daemon::DaemonState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    use std::str::FromStr;
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // Read old hotkey values to check for changes
    let (old_hotkey, old_shot) = if let Ok(c) = state.config.read() {
        (c.global_hotkey.clone(), c.screenshot_hotkey.clone())
    } else {
        ("".to_string(), "".to_string())
    };

    // Milestone 6-C: reject forbidden hotkeys BEFORE persisting, so a bad
    // value can never reach config.toml (previously the config saved first
    // and only failed later during OS re-registration, leaving disk + OS
    // state inconsistent).
    for (label, hotkey) in [
        ("Upload hotkey", &config.global_hotkey),
        ("Screenshot hotkey", &config.screenshot_hotkey),
    ] {
        if let Some(reason) = hotkey_rejection(hotkey) {
            return Err(format!("Invalid {} '{}': {}", label, hotkey, reason));
        }
        if tauri_plugin_global_shortcut::Shortcut::from_str(hotkey).is_err() {
            return Err(format!("Invalid {} '{}': unparseable shortcut format", label, hotkey));
        }
    }
    if config
        .global_hotkey
        .trim()
        .eq_ignore_ascii_case(config.screenshot_hotkey.trim())
    {
        return Err("Upload hotkey and Screenshot hotkey must be different".to_string());
    }

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

    // Re-register the screenshot hotkey if it changed
    if old_shot != config.screenshot_hotkey {
        let manager = app_handle.global_shortcut();
        if let Ok(old_s) = tauri_plugin_global_shortcut::Shortcut::from_str(&old_shot) {
            let _ = manager.unregister(old_s);
        }
        match tauri_plugin_global_shortcut::Shortcut::from_str(&config.screenshot_hotkey) {
            Ok(new_s) => {
                if manager.register(new_s).is_ok() {
                    daemon::log_message(&app_handle, &state.log_history, &format!("Registered screenshot shortcut: {}", config.screenshot_hotkey));
                }
            }
            Err(_) => daemon::log_message(&app_handle, &state.log_history, &format!("Warning: Invalid screenshot hotkey: {}", config.screenshot_hotkey)),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::hotkey_rejection;

    #[test]
    fn rejects_system_destructive_hotkeys() {
        let bad = [
            "Ctrl+V", "Ctrl+C", "Ctrl+X", "Ctrl+Z", "Ctrl+S", "Ctrl+A", "Ctrl+F4",
            "Alt+F4", "Alt+Tab", "Win+E", "Meta+L", "Cmd+Space", "A", "V", "Space",
            "Tab", "", "Ctrl+Foo+B", "F", "F99", "F0",
        ];
        for h in bad {
            assert!(hotkey_rejection(h).is_some(), "should reject {:?}", h);
        }
    }

    #[test]
    fn allows_safe_hotkeys() {
        let ok = [
            "F8", "F12", "F1", "Alt+V", "Alt+Shift+S", "Alt+X", "Ctrl+Alt+V",
            "Ctrl+Shift+P", "Ctrl+B", "Shift+D", "Alt+P",
        ];
        for h in ok {
            assert!(hotkey_rejection(h).is_none(), "should allow {:?}", h);
        }
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

/// Copy the full log history to the clipboard (Milestone 6-F).
#[tauri::command]
fn copy_logs(state: tauri::State<'_, daemon::DaemonState>) -> Result<(), String> {
    let logs = state
        .log_history
        .lock()
        .map_err(|_| "Failed to acquire log history lock".to_string())?;
    crate::injector::copy_to_clipboard(&logs.join("\n"))
}

/// Write the full log history to a file path chosen by the user via the
/// frontend save dialog (Milestone 6-F). Returns the path written.
#[tauri::command]
fn write_logs(path: String, state: tauri::State<'_, daemon::DaemonState>) -> Result<String, String> {
    let logs = state
        .log_history
        .lock()
        .map_err(|_| "Failed to acquire log history lock".to_string())?;
    let text = logs.join("\n");
    std::fs::write(&path, text).map_err(|e| format!("Failed to write log file: {}", e))?;
    Ok(path)
}

#[tauri::command]
async fn test_connection(
    host: String,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
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

    // Prefer password auth (explicit or from keyring) via the in-process SSH
    // client; fall back to the system ssh binary (key-based) when no password.
    let port_val = port.unwrap_or(22);
    let user_val = username.clone().unwrap_or_default();
    let identity = crate::ssh::identity_key(&user_val, host_trimmed, Some(port_val));
    let password = password
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .or_else(|| crate::ssh::get_stored_password(&identity));
    if let Some(pw) = password {
        return match crate::ssh::test_password_async(
            host_trimmed.to_string(),
            port_val,
            user_val,
            pw,
        )
        .await
        {
            Ok(()) => Ok("Connection Successful (password).".to_string()),
            Err(e) => Err(e),
        };
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
fn set_ssh_password(
    user: String,
    host: String,
    port: Option<u16>,
    password: String,
) -> Result<(), String> {
    let identity = crate::ssh::identity_key(user.trim(), host.trim(), port);
    if password.is_empty() {
        crate::ssh::clear_password(&identity)
    } else {
        crate::ssh::store_password(&identity, &password)
    }
}

#[tauri::command]
fn clear_ssh_password(
    user: String,
    host: String,
    port: Option<u16>,
) -> Result<(), String> {
    let identity = crate::ssh::identity_key(user.trim(), host.trim(), port);
    crate::ssh::clear_password(&identity)
}

#[tauri::command]
fn has_ssh_password(user: String, host: String, port: Option<u16>) -> bool {
    let identity = crate::ssh::identity_key(user.trim(), host.trim(), port);
    crate::ssh::has_stored_password(&identity)
}

#[tauri::command]
fn load_ssh_config(path: Option<String>) -> Result<Vec<ssh_config::SshHostEntry>, String> {
    let resolved = ssh_config::resolve_config_path(path.as_deref())
        .ok_or_else(|| "Could not determine home directory".to_string())?;
    if !resolved.exists() {
        return Err(format!("SSH config not found: {}", resolved.display()));
    }
    let content = std::fs::read_to_string(&resolved)
        .map_err(|e| format!("Failed to read {}: {}", resolved.display(), e))?;
    Ok(ssh_config::parse_ssh_config(&content))
}

/// Re-launch the app elevated (Windows only) so Enigo can reach
/// terminals that run as Administrator. UIPI otherwise blocks synthetic input
/// into a higher-integrity window, producing
/// "not all input events were sent ... blocked by UIPI".
#[cfg(windows)]
fn restart_as_admin(app: &tauri::AppHandle) {
    use std::os::windows::ffi::OsStrExt;
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
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
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
        // Must be the FIRST plugin: a second launch should surface the
        // existing window instead of spawning a duplicate tray + hotkey
        // instance (Milestone 6-E).
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app_handle, shortcut, event| {
                if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    if let Some(state) = app_handle.try_state::<daemon::DaemonState>() {
                        let (cfg_hotkey, cfg_shot) = if let Ok(cfg) = state.config.read() {
                            (cfg.global_hotkey.clone(), cfg.screenshot_hotkey.clone())
                        } else {
                            (String::new(), String::new())
                        };
                        use std::str::FromStr;
                        if let Ok(cs) = tauri_plugin_global_shortcut::Shortcut::from_str(&cfg_hotkey) {
                            if shortcut == &cs {
                                daemon::trigger_capture_and_paste(app_handle, &state);
                                return;
                            }
                        }
                        if let Ok(ss) = tauri_plugin_global_shortcut::Shortcut::from_str(&cfg_shot) {
                            if shortcut == &ss {
                                if let Err(e) = capture::capture_full_screen(app_handle, &state) {
                                    daemon::log_message(
                                        app_handle,
                                        &state.log_history,
                                        &format!("Failed to capture screen: {}", e),
                                    );
                                } else {
                                    capture::open_capture_overlay(app_handle);
                                }
                            }
                        }
                    }
                }
            })
            .build())
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Only the main Settings window hides-to-tray; the capture
                // overlay (label "capture") is allowed to close normally.
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
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
            // Register the screenshot (region-capture) hotkey
            if let Ok(shot) = tauri_plugin_global_shortcut::Shortcut::from_str(&initial_config.screenshot_hotkey) {
                let _ = app.handle().global_shortcut().register(shot);
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
            
            let mut tray_builder = TrayIconBuilder::with_id("main-tray")
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
                });

            // Set the tray icon if available (default_window_icon may return
            // None on macOS where per-window icons aren't used).
            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }
            let _tray = tray_builder.build(app)?;

            // Window is visible on launch (tauri.conf.json visible: true).
            // The on_window_event handler hides it to tray on close.
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_log_history,
            copy_logs,
            write_logs,
            test_connection,
            load_ssh_config,
            set_ssh_password,
            clear_ssh_password,
            has_ssh_password,
            capture::capture_region,
            capture::cancel_capture,
            capture::get_captured_image,
            capture::get_window_rects,
            capture::show_capture_overlay
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
