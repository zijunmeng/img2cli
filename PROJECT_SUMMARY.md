# img2cli — Project Summary

> **Version:** 0.3.6 | **Date:** 2026-08 | **Status:** Active Development

## 1. 项目概述

`img2cli` 是一个**跨平台系统托盘桌面应用**，专为多模态 AI 工作流设计。核心功能：将截图转换为 Markdown 图片路径并注入到终端 / AI CLI，同时保持剪贴板中的原始图片不变（可继续粘贴到微信 / Word 等聊天软件）。

### 核心价值
- **截图 → 路径注入**：按一个热键（Alt+V），截图自动上传到服务器并注入 Markdown 路径到终端。
- **内置截图**：按 Alt+Shift+S，拖拽选区直接截屏（Snipaste 风格），无需第三方截图工具。
- **不抢剪贴板**：`direct` 注入模式下全程不碰剪贴板，同一张截图既能贴终端（出路径）也能贴微信（出图）。
- **SSH 密码/密钥**：密码存 OS 钥匙串（Xshell 式），密钥走系统 ssh，自动路由到正确的服务器。

---

## 2. 版本演进

| 版本 | 关键特性 |
|---|---|
| v0.1.x | 纯 CLI 守护进程（Rust），命令行配置，scp 上传 |
| v0.2.x | 交互式菜单，多目标路由（窗口标题匹配），单引号包裹 |
| v0.3.0 | **Tauri v2 GUI 重写**：托盘常驻、Vue 3 设置面板、毛玻璃 UI |
| v0.3.1 | SSH config 导入、跨终端自动路由、Set-as-Default |
| v0.3.2 | 热键录制器、表格去滚动、密码保存状态指示器 |
| v0.3.3 | 热键录制逻辑修复、橙色激活标签页、背景光晕移位 |
| v0.3.4 | **内置区域截图**（xcap + 冻结帧覆盖层）、可配置截图热键 |
| v0.3.5 | **主题系统**（6 套）、**冻结帧截图**、**SSH 连接池**（keep-alive） |
| **v0.3.6** | **架构重构**：CapturedArtifact 统一模型（区域截图绕过剪贴板回读）、有界 JobManager 单 worker 串行管线、四大边界（RouteResolver / ArtifactTransport / CliAdapter / 注入）、SFTP 三段超时 + API mkdir、known_hosts TOFU、URL 转义、路由/传输/任务顺序测试 |

---

## 3. 技术栈

### 后端 (Rust)
| 依赖 | 版本 | 用途 |
|---|---|---|
| `tauri` | 2.0.0 | 应用框架（窗口、托盘、IPC、事件） |
| `tauri-plugin-autostart` | 2.0.0 | 开机自启 |
| `tauri-plugin-global-shortcut` | 2.0.0 | 全局热键注册 |
| `tauri-plugin-dialog` | 2.7.1 | 文件选择对话框（浏览 SSH config） |
| `russh` | 0.51 | 纯 Rust SSH 客户端（密码认证 + SFTP） |
| `russh-sftp` | 2.1 | SFTP 文件传输 |
| `xcap` | 0.5 | 跨平台屏幕捕获（Win/Mac only） |
| `enigo` | 0.5.0 | 键盘输入模拟（Unicode 注入） |
| `arboard` | 3.4.1 | 剪贴板读写（图片 + 文本） |
| `keyring` | 3.x | OS 钥匙串（Win Credential Manager / Mac Keychain / Linux Secret Service） |
| `image` | 0.25 | 图像处理（缩放、JPEG 编码、裁剪） |
| `rdev` | 0.5.3 | 键盘事件监听（全局快捷键） |
| `tokio` | 1.x | 异步运行时（russh + 连接池） |
| `serde` / `toml` | 1.0 / 0.8 | 序列化 + 配置文件 |

### 前端 (Vue 3)
| 依赖 | 版本 | 用途 |
|---|---|---|
| `vue` | ^3.4 | 响应式 UI 框架 |
| `vite` | ^5.4 | 构建工具 + 开发服务器 |
| `tailwindcss` | ^3.4 | 原子化 CSS |
| `@tauri-apps/api` | ^2.0 | Tauri 前端 API（invoke / event / window） |
| `@tauri-apps/plugin-dialog` | ^2.0 | 文件选择对话框 |

