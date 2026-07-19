//! Built-in screenshot region capture (Snipaste-style).
//!
//! Screenshot hotkey -> instantly captures screen to memory -> fullscreen transparent overlay ->
//! loads screen image -> drag a region -> crops from memory cache -> clipboard -> upload & paste.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use crate::daemon::{self, DaemonState};

/// Instantly captures the primary monitor screenshot to memory *before* overlay loads.
pub fn capture_full_screen(_app: &AppHandle, state: &DaemonState) -> Result<(), String> {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        let monitors = xcap::Monitor::all().map_err(|e| format!("List monitors failed: {e}"))?;
        let mon = monitors.first().ok_or_else(|| "No monitor found".to_string())?;
        let full = mon.capture_image().map_err(|e| format!("Capture screen failed: {e}"))?;
        if let Ok(mut lock) = state.captured_image.lock() {
            *lock = Some(full);
        }
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (_app, state);
        Err("Screenshot capture is not supported on this platform".to_string())
    }
}

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

#[tauri::command]
pub fn get_captured_image(state: tauri::State<'_, DaemonState>) -> Result<String, String> {
    let lock = state.captured_image.lock().map_err(|_| "Lock failed")?;
    if let Some(ref img) = *lock {
        let mut png_bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        
        let dynamic_img = image::DynamicImage::ImageRgba8(img.clone());
        dynamic_img.write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode captured image to PNG: {}", e))?;
            
        let b64 = base64_encode(&png_bytes);
        return Ok(format!("data:image/png;base64,{}", b64));
    }
    Err("No captured screenshot in memory".to_string())
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
        use std::borrow::Cow;
        
        let full = {
            let lock = state.captured_image.lock().map_err(|_| "Failed to lock captured image")?;
            lock.clone().ok_or_else(|| "No captured image in memory".to_string())?
        };

        let scale = {
            let monitors = xcap::Monitor::all().map_err(|e| format!("List monitors: {e}"))?;
            let mon = monitors.first().ok_or_else(|| "No monitor found".to_string())?;
            mon.scale_factor().unwrap_or(1.0)
        };

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

        let mut cb = arboard::Clipboard::new().map_err(|e| format!("Open clipboard: {e}"))?;
        cb.set_image(arboard::ImageData {
            width: cropped.width() as usize,
            height: cropped.height() as usize,
            bytes: Cow::Owned(cropped.into_raw()),
        })
        .map_err(|e| format!("Set clipboard image error: {e}"))?;

        // Triggers upload and injection
        daemon::trigger_capture_and_paste(&app_handle, state.inner());
        Ok(())
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

fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        match chunk.len() {
            3 => {
                let val = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
                result.push(CHARSET[((val >> 18) & 63) as usize] as char);
                result.push(CHARSET[((val >> 12) & 63) as usize] as char);
                result.push(CHARSET[((val >> 6) & 63) as usize] as char);
                result.push(CHARSET[(val & 63) as usize] as char);
            }
            2 => {
                let val = ((chunk[0] as u32) << 8) | (chunk[1] as u32);
                result.push(CHARSET[((val >> 10) & 63) as usize] as char);
                result.push(CHARSET[((val >> 4) & 63) as usize] as char);
                result.push(CHARSET[((val << 2) & 63) as usize] as char);
                result.push('=');
            }
            1 => {
                let val = chunk[0] as u32;
                result.push(CHARSET[((val >> 2) & 63) as usize] as char);
                result.push(CHARSET[((val << 4) & 63) as usize] as char);
                result.push('=');
                result.push('=');
            }
            _ => {}
        }
    }
    result
}
