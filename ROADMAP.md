# Roadmap - img2cli

This file tracks future architecture improvements, user experience features, and optimization milestones for the `img2cli` desktop application and daemon. Completed milestones are tracked in release notes and git history.

---

## 1. Platform Direction & Build Strategy
`img2cli` aims to remain a highly performant, single-binary background daemon with a lightweight, lazy-loaded configuration interface.
* **Primary Target**: Windows (`x86_64-pc-windows-gnu` / setup installer and portable zip).
* **Secondary Targets**: macOS (Apple Silicon & Intel DMG) and Linux (Portable AppImage).
* **Guiding Principle**: Zero-dependency background operation with low memory overhead (<25MB idle).

---

## 2. Milestone 1: Performance & Latency Optimization (Short-Term)

### A. Pre-emptive Screen Freezing (Zero UI Flicker)
* **Goal**: Drop region capture startup lag from `100ms - 200ms` down to near `0ms`.
* **Action**:
  1. Capture display pixels directly inside the Rust backend handler (`rdev` callback) using low-level OS drawing/capture APIs (Windows GDI, macOS Quartz/CGDisplay).
  2. Cache the captured image buffer instantly in memory.
  3. Wake and launch the fullscreen Tauri Webview window asynchronously to render the crop handlers over the cached memory image.

### B. SSH Keep-Alive Connection Pooling
* **Goal**: Reduce SFTP upload latency from `1.5s` down to under `200ms`.
* **Action**:
  1. Maintain a single multiplexed connection channel to the active remote host.
  2. Implement a background thread heartbeat ping loop to keep the SSH channel warm.
  3. Send captured screenshot byte streams directly through the pre-existing warm channel, avoiding TCP/SSH handshakes at trigger time.

---

## 3. Milestone 2: AI-First Pipeline Enhancements (Mid-Term)

### A. Local OCR & Code Block Extraction
* **Goal**: Automatically convert screenshot images into copyable Markdown text code blocks to save up to 90% of model API tokens.
* **Action**:
  1. Link Rust bindings to native platform OCR engines (Windows OCR Runtime, macOS Vision API).
  2. Scan cropped region pixels for code structures and terminal outputs.
  3. If text/code is recognized, automatically format it as a markdown code block (`````language ... `````) and paste it directly to the active cursor instead of uploading a graphical image.

### B. Zero-Trust API Key & Password Masking
* **Goal**: Prevent accidental leaks of credentials (e.g. OpenAI `sk-...`, passwords, private IPs) to public AI models.
* **Action**:
  1. Perform real-time OCR checks on selection crop.
  2. Run regex matching against standard private key formats, internal server IPs, and database strings.
  3. Alert the user with a single-click "Mask/Blur" option directly on the crop UI.

---

## 4. Milestone 3: Editor Annotation Overlays (Long-Term)

### A. Inline Crop Annotations
* **Goal**: Add vector-based drawing directly on the screenshot selection area.
* **Action**:
  1. Draw lightweight vector elements (Arrow, Highlight Rectangle, Mosaic/Blur brush) on a transparent HTML5 `<canvas>` inside the Vue crop overlay.
  2. Render annotations locally to the cropped image buffer before initiating compression or upload.

---

## 5. Milestone 4: Memory-Aware Screen Pinning (Future)

### A. Static Window Pinning (贴图)
* **Goal**: Pin screenshot captures on top of other applications without incurring Tauri Webview multi-process memory bloat.
* **Action**:
  1. Avoid full-blown Webview window instances for pinned frames.
  2. Explore creating lightweight, raw OS-native windows (via Rust `tao` or simple custom Win32/Cocoa window bindings) that render static image frames using CPU/GPU directly, keeping memory footprints under 30MB.

---

## 6. Milestone 5: Apple-Style UI Makeover (Medium-Term)

