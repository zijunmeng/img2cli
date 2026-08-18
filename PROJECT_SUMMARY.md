# img2cli — Project Summary

> **Version:** 0.4.8 | **Date:** 2026-08-18 | **Status:** Active Development（冲刺 v1.0.0）

## 1. 项目概述

`img2cli` 是一个**跨平台系统托盘桌面应用**，专为多模态 AI 工作流设计。核心功能：截图（含标注/贴图）→ 后台上传服务器 → 按热键把 Markdown 图片路径注入终端 / AI CLI，同时保持剪贴板中的原始图片不变（可继续粘贴到微信 / Word 等聊天软件）。

### 核心价值
- **截图即上传**：确认选区后 SFTP 上传立即在后台开始；注入热键按下时直接粘贴已送达路径（剪贴板图像未变时走指纹快速通道，零重传）。
- **Snipaste 级截图体验**：冻结帧 + 窗口/元素自动识别（Tab 循环）+ 显式确认模型 + 标注编辑器（箭头/画笔/马克笔/马赛克/文字/图形/逐段擦除）+ 贴到屏幕 + 一键保存 + Ctrl+C 复制退出。
- **不抢剪贴板**：`direct` 注入全程不碰剪贴板；同一张截图既能贴终端（出路径）也能贴微信（出图）。
- **SSH 密码/密钥**：密码存 OS 钥匙串（Xshell 式），密钥走系统 ssh，known_hosts TOFU 校验，按窗口标题/进程名自动路由。
- **可观测性**：热键注册、截图链路（按键收到 → 冻结 → 浮层刷新 → 浮层显示）全程打点；守护日志同时镜像到 `%TEMP%\img2cli\daemon.log`，软件冻死也可取证。

---

## 2. 版本演进

| 版本 | 关键特性 |
|---|---|
| v0.1.x | 纯 CLI 守护进程（Rust），命令行配置，scp 上传 |
| v0.2.x | 交互式菜单，多目标路由（窗口标题匹配），单引号包裹 |
| v0.3.0 | **Tauri v2 GUI 重写**：托盘常驻、Vue 3 设置面板、毛玻璃 UI |
| v0.3.1–0.3.5 | SSH config 导入、跨终端自动路由、热键录制器、**内置区域截图**（xcap 冻结帧）、主题系统（6 套）、SSH 连接池 |
| v0.3.6 | **架构重构**：CapturedArtifact 统一模型、有界 JobManager 单 worker 串行管线、四大边界（RouteResolver / ArtifactTransport / CliAdapter / 注入）、SFTP 三段超时 + mkdir、known_hosts TOFU |
| v0.3.7–0.3.11 | Direct 静默失败修复（fallback_to_copy 保险）、per-target 注入策略、wrap_single_quotes、实测矩阵 |
| v0.3.12 | **UX 简化批**：窗口可调大小、热键黑名单校验、单实例、日志复制/导出、Dracula 深蓝默认、注入模式收敛 5→3（auto/direct/copy）、**中文界面**（strings.js 字典） |
| v0.3.13–0.3.15 | 自窗识别（xcap 排除自进程的绕行）、默认主机=卡片标志、选区状态机、mouseup 吸附裁决、**截图即上传**管线拆分（后台上传 + 指纹快速注入）、预加载误导区移除 |
| v0.4.0–v0.4.1 | 手感批：窗口识别 P0 过滤（本屏/矩形求交/Z 序）、状态机 v3（确认后区内拖=移动）、**元素识别 + Tab 钻取**（EnumChildWindows）、Shift+R/`,`/`.` 区域历史、WASD 光标微调、host_policy 进程名匹配、删旧 CLI 树 |
| v0.4.2 | **标注编辑器**（flameshot 值对象模型 + drawObjects 单渲染器）+ 动作工具栏（📌/💾/📋/✓/⬆） |
| v0.4.3 | 修复批：💾/📌 死因=**ACL**（overlay 窗不在 capabilities → JS 插件调用被拒）→ Rust 侧自定义命令、Esc 捕获阶段、马赛克拖拽归一化、橡皮擦扫擦 |
| v0.4.4 | **Snipaste 对齐**（6-T 七项）：✓ 确认态模型、文本焦点、笔迹分裂擦除、Ctrl+C 复制退出、一键自动命名保存、**常驻热窗 + JPEG 显示层**、贴屏右键菜单 |
| v0.4.5–v0.4.8 | **可观测性与加固周**（2026-08-18 四连发）：热键注册 Ok/Err 全量日志（吞错修复）+ 截图链路四段打点 + IME 安全录入器（v0.4.5）→ state-not-managed 启动回归修复（v0.4.6）→ 死浮层看门狗 + keyup 录入兜底 + 配置未加载禁存（v0.4.7）→ 看门狗降级为**按键节奏恢复**（emits>shows 判死 + 下一键重建）+ **daemon.log 文件镜像**（v0.4.8） |

