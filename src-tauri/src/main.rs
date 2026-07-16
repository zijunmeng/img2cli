#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod daemon;

use config::AppConfig;
use tauri::Manager;

#[tauri::command]
fn get_config() -> Result<AppConfig, String> {
    Ok(AppConfig::load())
}

#[tauri::command]
fn save_config(config: AppConfig) -> Result<(), String> {
    config.save()
}

#[tauri::command]
async fn test_connection(
    host: String,
    port: Option<u16>,
    username: Option<String>,
) -> Result<String, String> {
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
    
    let dest = if let Some(user) = username {
        if user.is_empty() {
            host
        } else {
            format!("{}@{}", user, host)
        }
    } else {
        host
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
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Ensure configuration exists
            let config = AppConfig::load();
            let _ = config.save(); // Save default if it didn't exist
            
            // Start the daemon thread
            daemon::start_daemon(app.handle().clone());
            
            let window = app.get_webview_window("main").unwrap();
            let _ = window.hide();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            test_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