### A. Theme and Component Realignment
* **Goal**: Refactor the Settings Webview interface to match the Apple dark mode design system tokens defined in [DESIGN.md](DESIGN.md).
* **Action**:
  1. Replace orange-amber gradients (`bg-gradient-to-r from-orange-500 to-amber-500`) with flat Action Blue (`#2997ff`) highlights, borders, and active state changes.
  2. Implement frosted-glass panels (`backdrop-blur: 24px` over `rgba(255, 255, 255, 0.04)`) for the sidebar and main settings card panels.
  3. Clean up the typography system to enforce negative letter-spacing on display headings and align table layout styles.

---

## 7. Milestone 6: UX Simplification Batch (Short-Term, v0.3.12 candidate)

> Recorded 2026-08-14 after real-world v0.3.11 testing on Orca; items E–J added 2026-08-15. **Deferred by decision — do not implement until picked up deliberately.**

### A. Main Window Free Resizing
* **Defect**: The Settings window is locked at 800×600 and cannot be resized.
* **Root cause**: `"resizable": false` on the `main` window in `src-tauri/tauri.conf.json`.
* **Action**:
  1. Set `resizable: true` with `minWidth`/`minHeight` constraints.
  2. Make the `App.vue` layout responsive (it is currently designed for a fixed 800×600 canvas) — sidebar, settings cards, and the log view should reflow.

### B. Honest Hotkey Naming
* **Issue**: The setting is labelled "Paste Hotkey", but on hosts where the host policy forces Copy (Orca), it only uploads + copies to the clipboard — the actual paste is the user's manual `Ctrl+V`.
* **Action**: Rename the UI label to "Upload Hotkey"; keep the dynamic hint text consistent.

### C. Hotkey Blacklist Validation
* **Issue**: `save_config` accepts any parseable shortcut — there is no blocklist. Registering `Ctrl+V` globally hijacks paste system-wide and self-sabotages the Copy flow: every manual paste after Copy spawns a failed job (`No image found in clipboard`, observed in the 2026-08-14 log).
* **Action**:
  1. On save, reject destructive/reserved combos with a clear message: `Ctrl+C/V/X/Z/S/A/F4`, `Alt+Tab`, `Win+*`-class reservations.
  2. Keep the existing parse-failure rollback.

### D. Injection Mode Consolidation (5 → 3)
* **Issue**: Five modes (`Auto/Direct/Swap/PasteKeep/Copy`) are v0.3.8–v0.3.10 experiment sediment. Since v0.3.11 the per-host decision is owned by `host_policy` (baseline P1), which is the thing Swap/PasteKeep were manually working around; PasteKeep's premise (VSCode accepting synthetic Ctrl+V) is gone entirely.
* **Action**:
  1. Keep three modes: **Auto** (default — host policy decides the full table: plain terminal → Direct, Orca → Copy), **Direct** (force typing; never touches the clipboard), **Copy** (force clipboard + manual Ctrl+V).
  2. Remove Swap/PasteKeep from the UI; migrate existing config values (`swap → auto`, `paste_keep → copy`) while serde keeps accepting the old strings.
  3. Extend `host_policy.rs` from the Orca-only override to the full decision table, so Auto is genuinely automatic.

### E. Single-Instance Enforcement
* **Defect**: Launching img2cli while it is already running spawns a second process — two tray icons / two taskbar entries, and both instances attempt global-hotkey registration (conflict).
* **Action**:
  1. Add `tauri-plugin-single-instance`.
  2. On second launch, show + focus the existing `main` window (surface it from tray) and exit the new process.

### F. System Logs Export / One-Click Copy
* **Gap**: The System Logs tab has no way to get the logs out.
* **Action**: Add toolbar buttons to the System Logs tab:
  1. **Copy All** — write the full `log_history` to the clipboard.
  2. **Export…** — save to a `.log`/`.txt` file via the existing dialog plugin.

