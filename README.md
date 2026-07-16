# img2cli

English | [简体中文](./README_zh.md)

Paste screenshots into any AI CLI as a Markdown image link — **without losing the image from your clipboard.**

`img2cli` is a cross-platform **system-tray desktop app** (Rust + Tauri + Vue 3) built for multimodal AI workflows. Take a screenshot, focus your terminal, press **Alt+V**, and the Markdown path to that image is typed into your terminal — while the image itself stays in your clipboard, so you can still paste it into WeChat / Word / Slack with **Ctrl+V**.

## Why

Multimodal AI CLIs (Claude Code, Cursor, Gemini CLI, …) running over SSH can't accept a pasted image — they need a **text path**. img2cli bridges that gap: it reads the screenshot from your clipboard, uploads it to the server your terminal is connected to (or saves it locally), and injects the path as `![image](...)`.

It's **clipboard-preserving**: in `direct` injection mode it never touches your clipboard, so the same screenshot can go to chat apps (Ctrl+V — the image) **and** to the terminal (Alt+V — the path).

## Download

Get the latest build from [GitHub Releases](https://github.com/zijunmeng/img2cli/releases):

| OS | Asset |
|---|---|
| Windows (installer) | `img2cli_0.3.0_x64-setup.exe` / `img2cli_0.3.0_x64_en-US.msi` |
| Windows (portable, no install) | `img2cli-v0.3.0-windows-portable.zip` |
| macOS (universal) | `img2cli_0.3.0_universal.dmg` |
| Linux | `img2cli_0.3.0_amd64.deb` / `.rpm` / `.AppImage` |

> ⚠️ The binaries are currently **unsigned**. On first launch, Windows SmartScreen (and some antiviruses like 360) may warn — click *More info → Run anyway*, or add the app to your trust list. Code-signing is on the roadmap.

## How it works

1. Copy a screenshot to the clipboard (Win+Shift+S, macOS screenshot, etc.).
2. Focus the terminal where your AI CLI runs.
3. Press **Alt+V**.
4. img2cli captures + compresses the image, uploads it to the matched server (creating the remote folder if needed), and **types** `![image](/remote/path.jpg)` into the terminal.

## Features

- **System-tray resident** with a glassmorphism settings dashboard (double-click the tray icon).
- **Cross-terminal auto-routing** — detects the SSH host from the active window title via your `~/.ssh/config` and uploads there automatically. Works across VS Code, Xshell, MobaXterm, PuTTY, Windows Terminal, … with no manual pattern needed.
- **Password OR key login** — passwords are stored in the **OS keyring** (Windows Credential Manager / macOS Keychain / Linux Secret Service), Xshell-style; key-based servers keep using your SSH keys.
- **Load OpenSSH config** — import hosts from `~/.ssh/config` (or any file, via the Browse… picker) into your router targets.
- **Clipboard-preserving injection** — `direct` (native keystroke, default) or `swap` (quick clipboard swap) modes.
- **Configurable** — output format (Markdown / HTML / raw / base64), compression quality, max dimension, hotkey, launch-on-boot, notifications.
- **Windows:** "Restart as Administrator" tray option to inject into terminals that run elevated (UIPI).

### Routing priority

When you press Alt+V, the upload target is resolved in this order:

1. **Manual router targets** (explicit `match_pattern` matches the window title)
2. **ssh-config auto-detect** (the title contains a host alias/hostname from `~/.ssh/config`)
3. **Default SSH host** (if enabled)
4. **Local temp path** (fallback)

## SSH password security

Passwords are **never written to the config file**. They're stored encrypted in your OS keyring, keyed per host (`user@host:port`), and only your OS user can read them — like Xshell, not plaintext. Key-based servers need no password at all.

## Platform notes

- **Windows:** fully supported (auto-routing by window title, restart-as-admin).
- **macOS:** grant **Accessibility** permission (for the global hotkey, text injection, and reading the window title).
- **Linux:** X11 only (needs `xdotool`). Wayland compositors don't expose other windows' titles, so auto-routing falls back to the default host / local path.

## Configuration

Settings are edited in the GUI and stored at:

- Windows: `%APPDATA%\img2cli\config.toml`
- macOS / Linux: `~/.config/img2cli/config.toml`

## Build from source

Requires Node.js, Rust, and the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
npm install
npm run tauri dev      # run in development
npm run tauri build    # produce installers / portable build
```
