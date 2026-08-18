# img2cli

[English](./README.md) | 简体中文

**把截图以 Markdown 图片链接的形式粘贴进任何 AI CLI —— 而且不破坏你剪贴板里的图片。**

`img2cli` 是一个跨平台的**系统托盘桌面应用**（Rust + Tauri v2 + Vue 3），为多模态 AI 工作流而生。用**内置的 Snipaste 风格截图**（标注、贴图、复制、上传一应俱全）截一张图，图片立即在后台上传到你的服务器 —— 聚焦终端，按**注入热键**，Markdown 路径直接注入。图片本身仍留在剪贴板里，你照样能用 **Ctrl+V** 把原图贴进微信 / Word / 飞书。

## 下载

| 系统 | 文件 | 说明 |
|---|---|---|
| **Windows**（安装版） | `img2cli_0.4.8_x64-setup.exe` / `_x64_en-US.msi` | |
| **Windows**（免安装版） | `img2cli-v0.4.8-windows-portable.zip` | 解压即用 |
| **macOS**（通用版） | `img2cli_0.4.8_universal.dmg` | 支持 M1/M2/M3 + Intel |
| **Linux** | `img2cli_0.4.8_amd64.deb` / `.rpm` / `.AppImage` | 截图功能暂不可用（见平台说明） |