> 版本规则（2026-08-16 起）：十进制版本，补丁位 0–9，逢 10 进位（`0.4.9 → 0.5.0`）；历史 tag v0.3.10–v0.3.15 不回改。

---

## 3. 技术栈

### 后端 (Rust)
| 依赖 | 版本 | 用途 |
|---|---|---|
| `tauri` | 2.x | 应用框架（窗口、托盘、IPC、事件） |
| `tauri-plugin-single-instance` | 2.x | 单实例（二次启动唤起已运行实例） |
| `tauri-plugin-autostart` | 2.x | 开机自启 |
| `tauri-plugin-global-shortcut` | 2.x | 全局热键注册 |
| `tauri-plugin-dialog` | 2.x | 文件对话框（Rust 侧命令使用，绕开 overlay 窗 ACL） |
| `russh` / `russh-sftp` | 0.51 / 2.x | 纯 Rust SSH/SFTP（密码认证、三段超时、TOFU） |
| `xcap` | 0.5 | 屏幕捕获 + 窗口枚举（Win/Mac） |
| `enigo` | 0.5 | 键盘模拟（direct 注入） |
| `arboard` | 3.6.1 | 剪贴板读写（图片 + 文本；Win32 立即写 + 5×5ms 重试） |
| `keyring` | 3.x | OS 钥匙串 |
| `image` | 0.25 | 缩放、JPEG 编码、裁剪（含 JPEG q85 显示层） |
| `windows-sys` | 0.59 | Win32：子窗口枚举、光标定位、提权重启 |
| `base64` | 0.22 | 标注合成 dataURL 解码 |
| `tokio` / `serde` / `toml` / `chrono` | — | 异步运行时、序列化、配置、日志时间戳 |

### 前端 (Vue 3)
| 依赖 | 版本 | 用途 |
|---|---|---|
| `vue` | ^3.4 | 响应式 UI（单文件 App.vue：设置面板 + 截图覆盖层 + 贴图窗） |
| `vite` | ^5.4 | 构建 + 开发服务器 |
| `tailwindcss` | ^3.4 | 原子化 CSS（全部走 CSS 变量以适配主题） |
| `@tauri-apps/api` + `plugin-dialog` | ^2.0 | invoke / 事件 / 主窗口对话框 |

### CI/CD
- **GitHub Actions**：`ci.yml`（per-push 三平台 `cargo check --all-targets` + `cargo test`）+ `release.yml`（tag 触发，三平台 + Windows 便携 zip + macOS universal）。
- **本地双 target 检查**（CI 前置门）：`/tmp/fakepc` 伪造 GTK .pc 桩后 `cargo check --all-targets`（Linux）+ `--target x86_64-pc-windows-gnu`（Windows 交叉）。

---

## 4. 架构概览

```
┌──────────────────────────────────────────────────────────────────┐
│                        img2cli 应用进程                            │
│                                                                  │
│  ┌─────────────┐    IPC (invoke)    ┌──────────────────────┐     │
│  │  Vue 3 前端  │◄──────────────────▶│   Rust 后端 (Tauri)   │     │
│  │  (WebView)  │    事件 (listen)    │                      │     │
│  │             │                    │  Tauri Commands ×27   │     │
│  │ • Settings  │                    │  Daemon Thread        │     │
│  │ • Hosts Mgr │                    │  JobManager (串行)     │     │
│  │ • Logs      │                    │  System Tray          │     │
│  │ • Capture   │                    │  Global Hotkeys       │     │
│  │   Overlay   │                    │  Pin Windows          │     │
│  │ • Pin Pages │                    └──────────────────────┘     │
│  └─────────────┘                                                 │
└──────────────────────────────────────────────────────────────────┘
```

### 核心数据流（截图 → 注入，v0.3.15 起的拆分管线）

