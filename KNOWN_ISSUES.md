# Known Issues - img2cli

This file tracks current, observable defects, platform limitations, and security/UI boundaries. Future plans belong in [ROADMAP.md](ROADMAP.md); completed changes belong in git history.

---

## 1. Platform Support Matrix

| Area | Windows | macOS | Linux |
| --- | --- | --- | --- |
| **Release Status** | Primary supported target | Supported build, stabilizing | Experimental |
| **Global Keyboard Hooks** | Native Win32 Hooks | CoreGraphics event taps (Requires Accessibility) | X11/XRecord (Wayland requires DBus fallback) |
| **Active Window Title Routing** | Win32 Window Manager | Accessibility API (Requires Screen Recording) | X11/ActiveWindow (Wayland restricted) |
| **Text Injection Mode** | Enigo (Direct Unicode / Input Simulation) | Enigo (Requires Accessibility) | Enigo (Requires XTest extension) |
| **Keyring Security** | Windows Credential Manager | macOS Keychain | Secret Service / DBus |

---

## 2. Windows-Specific Issues

### A. Input Method Editor (IME) Interference
* **Symptom**: When `img2cli` types the remote image path into the terminal cursor, if the user's system active input method is in Chinese/non-English IME mode, the simulated keystrokes might trigger candidate word selection boxes rather than direct alphanumeric path characters.
* **Mitigation**: 
  1. Users can switch their settings to use **Clipboard Swap (Paste) Mode** which bypasses keyboard stroke simulation entirely by backing up the clipboard, executing `Ctrl+V`, and restoring the clipboard.
  2. Developers can enhance `injector.rs` to leverage Enigo's safe Unicode/text input mode where supported.

### B. Elevated Shells (UAC / Run as Administrator)
* **Symptom**: When the target focus terminal is running with elevated privileges (e.g. Administrator PowerShell or Command Prompt), `img2cli` keystrokes do not paste anything.
* **Root Cause**: Windows UIPI (User Interface Privilege Isolation) prevents standard-user processes from injecting inputs into higher-integrity process windows.
* **Solution**: The tray icon contains a `"Restart as Administrator"` action which prompts UAC elevation. Running `img2cli` as Administrator resolves this limitation.

---

## 3. macOS-Specific Issues

### A. System Permissions (Accessibility & Screen Recording)
* **Symptom**: Global keyboard shortcut presses or window title routing actions fail silently.
* **Root Cause**: macOS sandboxing requires explicit permissions:
  * **Accessibility (辅助功能)**: Required by `rdev` for global key hooks and `enigo` for clipboard/keystroke paste execution.
  * **Screen Recording (屏幕录制)**: Required by `xcap` to scrap screen frames for selection and to inspect active window titles.
* **Solution**: On launch, if permissions are missing, log warnings to the console. The user must manually enable these under System Settings -> Privacy & Security.

### B. Command Key vs Control Key Clipboard Mapping
* **Symptom**: Clipboard Swap Mode fails to paste on macOS.
* **Root Cause**: macOS uses `Cmd+V` (Command) to paste instead of `Ctrl+V` (Control).
* **Mitigation**: `injector.rs` must route keyboard keystroke commands based on target operating system compile flags (`target_os = "macos"`).

---

## 4. Linux-Specific Issues

### A. Wayland Security Restrictions
* **Symptom**: Blank screenshots or failed key recording inside GNOME/KDE Wayland sessions.
* **Root Cause**: Wayland blocks background screen capture and keyboard sniffing by design.
* **Workaround**: Currently, Wayland requires falling back to XWayland compatibility layers, or querying DBus Desktop Screenshot Portals (`org.freedesktop.portal.Screenshot`).

### B. PipeWire & libspa Gaps
* **Symptom**: Target Linux compilation or region capture crashes on older distros (like Ubuntu 22.04).
* **Root Cause**: The Rust screen capture crate `xcap` relies on active PipeWire headers which are missing or mismatched on legacy targets.

---

## 5. Cross-Platform Latency Gaps

### A. SFTP Connection Handshake Overhead
* **Symptom**: Up to 1-3 seconds lag between taking a screenshot and seeing the path pasted in the terminal.
* **Root Cause**: The SSH client negotiates authentication handshakes on *every* hotkey capture event.
* **Mitigation**: Connection pooling (Keep-Alive connection channel) is planned in [ROADMAP.md](ROADMAP.md).

### B. Webview Overlay Loading Latency
* **Symptom**: 50ms - 200ms screen freezing flicker when initiating region capture.
* **Root Cause**: Tauri Webview loads assets and initializes the HTML5 Vue engine after the hotkey is pressed.
* **Mitigation**: Rust-native pre-emptive screen freezing is planned in [ROADMAP.md](ROADMAP.md).

---

## 6. UI & Design System Alignment Gaps
* **Gradient Theme Mismatch**: The current GUI interface still uses orange-to-amber gradients (`bg-gradient-to-r from-orange-500 to-amber-500`) and custom styles that deviate from the pure Apple style defined in [DESIGN.md](DESIGN.md). Full visual style migration is scheduled as part of the roadmap.
