# Roadmap - img2cli

This file tracks future architecture improvements, user experience features, and optimization milestones for the `img2cli` desktop application and daemon. Completed milestones are tracked in release notes and git history.

---

## 0. Road to v1.0.0 (decided 2026-08-16)

**Goal: at v1.0.0, ship a feature-complete / highly stable / performant / lightweight product. ALL remaining debt is cleared before v0.4.5 (nothing slips to v0.5.x).**

| Version | Theme | Contents |
|---|---|---|
| v0.4.1 | Feel + defects | 6-R state machine v3 · 6-S element detect + Tab cycling · **overlay keys: Shift+R reuse last region, `,`/`.` cycle region history (last 8), WASD nudge cursor 1px** · IME×Direct real-test (force Copy per-host if conflicting) · host_policy → process-name detection · engineering cleanup (delete legacy `src/` CLI tree, orphan plugin-shell dep, dead_code, themed slider track) |
| v0.4.2 | **Annotation editor + action toolbar** | Toolset (user-final 2026-08-16): **箭头 arrow · 画笔 pen · 马克笔 marker/highlighter (multiply blend) · 马赛克 mosaic (incl. secure mode) · 文本 text · 圆形/矩形 circle/rect · 橡皮擦 object-eraser (alpha hit-test click-to-delete) · 撤销/重做 undo/redo (snapshot stack)** + color/thickness pickers — flameshot five-mechanism blueprint (`docs/REF_mining_20260809.md` §A) — PLUS the confirmed-selection action toolbar, Snipaste-style: **icon buttons with hover tooltips** for 📌 贴到屏幕 (pin, pulled forward from v0.4.3; ShareX interaction spec §C, webview-based first cut) / 💾 保存到文件 (dialog save + write_image) / 📋 复制到剪贴板 / ✓ upload+inject (existing flow) |
| v0.4.3 | Multi-monitor (+ pin upgrade) | per-monitor capture & coordinate mapping · optional native-window pin upgrade if the webview pin proves memory-heavy |
| v0.4.4 | Performance + intelligence | SSH keep-alive pool (<200ms uploads, M1-B) · Rust-native pre-emptive freeze (M1-A) · L-tail (cursor capture / focus-loss exit / sound) · OCR→markdown code block (Windows OCR Runtime first) · **长截屏 scrolling capture** (user-requested 2026-08-16: select region → loop {PostMessage WM_MOUSEWHEEL to target, Chromium needs SendInput fallback} → frame capture → row-overlap stitching → long image into the existing annotate/pin/upload pipeline; ShareX's implementation in `ref/pkg/ShareX-develop` is the reference) |
| v0.4.5 | Buffer | regression fixes only — the goal is for this version to be EMPTY |
| v1.0.0 | Ship | code signing (⚠️ requires purchasing a certificate — user decision), final polish, docs |

Open decisions: signing certificate budget (v1.0.0 stage); OCR scope = Windows OCR Runtime first, macOS Vision post-1.0 (assumed OK).

---

## 1. Platform Direction & Build Strategy
`img2cli` aims to remain a highly performant, single-binary background daemon with a lightweight, lazy-loaded configuration interface.
* **Primary Target**: Windows (`x86_64-pc-windows-gnu` / setup installer and portable zip).
* **Secondary Targets**: macOS (Apple Silicon & Intel DMG) and Linux (Portable AppImage).
* **Guiding Principle**: Zero-dependency background operation with low memory overhead (<25MB idle).
* **Version digits rule** (from 2026-08-16): read versions as decimals — the patch digit is 0–9 only; carry into minor at 10 (`0.3.9 → 0.4.0`, never `0.3.10`). Historical tags v0.3.10–v0.3.15 predate the rule and stay; the next release is **v0.4.0**.

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

## 4. Milestone 3: Annotation Editor (user-requested 2026-08-16 → **v0.4.2**, Road to v1.0.0)

### A. Annotation Tools on the Selection
* **Goal**: Snipaste-class editing inside the capture overlay's confirmed selection.
* **Toolset** (user's list): 箭头 arrow · 画笔 freehand pen · 马赛克 mosaic/pixelate · 文字 text · **保存图片 save-to-file** · (plus: highlight rect from the original plan, undo/redo, color/width pickers).
* **Architecture**:
  1. After a selection is confirmed (6-O state machine), show a tool toolbar; annotations render on a transparent HTML5 `<canvas>` layered over the frozen frame within the selection — stored as vector objects (arrow = line+head, pen = polyline, text = text box, mosaic = pixelate region sampled from the frozen pixels by down/up-scaling).
  2. On final confirm, rasterize annotations onto the image buffer BEFORE crop → clipboard → the existing upload/inject pipeline (annotated image flows through unchanged).
  3. Save-to-file: plugin-dialog `save()` + a `write_image` command (PNG/JPG).
  4. References in `ref/pkg`: flameshot & ksnip (annotation model, tool UX), ShareX (editor), greenshot (text/mosaic details).

---

## 5. Milestone 4: Screen Pinning / 贴图 (user-requested 2026-08-16 → **v0.4.3**, Road to v1.0.0)

### A. Pin the Capture to the Screen (Snipaste-style)
* **Goal**: Float the confirmed crop as an always-on-top, draggable, resizable mini-window; Esc/× to close; optionally scroll-to-zoom.
* **Action**:
  1. First cut: a small Tauri webview window rendering the crop (fast to build; acceptable memory for 1-2 pins).
  2. Memory-aware upgrade (original plan): raw OS-native windows (`tao`/custom Win32-Cocoa) rendering static frames, target <30MB per pin — `ref/pkg/wispterm` is the portable/native-window reference.
  3. Pin action joins the editor toolbar (Milestone 3) alongside copy/save/upload.

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

> Recorded 2026-08-14 after real-world v0.3.11 testing on Orca; items E–L added 2026-08-15.
> **Status: A–I shipped in v0.3.12; H, J, and the L core shipped in v0.3.13; M, N, O + dual-path hints shipped in v0.3.14 (2026-08-16). K dropped by decision. L remainder (cursor capture, focus-loss exit, sound, magnifier/crosshair/guides) still deferred → Milestone 3/4 (annotation + pinning) next.**

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

### K. ~~UI Font Customization~~ — DROPPED (2026-08-15, by decision)
* Removed from scope; not planned. (Reference screenshot kept at `docs/design-ref/snipaste-general-font.png`.)

### O. Overlay Selection State Machine: unconfirmed rects must not lock edit mode (v0.3.13 follow-up)
* **Defect** (reported 2026-08-16, fixed differently in v0.3.15 after user feedback): the preloaded last-region proposal locked the overlay into move-edit mode with no confirmation. v0.3.15 removed the preload/proposal outright — the overlay always opens as a fresh crosshair; drawing works anywhere including inside an existing selection; Alt+drag inside keeps move.

### P. Click-to-snap must be decided on mouseUP, not mouseDOWN (v0.3.15 follow-up) — SHIPPED v0.4.0
* mousedown always starts a draw; mouseup with a tiny rect + press-time hover snaps the window. Follow-ups moved to 6-R.

### R. Overlay state machine v3: confirmed-vs-unconfirmed rules + hover feedback (v0.4.0 follow-up)
* **Defects** (reported 2026-08-16):
  1. After a click CONFIRMS a window, dragging inside the selection should MOVE it — v0.3.15's fix2 made inside-drag start a new draw; that was over-correction for the (now-removed) preload lock-in. Correct model: UNCONFIRMED = drag anywhere draws (incl. inside detected windows), quick click confirms; CONFIRMED = inside-drag moves, handles resize, outside-drag draws fresh.
  2. Click-vs-drag threshold too small (4px) — jittery clicks become tiny draws instead of snapping the window. Raise to ~8-10px (Snipaste-ish).
  3. Hover outline vanishes the moment the button is pressed (hoverRect nulled on first mousemove while drawing) — keep it visible during the hold until movement passes the threshold.
* **Fix sketch**: rectMouseDown → plain = startMove (revert fix2's inside-draw), keep Alt variant as redundant; raise hasRect/draw threshold to 8px for the mouseup snap arbitration; hold hoverRect until draw actually exceeds threshold.

### T. Overlay UX v4 — Snipaste 行为对齐 (v0.4.3 用户报告, 2026-08-17)

1. **确认态模型**: 有选区 ≠ 确认。只有点击 ✓ 才算确认;确认前光标保持十字、**任意位置 (含选区内部) 拖拽 = 重画选区** (Tab/Shift+R/`,`/`.` 产生的选区同样未确认);确认后光标才变为带箭头十字、区内拖动 = 移动。需要显式 `confirmed` 状态 (当前"选区存在即已确认"的隐式模型是错的)。
2. **文本工具失效**: 点击文本工具后在选区内点击,输入框无法输入 (v0.4.2 起悬而未决; 嫌疑: capture-phase 键盘监听与 textarea 焦点/事件链,需带日志调试)。
3. **橡皮擦应为"逐段擦除"**: Snipaste 是擦除笔迹被扫过的**片段**(polyline 在命中处分裂),不是整对象删除。当前对象级删除不符合预期;需实现 stroke-splitting。
4. **Ctrl+C = 复制图像并退出**: 选区存在时 Ctrl+C 直接把 (含标注的) 图像复制到剪贴板并关闭 overlay — Snipaste 行为,发往任意处的入口。
5. **一键保存**: 点击 💾 不弹路径对话框 — 自动命名 `img2cli_YYYY-MM-DD_HH-mm-ss` 存入默认目录 (save_dir),保存后退出 overlay。(Snipaste: `Snipaste_2026-08-17_16-26-37` 格式,直接落盘;是否可配置默认保存目录随后议。)

### S. Snipaste-grade element detection + Tab cycling (v0.4.0 follow-up)
* **Wanted**: Snipaste detects many window ELEMENTS (button/input level) under one cursor position and cycles them with Tab.
* **Fix sketch**: backend — EnumChildWindows descent (greenshot's FindChildUnderPoint, ~3 levels, edge-rule: cursor on a rect edge returns the parent whole) exposing all candidates containing the cursor, Z-topmost-first (Windows-only via windows-sys; macOS via AX APIs later); frontend — candidate list at the cursor, Tab/Shift+Tab cycles the highlighted candidate, outline + size label track the active one.

### Q. Default host must be a FLAG on a target card, not a duplicated pinned card (v0.3.15 follow-up) — SHIPPED v0.4.0
* Pinned card removed; `is_default` flag on TargetConfig; DefaultSshResolver reads the flag with legacy config.ssh fallback; auto-migration seeds the flag.

### N. Merge Default-Host Form into the Card List (v0.3.13 follow-up)
* **Issue** (reported 2026-08-16): the Hosts & Targets page has two host-editing surfaces — the top "default host" form (`config.ssh`) and the Dynamic Router Targets card list. The form is load-bearing (DefaultSsh fallback route — most uploads actually go through it; the keyring password lives on it; "Set Default" copies into it) but duplicative UI.
* **Action**:
  1. Render `config.ssh` as the pinned first card of the list (默认 badge + pin icon); edit it through the same Add/Edit modal as targets.
  2. Backend `config.ssh` stays the storage/routing structure (zero routing/upload changes); "Set Default" becomes swapping a card into the default slot.
  3. Frontend-only refactor of the Hosts tab + a small setAsDefault adjustment.

### M. Own-Window Detection (v0.3.13 follow-up)
* **Defect** (reported 2026-08-16, v0.3.13): auto window detection never highlights img2cli's own Settings window — the frozen frame shows it, but hovering yields no outline.
* **Root cause (verified in xcap 0.5.2 source, `src/windows/impl_window.rs::is_valid_window`)**: xcap's Windows enumeration skips ALL windows owned by the current process (`lp_dw_process_id == GetCurrentProcessId() → false`), a defensive rule borrowed from WebRTC desktop-capture (GetWindowText on own-process windows can deadlock the message loop). The capture overlay itself is additionally excluded by our empty-title filter — only the overlay is intentional.
* **Fix sketch**: don't fight xcap — we know our own windows. In `capture_full_screen`, append the main window's rect via Tauri APIs (`app.get_webview_window("main")` → `is_visible()` + `outer_position()` + `outer_size()`, physical ÷ scale → CSS px) into `window_rects`. No Win32, no deadlock surface.

### L. Capture Options Settings Tab (Snipaste-style)
* **Reference**: [`docs/design-ref/snipaste-capture-options.png`](docs/design-ref/snipaste-capture-options.png) + [`docs/design-ref/snipaste-capture-appearance.png`](docs/design-ref/snipaste-capture-appearance.png). Snipaste's 截屏选项 exposes: 自动检测窗口 / 自动检测界面元素 / 捕捉鼠标指针 / 截屏时其他窗口激活自动退出 / 历史截屏区域数(8)+循环 / 历史记录数(20) / 音效文件 / 边框宽度(3px) / 遮罩颜色 / 显示锚点+锚点描边颜色 / 放大镜显示内容(遮罩/边框/锚点) / 全屏十字线 / 辅助线 / 显示快捷键提示 / 恢复默认.
* **Gap**: img2cli's capture overlay has no user-tunable options.
* **Action** (prioritized subset mapped to img2cli):
  1. 自动检测窗口 / 界面元素 toggles — the companion switches for item J (+ 隐藏 1x1 提示).
  2. 历史截屏区域 — remember the last N selection rects, cycle through them (high value for the AI-CLI workflow: repeated captures of the same terminal region).
  3. Selection appearance: border width, mask (dim) color, show anchors + color, show shortcut hints (hint already exists — make it toggleable).
  4. 捕捉鼠标指针 (draw the cursor into the captured image) and focus-loss auto-exit.
  5. 音效文件 + 恢复默认 per section.
  6. Bigger optional sub-features, later: magnifier (放大镜), fullscreen crosshair, guides.