### CI/CD
- **GitHub Actions** + `tauri-apps/tauri-action@v1`
- 三平台矩阵构建：`ubuntu-22.04`, `macos-latest`, `windows-latest`
- macOS universal binary（`--target universal-apple-darwin`）
- Windows 便携 zip（`Compress-Archive`）

---

## 4. 架构概览

```
┌──────────────────────────────────────────────────────────────────┐
│                        img2cli 应用进程                            │
│                                                                  │
│  ┌─────────────┐    IPC (invoke)    ┌──────────────────────┐     │
│  │  Vue 3 前端  │◄──────────────────▶│   Rust 后端 (Tauri)   │     │
│  │  (WebView)  │    事件 (listen)    │                      │     │
│  │             │                    │  ┌──────────────────┐ │     │
│  │ • Settings  │                    │  │  Tauri Commands   │ │     │
│  │ • Theme     │                    │  │  get_config       │ │     │
│  │ • Hosts Mgr │                    │  │  save_config      │ │     │
│  │ • Logs      │                    │  │  test_connection  │ │     │
│  │ • Capture   │                    │  │  load_ssh_config  │ │     │
│  │   Overlay   │                    │  │  set/get/clear_pw │ │     │
│  └─────────────┘                    │  │  capture_region    │ │     │
│                                     │  └──────────────────┘ │     │
│                                     │                        │     │
│                                     │  ┌──────────────────┐ │     │
│                                     │  │  Daemon Thread    │ │     │
│                                     │  │  (capture→inject) │ │     │
│                                     │  └──────────────────┘ │     │
│                                     │                        │     │
│                                     │  ┌──────────────────┐ │     │
│                                     │  │  System Tray      │ │     │
│                                     │  │  + Global Hotkeys │ │     │
│                                     │  └──────────────────┘ │     │
│                                     └──────────────────────┘     │
└──────────────────────────────────────────────────────────────────┘
```

### 核心数据流（Alt+V 粘贴路径）

```
用户按 Alt+V
    │
    ▼
daemon::trigger_capture_and_paste()
    │
    ├──▶ 1. clipboard::capture_and_save_image()
    │       ├── 读取剪贴板图片 (arboard::get_image)
    │       ├── 按最大尺寸缩放 (image::resize, Lanczos3)
    │       └── JPEG 编码保存到本地临时目录
    │
    ├──▶ 2. daemon::route() — 路由决策
    │       ├── get_active_window_title() — 获取前台窗口标题
    │       ├── ① 遍历 config.targets → match_pattern 匹配
    │       ├── ② 读 ~/.ssh/config → 标题包含别名/主机名?
    │       ├── ③ config.ssh.enabled → 默认 SSH
    │       └── ④ 以上都不匹配 → 本地路径
    │
    ├──▶ 3. 上传（如果有 SSH 目标）
    │       ├── 密码在钥匙串? → ssh::upload_via_sftp() (russh SFTP)
    │       │   ├── 建立连接（或复用连接池）
    │       │   ├── 认证（password）
    │       │   ├── mkdir -p 远程目录
    │       │   ├── SFTP 上传文件
    │       │   └── 返回远程路径
    │       └── 无密码 → daemon::upload_via_scp() (系统 scp)
    │           ├── ssh mkdir -p 远程目录
    │           └── scp 上传
    │
    ├──▶ 4. 格式化输出
    │       ├── markdown → ![image](/path/to/img.jpg)
    │       ├── html → <img src="/path/to/img.jpg" />
    │       ├── raw → /path/to/img.jpg
    │       └── base64 → data:image/jpeg;base64,...（跳过上传）
    │
    └──▶ 5. injector::inject_text()
            ├── direct 模式 → Enigo 逐字符 Unicode 键入
            └── swap 模式 → 备份剪贴板 → 写路径 → Ctrl+V → 恢复剪贴板
```

### 核心数据流（Alt+Shift+S 截图捕获）

```
用户按 Alt+Shift+S
    │
    ▼
capture::capture_full_screen() — 截取主屏幕到内存 (xcap)
    │
    ▼
capture::open_capture_overlay() — 打开全屏透明覆盖层窗口
    │
    ▼
Vue 覆盖层加载 (index.html?capture=1)
    │
    ├── get_captured_image() → 返回 base64 PNG → 覆盖层显示冻结帧
    │
    ▼
用户拖拽选区 → mouseup
    │
    ▼
capture::capture_region(x, y, w, h)
    ├── 关闭覆盖层
    ├── 从内存中的全屏截图裁剪选区（× DPI scale factor）
    ├── 写入剪贴板 (arboard::set_image)
    └── daemon::trigger_capture_and_paste() — 走上面的标准流程
```

