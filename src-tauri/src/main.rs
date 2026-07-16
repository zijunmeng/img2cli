use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            // Hide window on startup, letting it reside in tray
            let _ = window.hide();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
