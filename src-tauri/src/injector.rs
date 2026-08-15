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
        "swap" | "auto" => inject_swap(text),
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
pub(crate) fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to open clipboard: {}", e))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to set clipboard text: {}", e))?;
    Ok(())
}

/// Simulate Ctrl+V paste. On Windows, uses native Win32 `SendInput` with
/// virtual key codes (VK_CONTROL + 0x56) instead of Enigo's `Unicode('v')` —
/// Enigo's approach is blocked by UIPI in many contexts. On macOS/Linux, keeps
/// the Enigo fallback.
#[cfg(windows)]
fn simulate_paste() -> Result<(), String> {
    // Preflight: wait for the user's modifier keys + V to be fully released
    // before injecting (avoids shortcut recursion / stuck modifiers).
    if !windows_keys_released(1000) {
        return Err("preflight timed out: modifier keys still pressed after 1000ms".into());
    }
    windows_send_paste()
}

#[cfg(not(windows))]
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

// ── Windows native paste (Win32 SendInput) ────────────────────────────────

/// Wait for modifier keys + V to be released (spec §8.2).
/// Returns true if all clear within `timeout_ms`, false on timeout.
#[cfg(windows)]
fn windows_keys_released(timeout_ms: u64) -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    // VK_CONTROL(0x11) VK_MENU(0x12) VK_SHIFT(0x10) VK_LWIN(0x5B) VK_RWIN(0x5C) V(0x56)
    let vks: [i32; 6] = [0x11, 0x12, 0x10, 0x5B, 0x5C, 0x56];
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);
    loop {
        let all_released = vks.iter().all(|&vk| {
            let state = unsafe { GetAsyncKeyState(vk) };
            state >= 0 // MSB clear = key not currently pressed
        });
        if all_released {
            return true;
        }
        if start.elapsed() >= timeout {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// Send Ctrl+V as four virtual-key events in a single `SendInput` call
/// (spec §8.3). Returns Ok(()) if all 4 events were inserted, Err with
/// diagnostics otherwise.
#[cfg(windows)]
fn windows_send_paste() -> Result<(), String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    };
    use windows_sys::Win32::Foundation::GetLastError;

    const VK_CTRL: u16 = 0x11;
    const VK_V: u16 = 0x56;

    let kb = |vk: u16, flags: u32| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT { wVk: vk, wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 },
        },
    };

    // All four events in ONE SendInput call (spec §8.3 requirement).
    let inputs: [INPUT; 4] = [
        kb(VK_CTRL, 0),               // Ctrl down
        kb(VK_V, 0),                   // V down
        kb(VK_V, KEYEVENTF_KEYUP),     // V up
        kb(VK_CTRL, KEYEVENTF_KEYUP),  // Ctrl up
    ];

    let sent = unsafe {
        SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32)
    };

    if sent != inputs.len() as u32 {
        // Failure cleanup: best-effort V up + Ctrl up (spec §8.4).
        let cleanup: [INPUT; 2] = [kb(VK_V, KEYEVENTF_KEYUP), kb(VK_CTRL, KEYEVENTF_KEYUP)];
        let _ = unsafe {
            SendInput(cleanup.len() as u32, cleanup.as_ptr(), std::mem::size_of::<INPUT>() as i32)
        };
        let err = unsafe { GetLastError() };
        return Err(format!(
            "SendInput: requested=4 inserted={} last_error={} — possible UIPI or focus issue",
            sent, err
        ));
    }

    Ok(())
}
