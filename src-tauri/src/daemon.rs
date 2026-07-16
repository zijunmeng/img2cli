use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

pub struct DaemonState {
    pub running: Arc<Mutex<bool>>,
    pub log_history: Arc<Mutex<Vec<String>>>,
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

pub fn start_daemon(app_handle: AppHandle) -> DaemonState {
    let running = Arc::new(Mutex::new(true));
    let running_clone = running.clone();
    let log_history = Arc::new(Mutex::new(Vec::new()));
    let log_history_clone = log_history.clone();
    let app_clone = app_handle.clone();
    
    thread::spawn(move || {
        log_message(&app_clone, &log_history_clone, "Daemon background thread started.");
        
        loop {
            // Safe lock evaluation to check if thread should stop
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
    
    DaemonState { running, log_history }
}
