//! Built-in screenshot region capture (Snipaste-style).
//!
//! Screenshot hotkey -> fullscreen transparent overlay -> drag a region ->
//! `capture_region` grabs the primary monitor (xcap), crops to the selection,
//! puts it on the clipboard, then runs the existing clipboard -> inject flow.
//!
//! Note: xcap's Linux backend pulls in PipeWire/libspa which is incompatible
//! with older system libspa (e.g. Ubuntu 22.04), so the actual screen grab is
//! built only for Windows and macOS. On Linux the overlay is not opened.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::daemon::{self, DaemonState};

/// Open the fullscreen region-selection overlay (Windows / macOS only).
pub fn open_capture_overlay(app: &AppHandle) {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        if let Some(existing) = app.get_webview_window("capture") {
            let _ = existing.show();
            let _ = existing.set_focus();
            return;
        }
        let _ = WebviewWindowBuilder::new(
            app,
            "capture",
            WebviewUrl::App("index.html?capture=1".into()),
        )
        .title("")
        .fullscreen(true)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .build();
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = app;
    }
}

fn close_capture_overlay(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("capture") {
        let _ = win.close();
    }
}

/// Grab the selected region and run the inject flow (Windows / macOS only).
#[cfg(any(target_os = "windows", target_os = "macos"))]
fn do_capture(
    app_handle: &AppHandle,
    state: &DaemonState,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
) -> Result<(), String> {
    use std::borrow::Cow;
    use std::time::Duration;

    // The overlay was just closed; let the OS redraw so it isn't in the shot.
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

    daemon::trigger_capture_and_paste(app_handle, state);
    Ok(())
}

#[tauri::command]
pub fn capture_region(
    app_handle: AppHandle,
    state: tauri::State<'_, DaemonState>,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
) -> Result<(), String> {
    close_capture_overlay(&app_handle);
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        do_capture(&app_handle, state.inner(), x, y, w, h)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (state, x, y, w, h);
        Err("Screenshot capture is not supported on this platform".to_string())
    }
}

/// Cancel the capture (Esc / tiny selection in the overlay).
#[tauri::command]
pub fn cancel_capture(app_handle: AppHandle) -> Result<(), String> {
    close_capture_overlay(&app_handle);
    Ok(())
}
