use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

pub struct DaemonState {
    pub running: Arc<Mutex<bool>>,
}

pub fn log_message(app_handle: &AppHandle, message: &str) {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let formatted = format!("[{}] {}", timestamp, message);
    let _ = app_handle.emit("log_append", formatted);
}

pub fn start_daemon(app_handle: AppHandle) -> DaemonState {
    let running = Arc::new(Mutex::new(true));
    let running_clone = running.clone();
    let app_clone = app_handle.clone();
    
    thread::spawn(move || {
        log_message(&app_clone, "Daemon background thread started.");
        
        while *running_clone.lock().unwrap() {
            thread::sleep(std::time::Duration::from_millis(500));
            // Future hotkey & clipboard polling logic will go here.
        }
        
        log_message(&app_clone, "Daemon background thread stopped.");
    });
    
    DaemonState { running }
}
