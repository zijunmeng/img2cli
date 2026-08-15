//! Built-in screenshot region capture (Snipaste-style).
//!
//! Screenshot hotkey -> instantly captures screen to memory -> fullscreen transparent overlay ->
//! loads screen image -> drag a region -> crops from memory cache -> clipboard -> upload & paste.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use crate::daemon::{self, DaemonState};

/// Instantly captures the primary monitor screenshot to memory *before* overlay loads.
pub fn capture_full_screen(app: &AppHandle, state: &DaemonState) -> Result<(), String> {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        let monitors = xcap::Monitor::all().map_err(|e| format!("List monitors failed: {e}"))?;
        let mon = monitors.first().ok_or_else(|| "No monitor found".to_string())?;
        let full = mon.capture_image().map_err(|e| format!("Capture screen failed: {e}"))?;
        if let Ok(mut lock) = state.captured_image.lock() {
            *lock = Some(full);
        }

        // Auto window detection (Roadmap 6-J): snapshot the on-screen window
        // rects alongside the frozen frame. CSS px = physical px / scale.
        // Filter: titled, non-minimized, reasonably sized windows — empty
        // titles skip tool overlays (including our own capture window).
        let scale = mon.scale_factor().unwrap_or(1.0);
        let mut rects = Vec::new();
        if let Ok(windows) = xcap::Window::all() {
            for w in windows {
                let (Ok(title), Ok(x), Ok(y), Ok(width), Ok(height), Ok(minimized)) =
                    (w.title(), w.x(), w.y(), w.width(), w.height(), w.is_minimized())
                else {
                    continue;
                };
                if minimized || title.is_empty() || width < 40 || height < 40 {
                    continue;
                }
                rects.push(daemon::WindowRect {
                    x: (x as f32 / scale) as i32,
                    y: (y as f32 / scale) as i32,
                    w: (width as f32 / scale) as i32,
                    h: (height as f32 / scale) as i32,
                    title,
                });
            }
        }
        // xcap deliberately excludes windows owned by the current process
        // (WebRTC-borrowed deadlock guard; verified xcap 0.5.2
        // impl_window.rs), so our own Settings window never appears. We know
        // our windows — append the main window's rect via Tauri APIs
        // (Roadmap 6-M). No GetWindowText on own windows → no deadlock risk.
        if let Some(win) = app.get_webview_window("main") {
            let visible = win.is_visible().unwrap_or(false);
            let minimized = win.is_minimized().unwrap_or(false);
            if visible && !minimized {
                if let (Ok(pos), Ok(size)) = (win.outer_position(), win.outer_size()) {
                    rects.push(daemon::WindowRect {
                        x: (pos.x as f32 / scale) as i32,
                        y: (pos.y as f32 / scale) as i32,
                        w: (size.width as f32 / scale) as i32,
                        h: (size.height as f32 / scale) as i32,
                        title: "img2cli".to_string(),
                    });
                }
            }
        }
        if let Ok(mut lock) = state.window_rects.lock() {
            *lock = rects;
        }
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (app, state);
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
        .visible(false)
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

/// On-screen window rects (CSS px) captured with the frozen frame — consumed
/// by the overlay's auto window detection (Roadmap 6-J).
#[tauri::command]
pub fn get_window_rects(state: tauri::State<'_, DaemonState>) -> Result<Vec<daemon::WindowRect>, String> {
    Ok(state.window_rects.lock().map_err(|_| "Lock failed")?.clone())
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

        // v0.3.7: Alt+Z now captures ONLY — it puts the image in the clipboard
        // and stops. The screenshot subject and the AI CLI are rarely on screen
        // at the same time, so the old "capture + immediately upload + inject"
        // behavior sent the path into the wrong window. Now the user switches
        // to the AI CLI and presses Alt+V, which reads this clipboard image and
        // uploads + injects. The same clipboard image can also be Ctrl+V'd into
        // WeChat / QQ / Feishu.
        let cw = cropped.width();
        let ch = cropped.height();
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_image(arboard::ImageData {
                width: cw as usize,
                height: ch as usize,
                bytes: Cow::Owned(cropped.into_raw()),
            });
        }

        // Remember the confirmed selection (Roadmap 6-L) so the next capture
        // can preload it for repeat captures of the same region. Always
        // recorded; the capture_remember_region toggle gates the preload.
        if let Ok(mut cfg) = state.config.write() {
            cfg.last_capture_rect = Some(crate::config::CaptureRect {
                x: x,
                y: y,
                w: w as i32,
                h: h as i32,
            });
            let _ = cfg.save();
        }
        // Dynamic hint: use the configured hotkey + injection mode (spec §11.3)
        let (hotkey, mode) = if let Ok(cfg) = state.config.read() {
            (cfg.global_hotkey.clone(), cfg.injection_mode)
        } else {
            ("Alt+V".to_string(), crate::config::InjectionMode::Auto)
        };
        let hint = match mode {
            crate::config::InjectionMode::Copy => format!(
                "Screenshot captured. Switch to your AI CLI, press {} to upload, then Ctrl+V.",
                hotkey
            ),
            crate::config::InjectionMode::Auto => format!(
                "Screenshot captured. Ctrl+V pastes the image directly; press {} to upload + paste the path.",
                hotkey
            ),
            crate::config::InjectionMode::Direct => format!(
                "Screenshot captured. Ctrl+V pastes the image directly; press {} to upload + paste the path.",
                hotkey
            ),
        };
        daemon::log_message(&app_handle, &state.log_history, &hint);
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

/// Reveal the capture overlay after its frozen frame has rendered. The overlay
/// is built hidden (open_capture_overlay: visible(false)) so the WebView's
/// initial white frame never shows; the frontend calls this once the image is
/// on screen, giving a flash-free reveal (Snipaste-style). Done as a custom
/// command because the capture window isn't in the capability allowlist, so the
/// core `window.show()` IPC would be denied — custom commands aren't gated.
#[tauri::command]
pub fn show_capture_overlay(app_handle: AppHandle) {
    if let Some(win) = app_handle.get_webview_window("capture") {
        let _ = win.show();
        let _ = win.set_focus();
    }
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
