#![allow(dead_code)]
use crate::config::Config;
use enigo::{Enigo, Keyboard, Settings};
use rdev::{listen, Event, EventType, Key};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hotkey {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
    pub key: Key,
}

#[derive(Default, Debug, Clone)]
struct ModifierState {
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
}

fn parse_key(name: &str) -> Option<Key> {
    match name.to_lowercase().as_str() {
        "a" => Some(Key::KeyA),
        "b" => Some(Key::KeyB),
        "c" => Some(Key::KeyC),
        "d" => Some(Key::KeyD),
        "e" => Some(Key::KeyE),
        "f" => Some(Key::KeyF),
        "g" => Some(Key::KeyG),
        "h" => Some(Key::KeyH),
        "i" => Some(Key::KeyI),
        "j" => Some(Key::KeyJ),
        "k" => Some(Key::KeyK),
        "l" => Some(Key::KeyL),
        "m" => Some(Key::KeyM),
        "n" => Some(Key::KeyN),
        "o" => Some(Key::KeyO),
        "p" => Some(Key::KeyP),
        "q" => Some(Key::KeyQ),
        "r" => Some(Key::KeyR),
        "s" => Some(Key::KeyS),
        "t" => Some(Key::KeyT),
        "u" => Some(Key::KeyU),
        "v" => Some(Key::KeyV),
        "w" => Some(Key::KeyW),
        "x" => Some(Key::KeyX),
        "y" => Some(Key::KeyY),
        "z" => Some(Key::KeyZ),
        "0" => Some(Key::Num0),
        "1" => Some(Key::Num1),
        "2" => Some(Key::Num2),
        "3" => Some(Key::Num3),
        "4" => Some(Key::Num4),
        "5" => Some(Key::Num5),
        "6" => Some(Key::Num6),
        "7" => Some(Key::Num7),
        "8" => Some(Key::Num8),
        "9" => Some(Key::Num9),
        _ => None,
    }
}

impl Hotkey {
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts = s.split('+');
        let mut ctrl = false;
        let mut shift = false;
        let mut alt = false;
        let mut meta = false;
        let mut target_key = None;

        for part in parts {
            let p = part.trim().to_lowercase();
            match p.as_str() {
                "ctrl" | "control" => ctrl = true,
                "shift" => shift = true,
                "alt" => alt = true,
                "meta" | "super" | "win" | "cmd" => meta = true,
                other => {
                    if let Some(k) = parse_key(other) {
                        target_key = Some(k);
                    } else {
                        return Err(format!("Unknown key in hotkey: {}", other));
                    }
                }
            }
        }

        let key = target_key.ok_or_else(|| "No target key specified in hotkey (e.g. 'v')".to_string())?;

        Ok(Hotkey {
            ctrl,
            shift,
            alt,
            meta,
            key,
        })
    }
}

pub fn simulate_typing(text: &str) -> Result<(), String> {
    // Wait a brief moment for the user to release the physical hotkeys
    // so they do not interfere with simulated input modifiers.
    thread::sleep(Duration::from_millis(150));

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to initialize keyboard simulator: {:?}", e))?;

    if let Err(e) = enigo.text(text) {
        println!("Direct typing failed ({:?}). Retrying via Clipboard Paste (Ctrl+V)...", e);
        
        // Write the path text to the clipboard
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|err| format!("Failed to open clipboard for fallback: {}", err))?;
            
        clipboard.set_text(text.to_string())
            .map_err(|err| format!("Failed to set clipboard text: {}", err))?;

        // Simulate Ctrl+V pasting (or Cmd+V on macOS)
        #[cfg(target_os = "macos")]
        {
            use enigo::Direction::{Click, Press, Release};
            enigo.key(enigo::Key::Meta, Press).map_err(|err| err.to_string())?;
            enigo.key(enigo::Key::Unicode('v'), Click).map_err(|err| err.to_string())?;
            enigo.key(enigo::Key::Meta, Release).map_err(|err| err.to_string())?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            use enigo::Direction::{Click, Press, Release};
            enigo.key(enigo::Key::Control, Press).map_err(|err| err.to_string())?;
            enigo.key(enigo::Key::Unicode('v'), Click).map_err(|err| err.to_string())?;
            enigo.key(enigo::Key::Control, Release).map_err(|err| err.to_string())?;
        }
    }

    Ok(())
}

pub fn listen_for_hotkey<F>(config: &Config, callback: F) -> Result<(), String>
where
    F: FnMut() + Send + 'static,
{
    let target_hotkey = Hotkey::parse(&config.hotkey)?;
    println!("Listening for hotkey: {:?}", target_hotkey);

    let modifier_state = Arc::new(Mutex::new(ModifierState::default()));
    let callback_mutex = Arc::new(Mutex::new(callback));

    let handler = move |event: Event| {
        let mut state = modifier_state.lock().unwrap();
        match event.event_type {
            EventType::KeyPress(key) => {
                match key {
                    Key::ControlLeft | Key::ControlRight => state.ctrl = true,
                    Key::ShiftLeft | Key::ShiftRight => state.shift = true,
                    Key::Alt | Key::AltGr => state.alt = true,
                    Key::MetaLeft | Key::MetaRight => state.meta = true,
                    k => {
                        if k == target_hotkey.key
                            && state.ctrl == target_hotkey.ctrl
                            && state.shift == target_hotkey.shift
                            && state.alt == target_hotkey.alt
                            && state.meta == target_hotkey.meta
                        {
                            // Trigger the paste logic
                            let mut cb = callback_mutex.lock().unwrap();
                            cb();
                        }
                    }
                }
            }
            EventType::KeyRelease(key) => {
                match key {
                    Key::ControlLeft | Key::ControlRight => state.ctrl = false,
                    Key::ShiftLeft | Key::ShiftRight => state.shift = false,
                    Key::Alt | Key::AltGr => state.alt = false,
                    Key::MetaLeft | Key::MetaRight => state.meta = false,
                    _ => {}
                }
            }
            _ => {}
        }
    };

    listen(handler).map_err(|e| format!("rdev listener error: {:?}", e))
}
