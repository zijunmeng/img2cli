# Roadmap - img2cli

This file tracks future architecture improvements, user experience features, and optimization milestones for the `img2cli` desktop application and daemon. Completed milestones are tracked in release notes and git history.

---

## 0. Road to v1.0.0 (decided 2026-08-16)

**Goal: at v1.0.0, ship a feature-complete / highly stable / performant / lightweight product. ALL remaining debt is cleared before v0.4.5 (nothing slips to v0.5.x).**

| Version | Theme | Contents |
|---|---|---|
| v0.4.1 | Feel + defects | 6-R state machine v3 · 6-S element detect + Tab cycling · **overlay keys: Shift+R reuse last region, `,`/`.` cycle region history (last 8), WASD nudge cursor 1px** · IME×Direct real-test (force Copy per-host if conflicting) · host_policy → process-name detection · engineering cleanup (delete legacy `src/` CLI tree, orphan plugin-shell dep, dead_code, themed slider track) |
| v0.4.2 | **Annotation editor + action toolbar** | Toolset (user-final 2026-08-16): **箭头 arrow · 画笔 pen · 马克笔 marker/highlighter (multiply blend) · 马赛克 mosaic (incl. secure mode) · 文本 text · 圆形/矩形 circle/rect · 橡皮擦 object-eraser (alpha hit-test click-to-delete) · 撤销/重做 undo/redo (snapshot stack)** + color/thickness pickers — flameshot five-mechanism blueprint (`docs/REF_mining_20260809.md` §A) — PLUS the confirmed-selection action toolbar, Snipaste-style: **icon buttons with hover tooltips** for 📌 贴到屏幕 (pin, pulled forward from v0.4.3; ShareX interaction spec §C, webview-based first cut) / 💾 保存到文件 (dialog save + write_image) / 📋 复制到剪贴板 / ✓ upload+inject (existing flow) |
| v0.4.3 | 修复批 (re-planned; multi-monitor → v0.4.5) | 💾/📌 死因=**ACL** (overlay 窗不在 capabilities → JS 插件调用被拒) → Rust 侧自定义命令 + 日志 · Esc 捕获阶段 + 右键退出 · 马赛克拖拽归一化 · 橡皮擦命中域 + 扫擦 |
| v0.4.4 | Snipaste 交互对齐 — 6-T 七项 (re-planned; perf batch → v0.4.5) | ✓ 确认态模型 (✓=纯确认, ⬆=上传) · 文本焦点修复 · 笔迹分裂擦除 · Ctrl+C 复制退出 · 一键自动命名保存 · **常驻热窗 + JPEG 显示层** · 贴屏右键菜单 + 边缘缩放 |
| v0.4.5 | **热键修复 + 观测性** (2026-08-18 紧急插入: 截图热键在用户机坏死) | 6-U.9① 注册失败日志化 (启动 `let _ =` 吞错 + save_config `is_ok()` 吞错, 均改 Ok/Err 双路日志 + 回滚) · 6-U.9② 现场取证待新日志 · 截图热键链路全程打点 (`Screenshot hotkey received` → `Screen frozen in Xms` → `refreshing warm window` → `Capture overlay shown` = 链路终点) · 6-U.9③ 热键录入器 IME 修复 (`e.key==="Process"` → e.code 回退) · 6-U.8 copy_logs 打点 |
| v0.4.6 | 回归修复 (2026-08-18) | v0.4.5 `state not managed` 启动回归 (prewarm 挪到 manage 前引发的竞态, `a404b99`); 打点落地后**现场定位死因**: 热键链 4 行中前 3 行每按必出、第 4 行 `Capture overlay shown` 从不出现 → 断点 = 常驻浮层 webview 收到 capture-refresh 后未完成 渲染→show (6-U.9②) |
| v0.4.7 | **浮层修复批** (2026-08-18, 用户被挡无法测试 → 插队) | 6-U.9② **死浮层看门狗**: emit 后 400ms 仍隐藏 → 自动重建浮层窗 (新 webview 自加载自 show, 重建闸门防连点堆积) · `get_captured_image` 存活打点 (区分 webview 死 vs invoke 挂) · 6-U.10② **录入器 keyup 兜底** (IME 吞字母 keydown 时, 幸存的 keyUP 补齐组合) · 6-U.10③ **配置未加载禁存** (configLoaded 闸 + Save 禁用, 杜绝默认值覆盖 config.toml) |
| v0.4.8 | **最后一格, 全部清债** (原计划再顺延 — 补丁位 0.4.8–0.4.9, 0.5.0 前全部清完) | **多显示器** (per-monitor freeze + 坐标映射, 现只 `monitors.first()`) · **SSH 保活池** (<200ms uploads, M1-B) · **原生预冻结** (M1-A) · **OCR→markdown 代码块** (Windows OCR Runtime first) · **长截屏 scrolling capture** (select region → loop {PostMessage WM_MOUSEWHEEL to target, Chromium needs SendInput fallback} → frame capture → row-overlap stitching → long image into the existing annotate/pin/upload pipeline; ShareX's implementation in `ref/pkg/ShareX-develop` is the reference) · L-tail (cursor capture / focus-loss exit / sound) · **6-U: v0.4.4 用户反馈批** (U1–U7) |
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
6. **截图浮层秒出** (Snipaste 按下热键边框瞬间出现,我们有可感知延迟): 现路径 = 建窗 + 整屏 PNG base64 编码 (~MB 级 IPC) + webview 加载渲染后才 reveal。方向: ①**overlay 窗口常驻隐藏** (启动即建,热键只 show+设图,省掉建窗/webview 冷启动) ②缩小 IPC 载荷 (整屏 base64 是大头; JPEG 或写临时文件走 asset 协议)。
7. **贴屏窗交互残缺**: 右键应弹**菜单** (至少: 复制图片 / 销毁;可加 另存为/置顶切换),当前右键=直接关闭;尺寸调节不应只有滚轮 — 边缘拖拽 resize (窗口 resizable + 边缘热点)。

### U. v0.4.4 用户实测报告 (2026-08-18, 记录待修 — 全部进 v0.4.5)

1. **Tab 之后无法再自由选区**: 实测 Tab 切换候选后,选区即被"定死",不能再任意拖拽重画; Snipaste 的 Tab 只是切换高亮候选,之后仍可随时拖拽自由选区/继续 Tab。(工作假设: 我们 Tab 时立即把候选固化为选区并清掉 hover 高亮;是否真锁死编辑、锁在哪一步,待复现。)
2. **橡皮擦没有圆圈光标, 体感仍是"点击删对象"**: 期望 Snipaste 式 — 选中橡皮擦后出现 **~14px 的圆圈光标** (圈 = 实际擦除范围),按住扫过之处逐点擦除。现状: 无任何半径视觉反馈 (v0.4.4 T3 笔迹分裂已上线但 ERASE_RADIUS=10 看不见; 形状/文本/马赛克整删属设计)。需: 圆圈光标 (半径=擦除半径, 跟随鼠标) + 笔迹逐段消失的可见性验证。
3. **画笔/马克笔 Shift = 直线**: 按住 Shift 拖拽应绘制从落点到当前点的直线 (通用绘图惯例)。
4. **粗细/大小调节不足**: 画笔/马克笔线宽当前只有共享 toolSize 1–8,范围太窄; 橡皮擦半径固定 10px,未接进工具栏 −/+。需: 线宽档位扩大 (或按工具独立记忆), 橡皮擦大小可调。
5. **贴图窗边缘 resize 无效 + 右键直接关闭**: 实测仍只能移动 + 滚轮缩放 — 窗口虽 `resizable(true)` (main.rs T7 注释),undecorated 窗口边缘命中不生效,需 WM_NCHITTEST 边缘热点或显式缩放手柄; 右键未弹菜单而是把贴图关掉了 (与 T7 实现不符, 待复现 — 嫌疑: 双击右键被算成 dblclick 关闭路径 / WebView2 默认上下文菜单)。
6. **保存应免确认直达另存为**: 期望选区存在即可点 💾 直接弹"另存为"对话框,不需要先点 ✓; 实测必须先点对勾 (机制待复现)。注意 v0.4.4 T5 把 💾 改成了**无对话框一键落盘** — 与本次期望方向冲突,需决策: 恢复对话框 / 一键保存可配置 / 💾=一键、Shift+💾=另存为。
7. **热键→浮层仍有可感知延迟** (常驻热窗 + JPEG 显示层之后仍不"秒出"): 剩余成本嫌疑 (工作假设): xcap 整屏抓取 (GDI BitBlt) → JPEG 编码 → base64 IPC → `<img>` 解码渲染,**全部完成后才 reveal** (`show_capture_overlay` 在图片上屏后调用)。根治 = M1-A 原生预冻结 (先 show 再贴图 / 更快的抓取路径),v0.4.5 优先级提升。
8. **系统日志「复制全部」偶发不工作** (2026-08-18 报告 → 当日定位): 实测 — 旧实例 (00:06 启动) 点击后无任何 toast、记事本粘贴为空; **重启后 (01:53) 同按钮正常**。诊断结论: ①**主因 (已证)**: 该实例的 `copy_logs` invoke 挂起未执行 (无 toast = try/catch 都没走到; 同实例注入 job 的 arboard 写剪贴板正常 → 写路径无恙); 根因无法回溯 (无日志), 归类"偶发 IPC 挂起, 重启自愈"。②**次要陷阱 (行为如此)**: 用户习惯流程 "复制全部→F8→Ctrl+V" — F8 的 copy 模式注入会把剪贴板覆写成 `![image](路径)`, 日志必丢; 正确用法 = 复制全部后直接 Ctrl+V。已排除: arboard 写入丢失 (3.6.1 纯 Win32 立即写+重试, 已读源码) · Orca 拒多行文本 (记事本多行→Orca ✓)。**v0.4.5 加固**: copy_logs 命令加 daemon 日志 (执行/字节数/错误, 再发时有据可查) + 考虑改 async (挪离可疑线程)。
9. **v0.4.4 截图热键整链路失效** (2026-08-18 01:53 重启后, **①③ 已修 v0.4.5**): 实测 — Alt+X 按了无反应; 换 Control+J 能注册 (日志 `Registered screenshot shortcut: Control+J`) 但按下也无反应。**三个独立问题**: ① 启动注册吞错 (main.rs `let _ = register()`) + save_config `is_ok()` 吞错: Windows RegisterHotKey 独占, 旧实例/其他进程占键时注册静默失败, 无任何日志 → **已修 (v0.4.5): 启动/保存双路径 Ok/Err 全量日志 + 失败回滚**; ② 僵尸实例嫌疑 (工作假设, 未证): 旧实例可能仍在后台占键 — v0.4.5 的注册失败日志会直接点名 (`Failed to register ... Another instance/app may be using it`); ③ **热键录入器 IME 吃键** (`e.key==="Process"`) → **已修 (v0.4.5): keyFromEvent 走 e.code 回退**。**链路打点已埋 (v0.4.5)**: `Screenshot hotkey received` (处理器活) → `Screen frozen in Xms` (抓屏活) → `refreshing warm window` (浮层窗在) → `Capture overlay shown` (webview 渲染完成并 show = 全链通)。现场复测: 按 Alt+X 后看缺哪行, 即断点。

**v0.4.6 现场实测 (2026-08-18 11:06–11:11, 断点已定位)**: 启动注册 F8+Alt+X ✓ 无 state-not-managed ✓; 按 Alt+X 约 20 次 — **前 3 行每按必出** (handler 活 / 冻结 ~50ms @1920x1080 / 热窗在且事件已发), **第 4 行 `Capture overlay shown` 从未出现** → 断点锁定: **常驻浮层 webview 收到 capture-refresh 后没走完 get_captured_image→渲染→show**。剩余两种可能 (待 `get_captured_image` 加打点区分): (a) webview JS 死/监听器不在 (初始加载失败 或 WebView2 对隐藏 webview 的挂起/节流 — 常驻隐藏窗设计的固有风险); (b) webview 活着但 invoke 挂起 (与 6-U.8 copy_logs 挂死同类)。**修复设计 (待开工)**: ① `get_captured_image` 加日志 (收到调用 = webview 活); ② Rust 兜底 — emit 后 ~300ms 检查 `is_visible()`, 仍隐藏则强制 show + 日志 `webview unresponsive — forced show` (健康实例前端先 show 无闪, 死实例至少浮层出现不再卡死用户); ③ 评估 WebView2 挂起对策 (show 唤醒 / 禁用挂起)。

**附带确认**: 用户"改 Alt+Z 后有时改不回 Alt+X" — 注册日志显示所有成功提交的热键变更全部注册成功 (Alt+X↔Alt+Z↔Alt+V 反复切换均 ✓); "改不回"实为 6-U.10② IME 录入器 bug 的间歇性表现 (字母键 keydown 被吞 → 裸 `Alt` → 被拒), 时灵时不灵与输入法状态一致。"默认变成了 Alt+X" 是误读 — 出厂默认从未变 (Alt+Shift+S), 是 v0.4.6 修复后磁盘配置正常加载, 显示回用户自设的 Alt+X。
10. **v0.4.5 启动 "state not managed" 回归的三个连锁现象** (2026-08-18 用户报告, 回归本体**已修 v0.4.6** `a404b99`): ① 热键"变回默认 Alt+Shift+S" — get_config 启动失败 → 前端 `config` 保持 JS 默认对象, 界面显示默认值 (**非磁盘配置被改**; 磁盘仍是 Rust 侧独立加载的 Alt+X, 启动注册不受影响); ② **改不回 Alt+X** — 录入器只收到 Alt (裸修饰键): IME 在 `<input>` 里把字母 keydown 整个吞掉 (v0.4.5 的 Process 回退只救"到达的 Process 事件", 字母事件根本没到达 recorder) → 存成裸 `Alt` → 被 hotkey_rejection 拒 ("without modifiers must be a function key", 报错本身合理)。**修 (待做)**: 录入器不能只信 keydown —— 修饰键按下后若下一个非常规键 keydown 未到达, keyup 兜底或直接以 `e.code` 为主键源; ③ **隐性配置覆盖风险**: get_config 失败状态下若用户点过一次成功的"保存设置", 默认值 config 会整份写回 config.toml (targets/热键/主题全部归零) — v0.4.6 装上后需检查 Hosts & Targets 是否还在; 根治方向: loadConfig 失败时禁用 Save 按钮 (loaded 标志位)。

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