```
用户按截图热键 (默认 Alt+Shift+S)
    │  [日志: Screenshot hotkey received → Screen frozen in Xms]
    ▼
capture_full_screen() — xcap 冻结主屏 + 窗口矩形快照（含子元素钻取表）
    ▼
open_capture_overlay() — 常驻热窗 emit capture-refresh
    │  [emits+1；日志: refreshing warm window]
    ▼
Vue 覆盖层：get_captured_image()（JPEG q85 显示层）→ 渲染 → show_capture_overlay()
    │  [shows+1；日志: Capture overlay shown —— 链路终点]
    ▼
用户：Tab/点击/拖拽 选区 → （可选）标注 → 动作
    ├── ✓ 确认（进入移动/调整态）
    ├── 📋 / Ctrl+C —— compositeRegion() → copy_image → 退出
    ├── 💾 —— 一键自动命名 img2cli_时间戳.png → write_image → 退出
    ├── 📌 —— composite → pin_image（Rust 侧建窗 + 日志）→ 退出
    └── ⬆ / Enter×2 —— capture_region(x,y,w,h,annotated?)
            ├── 裁剪（物理分辨率；annotated dataURL 直接替换裁剪结果）
            ├── 写剪贴板（arboard set_image）
            ├── 记录 last_capture_rect + capture_history（最新在前，cap 8，去重）
            └── trigger_upload_only() —— 后台 Job：路由 → 压缩 → SFTP 上传
                  └── DaemonState.last_upload = {指纹, 送达路径}

用户按注入热键 (默认 Alt+V)
    ▼
trigger_capture_and_paste() → Job 快速通道：
    peek_clipboard_image() 指纹 == last_upload.fingerprint？
    ├── 是 → 直接注入已送达路径（跳过上传）
    └── 否 → 剪贴板图像 → 压缩 → 上传 → 注入
    ▼
host_policy（标题/进程名）→ effective mode：
    ├── direct → Enigo Unicode 键入（不碰剪贴板）
    └── copy   → 路径写剪贴板 + 模拟粘贴（Orca 等拒合成输入应用）
    └── direct 失败 → fallback_to_copy 保险
```

### 关键设计决策（踩坑沉淀）
1. **ACL 陷阱**：overlay/贴图窗不在 `capabilities/default.json`（`windows:["main"]`）→ JS 端插件调用被静默拒 → 对话框/建窗一律走 **Rust 侧自定义命令**（不受 ACL 管）。
2. **CJK IME 吃键**：字母/标点 keydown 的 `e.key === "Process"` → 一律用 **`e.code`** 匹配（overlay 热键、热键录入器 + keyup 兜底）。
3. **xcap 排除自进程窗口**（WebRTC 借用的防死锁规则）→ 自窗矩形用 Tauri API 补录。
4. **常驻热窗的生命周期**：webview 会死于环境因素（WebView2/驱动相关）→ v0.4.8 用 **emits/shows 计数**判死，**下一次按键**重建（按键节奏，不做后台自动重建 —— v0.4.7 的看门狗在主线程叠建窗导致整机卡死，已回退）。
5. **平台分歧警告**：Linux 的"多余 mut/未用 import"可能是 Windows cfg 块必需 → push 前双 target 检查。
6. **`app.manage()` 必须先于任何窗口创建**（v0.4.5 回归教训）：建窗慢会让窗口 JS 抢在 manage 前调命令 → "state not managed"。

---

## 5. 模块详解

### `main.rs` — 应用入口
插件链（single-instance 首位 → autostart → global-shortcut → dialog）、托盘（左键开设置、右键菜单：Show / Restart as Admin / Exit）、窗口关闭→隐藏、setup（加载配置 → daemon → **注册热键并 Ok/Err 双路打日志** → manage → 预热浮层）。

**热键处理器**（global-shortcut）：按键 → 注入热键比对 → `trigger_capture_and_paste`；截图热键比对 → `[日志] Screenshot hotkey received` → `capture_full_screen`（失败记日志）→ `open_capture_overlay`。

**`hotkey_rejection()`**（保存前校验）：解析失败 / 含 Meta / 裸键非 F1–F12 / 黑名单（Ctrl+C,V,X,Z,S,A,F4、Alt+F4、Tab）/ 两热键相同 → 拒绝。

### IPC 命令（27 个）
| 组 | 命令 |
|---|---|
| 配置 | `get_config` · `save_config`（校验→存盘→状态→热键重注册，失败回滚+日志） |
| 日志 | `get_log_history` · `copy_logs`（成功/失败均记 daemon 日志） · `write_logs` |
| 截图 | `capture_region(x,y,w,h,annotated?)` · `cancel_capture` · `get_captured_image`（JPEG q85 + 存活打点） · `get_window_rects` · `show_capture_overlay`（shows+1 + 终点日志） |
| 图像动作 | `write_image`（裸文件名→默认目录 + 日志） · `copy_image` · `save_image_dialog`（Rust 侧另存为对话框） |
| 贴图 | `pin_image`（存图+建窗一体） · `set/get_pin_image` · `create_pin` · `close_pin` · `resize_pin` · `drag_pin` |
| SSH | `test_connection` · `load_ssh_config` · `set/clear/has_ssh_password` |
| 其它 | `nudge_cursor`（WASD，DPI 换算） |