### G. Dracula as Default Dark + Deep-Blue Background (herdr reference)
* **Gap 1**: The default theme is still `apple-dark` (`default_theme()` in `config.rs`).
* **Gap 2**: The `dracula` theme's background reads **gray**, not deep blue: `bgApp #282a36` with desaturated gray sidebar/cards (`rgba(33,34,44,.6)` / `rgba(68,71,90,.4)`) — the blue tint gets washed out. Reference (herdr screenshot, 2026-08-15): base `#1e1e2e`, sidebar `#181825` (Catppuccin-Mocha-family deep blue), accent pink `#f92672`.
* **Action**:
  1. `default_theme()` → `"dracula"`.
  2. Retune the dracula entry in `App.vue` (~line 602): `bgApp #282a36 → #1e1e2e`; sidebar `rgba(33,34,44,.6) → #181825`-family; cards/borders shifted from gray rgba to blue-tinted surfaces (`#313244`-family). Keep the dracula accent `#bd93f9` unless asked otherwise.

### H. Hosts & Targets Redesign (Orca-style card list)
* **Gap**: The Hosts & Targets tab is a flat form-based editor — hard to scan and manage multiple targets.
* **Reference**: [`docs/design-ref/orca-ssh-hosts.png`](docs/design-ref/orca-ssh-hosts.png) (Orca's SSH manager, captured 2026-08-15). Card-based vertical list:
  * Per card: host name (bold) · status dot + text (green Connected / gray Disconnected) · connection details `user@host:port` · small note · right-aligned icon actions (view / refresh / edit / delete) · status-specific buttons (Connected → Disconnect; Disconnected → Test + Connect).
  * Header: title + subtitle; "Targets" section with **Import** and **Add Target** global actions.
  * Style: dark cards on a darker background, generous padding (≈16px in-card, ≈24px between), white primary / gray secondary text — mapped to the active theme's CSS variables, not hardcoded hex.
* **Action**:
  1. Rebuild the tab as a card list (one card per `TargetConfig`); the existing form opens only for Add/Edit.
  2. Status wiring: reuse the existing `test_connection` IPC for the Test button; cache the last result as the status dot.
  3. Import button → existing `load_ssh_config` (imports from `~/.ssh/config`).
  4. Connect/Disconnect don't map 1:1 (img2cli uploads on demand) — map them to enable/disable + password set/clear.

### I. UI Localization (中文界面)
* **Gap**: The Settings UI is English-only. The user's primary language is Chinese — they referenced Snipaste's "显示语言" setting (its settings UI ships in Simplified Chinese) as the expected experience. *(User's original phrasing said "英文版" but context makes clear they mean a Chinese version — confirm before implementing.)*
* **Action**:
  1. Add a lightweight frontend i18n layer (a `zh-CN` + `en` string dictionary; no heavy framework needed for a settings-sized UI).
  2. Add a `language` field to `AppConfig` + a Display Language dropdown in General Settings; default `zh-CN`.
  3. Translate the General Settings / Hosts & Targets / System Logs vocabulary; keep hotkeys, paths, and log lines (diagnostics) in English.

### J. Snipaste-Style Automatic Window Detection in the Capture Overlay
* **Gap**: Today the capture overlay (v0.3.7) requires a manual drag to define the region. Snipaste, on hotkey press, immediately outlines the window under the cursor (blue border — full screen or the app's window) with operation hints; click snaps that window.
* **Behavior wanted**: press screenshot hotkey → frozen overlay shows the hovered window auto-outlined (colored border) + hint tips; click = snap that window; drag = custom region (existing editor with handles stays).
* **Action**:
  1. Enumerate on-screen window rects per platform (Win32 `EnumWindows`+`GetWindowRect` filtering visible/owned; macOS `CGWindowList`; Linux X11) and expose them to the overlay alongside the frozen frame.
  2. In the overlay, hit-test cursor position against window rects (CSS→physical px conversion, same scale handling as `capture_region`), draw the hover outline + dimension labels + hint bar.
  3. Click-to-snap feeds the window rect into the existing crop path; drag still enters the adjustable-selection editor.
