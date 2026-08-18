# img2cli

English | [简体中文](./README_zh.md)

**Paste screenshots into any AI CLI as a Markdown image link — without losing the image from your clipboard.**

`img2cli` is a cross-platform **system-tray desktop app** (Rust + Tauri v2 + Vue 3) built for multimodal AI workflows. Take a screenshot with the **built-in Snipaste-style capture** (annotate, pin, copy, or upload), and the image is uploaded to your server in the background — focus your terminal, press the **inject hotkey**, and the Markdown path is injected. The image itself stays in your clipboard so you can still paste it into WeChat / Word / Slack with **Ctrl+V**.

## Download

| OS | Asset | Notes |
|---|---|---|
| **Windows** (installer) | `img2cli_0.4.8_x64-setup.exe` / `_x64_en-US.msi` | |
| **Windows** (portable) | `img2cli-v0.4.8-windows-portable.zip` | no install, runs from anywhere |
| **macOS** (universal) | `img2cli_0.4.8_universal.dmg` | M1/M2/M3 + Intel |
| **Linux** | `img2cli_0.4.8_amd64.deb` / `.rpm` / `.AppImage` | screenshot capture unavailable (see notes) |

→ **[GitHub Releases](https://github.com/zijunmeng/img2cli/releases)**

> ⚠️ **Unsigned binaries.** On first launch:
> - **Windows:** SmartScreen → *More info → Run anyway*; or add to antivirus trust list.
> - **macOS:** Right-click `img2cli.app` → *Open* → confirm. Then grant **Accessibility** + **Screen Recording** permissions in System Settings → Privacy & Security.

---

## Quick Start

1. **Install** — download the asset for your OS, install/drag to Applications.
2. **Screenshot** — press **Alt+Shift+S** (configurable; the screen freezes instantly):
   - **Drag** to select a region, or **click** a window to snap it, or **Tab** to cycle detected windows/elements.
   - **Annotate** — arrow / pen / marker / mosaic / text / rectangle / ellipse, with undo-redo, stroke-splitting eraser.
   - **Act** — ⬆ upload+inject later · 📋 / **Ctrl+C** copy the image · 💾 save to file · 📌 pin to screen.
   - Confirming the selection (✓ / Enter) unlocks move/resize; a second Enter uploads.
3. The crop goes to your clipboard **and** uploads in the background — no waiting.
4. **Paste path to terminal** — focus your AI CLI, press **Alt+V** (inject hotkey). The Markdown path `![image](/remote/path.jpg)` is injected; if the image is unchanged, the already-uploaded path is reused instantly.
5. **Paste image to chat** — press **Ctrl+V** in WeChat / Word. The original image is still in your clipboard.

---

## Features

### 📸 Built-in Screenshot Capture (Snipaste-style)
- **Screenshot Hotkey** (default `Alt+Shift+S`) with a **persistently warm overlay** — the screen is frozen to memory *before* the overlay appears; the frame ships to the UI as a compact JPEG so the overlay appears with no perceptible lag.
- **Window auto-detection** — outlines the window under the cursor; **Tab / Shift+Tab** cycles windows *and child elements* (buttons, editors) for precise snapping. Click = snap, drag = free region, both decided on mouse-up.
- **Explicit confirm model** — a selection is *not* a commitment: keep redrawing freely until you click ✓ (then drag inside moves it). Enter = confirm, then upload.
- **Region memory** — **Shift+R** recalls the last region; `,` / `.` cycle the last 8 regions. **WASD** nudges the cursor 1px.
- **Capture options** — border width, mask opacity, hint panel and window detection toggles.

### ✏️ Annotation Editor
- Tools: **arrow, pen, marker (multiply blend), mosaic, text, rectangle, ellipse** + color palette and thickness.
- **Undo / redo** (Ctrl+Z / Ctrl+Y), and a **stroke-splitting eraser** that erases pen/marker strokes segment-by-segment (Snipaste behavior) instead of deleting whole objects.
- Annotations render into the final crop at **full physical resolution** and flow through the whole pipeline (clipboard, history, upload, pin).

### 📌 Pin to Screen (贴图)
- Pin any (annotated) crop as a borderless always-on-top window.
- Drag to move, **wheel to zoom**, **right-click menu** (copy image / save as / destroy), double-click to close.

### 🔄 Clipboard-Preserving Injection
- **`auto` mode** (default): per-app host policy — keystroke injection where it works, clipboard mode for apps that reject synthetic input (e.g. Orca).
- **`direct`**: native Unicode keystroke injection via [Enigo](https://crates.io/crates/enigo); never touches the clipboard.
- **`copy`**: writes the path to the clipboard and simulates paste — for hosts that reject synthetic keystrokes.
- **Capture-then-upload**: confirming a region starts the SFTP upload in the background immediately; the inject hotkey pastes the delivered path instantly (fingerprint fast-path skips re-upload when the clipboard still holds the same image).

### 🔐 SSH Password + Key Login
- **Password auth**: stored in the **OS keyring** (Windows Credential Manager / macOS Keychain / Linux Secret Service) — never in config. Keyed per host (`user@host:port`).
- **Key auth**: system `ssh`/`scp` with your default SSH keys.
- **known_hosts**: new hosts are remembered on first connect (TOFU) and verified afterwards.

### 🌐 Cross-Terminal Auto-Routing
When the inject hotkey fires, the upload target is resolved in priority order:
1. **Router targets** — explicit `match_pattern` matches the active window title (or process name).
2. **ssh-config auto-detect** — the title contains a host alias/hostname from `~/.ssh/config`.
3. **Default SSH host** — the target card flagged as default.
4. **Local temp path** — fallback (no upload).

Works across **VS Code, Xshell, MobaXterm, PuTTY, Windows Terminal**, and more.

### 🔑 Hosts & Targets Management
- Orca-style **card list** — one card per target, a **Default** badge on exactly one, enable/disable per card, per-target connection test.
- **Load OpenSSH Config** — import hosts from `~/.ssh/config` (search, multi-select, dedupe).

### 🎨 UI
- **6 themes** (`dracula` default, `apple-dark`/`apple-light`, `nord`, `gruvbox`, `cyberpunk`), every surface adapts via CSS variables.
- **中文 / English interface** switch.
- Resizable main window, single-instance (second launch focuses the running app).
- **System Logs** panel with copy-all / export / clear — and every daemon event is mirrored to `%TEMP%\img2cli\daemon.log` so even a frozen app leaves a readable trail.
- **Press-to-record hotkeys** with blacklist validation (won't let you save combos that break the system, like Ctrl+C or Alt+F4). Works under CJK IMEs (physical-key matching + key-up fallback).

### 🖥️ System Tray
- Runs in the system tray; left-click opens Settings; closing the window hides to tray.
- **Windows:** "Restart as Administrator" tray option (for injecting into elevated terminals / UIPI).

### 📋 Configurable
- **Output format**: Markdown `![image](path)` / HTML `<img>` / raw path / inline Base64.
- **Compression**: JPEG quality (10–100), max dimension (auto-resize).
- **Wrap in quotes**, **auto-cleanup** of old screenshots, **launch on boot**, **desktop notifications**.

---

## How It Works

```
┌──────────────┐     ┌───────────────┐     ┌──────────────┐     ┌──────────────┐
│  Screenshot   │────▶│   Compress    │────▶│   Upload     │────▶│   Inject     │
│  (xcap/clip)  │     │  (JPEG)       │     │  (SFTP/SCP,  │     │  (on demand: │
│  + annotate   │     │               │     │  background) │     │  hotkey)     │
└──────────────┘     └───────────────┘     └──────────────┘     └──────────────┘
        │                                                               ▲
        └── crop → clipboard (image stays for Ctrl+V) ───────────────────┘
                                        route by title/process: ① target ② ssh-config ③ default ④ local
```

1. **Capture** — region screenshot (freeze-frame overlay) or clipboard image.
2. **Upload (background)** — the confirmed region is compressed and pushed via SFTP/SCP immediately; the delivered path is cached.
3. **Route** — active window title + process name → target → upload destination (same routing for the background upload).
4. **Inject (on demand)** — focus an AI CLI, press the inject hotkey: the Markdown path is typed/pasted. If the clipboard still holds the same image, the cached path is injected with no re-upload.

---

## Configuration

Settings are edited in the GUI and stored at:
- **Windows:** `%APPDATA%\img2cli\config.toml`
- **macOS / Linux:** `~/.config/img2cli/config.toml`

### Key settings

| Setting | Default | Description |
|---|---|---|
| `output_format` | `"markdown"` | `markdown` / `html` / `raw` / `base64` |
| `compress_quality` | `80` | JPEG quality (10–100) |
| `max_dimension` | `1024` | Max width/height in pixels |
| `wrap_single_quotes` | `false` | Wrap output in `'...'` |
| `global_hotkey` | `"Alt+V"` | Inject hotkey |
| `screenshot_hotkey` | `"Alt+Shift+S"` | Screenshot region capture hotkey |
| `injection_mode` | `"auto"` | `auto` (per-app policy) / `direct` (keystroke) / `copy` (clipboard+paste) |
| `theme` | `"dracula"` | UI theme |
| `language` | `"zh-CN"` | `zh-CN` / `en` |
| `capture_border_width` | `2` | Selection border (px) |
| `capture_mask_opacity` | `45` | Outside-selection dim (%) |
| `clean_keep_days` | `1` | Auto-delete screenshots older than N days |
| `launch_on_boot` | `true` | Start with the OS |

### Router targets

```toml
[[targets]]
enabled = true
type = "ssh"                    # "ssh" or "local"
match_pattern = "91_mengzijun"  # matches window title / process
host = "172.16.190.96"
port = 7525
username = "mengzijun"
remote_dir = "/tmp/img2cli"
is_default = true               # exactly one card carries the Default badge
```

---

## Platform Notes

| Feature | Windows | macOS | Linux |
|---|---|---|---|
| **Inject hotkey** | ✅ Full | ✅ Full (needs Accessibility) | ✅ X11 (Wayland limited) |
| **Screenshot capture** | ✅ Full | ✅ Full (needs Screen Recording) | ❌ Disabled (xcap/PipeWire incompat) |
| **Window-title routing** | ✅ Win32 | ✅ Accessibility API | ✅ X11 (Wayland: fallback) |
| **Restart as Admin** | ✅ | N/A | N/A |
| **Portable zip** | ✅ | N/A | N/A |

**macOS permissions required:**
- **Accessibility** — for global hotkeys + text injection (Enigo).
- **Screen Recording** — for screenshot capture (xcap).

---

## Troubleshooting

- **Screenshot hotkey does nothing** — open Settings → System Logs:
  - `Failed to register screenshot shortcut ... Another instance/app may be using it` → another app (or a zombie img2cli instance) holds the combo. Quit all img2cli instances (tray → Exit, or Task Manager) and start one.
  - `Screenshot hotkey received` lines appear but no overlay → **press the hotkey again**: the warm overlay webview is detected dead and rebuilt on the next press (at most one rebuild per keypress).
- **App frozen / UI dead** — the daemon log survives: `%TEMP%\img2cli\daemon.log` (`C:\Users\<you>\AppData\Local\Temp\img2cli\daemon.log`). The last lines show exactly where the pipeline stopped.
- **Webviews misbehave after force-killing the app** — delete `%LOCALAPPDATA%\com.img2cli.app` (WebView2 profile; it regenerates; your config is elsewhere) and restart.
- **Hotkey recorder won't take a letter under a CJK IME** — release the key: the key-up event finalizes the combo. Still failing → switch the IME to EN while recording.
- **Path pasted instead of what you copied** — the inject hotkey always injects the screenshot path; use **Ctrl+V** for manual clipboard pastes.

---

## Build from Source

### Prerequisites
- [Node.js](https://nodejs.org/) (LTS)
- [Rust](https://rustup.rs/) (stable)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)

### Build & Run
```bash
git clone https://github.com/zijunmeng/img2cli.git
cd img2cli
npm install
npm run tauri dev      # development (hot reload)
npm run tauri build    # production build → src-tauri/target/release/bundle/
```

### Tech Stack
- **Backend:** Rust, Tauri v2, russh, xcap, enigo, arboard, keyring, image
- **Frontend:** Vue 3, Vite, Tailwind CSS
- **CI/CD:** GitHub Actions + tauri-action (3-platform check + release)

---

## Architecture

```
src-tauri/src/
├── main.rs         App entry: tray, hotkeys (+ blacklist validation), IPC commands, window setup
├── config.rs       Configuration: AppConfig, TargetConfig, InjectionMode (TOML)
├── job.rs          JobManager + worker: capture → route → deliver → inject, background uploads
├── routing.rs      RouteResolver chain: targets → ssh-config → default SSH → local
├── host_policy.rs  Per-app injection policy (title/process → direct/copy)
├── transport.rs    ArtifactTransport: SFTP/SCP/local delivery + auth dispatch
├── cli_adapter.rs  CliAdapter: render delivered path → Markdown / HTML / raw
├── daemon.rs       DaemonState, logging (panel + daemon.log mirror), SCP engine
├── clipboard.rs    Clipboard capture, data-URL decode, image processing
├── injector.rs     Injection: direct (Enigo) / copy (clipboard + paste)
├── ssh.rs          SSH client: russh SFTP (timeouts, TOFU known_hosts), keyring, pooling
├── ssh_config.rs   OpenSSH config parser (~/.ssh/config)
└── capture.rs      Region capture: xcap freeze-frame + warm overlay + window detection

src/                 Vue 3 frontend
├── App.vue         Settings dashboard + capture overlay (annotation engine) + pin windows
├── strings.js      zh-CN / en localization
├── main.js         Vue app bootstrap
└── index.css       Tailwind + custom styles
```

---

## Known Issues

- **Unsigned binaries** → SmartScreen (Windows) / Gatekeeper (macOS) warnings. Code-signing is planned (v1.0.0).
- **Warm overlay webview can die** on some machines (WebView2/driver dependent). Press the screenshot hotkey again — v0.4.8 detects the dead overlay and rebuilds it on the next press. Under investigation.
- **Apps that reject synthetic input** (e.g. Orca) — handled automatically in `auto` mode (routed to `copy`); use Ctrl+V there.
- **UIPI** — can't inject into elevated terminals (Windows). Fix: "Restart as Administrator" tray option.
- **Linux screenshot** — disabled (xcap's PipeWire/libspa backend incompatible with older distros).

See [KNOWN_ISSUES.md](./KNOWN_ISSUES.md) for the full list.

---

## Roadmap

- [x] Annotation editor (arrows, pen, marker, mosaic, text, shapes, eraser)
- [x] Screen pinning (贴图)
- [x] Background upload + fast-path inject
- [ ] Multi-monitor capture
- [ ] SSH keep-alive pool (<200ms uploads)
- [ ] Local OCR & code-block extraction
- [ ] Scrolling capture (长截屏)
- [ ] Code-signing (Windows + macOS)

See [ROADMAP.md](./ROADMAP.md) for details.

---

## Acknowledgments

- [Tauri](https://tauri.app/) — the app framework
- [russh](https://crates.io/crates/russh) — pure-Rust SSH client
- [xcap](https://crates.io/crates/xcap) — cross-platform screen capture
- [enigo](https://crates.io/crates/enigo) — input simulation
- [arboard](https://crates.io/crates/arboard) — clipboard access
- [keyring](https://crates.io/crates/keyring) — OS credential storage
- [wispterm](https://github.com/nicepkg/wispterm) — design inspiration for portable packaging & freeze-frame capture
- [Snipaste](https://www.snipaste.com/) / [ShareX](https://getsharex.com/) / [Flameshot](https://flameshot.org/) — interaction references for capture, pinning and annotation

---

## License

MIT