→ **[GitHub Releases](https://github.com/zijunmeng/img2cli/releases)**

> ⚠️ **二进制未签名。** 首次启动时：
> - **Windows：** SmartScreen → *更多信息 → 仍要运行*；或加入杀软信任区。
> - **macOS：** 右键 `img2cli.app` → *打开* → 确认。然后在 *系统设置 → 隐私与安全性* 中授权 **辅助功能** + **屏幕录制**。

---

## 快速上手

1. **安装** —— 下载对应系统的安装包，安装 / 拖到应用程序。
2. **截图** —— 按 **Alt+Shift+S**（可配置；屏幕瞬间定格）：
   - **拖拽**自由框选，或**点击**窗口直接吸附，或 **Tab** 在识别到的窗口/界面元素间循环切换。
   - **标注** —— 箭头 / 画笔 / 马克笔 / 马赛克 / 文字 / 矩形 / 椭圆，支持撤销重做、逐段擦除的橡皮擦。
   - **动作** —— ⬆ 上传+稍后注入 · 📋 / **Ctrl+C** 复制图像 · 💾 保存文件 · 📌 贴到屏幕。
   - 点 ✓（或 Enter）确认选区后进入移动/调整模式；再按 Enter 上传。
3. 裁剪结果**同时**进入剪贴板**和**后台上传队列 —— 不用等。
4. **粘贴路径到终端** —— 聚焦 AI CLI，按 **Alt+V**（注入热键），Markdown 路径 `![image](/远程/路径.jpg)` 注入；图片未变时直接复用已上传的路径，秒出。
5. **贴图到聊天** —— 在微信 / Word 里按 **Ctrl+V**，原图照常粘贴。

---

## 功能特性

### 📸 内置区域截图（Snipaste 风格）
- 专用**截图热键**（默认 `Alt+Shift+S**`）+ **常驻热覆盖层** —— 覆盖层出现前屏幕已抓取到内存（零闪烁）；冻结帧以紧凑 JPEG 送入界面，浮层几乎无感延迟。
- **窗口自动识别** —— 光标下的窗口自动描边；**Tab / Shift+Tab** 在窗口*及其子元素*（按钮、编辑器）间循环，精准吸附。点击 = 吸附，拖拽 = 自由选区，mouseup 时裁决。
- **显式确认模型** —— 有选区 ≠ 确定：点 ✓ 之前可任意重画；确认后区内拖动 = 移动。Enter = 先确认、再上传。
- **区域记忆** —— **Shift+R** 回到上次区域；`,` / `.` 循环最近 8 个区域；**WASD** 光标 1 像素微调。
- **截屏选项** —— 选区边框宽度、遮罩浓度、提示面板与窗口识别开关。

### ✏️ 标注编辑器
- 工具：**箭头、画笔、马克笔（multiply 混合）、马赛克、文字、矩形、椭圆** + 取色板 + 粗细调节。
- **撤销 / 重做**（Ctrl+Z / Ctrl+Y）；**逐段擦除橡皮擦** —— 笔迹被扫过的片段消失（Snipaste 行为），不是整对象删除。
- 标注以**全物理分辨率**合成进最终裁剪，流经整条管线（剪贴板、历史、上传、贴图）。

### 📌 贴到屏幕（贴图）
- 任意（含标注的）裁剪贴为无边框置顶窗口。
- 拖动移动、**滚轮缩放**、**右键菜单**（复制图像 / 另存为 / 销毁）、双击关闭。

### 🔄 不抢剪贴板注入
- **`auto` 模式**（默认）：按目标应用策略 —— 可注入的用键入，拒绝合成输入的应用（如 Orca）自动切剪贴板模式。
- **`direct`**：通过 [Enigo](https://crates.io/crates/enigo) 模拟原生 Unicode 键盘输入，**全程不碰剪贴板**。
- **`copy`**：路径写入剪贴板 + 模拟粘贴 —— 供拒绝合成键入的主机使用。
- **截图即上传**：确认选区后 SFTP 上传立即在后台开始；注入热键粘贴已送达路径（剪贴板图像未变时走指纹快速通道，跳过重复上传）。

### 🔐 SSH 密码 + 密钥登录
- **密码认证**：密码存在**系统钥匙串**（Windows 凭据管理器 / macOS Keychain / Linux Secret Service）—— 永不写进配置文件。按主机（`用户@主机:端口`）存储。
- **密钥认证**：通过系统 `ssh`/`scp` 使用默认 SSH 密钥。
- **known_hosts**：首次连接记住主机（TOFU），之后校验。

### 🌐 跨终端自动路由
注入热键触发时，上传目标按优先级解析：
1. **路由目标** —— 窗口标题（或进程名）匹配显式 `match_pattern`。
2. **ssh-config 自动识别** —— 标题里包含 `~/.ssh/config` 中的主机别名/主机名。
3. **默认 SSH 主机** —— 带 Default 标志的目标卡片。
4. **本地临时路径** —— 兜底（不上传）。

支持 **VS Code、Xshell、MobaXterm、PuTTY、Windows Terminal** 等主流终端。

### 🔑 主机与目标管理
- Orca 风格**卡片列表** —— 一个目标一张卡，恰好一张带 **Default** 标志，逐卡启用/停用、连接测试。
- **加载 OpenSSH 配置** —— 从 `~/.ssh/config` 导入主机（搜索、多选、去重）。

### 🎨 界面
- **6 套主题**（默认 `dracula`，另有 `apple-dark`/`apple-light`、`nord`、`gruvbox`、`cyberpunk`），所有界面元素通过 CSS 变量自动适配。
- **中文 / English** 界面切换。
- 主窗口可调大小；**单实例**（二次启动唤起已运行实例）。
- **系统日志**面板：一键复制 / 导出 / 清空 —— 所有守护事件同时镜像到 `%TEMP%\img2cli\daemon.log`，软件冻死也有完整日志可查。
- **按键录制式热键** + 黑名单校验（拦下 Ctrl+C、Alt+F4 这类会破坏系统的组合）。中文输入法下可用（物理键匹配 + keyup 兜底）。

### 🖥️ 系统托盘常驻
- 后台运行，系统托盘常驻；左键打开设置；关闭窗口 → 隐藏到托盘（不退出）。
- **Windows：** 托盘"以管理员身份重启"选项（用于注入到管理员权限的终端 / 绕过 UIPI）。

### 📋 可配置项
- **输出格式**：Markdown `![image](path)` / HTML `<img>` / 裸路径 / 内联 Base64。
- **压缩**：JPEG 质量（10–100）、最大边长（自动缩放）。
- **单引号包裹**、过期截图自动清理、**开机自启**、**桌面通知**。

---

## 工作原理

```
┌──────────────┐     ┌───────────────┐     ┌──────────────┐     ┌──────────────┐
│    截图       │────▶│     压缩       │────▶│    上传       │────▶│    注入       │
│ (xcap/剪贴板) │     │   (JPEG)      │     │ (SFTP/SCP,   │     │ (按需: 热键)  │
│   + 标注      │     │               │     │   后台进行)   │     │              │
└──────────────┘     └───────────────┘     └──────────────┘     └──────────────┘
        │                                                               ▲
        └── 裁剪 → 剪贴板（图像保留，供 Ctrl+V）──────────────────────────┘
                                 按标题/进程路由: ① 目标卡 ② ssh-config ③ 默认主机 ④ 本地
```

1. **截图** —— 区域截图（冻结帧覆盖层）或剪贴板图像。
2. **上传（后台）** —— 确认的区域立即压缩并 SFTP/SCP 推送；送达路径缓存。
3. **路由** —— 前台窗口标题 + 进程名 → 目标 → 上传目的地。
4. **注入（按需）** —— 聚焦 AI CLI 按注入热键：键入/粘贴 Markdown 路径；剪贴板还是同一张图时直接注入缓存路径，不重传。

---

## 配置

设置在 GUI 中编辑，存储于：
- **Windows：** `%APPDATA%\img2cli\config.toml`
- **macOS / Linux：** `~/.config/img2cli/config.toml`

### 关键设置

| 设置 | 默认值 | 说明 |
|---|---|---|
| `output_format` | `"markdown"` | `markdown` / `html` / `raw` / `base64` |
| `compress_quality` | `80` | JPEG 质量（10–100） |
| `max_dimension` | `1024` | 最大宽/高（像素） |
| `wrap_single_quotes` | `false` | 输出用 `'...'` 包裹 |
| `global_hotkey` | `"Alt+V"` | 注入热键 |
| `screenshot_hotkey` | `"Alt+Shift+S"` | 截图热键 |
| `injection_mode` | `"auto"` | `auto`（按应用策略）/ `direct`（键入）/ `copy`（剪贴板+粘贴） |
| `theme` | `"dracula"` | 界面主题 |
| `language` | `"zh-CN"` | `zh-CN` / `en` |
| `capture_border_width` | `2` | 选区边框（px） |
| `capture_mask_opacity` | `45` | 选区外压暗（%） |
| `clean_keep_days` | `1` | 自动清理 N 天前的截图 |
| `launch_on_boot` | `true` | 随系统启动 |

### 路由目标

```toml
[[targets]]
enabled = true
type = "ssh"                    # "ssh" 或 "local"
match_pattern = "91_mengzijun"  # 匹配窗口标题 / 进程名
host = "172.16.190.96"
port = 7525
username = "mengzijun"
remote_dir = "/tmp/img2cli"
is_default = true               # 恰好一张卡片带 Default 标志
```

---

## 平台说明

| 功能 | Windows | macOS | Linux |
|---|---|---|---|
| **注入热键** | ✅ 完整 | ✅ 完整（需辅助功能） | ✅ X11（Wayland 受限） |
| **区域截图** | ✅ 完整 | ✅ 完整（需屏幕录制） | ❌ 禁用（xcap/PipeWire 不兼容） |
| **窗口标题路由** | ✅ Win32 | ✅ Accessibility API | ✅ X11（Wayland: 回退） |
| **以管理员重启** | ✅ | 无 | 无 |
| **免安装 zip** | ✅ | 无 | 无 |

**macOS 需要的权限：**
- **辅助功能** —— 全局热键 + 文字注入（Enigo）。
- **屏幕录制** —— 截图（xcap）。

---

## 排障指南

- **截图热键没反应** —— 打开 设置 → 系统日志：
  - `Failed to register screenshot shortcut ... Another instance/app may be using it` → 别的应用（或没退干净的 img2cli 实例）占着组合键。把所有 img2cli 退出（托盘 → Exit，或任务管理器），只启动一个。
  - 有 `Screenshot hotkey received` 日志行但浮层不出 → **再按一次热键**：热浮层 webview 已死会被检测到，下一次按键重建（每次按键至多一次重建）。
- **软件卡死 / 界面无响应** —— 守护日志还在：`%TEMP%\img2cli\daemon.log`（`C:\Users\<你>\AppData\Local\Temp\img2cli\daemon.log`），最后几行就是管线停住的位置。
- **强杀进程后 Webview 行为异常** —— 删除 `%LOCALAPPDATA%\com.img2cli.app`（WebView2 档案，会自动重建；配置在别处不受影响）后重启。
- **中文输入法下热键录制器录不进字母** —— 松开按键即可：keyup 事件会补齐组合。仍不行 → 录制时把输入法切到 EN。
- **想粘贴的内容变成了路径** —— 注入热键永远注入截图路径；手动粘贴请用 **Ctrl+V**。

---

## 从源码构建

### 前置
- [Node.js](https://nodejs.org/)（LTS）
- [Rust](https://rustup.rs/)（stable）
- [Tauri v2 前置依赖](https://v2.tauri.app/start/prerequisites/)

### 构建与运行
```bash
git clone https://github.com/zijunmeng/img2cli.git
cd img2cli
npm install
npm run tauri dev      # 开发（热重载）
npm run tauri build    # 生产构建 → src-tauri/target/release/bundle/
```

### 技术栈
- **后端：** Rust、Tauri v2、russh、xcap、enigo、arboard、keyring、image
- **前端：** Vue 3、Vite、Tailwind CSS
- **CI/CD：** GitHub Actions + tauri-action（三平台检查 + 发版）

---

## 架构

```
src-tauri/src/
├── main.rs         入口：托盘、热键（+黑名单校验）、IPC 命令、窗口装配
├── config.rs       配置：AppConfig、TargetConfig、InjectionMode（TOML）
├── job.rs          JobManager + worker：截图 → 路由 → 交付 → 注入、后台上传
├── routing.rs      RouteResolver 路由链：目标卡 → ssh-config → 默认 SSH → 本地
├── host_policy.rs  按应用注入策略（标题/进程 → direct/copy）
├── transport.rs    ArtifactTransport：SFTP/SCP/本地交付 + 认证分发
├── cli_adapter.rs  CliAdapter：送达路径 → Markdown / HTML / 裸路径
├── daemon.rs       DaemonState、日志（面板 + daemon.log 镜像）、SCP 引擎
├── clipboard.rs    剪贴板捕获、data-URL 解码、图像处理
├── injector.rs     注入：direct（Enigo）/ copy（剪贴板+粘贴）
├── ssh.rs          SSH 客户端：russh SFTP（超时、TOFU known_hosts）、钥匙串、连接池
├── ssh_config.rs   OpenSSH 配置解析器（~/.ssh/config）
└── capture.rs      区域截图：xcap 冻结帧 + 常驻热浮层 + 窗口识别

src/                 Vue 3 前端
├── App.vue         设置面板 + 截图覆盖层（标注引擎）+ 贴图窗口
├── strings.js      zh-CN / en 国际化
├── main.js         Vue 启动
└── index.css       Tailwind + 自定义样式
```

---

## 已知问题

- **二进制未签名** → SmartScreen（Windows）/ Gatekeeper（macOS）警告。代码签名已列入计划（v1.0.0）。
- **热浮层 webview 偶发死亡**（与 WebView2/驱动相关，因机而异）。再按一次截图热键即可 —— v0.4.8 会检测到死浮层并在下一次按键重建。持续调查中。
- **拒绝合成输入的应用**（如 Orca）—— `auto` 模式自动处理（路由到 `copy`）；在这些应用里用 Ctrl+V。
- **UIPI** —— 无法注入管理员权限终端（Windows）。解决：托盘"以管理员身份重启"。
- **Linux 截图** —— 禁用（xcap 的 PipeWire/libspa 后端与旧发行版不兼容）。

完整列表见 [KNOWN_ISSUES.md](./KNOWN_ISSUES.md)。

---

## 路线图

- [x] 标注编辑器（箭头、画笔、马克笔、马赛克、文字、图形、橡皮擦）
- [x] 贴到屏幕（贴图）
- [x] 后台上传 + 快速通道注入
- [ ] 多显示器截图
- [ ] SSH 保活连接池（上传 <200ms）
- [ ] 本地 OCR 转代码块
- [ ] 长截屏（滚动截图）
- [ ] 代码签名（Windows + macOS）

详见 [ROADMAP.md](./ROADMAP.md)。

---

## 致谢

- [Tauri](https://tauri.app/) —— 应用框架
- [russh](https://crates.io/crates/russh) —— 纯 Rust SSH 客户端
- [xcap](https://crates.io/crates/xcap) —— 跨平台屏幕捕获
- [enigo](https://crates.io/crates/enigo) —— 输入模拟
- [arboard](https://crates.io/crates/arboard) —— 剪贴板访问
- [keyring](https://crates.io/crates/keyring) —— 系统凭据存储
- [wispterm](https://github.com/nicepkg/wispterm) —— 便携打包与冻结帧截图的设计参考
- [Snipaste](https://www.snipaste.com/) / [ShareX](https://getsharex.com/) / [Flameshot](https://flameshot.org/) —— 截图、贴图与标注的交互参考

---

## 许可证

MIT
