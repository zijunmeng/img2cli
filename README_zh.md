# img2cli

[English](./README.md) | 简体中文

把截图以 Markdown 图片链接的形式粘贴进任何 AI CLI —— **而且不破坏你剪贴板里的图片。**

`img2cli` 是一个跨平台的**系统托盘桌面应用**（Rust + Tauri + Vue 3），为多模态 AI 工作流而生。截图 → 聚焦到终端 → 按 **Alt+V**，图片的 Markdown 路径就会被"敲"进你的终端；而图片本身仍留在剪贴板里，所以你照样能用 **Ctrl+V** 把原图贴进微信 / Word / 飞书。

## 为什么需要它

通过 SSH 远程运行的多模态 AI CLI（Claude Code、Cursor、Gemini CLI 等）**接收不了粘贴的图片**，它们只认**文本路径**。img2cli 就是来补这个缺口的：读取剪贴板里的截图，上传到你终端所连的服务器（或存到本地），再把路径以 `![image](...)` 注入进去。

它是**不抢剪贴板**的：默认 `direct`（直接键入）模式下全程不碰剪贴板，所以同一张截图既能贴聊天软件（Ctrl+V，出图）**也能**喂给终端（Alt+V，出路径）。

## 下载

从 [GitHub Releases](https://github.com/zijunmeng/img2cli/releases) 下载最新版：

| 系统 | 文件 |
|---|---|
| Windows（安装版） | `img2cli_0.3.2_x64-setup.exe` / `img2cli_0.3.2_x64_en-US.msi` |
| Windows（免安装便携版） | `img2cli-v0.3.2-windows-portable.zip` |
| macOS（通用版） | `img2cli_0.3.2_universal.dmg` |
| Linux | `img2cli_0.3.2_amd64.deb` / `.rpm` / `.AppImage` |

> ⚠️ 目前二进制**未签名**。首次启动时 Windows SmartScreen（以及 360 等部分杀软）可能拦截 —— 点 *更多信息 → 仍要运行*，或把程序加入信任区即可。代码签名已在规划中。

## 工作流程

1. 截图并放入剪贴板（Win+Shift+S、macOS 截图等）。
2. 把焦点切到运行 AI CLI 的终端。
3. 按 **Alt+V**。
4. img2cli 抓取并压缩图片，上传到匹配的服务器（远程目录不存在会自动创建），再把 `![image](/远程/路径.jpg)` **注入**终端。

## 功能特性

- **常驻系统托盘**，带毛玻璃风格的设置面板（双击托盘图标打开）。
- **跨终端自动路由** —— 根据当前窗口标题，结合你的 `~/.ssh/config` 自动识别要上传的 SSH 主机。VS Code、Xshell、MobaXterm、PuTTY、Windows Terminal 等都适用，无需手动配匹配规则。
- **密码 或 密钥登录** —— 密码存在**系统钥匙串**（Windows 凭据管理器 / macOS 钥匙串 / Linux Secret Service），和 Xshell 一样；密钥服务器继续用你的 SSH key。
- **加载 OpenSSH 配置** —— 可从 `~/.ssh/config`（或通过"浏览…"任意文件）导入主机到路由目标。
- **不抢剪贴板的注入** —— `direct`（原生键入，默认）或 `swap`（快速剪贴板置换）模式。
- **可配置** —— 输出格式（Markdown / HTML / 原始路径 / base64）、压缩质量、最大尺寸、**按键录制式全局热键**、开机自启、通知。
- **Windows：** 托盘"以管理员身份重启"选项，可注入到以管理员权限运行的终端（绕过 UIPI 限制）。

### 路由优先级

按下 Alt+V 时，上传目标按以下顺序解析：

1. **手动路由目标**（窗口标题匹配显式的 `match_pattern`）
2. **ssh-config 自动识别**（标题里包含 `~/.ssh/config` 中的主机别名/主机名）
3. **默认 SSH 主机**（若已启用）
4. **本地临时路径**（兜底）

## SSH 密码安全

密码**永远不会写进配置文件**。它们加密存储在系统钥匙串里，按主机（`用户@主机:端口`）作为键名，只有当前系统用户能读取 —— 和 Xshell 一样，不是明文。密钥服务器则根本不需要密码。

## 平台说明

- **Windows：** 完整支持（按窗口标题自动路由、以管理员重启）。
- **macOS：** 需授予**辅助功能（Accessibility）**权限（用于全局热键、文字注入、读取窗口标题）。
- **Linux：** 仅 X11（需安装 `xdotool`）。Wayland 合成器不允许读取其它窗口标题，因此自动路由会回落到默认主机 / 本地路径。

## 配置

设置在 GUI 里编辑，存储于：

- Windows：`%APPDATA%\img2cli\config.toml`
- macOS / Linux：`~/.config/img2cli/config.toml`

## 从源码构建

需要 Node.js、Rust 以及 [Tauri v2 前置依赖](https://v2.tauri.app/start/prerequisites/)。

```bash
npm install
npm run tauri dev      # 开发模式运行
npm run tauri build    # 生成安装包 / 便携版
```