---

## 5. 模块详解

### `main.rs` — 应用入口

**职责：** Tauri 应用初始化、系统托盘、全局热键、IPC 命令注册、窗口管理。

**关键函数：**
- `main()` — Builder 链：注册插件（autostart / global-shortcut / dialog）、设置托盘菜单、窗口事件拦截（close → hide）、setup 闭包（加载配置 → 启动 daemon → 注册快捷键）
- `restart_as_admin()` (Windows) — `ShellExecuteW(verb="runas")` 重启提权
- `recordHotkeyKeydown()` — 前端热键录制器（按键捕获 → 组合键字符串）

**IPC 命令（8 个）：**
| 命令 | 参数 | 返回 | 功能 |
|---|---|---|---|
| `get_config` | — | `AppConfig` | 读取配置 |
| `save_config` | `config: AppConfig` | `()` | 保存配置 + 热键重注册 |
| `get_log_history` | — | `Vec<String>` | 获取日志历史 |
| `test_connection` | `host, port, username, password` | `String` | 测试 SSH 连接（密码/密钥） |
| `load_ssh_config` | `path: Option<String>` | `Vec<SshHostEntry>` | 解析 OpenSSH config |
| `set_ssh_password` | `user, host, port, password` | `()` | 存密码到钥匙串 |
| `clear_ssh_password` | `user, host, port` | `()` | 从钥匙串删密码 |
| `has_ssh_password` | `user, host, port` | `bool` | 检查钥匙串是否有密码 |

**额外命令（capture.rs）：**
| 命令 | 参数 | 返回 | 功能 |
|---|---|---|---|
| `capture_full_screen` | — | `()` | 截取全屏到内存（截图热键触发） |
| `get_captured_image` | — | `String` (base64) | 获取内存中的截图（覆盖层加载用） |
| `capture_region` | `x, y, w, h` | `()` | 裁剪选区 → 剪贴板 → 触发注入 |
| `cancel_capture` | — | `()` | 取消截图（关闭覆盖层） |

### `config.rs` — 配置管理

**数据结构：**

```rust
pub struct AppConfig {
    pub save_dir: Option<PathBuf>,           // 自定义临时目录
    pub output_format: String,               // markdown/html/raw/base64
    pub compress_quality: u8,                // JPEG 质量 10-100
    pub max_dimension: Option<u32>,          // 最大宽/高
    pub workspace_aware: bool,               // 工作区感知（保留字段）
    pub wrap_single_quotes: bool,            // 单引号包裹
    pub launch_on_boot: bool,                // 开机自启
    pub enable_notifications: bool,          // 桌面通知
    pub global_hotkey: String,               // 粘贴热键 "Alt+V"
    pub screenshot_hotkey: String,           // 截图热键 "Alt+Shift+S"
    pub upload_strategy: String,             // eager/lazy（保留字段）
    pub injection_mode: String,              // direct/swap
    pub clean_keep_days: u32,                // 自动清理天数
    pub theme: String,                       // UI 主题
    pub ssh: SshConfig,                      // 默认 SSH 配置
    pub targets: Vec<TargetConfig>,          // 路由目标列表
}

pub struct SshConfig {
    pub enabled: bool,
    pub host: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub remote_dir: String,
    pub match_pattern: Option<String>,
    pub remember_password: bool,
}

pub struct TargetConfig {
    pub enabled: bool,
    pub r#type: String,            // "ssh" or "local"
    pub match_pattern: String,     // 窗口标题匹配关键词
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub remote_dir: Option<String>,
    pub local_dir: Option<String>,
    pub remember_password: Option<bool>,
}
```

**存储：** TOML 格式，位于 `~/.config/img2cli/config.toml`（Linux/Mac）或 `%APPDATA%\img2cli\config.toml`（Windows）。

### 管线架构（v0.3.6 重构）

热键触发 → `job.rs` 的 `JobManager` 入队（有界 cap 8、非阻塞）→ 单 worker 串行执行 `process_job`。

**编排 `job.rs`：** capture → route → deliver → format → inject（薄编排器，不再一把抓）。

