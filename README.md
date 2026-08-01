# img2cli

English | [简体中文](./README_zh.md)

**Paste screenshots into any AI CLI as a Markdown image link — without losing the image from your clipboard.**

`img2cli` is a cross-platform **system-tray desktop app** (Rust + Tauri v2 + Vue 3) built for multimodal AI workflows. Take a screenshot — either with the **built-in region capture** or your OS tool — focus your terminal, press **Alt+V**, and the Markdown path to that image is injected directly. The image itself stays in your clipboard so you can still paste it into WeChat / Word / Slack with **Ctrl+V**.

## Download

| OS | Asset | Notes |
|---|---|---|
| **Windows** (installer) | `img2cli_0.3.6_x64-setup.exe` / `_x64_en-US.msi` | |
| **Windows** (portable) | `img2cli-v0.3.6-windows-portable.zip` | no install, runs from anywhere |
| **macOS** (universal) | `img2cli_0.3.6_universal.dmg` | M1/M2/M3 + Intel |
| **Linux** | `img2cli_0.3.6_amd64.deb` / `.rpm` / `.AppImage` | screenshot capture unavailable (see notes) |

→ **[GitHub Releases](https://github.com/zijunmeng/img2cli/releases)**

> ⚠️ **Unsigned binaries.** On first launch:
> - **Windows:** SmartScreen → *More info → Run anyway*; or add to antivirus trust list.
> - **macOS:** Right-click `img2cli.app` → *Open* → confirm. Then grant **Accessibility** + **Screen Recording** permissions in System Settings → Privacy & Security.

---

## Quick Start

1. **Install** — download the asset for your OS, install/drag to Applications.
2. **Screenshot** — press **Alt+Shift+S** (built-in region capture: drag to select), or use your OS screenshot tool (Win+Shift+S / macOS screenshot).
3. **Paste to terminal** — focus your terminal / AI CLI, press **Alt+V**. The Markdown path `![image](/remote/path.jpg)` is injected.
4. **Paste image to chat** — press **Ctrl+V** in WeChat / Word. The original image is still in your clipboard (injection never touches it in `direct` mode).

---

## Features

### 📸 Built-in Screenshot Capture
- Dedicated **Screenshot Hotkey** (default `Alt+Shift+S`) opens a fullscreen overlay.
- **Freeze-frame** capture: screen is grabbed to memory *before* the overlay opens (zero flicker), then the overlay shows the frozen image for region selection.
- **Drag-to-select** a region (Snipaste-style). Release → img2cli crops, compresses, and runs the upload+inject pipeline.
- Powered by [`xcap`](https://crates.io/crates/xcap) (Windows / macOS).

### 🔄 Clipboard-Preserving Injection
- **`direct` mode** (default): native Unicode keystroke injection via [Enigo](https://crates.io/crates/enigo). Bypasses IME. Never touches the clipboard.
- **`swap` mode**: backs up clipboard → writes path → simulates Ctrl+V → restores clipboard. Use this if `direct` drops characters (Chinese IME interference).

### 🔐 SSH Password + Key Login
- **Password auth**: stored in the **OS keyring** (Windows Credential Manager / macOS Keychain / Linux Secret Service) — Xshell-style, never in config. Keyed per host (`user@host:port`).
- **Key auth**: uses system `ssh`/`scp` with your default SSH keys (no password needed).
- **Routing**: upload target is resolved automatically (see below).
- **Connection pooling**: cached russh handle (keep-alive) to reduce per-capture SFTP latency.

### 🌐 Cross-Terminal Auto-Routing
When you press Alt+V, the upload target is resolved in priority order:
1. **Manual router targets** — explicit `match_pattern` matches the active window title.
2. **ssh-config auto-detect** — the title contains a host alias/hostname from `~/.ssh/config`.
3. **Default SSH host** — if enabled.
4. **Local temp path** — fallback (no upload).

Works across **VS Code, Xshell, MobaXterm, PuTTY, Windows Terminal**, and more.

### 🔑 Load OpenSSH Config
- Import hosts from `~/.ssh/config` (or any file via the **Browse…** picker) into your router targets.
- Parser handles aliases, HostName, User, Port; skips wildcard hosts.

### 🎨 Theme System (6 Themes)
- `apple-dark` (default), `apple-light`, `dracula`, `nord`, `gruvbox`, `cyberpunk`.
- Compact `<select>` picker with an accent-color swatch.
- Every surface (canvas, sidebar, cards, inputs, toggles, buttons, table, logs) adapts via CSS variables.

### ⌨️ Press-to-Record Hotkeys
- Click the hotkey field, press your key combo — no text typing.
- Two configurable hotkeys: **Paste** (default `Alt+V`) and **Screenshot** (default `Alt+Shift+S`).

### 🖥️ System Tray
- Runs in the system tray (like Snipaste / OneDrive).
- Double-click the tray icon to open Settings.
- Closing the Settings window hides it to tray (doesn't quit).
- **Windows:** "Restart as Administrator" tray option (for injecting into elevated terminals / UIPI).

### 📋 Configurable
- **Output format**: Markdown `![image](path)` / HTML `<img>` / raw path / inline Base64.
- **Compression**: JPEG quality (10–100), max dimension (auto-resize).
- **Wrap in quotes**: wrap the output in `'...'` (prevents Bash history expansion).
- **Auto-cleanup**: delete screenshots older than N days.
- **Launch on boot**, **desktop notifications**.

---

## How It Works

```
┌──────────────┐     ┌───────────────┐     ┌──────────────┐     ┌──────────────┐
│  Screenshot   │────▶│   Compress    │────▶│   Upload     │────▶│   Inject     │
│  (xcap/clip)  │     │  (JPEG, ≤1024)│     │  (SFTP/SCP)  │     │  (Enigo)     │
└──────────────┘     └───────────────┘     └──────────────┘     └──────────────┘
                                                     │
                    ┌────────────────────────────────┘
                    ▼
          ┌─────────────────┐
          │  Route by title  │
          │  ① Manual target │
          │  ② ssh-config    │
          │  ③ Default SSH   │
          │  ④ Local path    │
          └─────────────────┘
```

1. **Capture** — clipboard image (Alt+V) or region screenshot (Alt+Shift+S).
2. **Compress** — resize to max dimension, JPEG encode at configured quality.
3. **Route** — detect the active window title → match a target → determine upload destination.
4. **Upload** — SFTP (password/keyring via russh) or SCP (system ssh keys). Remote dir auto-created (`mkdir -p`).
5. **Inject** — type the Markdown path into the focused terminal via Enigo (direct) or clipboard swap.

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
| `wrap_single_quotes` | `true` | Wrap output in `'...'` |
| `global_hotkey` | `"Alt+V"` | Paste hotkey |
| `screenshot_hotkey` | `"Alt+Shift+S"` | Screenshot region capture hotkey |
| `injection_mode` | `"direct"` | `direct` (keystroke) / `swap` (clipboard) |
| `theme` | `"apple-dark"` | UI theme |
| `clean_keep_days` | `1` | Auto-delete screenshots older than N days |
| `launch_on_boot` | `true` | Start with the OS |

### SSH config

```toml
[ssh]
enabled = true
host = "172.16.190.96"
port = 7525
username = "mengzijun"
remote_dir = "/tmp/img2cli"
```

### Router targets

```toml
[[targets]]
enabled = true
type = "ssh"                # "ssh" or "local"
match_pattern = "91_mengzijun"  # matches window title
host = "172.16.190.96"
port = 7525
username = "mengzijun"
remote_dir = "/tmp/img2cli"
```

---

## Platform Notes

| Feature | Windows | macOS | Linux |
|---|---|---|---|
| **Paste (Alt+V)** | ✅ Full | ✅ Full (needs Accessibility) | ✅ X11 (Wayland limited) |
| **Screenshot capture** | ✅ Full | ✅ Full (needs Screen Recording) | ❌ Disabled (xcap/PipeWire incompat) |
| **Window-title routing** | ✅ Win32 | ✅ Accessibility API | ✅ X11 (Wayland: fallback) |
| **Restart as Admin** | ✅ | N/A | N/A |
| **Portable zip** | ✅ | N/A | N/A |

**macOS permissions required:**
- **Accessibility** — for global hotkeys + text injection (Enigo).
- **Screen Recording** — for screenshot capture (xcap).

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
- **Backend:** Rust, Tauri v2, russh, xcap, enigo, arboard, keyring
- **Frontend:** Vue 3, Vite, Tailwind CSS
- **CI/CD:** GitHub Actions + tauri-action

---

## Architecture

```
src-tauri/src/
├── main.rs         App entry: tray, hotkeys, IPC commands, window setup
├── config.rs       Configuration: AppConfig, SshConfig, TargetConfig (TOML)
├── job.rs          JobManager + worker: capture → route → deliver → inject (orchestrator)
├── routing.rs      RouteResolver chain: manual → ssh-config → default SSH → local
├── transport.rs    ArtifactTransport: SFTP/SCP/local delivery + auth dispatch
├── cli_adapter.rs  CliAdapter: render delivered path → Markdown / HTML / raw
├── daemon.rs       Daemon state, helpers, SCP upload engine
├── clipboard.rs    Clipboard capture + image processing (resize / compress)
├── injector.rs     Text injection: direct (Enigo) / swap (clipboard)
├── ssh.rs          SSH client: russh SFTP (timeouts, TOFU), keyring, pooling
├── ssh_config.rs   OpenSSH config parser (~/.ssh/config)
└── capture.rs      Screenshot region capture: xcap + freeze-frame overlay

src/                 Vue 3 frontend
├── App.vue         Settings dashboard + theme system + capture overlay
├── main.js          Vue app bootstrap
└── index.css        Tailwind + custom styles
```

---

## Known Issues

- **Unsigned binaries** → SmartScreen (Windows) / Gatekeeper (macOS) warnings. Code-signing is planned.
- **IME interference** — Chinese input method can eat the first characters of `direct` injection. Fix: switch to `swap` mode.
- **UIPI** — can't inject into elevated terminals (Windows). Fix: "Restart as Administrator" tray option.
- **Linux screenshot** — disabled (xcap's PipeWire/libspa backend incompatible with older distros).

See [KNOWN_ISSUES.md](./KNOWN_ISSUES.md) for the full list.

---

## Roadmap

- [ ] Code-signing (Windows + macOS) — fix SmartScreen / Gatekeeper
- [ ] Local OCR & code-block extraction
- [ ] Annotation overlays (arrows, highlights, blur)
- [ ] Screen pinning (贴图)

See [ROADMAP.md](./ROADMAP.md) for details.

---

## Acknowledgments

- [Tauri](https://tauri.app/) — the app framework
- [russh](https://crates.io/crates/russh) — pure-Rust SSH client
- [xcap](https://crates.io/crates/xcap) — cross-platform screen capture
- [enigo](https://crates.io/crates/enigo) — input simulation
- [keyring](https://crates.io/crates/keyring) — OS credential storage
- [wispterm](https://github.com/nicepkg/wispterm) — design inspiration for portable packaging & freeze-frame capture

---

## License

MIT
