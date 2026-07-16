# Specification: img2cli v2.0 - Tauri GUI & Clipboard-Free Text Injection

## 1. Goal

Provide a premium, double-click-to-run desktop application for `img2cli` using **Tauri v2** to eliminate manual console compilation and configuration. The program runs natively in the **System Tray** (like Snipaste/OneDrive), manages SSH host profiles visually (with a "Test Connection" tool), and supports a **dedicated global shortcut** (e.g., `Alt + V`) to paste Markdown links into the active terminal using **direct OS-level text injection** (leaving the system clipboard untouched so images can still be pasted into WeChat/Word).

---

## 2. Architecture & Components

The application is structured into a lightweight frontend (WebView container) and a native Rust backend running background daemon workers.

```mermaid
graph TD
    subgraph Frontend (Tauri WebView)
        UI[HTML/CSS/JS Settings Window]
        LogView[Logs & History Tab]
        HostMgr[Host CRUD Manager]
    end

    subgraph Backend (Tauri Rust Core)
        Config[Config Manager: config.toml]
        Tray[System Tray Service]
        Hotkey[Tauri Global Shortcut Plugin]
        Worker[Background Listener Thread]
        Injector[OS-Level Text Injector]
    end

    UI <-->|Tauri IPC Commands / Events| Backend
    Hotkey -->|On Trigger| Worker
    Worker -->|1. Capture Clipboard Image| Clipboard[System Clipboard]
    Worker -->|2. Compress & Upload| SSH[Remote Server via SCP]
    Worker -->|3. Get Markdown Link| Injector
    Injector -->|4. Send Input Events| Terminal[Active Window / Focus Target]
```

### 2.1 Backend Daemon (Rust)
*   **Window Manager & System Tray**: Handles tray icon creation, minimize-to-tray window hide/show toggles, and context menus (Lock to Server, Auto Route, Pause/Resume, Exit).
*   **Background Worker Thread**: Monitors the system clipboard for new images, runs JPEG compression, and handles SFTP/SCP uploads.
*   **Tauri Global Shortcut Plugin**: Registers the custom global hotkey configured by the user.
*   **OS-Level Text Injector**:
    *   **Windows**: Calls the Win32 `SendInput` API using the `KEYEVENTF_UNICODE` flag to type Unicode text directly into the active window. Bypasses character-by-character typing lag and IME state conflicts.
    *   **macOS**: Uses the AppleScript keystroke API or CoreGraphics events to input text directly to the focused process.

### 2.2 Frontend Settings GUI (HTML/CSS/JS)
*   **Aesthetics**: Premium modern dark-theme with glassmorphic components, subtle orange highlight borders, and smooth transition animations.
*   **Host Manager (CRUD)**: Visual tables to manage targets with a dedicated "Test Connection" button calling backend SSH validation.
*   **Hotkey Recorder**: Custom key recorder component capturing key combinations (Ctrl, Alt, Shift, keys) to save configuration values.
*   **Real-time Logs Panel**: Automatically streams terminal-like logs from backend stderr/stdout outputs using Tauri event listeners.

---

## 3. Configuration Schema (`config.toml`)

The configuration file is expanded to support GUI and hotkey preferences.

```toml
# Main settings
output_format = "markdown"             # markdown, html, raw, or base64
compress_quality = 80                  # 0-100
max_dimension = 1024                   # pixel width/height constraint
wrap_single_quotes = true              # Prevent history expansion in Bash
launch_on_boot = true                  # Autostart on system launch
enable_notifications = true            # System desktop notifications toggle
global_hotkey = "Alt+V"                # Customized capture-and-paste hotkey

# Default SSH destination config
[ssh]
enabled = false
host = "your_ssh_alias_or_ip"
port = 22
username = "your_username"
remote_dir = "/tmp/img2cli"
match_pattern = ""

# Auto Route target servers list
[[ssh_targets]]
enabled = true
match_pattern = "S90"
host = "172.16.190.90"
port = 22
username = "user"
remote_dir = "/s1/SHARE/user/tmp/img2cli"
```

---

## 4. Key User Flows

### 4.1 First Launch & System Tray Resident Flow
1.  User downloads `img2cli-windows.exe` and double-clicks to execute.
2.  The program starts in the background and places a custom icon in the Windows System Tray.
3.  A system notification alerts: *"img2cli is running in the background. Press Alt+V to paste screenshot Markdown links."*
4.  Double-clicking the tray icon (or selecting "Open Settings" in the tray context menu) displays the main GUI Configuration Window.
5.  Closing the configuration window hides it back to the system tray instead of exiting.

### 4.2 Adding & Testing an SSH Target
1.  User opens the "Hosts" tab in the GUI and clicks **`+ Add New Server`**.
2.  An overlay dialog prompts for Name, Hostname, Username, Port, and Target Folder.
3.  User clicks **`Test Connection`**.
4.  Tauri backend runs a fast SSH key check and connection attempt:
    *   *Success*: Display a green checkmark next to the test button.
    *   *Failure*: Display a red error tooltip detailing the SSH connection error (e.g. *Timeout*, *Auth Failure*).
5.  Clicking `Save` persists the target into `config.toml` and updates the running router state.

### 4.3 Upload & Direct Text Injection Flow (WeChat-Compatible)
1.  User copies an image to their clipboard (e.g., using Snipaste/PrintScreen).
2.  **Paste to WeChat**: User presses `Ctrl + V` in WeChat. The original image pastes normally.
3.  **Paste to Terminal**: User moves focus to their terminal window and presses the custom shortcut **`Alt + V`**.
4.  `img2cli` intercepts `Alt + V`, takes the image from the clipboard, compresses it, and uploads it to the active SSH server (matched by active window title auto-routing).
5.  `img2cli` generates the markdown path (e.g., `'![image](/s1/SHARE/...)'`).
6.  The backend calls `SendInput` (Windows) to instantly inject the Markdown text directly into the cursor focus of the active terminal.
7.  **Result**: The Markdown text is pasted to the terminal, and the original image remains in the system clipboard (allowing the user to paste it into WeChat at any time).

---

## 5. Security & System Integration

*   **Autostart Integration**: Autostart is registered via the OS startup registry (Windows) or LaunchAgents (macOS).
*   **Accessibility Permissions (macOS)**: AppleScript text injection and global hotkey capture require macOS Accessibility permissions. If not authorized, the app prompts a friendly popup guide to opening System Preferences.
*   **Wayland Compatibility (Linux)**: Direct keyboard emulation has limited support in Wayland. TUI mode or custom X11 fallbacks will be provided for Linux desktop users.