### `config.rs` — 配置
```rust
pub struct AppConfig {
    // 输出/压缩
    output_format, compress_quality, max_dimension, wrap_single_quotes, // 默认 false
    // 热键/注入
    global_hotkey("Alt+V"), screenshot_hotkey("Alt+Shift+S"),
    injection_mode: InjectionMode,        // Auto(默认, alias "swap") | Direct | Copy(alias "paste_keep")
    fallback_to_copy: bool,               // direct 失败保险
    // 截图选项
    capture_auto_detect, capture_show_hints, capture_border_width(2),
    capture_mask_opacity(45), last_capture_rect, capture_history(Vec, cap 8),
    // 界面/系统
    theme("dracula"), language("zh-CN"), launch_on_boot, enable_notifications,
    clean_keep_days, save_dir,
    // 路由
    ssh: SshConfig(legacy 兜底), targets: Vec<TargetConfig { …, is_default }>
}
```

### `daemon.rs` — 状态与日志
```rust
pub struct DaemonState {
    running, log_history,                    // 日志：内存(cap 100) + emit + daemon.log 文件镜像
    config: Arc<RwLock<AppConfig>>,
    captured_image,                          // 冻结帧（原图，物理分辨率）
    window_rects,                            // 窗口/元素矩形快照（CSS px，Z 序，子窗紧随父窗）
    last_upload: Option<LastUpload>,         // {指纹, 送达路径} —— 注入快速通道
    pins: HashMap<u32, String>,              // 贴图 id → dataURL
    overlay_emits / overlay_shows,           // 浮层存活计数（emits>shows = 热窗已死）
}
```
`log_message()`：`[时间戳] 消息` → 内存 + `%TEMP%\img2cli/daemon.log` 追加 + `log_append` 事件。

### `job.rs` — 管线编排
有界队列（cap 8）单 worker 串行。两条路径：
- **upload-only**（截图确认后）：路由 → 压缩 → 上传 → `last_upload` 存储 + "Background upload ready" 日志。
- **inject**（注入热键）：指纹快速通道 → 否则剪贴板取图上传 → `resolve_effective_mode`（Auto 打 `[auto] host policy:` 日志）→ `inject_with_fallback`。

### `host_policy.rs` — 按应用策略
`resolve_injection_mode(标题, 进程名, 全局模式)`：规则表匹配（如 "orca" → Copy）；Auto 默认 Direct。

### `capture.rs` — 区域截图
- `capture_full_screen`：xcap 冻结 + **窗口枚举**（标题/尺寸/最小化过滤、本屏过滤、监视器矩形求交、EnumChildWindows 子元素、自窗 Tauri API 补录）+ 冻结耗时日志。
- `prewarm_capture_overlay`（启动建常驻隐藏窗，**必须在 manage 之后**）/ `open_capture_overlay`（热窗 emit / 死窗判定的按键节奏重建）。
- `capture_region`：物理分辨率裁剪或 annotated 合成替换 → 剪贴板 → 历史记录 → 后台上传。

### `injector.rs` / `clipboard.rs` / `ssh.rs` / `routing.rs` / `transport.rs` / `cli_adapter.rs` / `ssh_config.rs`
注入（direct=Enigo / copy=arboard+粘贴）；剪贴板（peek 指纹、dataURL 解码、缩放压缩）；SSH（russh SFTP 三段超时、TOFU known_hosts、钥匙串、连接池）；路由链（目标卡→ssh-config→默认 SSH→本地）；交付（SFTP/SCP/本地）；渲染（Markdown/HTML/raw + URL 转义）；OpenSSH 解析。

---

## 6. 前端（App.vue ~1900 行 + strings.js）

三个页面形态由 URL 参数决定：
- **主窗口**：三 Tab（常规设置 / 主机与目标 / 系统日志）+ 目标编辑弹窗 + SSH 导入弹窗 + toast。`configLoaded` 闸：配置加载失败禁用保存（防默认值覆盖）。
- **`?capture=1` 覆盖层**：常驻热窗，`capture-refresh` 事件驱动 `loadCaptureImage()`（重置会话）；确认态模型（✓ 前 `e.code` 十字重画，✓ 后移动）；标注引擎（值对象 + `drawObjects` 双渲染：活画布裁剪到选区 / 确认时物理分辨率合成）；笔迹分裂橡皮擦；Ctrl+C / Enter 双击语义 / Esc（捕获阶段）/ Shift+R / `,`/`.` / WASD / Tab。
- **`?pin=ID` 贴图窗**：右键菜单（复制/另存为/销毁）、拖动、滚轮缩放、双击关闭。

