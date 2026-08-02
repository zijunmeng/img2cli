use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::thread;
use std::time::Duration;
use arboard::Clipboard;

pub fn inject_text(text: &str, mode: &str) -> Result<(), String> {
    match mode {
        "direct" => inject_direct(text),
        "swap" => inject_swap(text),
        "copy" => copy_to_clipboard(text),
        _ => inject_direct(text),
    }
}

/// Copy the text to the clipboard and stop — no keystroke injection.
///
/// For terminals that reject synthetic input (notably VSCode's remote terminal
/// running an AI CLI like Claude Code): img2cli's Enigo-based `direct`/`swap`
/// injection is ignored, so nothing appears. With `copy`, img2cli just puts the
/// path on the clipboard and the user pastes manually (a real keypress the
/// terminal accepts). Claude Code turns a pasted absolute image path into an
/// attached image (`[Image #N]`), so this is the reliable path for that case.
fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to open clipboard: {}", e))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to set clipboard text: {}", e))?;
    Ok(())
}

pub fn inject_direct(text: &str) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to initialize Enigo: {:?}", e))?;
    enigo.text(text)
        .map_err(|e| format!("Failed to inject text: {:?}", e))
}

pub fn inject_swap(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to open clipboard: {}", e))?;
        
    // 1. Backup existing clipboard content (image or text)
    let backup_image = clipboard.get_image().ok();
    let backup_text = if backup_image.is_none() {
        clipboard.get_text().ok()
    } else {
        None
    };
    
    // 2. Set new Markdown link text to clipboard
    clipboard.set_text(text.to_string())
        .map_err(|e| format!("Failed to set clipboard text: {}", e))?;
        
    // 3. Simulate Ctrl+V (or Cmd+V on macOS)
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to initialize Enigo: {:?}", e))?;
        
    #[cfg(target_os = "macos")]
    let modifier = Key::Meta; // Cmd key
    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;
    
    // Briefly sleep to ensure target focus window is ready
    thread::sleep(Duration::from_millis(50));
    
    enigo.key(modifier, Direction::Press)
        .map_err(|e| format!("Failed to press modifier: {:?}", e))?;
    enigo.key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| format!("Failed to click V key: {:?}", e))?;
    enigo.key(modifier, Direction::Release)
        .map_err(|e| format!("Failed to release modifier: {:?}", e))?;
        
    // 4. Wait for the OS paste event to complete
    thread::sleep(Duration::from_millis(100));
    
    // 5. Restore original clipboard content
    if let Some(img) = backup_image {
        let _ = clipboard.set_image(img);
    } else if let Some(txt) = backup_text {
        let _ = clipboard.set_text(txt);
    }
    
    Ok(())
}
