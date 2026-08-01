# img2cli — Software Architecture Document

> **Version:** 0.3.6 | **Last Updated:** 2026-08-02  
> **Status:** Living document — update with each major version

---

## 1. 系统概述

### 1.1 定位

img2cli 是一个**系统级截图工具**，解决多模态 AI CLI（如 Claude Code、Cursor）无法直接接收粘贴图片的问题。它充当**截图 → 文本路径**的转换桥梁：截取屏幕图像，上传到远程服务器（或保存到本地），然后将文件路径以 Markdown 格式注入到当前焦点终端。

### 1.2 核心设计约束

| 约束 | 设计决策 |
|---|---|
| 不破坏用户剪贴板 | `direct` 模式用 Enigo 键盘注入，不碰剪贴板 |
| 跨平台（Win/Mac/Linux） | Tauri v2（Rust 后端 + WebView 前端） |
| 后台常驻，低资源占用 | 系统托盘 + 独立守护线程 |
| 支持密码 & 密钥 SSH 认证 | russh（密码，纯 Rust）+ 系统 ssh/scp（密钥） |
| 无需用户手动配置终端匹配 | 自动读取窗口标题 + `~/.ssh/config` 路由 |
| 内置截图，不依赖外部工具 | xcap 屏幕捕获 + 冻结帧覆盖层 |

---

