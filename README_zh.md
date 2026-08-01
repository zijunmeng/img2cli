# img2cli

[English](./README.md) | 简体中文

**把截图以 Markdown 图片链接的形式粘贴进任何 AI CLI —— 而且不破坏你剪贴板里的图片。**

`img2cli` 是一个跨平台的**系统托盘桌面应用**（Rust + Tauri v2 + Vue 3），为多模态 AI 工作流而生。截图（内置区域截图或系统截图工具）→ 聚焦终端 → 按 **Alt+V**，图片的 Markdown 路径就会被注入终端。而图片本身仍留在剪贴板里，你照样能用 **Ctrl+V** 把原图贴进微信 / Word / 飞书。

## 下载

| 系统 | 文件 | 说明 |
|---|---|---|
| **Windows**（安装版） | `img2cli_0.3.6_x64-setup.exe` / `_x64_en-US.msi` | |
| **Windows**（免安装版） | `img2cli-v0.3.6-windows-portable.zip` | 解压即用 |
| **macOS**（通用版） | `img2cli_0.3.6_universal.dmg` | 支持 M1/M2/M3 + Intel |
| **Linux** | `img2cli_0.3.6_amd64.deb` / `.rpm` / `.AppImage` | 截图功能暂不可用（见平台说明） |

→ **[GitHub Releases](https://github.com/zijunmeng/img2cli/releases)**

> ⚠️ **二进制未签名。** 首次启动时：
> - **Windows：** SmartScreen → *更多信息 → 仍要运行*；或加入杀软信任区。
> - **macOS：** 右键 `img2cli.app` → *打开* → 确认。然后在 *系统设置 → 隐私与安全性* 中授权 **辅助功能** + **屏幕录制**。

---

## 快速上手

1. **安装** —— 下载对应系统的安装包，安装 / 拖到应用程序。
2. **截图** —— 按 **Alt+Shift+S**（内置区域截图：拖拽框选），或用系统截图工具（Win+Shift+S / macOS 截图）。
3. **粘贴到终端** —— 把焦点切到终端 / AI CLI，按 **Alt+V**，Markdown 路径 `![image](/远程/路径.jpg)` 自动注入。
4. **贴图到聊天** —— 在微信 / Word 里按 **Ctrl+V**，原图照常粘贴（`direct` 模式不碰剪贴板）。

---

## 功能特性

### 📸 内置区域截图
- 专用**截图热键**（默认 `Alt+Shift+S`）打开全屏覆盖层。
- **冻结画面**：截图前先抓取屏幕到内存（零闪烁），覆盖层显示冻结帧供选区。
- **拖拽框选**区域（Snipaste 风格）。松开 → img2cli 裁剪、压缩、上传、注入。
- 基于 [`xcap`](https://crates.io/crates/xcap)（Windows / macOS）。

### 🔄 不抢剪贴板注入
- **`direct` 模式**（默认）：通过 [Enigo](https://crates.io/crates/enigo) 模拟原生 Unicode 键盘输入，绕过输入法，**全程不碰剪贴板**。
- **`swap` 模式**：备份剪贴板 → 写入路径 → 模拟 Ctrl+V → 恢复剪贴板。适合 `direct` 模式丢字符（中文输入法干扰）时使用。

### 🔐 SSH 密码 + 密钥登录
- **密码认证**：密码存在**系统钥匙串**（Windows 凭据管理器 / macOS Keychain / Linux Secret Service）—— 和 Xshell 一样，**永不写进配置文件**。按主机（`用户@主机:端口`）加密存储。
- **密钥认证**：通过系统 `ssh`/`scp` 使用默认 SSH 密钥（无需密码）。
- **连接池**：缓存 russh 连接（keep-alive），减少每次截图的 SFTP 握手延迟。

### 🌐 跨终端自动路由
按下 Alt+V 时，上传目标按以下优先级解析：
1. **手动路由目标** —— 窗口标题匹配显式的 `match_pattern`。
2. **ssh-config 自动识别** —— 标题里包含 `~/.ssh/config` 中的主机别名/主机名。
3. **默认 SSH 主机** —— 若已启用。
4. **本地临时路径** —— 兜底（不上传）。

支持 **VS Code、Xshell、MobaXterm、PuTTY、Windows Terminal** 等主流终端。

### 🔑 加载 OpenSSH 配置
- 从 `~/.ssh/config`（或通过"浏览…"选择任意文件）导入主机到路由目标列表。
- 解析器支持别名、HostName、User、Port；自动跳过通配符主机。

### 🎨 主题系统（6 套主题）
- `apple-dark`（默认）、`apple-light`、`dracula`、`nord`、`gruvbox`、`cyberpunk`。
- 紧凑的下拉菜单选择器，带当前主题的色彩指示。
- 所有界面元素（背景、侧边栏、卡片、输入框、开关、按钮、表格、日志）通过 CSS 变量自动适配主题。

### ⌨️ 按键录制式热键
- 点击热键输入框，直接按下组合键即可录制 —— 无需手动输入文本。
- 两个可配置热键：**粘贴**（默认 `Alt+V`）和**截图**（默认 `Alt+Shift+S`）。

### 🖥️ 系统托盘常驻
- 后台运行，系统托盘常驻（如 Snipaste / OneDrive）。
- 双击托盘图标打开设置。
- 关闭设置窗口 → 隐藏到托盘（不退出）。
- **Windows：** 托盘"以管理员身份重启"选项（用于注入到管理员权限的终端 / 绕过 UIPI）。

### 📋 高度可配置
- **输出格式**：Markdown `![image](路径)` / HTML `<img>` / 原始路径 / 内联 Base64。
- **压缩**：JPEG 质量（10–100），最大尺寸（自动缩放）。
- **单引号包裹**：将输出包裹在 `'...'` 中（防止 Bash 历史扩展）。
- **自动清理**：自动删除超过 N 天的截图。
- **开机自启**、**桌面通知**。

---

## 工作原理

```
┌──────────────┐     ┌───────────────┐     ┌──────────────┐     ┌──────────────┐
│   截图        │────▶│    压缩        │────▶│    上传      │────▶│    注入      │
│ (xcap/剪贴板) │     │ (JPEG, ≤1024)  │     │  (SFTP/SCP)  │     │   (Enigo)    │
└──────────────┘     └───────────────┘     └──────────────┘     └──────────────┘
                                                     │
                    ┌────────────────────────────────┘
                    ▼
          ┌─────────────────┐
          │  按窗口标题路由   │
          │  ① 手动目标       │
          │  ② ssh-config    │
          │  ③ 默认 SSH      │
          │  ④ 本地路径       │
          └─────────────────┘
```

1. **截图** —— 剪贴板图片（Alt+V）或区域截图（Alt+Shift+S）。
2. **压缩** —— 等比缩放到最大尺寸，按配置质量编码为 JPEG。
3. **路由** —— 检测当前窗口标题 → 匹配目标 → 确定上传目的地。
4. **上传** —— SFTP（密码/钥匙串 via russh）或 SCP（系统 SSH 密钥）。远程目录自动创建（`mkdir -p`）。
5. **注入** —— 通过 Enigo 将 Markdown 路径"键入"当前焦点终端（direct 模式）或剪贴板置换（swap 模式）。

---

## 配置

设置在 GUI 里编辑，存储于：
- **Windows：** `%APPDATA%\img2cli\config.toml`
- **macOS / Linux：** `~/.config/img2cli/config.toml`

### 主要配置项

| 配置项 | 默认值 | 说明 |
|---|---|---|
| `output_format` | `"markdown"` | `markdown` / `html` / `raw` / `base64` |
| `compress_quality` | `80` | JPEG 压缩质量（10–100） |
| `max_dimension` | `1024` | 最大宽/高（像素） |
| `wrap_single_quotes` | `true` | 输出用单引号 `'...'` 包裹 |
| `global_hotkey` | `"Alt+V"` | 粘贴热键 |
| `screenshot_hotkey` | `"Alt+Shift+S"` | 截图区域选择热键 |
| `injection_mode` | `"direct"` | `direct`（键入）/ `swap`（剪贴板置换） |
| `theme` | `"apple-dark"` | UI 主题 |
| `clean_keep_days` | `1` | 自动清理超过 N 天的截图 |
| `launch_on_boot` | `true` | 开机自启 |

### SSH 配置

```toml
[ssh]
enabled = true
host = "172.16.190.96"
port = 7525
username = "mengzijun"
remote_dir = "/tmp/img2cli"
```

### 路由目标

```toml
[[targets]]
enabled = true
type = "ssh"                    # "ssh" 或 "local"
match_pattern = "91_mengzijun"  # 匹配窗口标题的关键词
host = "172.16.190.96"
port = 7525
username = "mengzijun"
remote_dir = "/tmp/img2cli"
```

---

## 平台说明

| 功能 | Windows | macOS | Linux |
|---|---|---|---|
| **粘贴（Alt+V）** | ✅ 完整 | ✅ 完整（需辅助功能权限） | ✅ X11（Wayland 受限） |
| **截图捕获** | ✅ 完整 | ✅ 完整（需屏幕录制权限） | ❌ 禁用（xcap/PipeWire 不兼容） |
| **窗口标题路由** | ✅ Win32 | ✅ Accessibility API | ✅ X11（Wayland: 回退） |
| **管理员重启** | ✅ | 不适用 | 不适用 |
| **便携版** | ✅ | 不适用 | 不适用 |

**macOS 需要授权的权限：**
- **辅助功能（Accessibility）** —— 用于全局热键 + 文字注入（Enigo）。
- **屏幕录制（Screen Recording）** —— 用于截图捕获（xcap）。

---

## 从源码构建

### 前置要求
- [Node.js](https://nodejs.org/)（LTS）
- [Rust](https://rustup.rs/)（stable）
- [Tauri v2 前置依赖](https://v2.tauri.app/start/prerequisites/)

### 构建与运行
```bash
git clone https://github.com/zijunmeng/img2cli.git
cd img2cli
npm install
npm run tauri dev      # 开发模式（热重载）
npm run tauri build    # 生产构建 → src-tauri/target/release/bundle/
```

### 技术栈
- **后端：** Rust, Tauri v2, russh, xcap, enigo, arboard, keyring
- **前端：** Vue 3, Vite, Tailwind CSS
- **CI/CD：** GitHub Actions + tauri-action

---

## 项目结构

```
src-tauri/src/
├── main.rs         应用入口：托盘、热键、IPC 命令、窗口管理
├── config.rs       配置管理：AppConfig, SshConfig, TargetConfig (TOML)
├── job.rs          JobManager + worker：截图 → 路由 → 交付 → 注入（编排器）
├── routing.rs      RouteResolver 路由链：手动 → ssh-config → 默认 SSH → 本地
├── transport.rs    ArtifactTransport：SFTP/SCP/本地 交付 + 认证分发
├── cli_adapter.rs  CliAdapter：把交付路径渲染成 Markdown / HTML / raw
├── daemon.rs       守护状态、辅助函数、SCP 上传引擎
├── clipboard.rs    剪贴板捕获 + 图像处理（缩放 / 压缩）
├── injector.rs     文字注入：direct (Enigo) / swap (剪贴板置换)
├── ssh.rs          SSH 客户端：russh SFTP（超时、TOFU）、钥匙串、连接池
├── ssh_config.rs   OpenSSH 配置解析器 (~/.ssh/config)
└── capture.rs      截图区域捕获：xcap + 冻结帧覆盖层

src/                 Vue 3 前端
├── App.vue         设置面板 + 主题系统 + 截图覆盖层
├── main.js          Vue 应用启动
└── index.css        Tailwind + 自定义样式
```

---

## 已知问题

- **未签名二进制** → SmartScreen（Windows）/ Gatekeeper（macOS）拦截。代码签名已在规划中。
- **输入法干扰** —— 中文输入法可能吃掉 `direct` 注入的前几个字符。解决：切换到 `swap` 模式。
- **UIPI** —— 无法注入到以管理员权限运行的终端（Windows）。解决：托盘"以管理员身份重启"。
- **Linux 截图** —— 已禁用（xcap 的 PipeWire/libspa 后端与旧版发行版不兼容）。

详见 [KNOWN_ISSUES.md](./KNOWN_ISSUES.md)。

---

## 路线图

- [ ] 代码签名（Windows + macOS）—— 解决 SmartScreen / Gatekeeper
- [ ] 本地 OCR & 代码块提取
- [ ] 标注覆盖层（箭头、高亮、马赛克）
- [ ] 屏幕贴图

详见 [ROADMAP.md](./ROADMAP.md)。

---

## 致谢

- [Tauri](https://tauri.app/) —— 应用框架
- [russh](https://crates.io/crates/russh) —— 纯 Rust SSH 客户端
- [xcap](https://crates.io/crates/xcap) —— 跨平台屏幕捕获
- [enigo](https://crates.io/crates/enigo) —— 输入模拟
- [keyring](https://crates.io/crates/keyring) —— 系统凭证存储
- [wispterm](https://github.com/nicepkg/wispterm) —— 便携打包 & 冻结帧截图的设计灵感

---

## 许可证

MIT
