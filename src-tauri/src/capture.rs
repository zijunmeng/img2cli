//! Built-in screenshot region capture (Snipaste-style).
//!
//! Flow: screenshot hotkey -> open a fullscreen transparent overlay -> user
//! drags a region -> `capture_region` closes the overlay, grabs the primary
//! monitor via xcap, crops to the selection, puts the region on the clipboard,
//! then runs the existing clipboard -> compress/route/upload/inject pipeline.

use std::borrow::Cow;
use std::time::Duration;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::daemon::{self, DaemonState};

/// Open the fullscreen, transparent, always-on-top region-selection overlay.
pub fn open_capture_overlay(app: &AppHandle) {
    if let Some(existing) = app.get_webview_window("capture") {
        let _ = existing.show();
        let _ = existing.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "capture", WebviewUrl::App("index.html?capture=1".into()))
        .title("")
        .fullscreen(true)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .build();
}

fn close_capture_overlay(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("capture") {
        let _ = win.close();
    }
}

/// Capture the selected region (CSS px, relative to the overlay = primary
/// monitor origin), put it on the clipboard, then trigger the standard
/// clipboard -> inject flow.
#[tauri::command]
pub fn capture_region(
    app_handle: AppHandle,
    state: tauri::State<'_, DaemonState>,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
) -> Result<(), String> {
    // Close the overlay first so it isn't in the shot; let the OS redraw.
    close_capture_overlay(&app_handle);
    std::thread::sleep(Duration::from_millis(180));

    let monitors = xcap::Monitor::all().map_err(|e| format!("list monitors: {e}"))?;
    let mon = monitors
        .first()
        .ok_or_else(|| "no monitor found".to_string())?;
    let scale = mon.scale_factor().unwrap_or(1.0);
    let full = mon
        .capture_image()
        .map_err(|e| format!("screen capture: {e}"))?;

    // Selection coords are CSS px; xcap image is physical px (× scale factor).
    let cx = ((x as f32) * scale).max(0.0) as u32;
    let cy = ((y as f32) * scale).max(0.0) as u32;
    let cw = (((w as f32) * scale) as u32)
        .max(1)
        .min(full.width().saturating_sub(cx));
    let ch = (((h as f32) * scale) as u32)
        .max(1)
        .min(full.height().saturating_sub(cy));
    let cropped = image::imageops::crop_imm(&full, cx, cy, cw, ch).to_image();

    let mut cb = arboard::Clipboard::new().map_err(|e| format!("open clipboard: {e}"))?;
    cb.set_image(arboard::ImageData {
        width: cropped.width() as usize,
        height: cropped.height() as usize,
        bytes: Cow::Owned(cropped.into_raw()),
    })
    .map_err(|e| format!("set clipboard image: {e}"))?;

    // Reuse the standard clipboard -> compress/route/upload/inject flow.
    daemon::trigger_capture_and_paste(&app_handle, state.inner());
    Ok(())
}

/// Cancel the capture (Esc / right-click in the overlay).
#[tauri::command]
pub fn cancel_capture(app_handle: AppHandle) -> Result<(), String> {
    close_capture_overlay(&app_handle);
    Ok(())
}