## 2. 架构分层

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Presentation Layer                            │
│                                                                     │
│  ┌──────────────────┐  ┌──────────────────┐  ┌─────────────────┐  │
│  │  Settings Window  │  │  Capture Overlay  │  │  System Tray    │  │
│  │  (Vue 3 + TW)     │  │  (Vue 3 + TW)     │  │  (OS Native)    │  │
│  │                   │  │                   │  │                 │  │
│  │  • General tab    │  │  • Frozen frame   │  │  • Show Settings│  │
│  │  • Hosts tab      │  │  • Drag-select    │  │  • Restart Admin│  │
│  │  • Logs tab       │  │  • Crop & capture │  │  • Exit         │  │
│  │  • Theme picker   │  │                   │  │                 │  │
│  └────────┬─────────┘  └────────┬──────────┘  └────────┬────────┘  │
│           │ IPC (invoke)        │ IPC (invoke)          │ Menu Event │
├───────────┼─────────────────────┼──────────────────────┼───────────┤
│           │   Application Layer  │                      │            │
│           ▼                     ▼                      ▼            │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    Tauri v2 Runtime                           │   │
│  │                                                                │   │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐  │   │
│  │  │ IPC Handler  │  │ Window Mgr    │  │ Global Shortcut     │  │   │
│  │  │ (12 commands)│  │ (main/capture)│  │ (Alt+V / Alt+Shift+S│  │   │
│  │  └──────┬──────┘  └──────────────┘  └──────────┬──────────┘  │   │
│  └─────────┼──────────────────────────────────────┼─────────────┘   │
├────────────┼──────────────────────────────────────┼─────────────────┤
│            │       Domain / Service Layer           │                 │
│            ▼                                        ▼                 │
│  ┌──────────────────────┐              ┌──────────────────────┐      │
│  │   Daemon Worker       │              │   Capture Service     │      │
│  │   (std::thread)       │              │   (xcap + overlay)    │      │
│  │                       │              │                       │      │
│  │  capture → compress   │◄────注入─────│  freeze → overlay →   │      │
│  │  → route → upload     │              │  select → crop        │      │
│  │  → inject             │              │                       │      │
│  └──┬─────┬─────┬───┬───┘              └──────────────────────┘      │
│     │     │     │   │                                                 │
├─────┼─────┼─────┼───┼─────────────────────────────────────────────────┤
│     │  Infrastructure Layer                                             │
│     ▼     ▼     ▼   ▼                                                  │
│  ┌──────┐┌──────┐┌────────┐┌──────────┐                               │
│  │Clip- ││Image ││SSH/    ││Injector  │                               │
│  │board ││Process││SFTP    ││(Enigo)   │                               │
│  │(ar-  ││(image)││(russh/ ││          │                               │
│  │board)││       ││scp)    ││direct /  │                               │
│  │      ││resize ││        ││swap      │                               │
│  │get_  ││JPEG   ││upload  ││          │                               │
│  │image ││encode ││mkdir-p ││type() /  │                               │
│  │      ││crop   ││keyring ││Ctrl+V    │                               │
│  └──────┘└──────┘└────────┘└──────────┘                               │
│                                                                        │
│  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌──────────────────┐     │
│  │ Config   │  │ SSH Config│  │ Keyring  │  │ Window Title     │     │
│  │ (TOML)   │  │ Parser    │  │ (OS)     │  │ Detector         │     │
│  │          │  │           │  │          │  │ (Win32/Apple/X11)│     │
│  └──────────┘  └───────────┘  └──────────┘  └──────────────────┘     │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. 组件分解

### 3.1 组件依赖图

```
                    ┌──────────┐
                    │ main.rs  │ (入口 + 编排)
                    └────┬─────┘
          ┌──────┬──────┼──────┬──────┬──────┐
          ▼      ▼      ▼      ▼      ▼      ▼
    ┌─────┐┌─────┐┌─────────┐┌─────┐┌─────┐┌────────┐
    │conf-││clip-││ daemon  ││inject││ ssh ││capture │
    │ig   ││board││  .rs    ││or.rs││ .rs ││ .rs    │
    │ .rs ││ .rs ││         ││     ││     ││        │
    └──┬──┘└─────┘└────┬────┘└──┬──┘└──┬──┘└───┬────┘
       │               │        │       │       │
       │          ┌────┼────┐   │       │       │
       │          ▼    ▼    ▼   │       │       │
       │     ┌──────┐│┌────────┐│┌──────┐│  ┌───┴───┐
       │     │inject│││ ssh.rs │││keyring││  │ xcap  │
       │     │or.rs │││(russh) │││ (OS)  ││  │(screen)│
       │     └──────┘│└────────┘│└──────┘│  └───────┘
       │             │          │        │
       │        ┌────┘     ┌────┘   ┌────┘
       │        ▼          ▼        ▼
       │   ┌────────┐ ┌──────┐ ┌──────────┐
       └──▶│ssh_conf│ │ image│ │ arboard  │
           │ig.rs   │ │ crate│ │ (clip-   │
           └────────┘ └──────┘ │  board)  │
                               └──────────┘
```

### 3.2 组件职责

| 组件 | 文件 | 职责 | 依赖 |
|---|---|---|---|
| **App Entry** | `main.rs` | Tauri 应用生命周期、IPC 命令注册、托盘菜单、全局热键分发、窗口创建/管理、Setup 初始化 | 所有模块 |
| **Config Manager** | `config.rs` | 配置序列化/反序列化（TOML）、默认值、配置文件路径管理 | `serde`, `toml` |
| **Job Manager** | `job.rs` | 单 worker 串行任务管线 + 有界队列（cap 8）；`process_job` 编排：capture → route → deliver → format → inject。热键只入队、不阻塞 | `routing`, `transport`, `cli_adapter`, `clipboard`, `injector`, `ssh_config` |
| **Routing** | `routing.rs` | `RouteResolver` 链（手动规则 → ssh-config 自动探测 → 默认 SSH → 本地回退），产出 `DeliveryTarget`（`SshTarget`/`LocalTarget`）。**只选目标**，不读 keyring、不上传 | `config`, `ssh_config` |
| **Transport** | `transport.rs` | `ArtifactTransport`：SFTP/SCP/本地交付 + 认证分发（keyring 有密码→SFTP，否则→SCP）。keyring 三态（无密码/有密码/服务不可用）区分处理 | `routing`, `ssh`, `daemon` |
| **CLI Adapter** | `cli_adapter.rs` | `CliAdapter`：把交付路径渲染成 Markdown / HTML / raw（带 URL 转义，防 `)` 破坏 markdown）。未来 CLI 检测的 seam（v0.5） | `transport` |
| **Daemon** | `daemon.rs` | 守护状态（`DaemonState`）、日志广播、过期文件清理线程、SCP 上传引擎、活动窗口标题获取 | `tauri`, 平台 API |
| **Clipboard** | `clipboard.rs` | 剪贴板图像读取、RGBA → DynamicImage、缩放（Lanczos3）、JPEG/Base64 编码。拆成 `capture_and_save_image`（剪贴板入口）+ `process_and_save_image`（共享处理，供 CapturedArtifact 复用） | `arboard`, `image` |
| **Injector** | `injector.rs` | 文字注入到当前焦点窗口。`direct`（Enigo 键盘模拟）/ `swap`（剪贴板置换 + Ctrl+V） | `enigo`, `arboard` |
| **SSH Client** | `ssh.rs` | SSH/SFTP 连接（**三段超时**：连接/认证/传输）、密码认证（russh）、密钥认证（系统 scp）、**SFTP API mkdir**、known_hosts **TOFU**（`public_key_base64` 指纹）、连接池、钥匙串三态查询 | `russh`, `russh-sftp`, `keyring`, `tokio` |
| **SSH Config Parser** | `ssh_config.rs` | 解析 `~/.ssh/config`，提取主机条目（别名、HostName、User、Port），跳过通配符 | `serde` |
| **Screenshot Capture** | `capture.rs` | 屏幕截取（xcap）、冻结帧缓存、全屏覆盖层、区域裁剪（DPI）。区域截图直接生成 `CapturedArtifact` 入管线，**绕过剪贴板回读** | `xcap`, `tauri::WebviewWindow`, `image` |
| **Frontend** | `App.vue` | 设置面板 UI、主题系统（CSS 变量）、截图覆盖层（拖拽选区）、热键录制器、密码状态显示 | `Vue 3`, `Tailwind`, `@tauri-apps/api` |

---

## 4. 线程模型

```
┌─────────────────────────────────────────────────┐
│                 img2cli 进程                      │
│                                                  │
│  ┌──────────────┐     主线程 (Tauri 事件循环)     │
│  │  Main Thread  │                              │
│  │  • 热键监听    │     • IPC 命令处理            │
│  │  • 托盘事件    │     • 窗口管理 / Vue 渲染      │
│  │  • Setup 初始化│                              │
│  └───────┬───────┘                              │
│          │                                      │
│          │ Alt+V: 快照 config → 包成 TransferJob │
│          │   → JobManager.submit()（非阻塞）      │
│          │          │                            │
│          │          ▼                            │
│          │     ┌──────────────────┐             │
│          │     │  JobManager       │  有界队列    │
│          │     │  + 单 worker 线程 │  (cap 8)     │
│          │     │  (FIFO 串行消费)  │             │
│          │     │                   │             │
│          │     │  process_job:     │  一次只跑    │
│          │     │  capture → route  │  一个任务    │
│          │     │  → deliver        │  （注入严格  │
│          │     │  → format → inject│   串行）     │
│          │     └──────────────────┘             │
│          │                                      │
│          │ Alt+Shift+S:                         │
│          │   freeze → overlay → 拖选 →          │
│          │   CapturedArtifact → submit()         │
│          │   （绕过剪贴板回读）                    │
│          │                                      │
│  ┌───────┴───────┐                              │
│  │ Daemon Cleanup │  守护清理线程                  │
│  │ Thread         │  • 定期删除过期截图            │
│  └───────────────┘                              │
│                                                  │
│  ┌───────────────┐                              │
│  │ Tokio Runtime  │  SSH 异步运行时 (multi-thread)│
│  │ (static OnceLock)│  • russh 连接/认证/SFTP      │
│  │                │  • worker 用 block_on 桥接    │
│  └───────────────┘                              │
└─────────────────────────────────────────────────┘
```

### 设计决策：为什么用单 worker 串行 JobManager，而不是每次热键 `thread::spawn`？

全局热键回调在主线程同步执行，不能直接做网络上传（1-3s）否则冻结 UI。早期版本每次热键 `std::thread::spawn` 一个独立线程，但有两个问题：① 连按热键时多个上传并发，注入顺序乱、`inject_swap` 的剪贴板 backup/restore 互相踩；② 线程与资源不可控。

v0.3.6 改为**单 worker + 有界队列（`JobManager`）**：热键只做"快照 config → 包成 `TransferJob` → `submit()`"（非阻塞；队列满则丢弃最新并记日志），一个长期 worker 线程按 FIFO 串行处理。这样注入严格串行、剪贴板 swap 不交错、任务顺序 = 截图顺序。对 img2cli 这种人工节奏的工具，**顺序正确比并发吞吐重要**——所以第一版不做并发上传。

worker 是同步 OS 线程（不是 tokio task），因为管线主体是同步的；SSH 上传用 russh（async），通过静态 multi-thread Tokio runtime 的 `block_on()` 桥接。SFTP 三段（连接 / 认证 / 传输）各有超时，卡死的服务器不会阻塞 worker；worker 还用 `catch_unwind` 包住单个 job，一个 panic 不会拖死整条队列。

---

## 5. 通信模式

### 5.1 IPC 通信（前端 ↔ 后端）

```
Vue 前端                    Tauri IPC                   Rust 后端
│                           │                           │
│  invoke('get_config')────▶│──────────────────────────▶│  get_config()
│                           │                           │
│  ◀──── AppConfig ─────────│◀─────── AppConfig ────────│
│                           │                           │
│  invoke('save_config',    │──────────────────────────▶│  save_config()
│    {config})              │                           │  + 热键重注册
│                           │                           │
│  invoke('capture_region', │──────────────────────────▶│  capture_region()
│    {x,y,w,h})             │                           │
│                           │                           │
│  listen('log_append')◀────│◀──── emit('log_append')───│  daemon 日志推送
│                           │                           │
```

**IPC 设计原则：**
- 命令使用 `snake_case`（Rust 惯例），前端 `invoke('snake_case')`
- 复杂参数用对象包装（`{config: config.value}`）
- 错误以 `String` 返回（`Result<T, String>`），前端 `catch` 显示 toast
- 日志用**事件推送**（`emit` / `listen`），非轮询

### 5.2 事件流（后端 → 前端）

| 事件名 | 触发时机 | 载荷 | 消费者 |
|---|---|---|---|
| `log_append` | daemon 每条日志 | `String`（日志文本） | App.vue Logs 面板 |

### 5.3 窗口间通信

```
主窗口 (label: "main")          截图窗口 (label: "capture")
│                               │
│  index.html                   │  index.html?capture=1
│                               │
│  • Settings UI                │  • 全屏透明覆盖层
│  • 主题选择                   │  • 冻结帧背景
│  • Hosts 管理                 │  • 拖拽选区
│                               │
│  通过 DaemonState 共享:        │  通过 DaemonState 共享:
│  • config (RwLock)            │  • captured_image (Mutex)
│  • log_history (Mutex)        │
│                               │
└───────────────────────────────┘
     窗口间不直接通信，通过 Rust 后端共享状态
```

---

## 6. 路由架构

### 6.1 路由决策树

```
              用户按 Alt+V
                   │
                   ▼
          ┌─────────────────┐
          │ 获取窗口标题      │
          │ get_active_      │
          │ window_title()   │
          └────────┬────────┘
                   │
                   ▼
          ┌─────────────────┐     匹配
          │ ① 遍历 targets   │────────▶ 使用该 target
          │   match_pattern  │           的 host/port/user
          │   ∈ title?       │           /remote_dir
          └────────┬────────┘
             不匹配 │
                   ▼
          ┌─────────────────┐     匹配
          │ ② 读 ~/.ssh/config│────────▶ 构造 SshConfig
          │   alias/host ∈   │           (alias 作为标识)
          │   title?         │           remote_dir = 默认
          │   取最长匹配      │           或 config.ssh.remote_dir
          └────────┬────────┘
             不匹配 │
                   ▼
          ┌─────────────────┐     enabled
          │ ③ config.ssh     │────────▶ 使用默认 SSH
          │   .enabled?      │           配置
          └────────┬────────┘
             未启用 │
                   ▼
          ┌─────────────────┐
          │ ④ 本地临时路径    │────────▶ 不上传
          │   (兜底)         │           注入本地路径
          └─────────────────┘
```

### 6.2 上传策略选择

```
确定目标后:
    │
    ├── 钥匙串有密码? (get_stored_password)
    │       │
    │       ▼ YES
    │   ssh::upload_via_sftp()     ← russh (纯 Rust SSH)
    │       ├── tokio runtime
    │       ├── connect + auth_password
    │       ├── mkdir -p remote_dir (exec channel)
    │       ├── SFTP create + write
    │       └── disconnect
    │
    └── 无密码
            │
            ▼
        upload_via_scp()           ← 系统 ssh/scp (子进程)
            ├── Command::new("ssh")
            │   .arg("-p").arg(port)
            │   .arg("mkdir -p ...")
            ├── Command::new("scp")
            │   .arg("-P").arg(port)
            │   .arg(local).arg(dest)
            └── 使用默认 SSH 密钥认证
```

### 设计决策：为什么 russh + scp 双路径？

GUI 应用没有终端（TTY），系统 `ssh`/`scp` 无法弹出密码提示（`BatchMode=yes` 禁止交互）。因此：
- **密码认证** → 必须用 `russh`（程序内完成认证，不需要 TTY）
- **密钥认证** → 系统 `ssh`/`scp` 即可（密钥认证不需要交互）
- 双路径确保两种认证方式都能工作

---

## 7. 截图架构（冻结帧模式）

### 7.1 为什么用冻结帧？

**问题：** 直接在覆盖层打开后截图，覆盖层本身会出现在截图中（它是最顶层窗口）。

**朴素方案：** 关闭覆盖层 → 等待 180ms（OS 重绘）→ 截图。缺点：闪烁 + 时间不确定。

**冻结帧方案（Snipaste 风格）：**
1. 按 Alt+Shift+S 时，**先截图到内存**（此时覆盖层还没打开）
2. 然后打开覆盖层，将内存中的截图作为**背景**显示
3. 用户在"冻结的画面"上拖拽选区
4. 选区确定后，从内存中的截图**裁剪**（不需要再次截屏）

```
时间线:
───┬───────────┬───────────────────────┬─────────────┬──▶ 时间
   │           │                       │             │
   │ 按        │ 覆盖层加载             │ 用户松开     │
   │ Alt+Shift+S│ (显示冻结帧)          │ 鼠标         │
   │           │                       │             │
   ▼           │                       ▼             │
 截图到内存     │ <── 用户在冻结帧       从内存裁剪      │
 (xcap)        │     上拖拽选区 ──>     (crop_imm)    │
               │                       │             │
               │                       ▼             │
               │                     写入剪贴板       │
               │                     → trigger_      │
               │                       capture_and   │
               │                       _paste()      │
```

### 7.2 DPI 缩放

覆盖层的选区坐标是 **CSS 像素**，xcap 截图是**物理像素**。裁剪时乘以 `scale_factor`：

```rust
let scale = monitor.scale_factor().unwrap_or(1.0);  // e.g., 2.0 on Retina
let cx = (x as f32 * scale) as u32;  // CSS px → physical px
let cy = (y as f32 * scale) as u32;
let cw = (w as f32 * scale) as u32;
let ch = (h as f32 * scale) as u32;
let cropped = crop_imm(&full_image, cx, cy, cw, ch).to_image();
```

---

## 8. 主题架构

### 8.1 CSS 变量驱动

```css
/* 根元素 :style 绑定 */
:root {
  --bg-app: #08080c;              /* 应用背景 */
  --bg-sidebar: rgba(...);        /* 侧边栏 */
  --bg-card: rgba(...);           /* 卡片 */
  --color-border: rgba(...);      /* 细线边框 */
  --color-accent: #2997ff;        /* 强调色 */
  --color-text-primary: #f8fafc;  /* 主文字 */
  --color-text-secondary: #94a3b8;/* 次要文字 */
  --bg-input: #020617;            /* 输入框 */
  --bg-toggle: rgba(...);         /* 开关轨道 */
  --bg-button: #1e293b;           /* 按钮 */
  /* ... 共 15 个变量 */
}
```

**切换原理：** `currentTheme = computed(() => themes[config.theme])` → 根元素 `:style` 绑定更新 CSS 变量 → 所有 Tailwind `var(--xxx)` 引用自动适配 → **零重新渲染、零重新构建**。

### 8.2 Tailwind 集成

```html
<!-- 不是硬编码颜色，而是引用 CSS 变量 -->
<div class="bg-[var(--bg-card)] border border-[var(--color-border)]">
  <p class="text-[var(--color-text-primary)]">标题</p>
  <p class="text-[var(--color-text-secondary)]">说明</p>
</div>
```

Tailwind 的任意值语法 `bg-[var(--xxx)]` 编译为 `background-color: var(--xxx)`，运行时由 CSS 变量提供实际值。

---

## 9. 安全架构

### 9.1 密码存储

```
用户在 UI 输入密码
       │
       ▼
  set_ssh_password(user, host, port, password)
       │
       ▼
  identity_key(user, host, port) → "mengzijun@172.16.190.96:7525"
       │
       ▼
  keyring::Entry::new("img2cli", identity)
       │
       ├── Windows: DPAPI (CryptProtectData) → 加密绑当前用户
       ├── macOS:   Keychain Services → 需用户授权访问
       └── Linux:   Secret Service / gnome-keyring → 需 DBus
       │
       ▼
  entry.set_password(password)
       │
       ▼
  密码加密存储在 OS 钥匙串中
  (config.toml 中永远没有密码字段)
```

### 9.2 剪贴板安全

| 模式 | 剪贴板状态 | 风险 |
|---|---|---|
| `direct` | 全程不触碰 | 无（图片始终保留） |
| `swap` | 短暂覆写（~100ms）→ 立即恢复 | 极低（100ms 窗口内其他进程可能读取） |

### 9.3 SSH 主机密钥（TOFU）

**当前：** TOFU（Trust On First Use）。`check_server_key()` 用主机公钥的 `public_key_base64` 作为指纹：

- **首次连接** → 指纹写入 `~/.config/img2cli/known_hosts`，接受；
- **后续连接** → 指纹匹配则接受；
- **指纹变化** → 拒绝（疑似 MITM）。

**剩余风险：** 首次连接仍无条件信任（无带外验证），但对 img2cli 这种内网/已知服务器场景足够。未来可对接系统 `~/.ssh/known_hosts`，或提供指纹确认 UI。

---

## 10. 平台抽象层

### 10.1 条件编译策略

```rust
// xcap 截图：仅 Windows/macOS
#[cfg(any(target_os = "windows", target_os = "macos"))]
fn do_capture(...) { /* xcap Monitor::all().capture_image() */ }

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn do_capture(...) { Err("not supported on this platform") }
```

### 10.2 平台差异表

| 功能 | Windows | macOS | Linux | 抽象方式 |
|---|---|---|---|---|
| 窗口标题获取 | Win32 `GetForegroundWindow` | `osascript` 子进程 | `xdotool` 子进程 | `get_active_window_title()` + `cfg` |
| 键盘注入 | `SendInput` (Win32) | CoreGraphics 事件 | XTest 扩展 | Enigo 统一接口 |
| 剪贴板 | Win32 API | NSPasteboard | X11 PRIMARY/CLIPBOARD | arboard 统一接口 |
| 钥匙串 | Credential Manager / DPAPI | Keychain | Secret Service (DBus) | keyring crate 统一接口 |
| 屏幕截图 | DXGI/Desktop Duplication | CGDisplay / ScreenCaptureKit | X11 GetImage | xcap crate（Linux 禁用） |
| 提权重启 | `ShellExecuteW(verb="runas")` | N/A | N/A | `#[cfg(windows)]` |
| 全局热键 | `RegisterHotKey` | Carbon Event Tap | X11 GrabKey | tauri-plugin-global-shortcut |

### 10.3 Cargo 平台特定依赖

```toml
[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.59", features = ["Win32_UI_Shell", ...] }
keyring = { version = "3", features = ["windows-native"] }

[target.'cfg(target_os = "macos")'.dependencies]
keyring = { version = "3", features = ["apple-native"] }

[target.'cfg(target_os = "linux")'.dependencies]
keyring = { version = "3", features = ["sync-secret-service", "crypto-rust"] }

[target.'cfg(any(target_os = "windows", target_os = "macos"))'.dependencies]
xcap = "0.5"
```

---

## 11. 关键设计决策记录

### ADR-01: 为什么选 Tauri v2 而不是 Electron？

| 维度 | Tauri v2 | Electron |
|---|---|---|
| 二进制大小 | ~3-5 MB | ~80-150 MB |
| 内存占用 | ~25 MB idle | ~100-200 MB |
| 后端语言 | Rust（内存安全、高性能） | Node.js |
| WebView | 系统 WebView2/WKWebView | 内嵌 Chromium |
| 安全 | 默认 CSP、权限系统 | 需额外配置 |

img2cli 是一个后台常驻工具，低内存 + 小体积是关键需求。Tauri 的 Rust 后端也适合做 SSH/截图等系统级操作。

### ADR-02: 为什么用 russh + scp 双路径？

GUI 应用无 TTY，系统 `ssh`/`scp` 的 `BatchMode=yes` 禁止密码交互。因此：
- 密码认证 → `russh`（纯 Rust，程序内完成，不依赖 TTY）
- 密钥认证 → 系统 `ssh`/`scp`（子进程，密钥认证无需交互）

### ADR-03: 为什么截图用"冻结帧"而不是"截图后关闭覆盖层"？

| 方案 | 优点 | 缺点 |
|---|---|---|
| 先截后关（朴素） | 实现简单 | 闪烁、覆盖层可能出现在截图中、时间不确定 |
| 冻结帧（Snipaste） | 零闪烁、覆盖层永不在截图中 | 内存占用（缓存整屏） |

### ADR-04: 为什么热键处理用 `std::thread` 而不是 `async`？

热键回调在主线程同步执行，不能直接做网络上传。v0.3.6 用一个**长期单 worker 线程**（`JobManager`）串行消费有界队列，而不是每次热键 spawn 新线程或改用 async：连按热键时队列保证 FIFO、注入严格串行、剪贴板 swap 不交错（详见 §4）。worker 是同步线程（不是 async task），SSH 上传通过静态 multi-thread Tokio runtime 的 `block_on()` 桥接。

### ADR-05: 为什么 xcap 在 Linux 上被禁用？

xcap 0.5 的 Linux 后端依赖 PipeWire + libspa。`libspa-0.8.0` 的 Rust 绑定引用了 `spa_video_info_raw.flags` 字段，但 Ubuntu 22.04 的系统 libspa 版本较旧，没有该字段 → 编译失败。解决方案是将 xcap 限制为 `cfg(any(windows, macos))`。

---

## 12. 扩展点

### 12.1 新增主题

在 `App.vue` 的 `themes` 对象中添加：
```javascript
'my-theme': {
  bgApp: '#...', bgSidebar: '...', bgCard: '...',
  colorBorder: '...', colorAccent: '#...', colorAccentHover: '...',
  colorAccentDim: '...', textPrimary: '#...', textSecondary: '#...',
  bgInput: '#...', colorInputBorder: '...',
  bgToggle: '...', colorToggleKnob: '#...',
  bgButton: '...', bgButtonHover: '...'
}
```

### 12.2 新增输出格式

在 `cli_adapter.rs` 新增一个 `CliAdapter` 实现，或扩展 `adapter_for()` 的格式分发；新的图像编码（如 WebP）加在 `clipboard.rs` 的 `process_and_save_image()`。渲染逻辑现在集中在 `cli_adapter`，不再散落在 daemon.rs。

### 12.3 新增平台支持

1. 在 `capture.rs` 中为新平台添加 `#[cfg(target_os = "xxx")]` 分支。
2. 在 `Cargo.toml` 中配置平台特定的 keyring 后端。
3. 在 `daemon.rs` 的 `get_active_window_title()` 中添加新平台的窗口标题获取。
4. 在 `release.yml` 的矩阵中添加新平台。

### 12.4 新增 Tauri IPC 命令

1. 在 `main.rs` 中定义 `#[tauri::command] fn my_command(...) -> Result<T, String>`。
2. 在 `invoke_handler` 的 `generate_handler![]` 中注册。
3. 前端调用 `await invoke('my_command', { ... })`。

---

## 13. 技术债务

| 项目 | 描述 | 优先级 | 计划 |
|---|---|---|---|
| 未签名二进制 | SmartScreen/Gatekeeper 拦截 | 高 | 购买代码签名证书 |
| SSH 连接池保活 | warm 连接已复用，但无周期心跳 | 低 | 加 keep-alive ping 循环 |
| `upload_strategy` 未实现 | 配置项声明但未使用 | 低 | eager（复制即上传）模式 |
| Wayland 窗口标题 | 无法获取其他窗口标题 | 低 | D-Bus Screenshot Portal |
| Linux 截图 | xcap PipeWire 不兼容 | 低 | 尝试 X11-only 回退 |
| 代码未签名 → 360 误报 | 木马误报 | 高 | 代码签名 |

---

*Document version: 0.3.6 | Last updated: 2026-08-02 | Maintainer: Mengzijun*