**路由 `routing.rs`（`RouteResolver` 链）：**
1. 手动 targets（`match_pattern` 匹配窗口标题）
2. ssh-config 自动检测（`~/.ssh/config` 主机别名/主机名出现在标题）
3. 默认 SSH（`config.ssh.enabled`）
4. 本地回退（`LocalFallback`，总是命中，保证有目标）

**交付 `transport.rs`（`ArtifactTransport`）：**
- keyring 有密码 → `ssh::upload_via_sftp()`（russh SFTP，三段超时，连接池）
- 无密码 → `upload_via_scp()`（系统 scp，走 SSH 密钥）
- 本地 → 文件拷贝（自拷贝守卫）
- keyring 服务不可用 → 结构化错误（**不**静默回退到密钥认证）

**渲染 `cli_adapter.rs`（`CliAdapter`）：** 交付路径 → Markdown / HTML / raw（带 URL 转义，防 `)` 破坏 markdown）。

**守护 `daemon.rs`：** 状态（`DaemonState`）、日志广播、过期文件清理线程、SCP 上传引擎、活动窗口标题获取——不再承载业务管线。

**DaemonState（共享状态）：**
```rust
pub struct DaemonState {
    pub config: Arc<RwLock<AppConfig>>,
    pub log_history: Arc<Mutex<Vec<String>>>,
    pub captured_image: Arc<Mutex<Option<RgbaImage>>>,  // 截图冻结帧缓存
}
```

### `ssh.rs` — SSH 客户端

**功能：**
- **密码认证 SFTP 上传**（russh）：`upload_via_sftp_async()` — 连接 → 认证 → mkdir -p → SFTP put
- **连接池**：静态 tokio runtime + `CachedConnection`（host/port/user/handle），避免每次握手
- **钥匙串管理**：`store_password()` / `get_stored_password()` / `clear_password()` / `has_stored_password()`
- **连接测试**：`test_password_async()` — 连接 + 认证 + 断开
- **身份键**：`identity_key(user, host, port)` → `"user@host:port"` 作为钥匙串条目名

**keyring 后端（按平台）：**
- Windows: `windows-native`（Credential Manager / DPAPI）
- macOS: `apple-native`（Keychain）
- Linux: `sync-secret-service`（gnome-keyring / KWallet，需 libdbus）

### `ssh_config.rs` — OpenSSH 配置解析器

**功能：** 解析 `~/.ssh/config` 文本，提取可连接的主机条目。

**解析规则：**
- `Host` 行：跳过通配符（`*` / `?`）和取反（`!`）模式
- 键不区分大小写（`HostName` = `hostname`）
- 多模式 Host 行（`Host A B`）→ 每个非通配符模式独立条目
- 无 `HostName` 时用别名作为主机名
- 端口默认 22，用户名默认 `$USER` / `$USERNAME`
- 支持 `~` 路径展开（自定义路径加载）

### `clipboard.rs` — 剪贴板 + 图像处理

**核心函数：** `capture_and_save_image(config, dest_path) -> Result<String>`

**流程：**
1. `arboard::Clipboard::new()` → 打开剪贴板
2. `clipboard.get_image()` → 读取 RGBA 图像数据
3. 按 `max_dimension` 等比缩放（Lanczos3 滤波器）
4. 按 `compress_quality` JPEG 编码
5. 保存到 `dest_path`
6. 按 `output_format` 格式化返回路径字符串

### `injector.rs` — 文字注入

**两种模式：**

**`direct` 模式（默认）：**
- 使用 Enigo 的 `text()` 方法模拟 Unicode 键盘输入
- 逐字符"键入"到当前焦点窗口
- **不碰剪贴板** —— 图片始终保留
- 平台：Windows（SendInput）、macOS（CoreGraphics）、Linux（XTest）

**`swap` 模式：**
- 备份当前剪贴板内容
- 将 Markdown 路径写入剪贴板
- 模拟 `Ctrl+V`（或 macOS `Cmd+V`）粘贴
- 恢复原始剪贴板内容
- 适合 `direct` 模式丢字符的场景（IME 干扰）

### `capture.rs` — 截图区域捕获

**平台：** Windows + macOS（Linux 禁用，xcap 的 PipeWire/libspa 后端不兼容 Ubuntu 22.04）

