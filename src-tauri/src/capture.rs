//! Built-in screenshot region capture (Snipaste-style).
//!
//! Screenshot hotkey -> instantly captures screen to memory -> fullscreen transparent overlay ->
//! loads screen image -> drag a region -> crops from memory cache -> clipboard -> upload & paste.

use tauri::{AppHandle, Manager};
// The overlay window builder only runs on capture-capable platforms.
#[cfg(any(target_os = "windows", target_os = "macos"))]
use tauri::{WebviewUrl, WebviewWindowBuilder};
use crate::daemon::{self, DaemonState};

/// Instantly captures the primary monitor screenshot to memory *before* overlay loads.
pub fn capture_full_screen(app: &AppHandle, state: &DaemonState) -> Result<(), String> {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        let started = std::time::Instant::now();
        let monitors = xcap::Monitor::all().map_err(|e| format!("List monitors failed: {e}"))?;
        let mon = monitors.first().ok_or_else(|| "No monitor found".to_string())?;
        let full = mon.capture_image().map_err(|e| format!("Capture screen failed: {e}"))?;
        if let Ok(mut lock) = state.captured_image.lock() {
            *lock = Some(full);
        }
        // 6-U.9/6-U.7: how long the freeze itself takes (and at what res) —
        // feeds both the dead-hotkey diagnosis and the overlay-lag work.
        daemon::log_message(
            app,
            &state.log_history,
            &format!(
                "Screen frozen in {}ms ({}x{} physical)",
                started.elapsed().as_millis(),
                mon.width().unwrap_or(0),
                mon.height().unwrap_or(0),
            ),
        );

        // Auto window detection (Roadmap 6-J): snapshot the on-screen window
        // rects alongside the frozen frame. CSS px = physical px / scale.
        // Filters (greenshot-verified P0s, v0.4.0): titled, non-minimized,
        // reasonably sized; only windows on the CAPTURED monitor (other
        // monitors' windows would map outside the frozen frame); rects
        // intersected with the monitor bounds (maximized/snapped overhang,
        // off-screen junk). xcap rects are client-area on Windows — no DWM
        // shadow margin involved — and cloaked/UWP-ghost filtering is already
        // handled inside xcap.
        let scale = mon.scale_factor().unwrap_or(1.0);
        let mon_id = mon.id().ok();
        // width()/height() return XCapResult<u32>; on failure, skip clamping.
        let mon_w = mon.width().unwrap_or(u32::MAX) as i32;
        let mon_h = mon.height().unwrap_or(u32::MAX) as i32;
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
                if w.current_monitor().ok().and_then(|m| m.id().ok()) != mon_id {
                    continue; // lives on another monitor
                }
                // Intersect with the monitor (physical px).
                let left = x.max(0);
                let top = y.max(0);
                let right = (x + width as i32).min(mon_w);
                let bottom = (y + height as i32).min(mon_h);
                if right - left < 40 || bottom - top < 40 {
                    continue;
                }
                rects.push(daemon::WindowRect {
                    x: (left as f32 / scale) as i32,
                    y: (top as f32 / scale) as i32,
                    w: ((right - left) as f32 / scale) as i32,
                    h: ((bottom - top) as f32 / scale) as i32,
                    title,
                });

                // 6-S (Windows): also collect this window's CHILD elements —
                // EnumChildWindows walks all descendants. Children land right
                // after their parent, so the overlay's Tab cycling drills
                // down: top window → deeper controls.
                #[cfg(target_os = "windows")]
                {
                    use windows_sys::Win32::Foundation::{HWND, RECT};
                    use windows_sys::Win32::UI::WindowsAndMessaging::{
                        EnumChildWindows, GetWindowRect, IsWindowVisible,
                    };
                    unsafe extern "system" fn enum_child(hwnd: HWND, lparam: isize) -> i32 {
                        let out = unsafe { &mut *(lparam as *mut Vec<(i32, i32, i32, i32)>) };
                        unsafe {
                            if IsWindowVisible(hwnd) != 0 {
                                let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                                if GetWindowRect(hwnd, &mut r) != 0 {
                                    out.push((r.left, r.top, r.right - r.left, r.bottom - r.top));
                                }
                            }
                        }
                        1 // continue
                    }
                    let hwnd = w.id().unwrap_or(0) as usize as HWND;
                    if !hwnd.is_null() {
                        let mut children: Vec<(i32, i32, i32, i32)> = Vec::new();
                        unsafe {
                            EnumChildWindows(hwnd, Some(enum_child), &mut children as *mut _ as isize);
                        }
                        for (cx, cy, cw, ch) in children {
                            let left = cx.max(0);
                            let top = cy.max(0);
                            let right = (cx + cw).min(mon_w);
                            let bottom = (cy + ch).min(mon_h);
                            if right - left < 8 || bottom - top < 8 {
                                continue;
                            }
                            rects.push(daemon::WindowRect {
                                x: (left as f32 / scale) as i32,
                                y: (top as f32 / scale) as i32,
                                w: ((right - left) as f32 / scale) as i32,
                                h: ((bottom - top) as f32 / scale) as i32,
                                title: String::new(),
                            });
                        }
                    }
                }
                if rects.len() > 600 {
                    break; // defensive cap for pathological window trees
                }
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
/// v0.4.4 (T6): the overlay window is created ONCE (hidden, at startup via
/// `prewarm_capture_overlay`) and kept alive — the hotkey only refreshes it,
/// so there is no window/webview cold start between hotkey and first paint.
pub fn open_capture_overlay(app: &AppHandle, state: &DaemonState) {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        if let Some(existing) = app.get_webview_window("capture") {
            use tauri::Emitter;
            daemon::log_message(
                app,
                &state.log_history,
                "Capture overlay: refreshing warm window (capture-refresh sent)",
            );
            let _ = existing.emit_to("capture", "capture-refresh", ());
            return;
        }
        // First run (or the window died): build it now, hidden — the frontend
        // loads and invokes show_capture_overlay itself after rendering.
        daemon::log_message(
            app,
            &state.log_history,
            "Capture overlay: warm window missing — rebuilding now",
        );
        let build = WebviewWindowBuilder::new(
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
        if let Err(e) = build {
            daemon::log_message(
                app,
                &state.log_history,
                &format!("Capture overlay: window build failed: {}", e),
            );
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (app, state);
    }
}

/// Build the hidden overlay window at startup (T6: instant hotkey response).
pub fn prewarm_capture_overlay(app: &AppHandle, state: &DaemonState) {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        if app.get_webview_window("capture").is_none() {
            let build = WebviewWindowBuilder::new(
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
            // 6-U.9: a failed prewarm used to be silent; the first hotkey then
            // pays a full window build (or nothing shows at all).
            if let Err(e) = build {
                daemon::log_message(
                    app,
                    &state.log_history,
                    &format!("Capture overlay: prewarm build failed: {}", e),
                );
            }
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (app, state);
    }
}

fn close_capture_overlay(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("capture") {
        // T6: hide, don't destroy — the window stays warm for the next hotkey.
        let _ = win.hide();
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
        // T6 (v0.4.4): the overlay DISPLAY layer ships as JPEG — a 4K PNG is
        // a multi-MB base64 payload over IPC (the visible hotkey→frame lag);
        // JPEG q85 is ~10x smaller. Final crops still come from the pristine
        // RgbaImage in Rust, so delivered quality is unaffected; only
        // ANNOTATED composites sample this display layer.
        use image::codecs::jpeg::JpegEncoder;
        use image::ExtendedColorType;
        #[allow(unused_imports)]
        use image::ImageEncoder as _;
        let mut bytes = Vec::new();
        let mut enc = JpegEncoder::new_with_quality(&mut bytes, 85);
        enc.encode(img.as_raw(), img.width(), img.height(), ExtendedColorType::Rgba8)
            .map_err(|e| format!("JPEG encode failed: {}", e))?;
        let b64 = base64_encode(&bytes);
        return Ok(format!("data:image/jpeg;base64,{}", b64));
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
    annotated: Option<String>,
) -> Result<(), String> {
    close_capture_overlay(&app_handle);
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        use std::borrow::Cow;
        // v0.4.2: when the overlay composited annotations (canvas rasterized
        // over the frozen frame), that data URL REPLACES the plain crop —
        // clipboard, history and the background upload all use the annotated
        // image from here on.
        let cropped = match annotated {
            Some(data_url) => crate::clipboard::decode_data_url_image(&data_url)
                .map_err(|e| format!("annotated composite invalid: {}", e))?,
            None => {
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

                image::imageops::crop_imm(&full, cx, cy, cw, ch).to_image()
            }
        };
        // v0.3.15 capture-then-upload: keep a copy for the background upload
        // job (the clipboard takes ownership of the original buffer below).
        let for_upload = cropped.clone();

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

        // Remember the confirmed selection: last_capture_rect for compat plus
        // the newest-first history (v0.4.1: Shift+R / `,` / `.` in the
        // overlay). Consecutive duplicates are skipped; history caps at 8.
        if let Ok(mut cfg) = state.config.write() {
            let r = crate::config::CaptureRect {
                x: x,
                y: y,
                w: w as i32,
                h: h as i32,
            };
            cfg.last_capture_rect = Some(r);
            if cfg.capture_history.first() != Some(&r) {
                cfg.capture_history.insert(0, r);
                cfg.capture_history.truncate(8);
            }
            let _ = cfg.save();
        }

        // v0.3.15 "capture = upload": ship the region to the server in the
        // background right away; the inject hotkey pastes the path instantly.
        daemon::trigger_upload_only(&app_handle, &state, for_upload);
        // Dynamic hint: use the configured hotkey + injection mode (spec §11.3)
        let (hotkey, mode) = if let Ok(cfg) = state.config.read() {
            (cfg.global_hotkey.clone(), cfg.injection_mode)
        } else {
            ("Alt+V".to_string(), crate::config::InjectionMode::Auto)
        };
        let hint = match mode {
            crate::config::InjectionMode::Copy => format!(
                "Screenshot captured. Uploading in the background — press {} to copy the path, then Ctrl+V.",
                hotkey
            ),
            _ => format!(
                "Screenshot captured. Uploading in the background — Ctrl+V pastes the image, or press {} to paste its path.",
                hotkey
            ),
        };
        daemon::log_message(&app_handle, &state.log_history, &hint);
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (state, x, y, w, h, &annotated);
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
pub fn show_capture_overlay(app_handle: AppHandle, state: tauri::State<'_, DaemonState>) {
    // 6-U.9: chain terminus — the overlay webview invokes this only after the
    // frozen frame has rendered. This line appearing = the whole hotkey →
    // capture → emit → render → show chain worked; anything above it that is
    // missing points at the broken link.
    let Some(win) = app_handle.get_webview_window("capture") else {
        daemon::log_message(&app_handle, &state.log_history, "Capture overlay: show requested but window not found");
        return;
    };
    match win.show() {
        Ok(()) => {
            let _ = win.set_focus();
            daemon::log_message(&app_handle, &state.log_history, "Capture overlay shown");
        }
        Err(e) => daemon::log_message(&app_handle, &state.log_history, &format!("Capture overlay: show failed: {}", e)),
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
