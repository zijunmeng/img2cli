//! Text injection / clipboard delivery for the paste link.
//!
//! Three modes (consolidated from five in v0.3.12 — Swap/PasteKeep relied on
//! simulating a Ctrl+V keystroke, which Chromium-family hosts reject and
//! which plain terminals never uniquely needed over typing):
//!
//! | mode     | behaviour                                | use case                      |
//! |----------|------------------------------------------|-------------------------------|
//! | `direct` | Enigo Unicode keystrokes (no clipboard)  | plain terminals, VSCode+admin  |
//! | `copy`   | clipboard only, no keystroke             | hosts rejecting synthetic      |
//! |          |                                          | input (Orca) — manual Ctrl+V   |
//! | `auto`   | resolved by `host_policy` BEFORE dispatch| the recommended default        |
//! |          | (never reaches this module as "auto")    |                               |
//!
//! Direct is best-effort: Enigo returns `Ok` once events are enqueued, with
//! no delivery acknowledgement. The job layer therefore also copies the path
//! to the clipboard as insurance when `fallback_to_copy` is on (P0,
//! docs/ISSUES_20260809.md §2).

use arboard::Clipboard;
use enigo::{Enigo, Keyboard, Settings};

pub fn inject_text(text: &str, mode: &str) -> Result<(), String> {
    match mode {
        "copy" => copy_to_clipboard(text),
        // "auto" is resolved by host_policy before dispatch; any stray
        // occurrence falls through to Direct (plain-terminal behavior).
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

/// Copy the text to the clipboard and stop — no keystroke injection. The user
/// pastes manually (Ctrl+V). For hosts (Orca, Chromium-family) that reject all
/// synthetic input; a real user keypress is the only thing that gets through.
pub(crate) fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to open clipboard: {}", e))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to set clipboard text: {}", e))?;
    Ok(())
}