**冻结帧流程（Snipaste 风格）：**
1. `capture_full_screen()` — xcap 截取主显示器到 `DaemonState.captured_image`（内存缓存）
2. `open_capture_overlay()` — 创建全屏透明窗口（`index.html?capture=1`）
3. Vue 覆盖层：加载冻结帧（`get_captured_image()` → base64 PNG）→ 拖拽选区
4. `capture_region(x, y, w, h)` — 从内存裁剪选区（× scale_factor DPI 缩放）→ 写剪贴板 → 触发标准注入流程

---

## 6. 主题系统

### CSS 变量

```css
:root {
  --bg-app, --bg-sidebar, --bg-card,    /* 背景层 */
  --color-border,                        /* 边框 */
  --color-accent, --color-accent-hover, --color-accent-dim,  /* 强调色 */
  --color-text-primary, --color-text-secondary,  /* 文字 */
  --bg-input, --color-input-border,      /* 输入框 */
  --bg-toggle, --color-toggle-knob,      /* 开关 */
  --bg-button, --bg-button-hover;        /* 按钮 */
}
```

### 6 套主题

| 主题 | 背景 | 强调色 | 风格 |
|---|---|---|---|
| `apple-dark` | `#08080c` | `#2997ff` | Apple 暗色 |
| `apple-light` | `#e9ebef` | `#0071e3` | Apple 浅色 |
| `dracula` | `#282a36` | `#bd93f9` | Dracula |
| `nord` | `#2e3440` | `#88c0d0` | Nord |
| `gruvbox` | `#282828` | `#fe8019` | Gruvbox |
| `cyberpunk` | `#0f0f1b` | `#ff007f` | Cyberpunk |

所有 Tailwind 类名使用 `var(--xxx)` 引用这些变量，切换主题时自动适配。

---

## 7. 安全模型

### SSH 密码存储
- **永不写入配置文件**（config.toml 中没有密码字段）
- 加密存储在 OS 钥匙串：
  - Windows: DPAPI 加密（绑当前 Windows 用户）
  - macOS: Keychain（需用户授权访问）
  - Linux: Secret Service / gnome-keyring（需 DBus）
- 按主机身份键 `user@host:port` 独立存储
- 前端通过 `has_ssh_password` 检查状态（不返回密码本身）

### 剪贴板安全
- `direct` 模式：全程不碰剪贴板（图片始终保留）
- `swap` 模式：短暂覆写 → 模拟粘贴 → 立即恢复（~100ms 窗口）
- 截图覆盖层使用冻结帧（不实时读取屏幕，防止覆盖层出现在截图中）

### 主机密钥验证
- 当前实现：**接受所有主机密钥**（`check_server_key() → Ok(true)`，等同于 `StrictHostKeyChecking=no`）
- 这是为了简化首次连接体验，但存在 MITM 风险
- TODO：实现 known_hosts 验证

---

## 8. CI/CD

### GitHub Actions (`release.yml`)

**触发：** `push` 到 `v*` 标签

**矩阵：** `ubuntu-22.04` / `macos-latest` / `windows-latest`（`fail-fast: false`）

**步骤：**
1. Checkout
2. Setup Node.js (LTS)
3. Install Rust toolchain (stable) + macOS universal targets
4. Ubuntu: apt 安装系统依赖（GTK, WebKit, dbus, X11, EGL, PipeWire）
5. `npm install`
6. `tauri-apps/tauri-action@v1` — 构建 + 创建/更新 GitHub Release
   - `tagName: v__VERSION__`（从 tauri.conf.json 版本号解析）
   - 上传 .msi / .exe / .deb / .rpm / .AppImage / .dmg
7. Windows: 额外打包便携 zip（`Compress-Archive img2cli.exe`）
8. Windows: 上传便携 zip 到 Release

### Ubuntu 额外依赖
```
libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
libdbus-1-dev libxcb1-dev libxext-dev libxfixes-dev libxrandr-dev libegl-dev
libgl1-mesa-dev libpipewire-0.3-dev
```

---

## 9. 版本管理

### 版本号位置（必须一致）
- `src-tauri/Cargo.toml` → `version = "0.3.6"`
- `src-tauri/tauri.conf.json` → `"version": "0.3.6"`
- `package.json` → `"version": "0.3.6"`
- `src/App.vue` 侧边栏 → `Settings v0.3.6`

