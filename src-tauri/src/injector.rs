//! Text injection / clipboard delivery for the paste link.
//!
//! Four modes — `injection_mode` controls HOW the text is delivered; the text
//! CONTENT itself is always decided upstream by `cli_adapter` (+ quote wrap),
//! never by this module:
//!
//! | mode        | behaviour                                         | use case                  |
//! |-------------|---------------------------------------------------|---------------------------|
//! | `direct`    | Enigo Unicode keystrokes (no clipboard)           | local terminals, no IME   |
//! | `swap`      | write text → Ctrl+V → wait 500ms → restore clip   | preserve original clipboard|
//! | `paste_keep`| write text → Ctrl+V → wait, DON'T restore         | VSCode remote (async paste)|
//! | `copy`      | write text to clipboard only, no keystroke        | manual Ctrl+V (VSCode etc)|
//!
//! The 500ms post-paste wait (up from 100ms) matters for VSCode Remote-SSH:
//! the paste is asynchronous (local clipboard → SSH → remote terminal), and
//! restoring the clipboard too early races with the remote read.

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::thread;
use std::time::Duration;
use arboard::Clipboard;

/// How long to wait after simulating Ctrl+V before restoring the clipboard
/// (swap mode) or returning (paste_keep). VSCode Remote-SSH pastes
/// asynchronously over SSH, so this needs to be generous.
const POST_PASTE_WAIT_MS: u64 = 500;

pub fn inject_text(text: &str, mode: &str) -> Result<(), String> {
    match mode {
        "direct" => inject_direct(text),
        "swap" => inject_swap(text),
        "paste_keep" => inject_paste_keep(text),
        "copy" => copy_to_clipboard(text),
        _ => inject_direct(text),
    }
}

/// Type the text via Enigo Unicode keystrokes. No clipboard involved.
pub fn inject_direct(text: &str) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to initialize Enigo: {:?}", e))?;
    enigo.text(text)
        .map_err(|e| format!("Failed to inject text: {:?}", e))
}

/// Write text to clipboard → simulate Ctrl+V → wait → **restore** the original
/// clipboard (image or text). Preserves the user's clipboard for WeChat etc.
pub fn inject_swap(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to open clipboard: {}", e))?;

    // 1. Backup existing clipboard (image or text)
    let backup_image = clipboard.get_image().ok();
    let backup_text = if backup_image.is_none() {
        clipboard.get_text().ok()
    } else {
        None
    };

    // 2. Write the paste text
    clipboard.set_text(text.to_string())
        .map_err(|e| format!("Failed to set clipboard text: {}", e))?;

    // 3. Simulate paste
    simulate_paste()?;

    // 4. Wait for the (possibly async, remote) paste to complete
    thread::sleep(Duration::from_millis(POST_PASTE_WAIT_MS));

    // 5. Restore original clipboard
    if let Some(img) = backup_image {
        let _ = clipboard.set_image(img);
    } else if let Some(txt) = backup_text {
        let _ = clipboard.set_text(txt);
    }

    Ok(())
}

/// Write text to clipboard → simulate Ctrl+V → wait → **don't restore**.
///
/// For VSCode Remote-SSH where `swap`'s restore races with the async remote
/// paste: leaving the text on the clipboard avoids the race (at the cost of
/// not preserving the original clipboard). If this works where `swap` doesn't,
/// the root cause is confirmed as the restore-timing race.
pub fn inject_paste_keep(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to open clipboard: {}", e))?;
    clipboard.set_text(text.to_string())
        .map_err(|e| format!("Failed to set clipboard text: {}", e))?;
    simulate_paste()?;
    thread::sleep(Duration::from_millis(POST_PASTE_WAIT_MS));
    // Deliberately do NOT restore — the text stays on the clipboard.
    Ok(())
}

/// Copy the text to the clipboard and stop — no keystroke injection. The user
/// pastes manually (Ctrl+V). For terminals (VSCode remote) that reject all
/// synthetic input; a real user keypress is the only thing that gets through.
fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to open clipboard: {}", e))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to set clipboard text: {}", e))?;
    Ok(())
}

/// Simulate Ctrl+V (Windows/Linux) or Cmd+V (macOS) via Enigo.
fn simulate_paste() -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to initialize Enigo: {:?}", e))?;

    #[cfg(target_os = "macos")]
    let modifier = Key::Meta; // Cmd key
    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;

    // Brief pause so the target window has focus after the clipboard write.
    thread::sleep(Duration::from_millis(50));

    enigo.key(modifier, Direction::Press)
        .map_err(|e| format!("Failed to press modifier: {:?}", e))?;
    enigo.key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| format!("Failed to click V key: {:?}", e))?;
    enigo.key(modifier, Direction::Release)
        .map_err(|e| format!("Failed to release modifier: {:?}", e))?;

    Ok(())
}