`strings.js`：~160 键 zh-CN 字典（键即英文原文，`t()` 缺省回退键名）。

---

## 7. 安全模型

- **SSH 密码**：永不入 config.toml；OS 钥匙串按 `user@host:port` 存储。
- **主机密钥**：TOFU —— 首连记录 known_hosts，之后校验。
- **剪贴板**：direct 模式全程不碰；copy 模式短暂覆写。
- **覆盖层**：冻结帧（不实时读屏，覆盖层不会出现在截图里）。

---

## 8. 版本管理与发版流程

**版本号位置（5 处一致）**：`src-tauri/Cargo.toml` · `src-tauri/tauri.conf.json` · `package.json` · `src/App.vue`（APP_VERSION）· `Cargo.lock`（img2cli 行，python 同步）。

**规则**：十进制补丁位（0–9，逢 10 进位）；单一 `dev` 分支开发；CI 是唯一构建门（本机不能构建，无 dbus/sudo）。

```bash
# 发版：dev → 双 target 本机检查 → push → CI 三平台绿
git push origin dev:main          # ff
git tag -a vX.Y.Z <sha> && git push origin vX.Y.Z   # release.yml 出 8 assets
```

---

## 9. 已知限制（2026-08-18）

| 限制 | 平台 | 状态/缓解 |
|---|---|---|
| 未签名 → SmartScreen/Gatekeeper | 全部 | v1.0.0 计划购证书签名 |
| **热浮层 webview 偶发死亡** | Windows | v0.4.8 按键节奏恢复（再按一次重建）+ daemon.log 取证；根因（WebView2 挂起/驱动）调查中 |
| 只支持主显示器 | 全部 | v0.4.9 多显示器 |
| IME × direct 注入打字 | Windows | 未实测；auto 模式 + per-host copy 兜底 |
| 拒合成输入应用（Orca 等） | Windows | host_policy 自动路由 copy |
| UIPI 管理员终端 | Windows | 托盘"以管理员重启" |
| Linux 截图禁用 | Linux | xcap/PipeWire 不兼容旧发行版 |
| U1–U7 用户反馈批（Tab 后自由选区/橡皮擦圆圈光标/Shift 直线/粗细调节/贴图边缘 resize/免确认另存为/秒出延迟） | — | ROADMAP 6-U，v0.4.9 批 |

---

## 10. 路线图（摘要）

- **v0.4.9（最后一个 0.4.x 槽位）**：多显示器 · SSH 保活池（<200ms）· 原生预冻结 · OCR→代码块 · 长截屏 · 6-U 用户反馈批。
- **v1.0.0**：代码签名（证书采购待决策）+ 最终打磨 + 文档。

详见 [ROADMAP.md](./ROADMAP.md)（含 §0 Road-to-v1.0.0 总表与 6-U 现场诊断全记录）。

---

## 11. 项目结构

```
img2cli/
├── src-tauri/src/
│   ├── main.rs         入口（托盘/热键/IPC/setup）
│   ├── config.rs       配置（TOML + InjectionMode）
│   ├── job.rs          JobManager 管线（上传/注入双路径）
│   ├── routing.rs      路由链
│   ├── host_policy.rs  按应用注入策略
│   ├── transport.rs    交付（SFTP/SCP/本地）
│   ├── cli_adapter.rs  输出渲染
│   ├── daemon.rs       DaemonState + 日志（面板+文件镜像）
│   ├── clipboard.rs    剪贴板 + 图像处理 + dataURL 解码
│   ├── injector.rs     注入（direct/copy）
│   ├── ssh.rs          SSH（SFTP/TOFU/钥匙串/池）
│   ├── ssh_config.rs   OpenSSH 解析
│   └── capture.rs      截图（冻结帧/热窗/窗口识别）
├── src/                Vue 3（App.vue + strings.js + main.js + index.css）
├── .github/workflows/  ci.yml（检查+测试）/ release.yml（发版）
├── docs/               HANDOFF / ISSUES / REF_mining 等过程文档
├── README.md / README_zh.md / KNOWN_ISSUES.md / ROADMAP.md / PROJECT_SUMMARY.md
└── DESIGN.md / ARCHITECTURE.md / AGENTS.md
```

---

*Document generated: 2026-08-18 | img2cli v0.4.8（上一版：v0.3.6, 2026-08-02）*