### 发版流程
```bash
# 1. 修改版本号（4 处）
# 2. 提交
git add -A && git commit -m "release: vX.Y.Z — ..."
# 3. 打标签
git tag vX.Y.Z
# 4. 推送
git push origin <branch>
git push origin vX.Y.Z
# 5. GitHub Actions 自动构建 + 发布
```

---

## 10. 已知限制

| 限制 | 平台 | 原因 | 缓解 |
|---|---|---|---|
| 未签名 → SmartScreen/Gatekeeper | 全部 | 无代码签名证书 | 右键打开 / xattr / 加入信任区 |
| IME 干扰注入 | Windows | 中文输入法吃掉首字符 | 切换 swap 模式 |
| UIPI 限制 | Windows | 无法注入到管理员终端 | 托盘"以管理员重启" |
| 截图不可用 | Linux | xcap 的 PipeWire/libspa 后端不兼容旧发行版 | 用系统截图 + Alt+V |
| Wayland 窗口标题 | Linux | Wayland 安全策略不允许读取其他窗口 | 回退到默认主机/本地 |
| macOS 无托盘图标 | macOS | `default_window_icon()` 返回 None | 窗口可见（visible: true），通过窗口操作 |
| 接受所有主机密钥 | 全部 | 简化首次连接 | TODO: known_hosts 验证 |
| SFTP 连接无持久化 | 全部 | 每次 Alt+V 新建连接 | 连接池已部分实现 |

---

## 11. 未来路线图（来自 ROADMAP.md）

1. **代码签名**（Windows OV + macOS Developer ID + Notarization）—— 彻底解决 SmartScreen/Gatekeeper
2. **SSH 连接池完善** —— keep-alive 心跳 + 连接复用，延迟从 1.5s 降到 <200ms
3. **本地 OCR** —— 截图自动转 Markdown 文本（Windows OCR / macOS Vision），节省 90% API token
4. **零信任扫描** —— 检测截图中的 API Key / 密码 / 内网 IP，提示遮挡
5. **标注覆盖层** —— 截图选区内绘制箭头、高亮、马赛克
6. **屏幕贴图** —— 截图钉在桌面上（类 Snipaste 贴图功能）

---

## 12. 开发指南

### 本地开发

```bash
# 安装依赖
npm install

# 开发模式（热重载）
npm run tauri dev

# 生产构建
npm run tauri build

# 仅构建前端（验证 Vue 编译）
npm run build
```

### 前端预览（无 Tauri 后端）

```bash
npm run dev
# 打开 http://localhost:1420
# 主题预览: http://localhost:1420/?theme=apple-light
```

### 截图覆盖层预览

```
http://localhost:1420/?capture=1
```

### 项目结构

```
img2cli/
├── src-tauri/              Rust 后端
│   ├── src/
│   │   ├── main.rs         入口（IPC + 托盘 + 热键）
│   │   ├── config.rs       配置（TOML）
│   │   ├── job.rs          JobManager + 串行管线编排
│   │   ├── routing.rs      RouteResolver 路由链
│   │   ├── transport.rs    ArtifactTransport 交付 + 认证分发
│   │   ├── cli_adapter.rs  CliAdapter 渲染（Markdown/HTML/raw）
│   │   ├── daemon.rs       守护状态 + 辅助 + SCP 引擎
│   │   ├── clipboard.rs    剪贴板 + 图像处理
│   │   ├── injector.rs     文字注入
│   │   ├── ssh.rs          SSH 客户端 + 钥匙串 + TOFU
│   │   ├── ssh_config.rs   OpenSSH 解析器
│   │   └── capture.rs      截图捕获（冻结帧）
│   ├── Cargo.toml          Rust 依赖
│   ├── tauri.conf.json     Tauri 配置
│   ├── capabilities/       Tauri 权限
│   └── icons/              应用图标
├── src/                    Vue 前端
│   ├── App.vue             主组件（设置 + 主题 + 截图覆盖层）
│   ├── main.js             Vue 启动
│   └── index.css           样式
├── .github/workflows/      CI/CD
│   ├── ci.yml              per-push 编译校验（含测试）
│   └── release.yml         tag 触发的发布工作流
├── package.json            npm 依赖
├── DESIGN.md               设计规范（Apple 风格 token）
├── KNOWN_ISSUES.md         已知问题
├── ROADMAP.md              路线图
├── README.md               英文文档
└── README_zh.md            中文文档
```

---

*Document generated: 2026-08-02 | img2cli v0.3.6*
