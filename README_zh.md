# img2cli

[English](./README.md) | 简体中文

`img2cli` 是一个使用 Rust 编写的快速、轻量、跨平台的命令行实用工具。它允许您直接将本地系统剪贴板中的截图自动上传并转换为文件路径文本（特别适用于通过 SSH 远程运行的多模态 AI Agent 终端工具，如 `agy cli`）。

## 工作原理

当工具在后台运行时，您在本地复制一张图片（例如使用 Snipaste 等截图工具）会**自动触发**以下操作：
1. 从系统剪贴板中读取截图数据。
2. 对图片进行压缩和尺寸优化（如自动调整大小并转为 JPEG），以节省网络上传时间和 API Token 消耗。
3. 将图片保存在本地。
4. 如果启用了远程上传，它会自动通过 `scp` 将图片传输到您的远程 Linux 服务器。
5. 自动格式化输出路径（如：绝对路径、Markdown 图片链接或 HTML 标签）。
6. 将格式化后的远程文件路径文本**写回您的系统剪贴板**，并发送气泡通知。
7. 您只需在终端中按下粘贴键（`Ctrl+V`、`Shift+Insert` 或鼠标右键），即可直接键入路径！

## 安装方式

### 方式 A：直接下载预编译二进制包（推荐）

您可以直接从本仓库的 **GitHub Releases** 页面下载适用于 **Windows**、**macOS** 和 **Linux** 的预编译好的可执行文件：

*   **Windows**: 下载 `img2cli-windows.exe`。
*   **Linux**: 下载 `img2cli-linux`（Linux 运行依赖见下方“环境依赖”说明）。
*   **macOS**: 下载 `img2cli-macos`。

### 方式 B：从源码编译

#### 1. 环境依赖 (仅 Linux 需要)

在 Linux (X11) 上运行，您需要安装 X11 运行库和开发库：
*   Debian/Ubuntu 系列系统：
    ```bash
    sudo apt-get install libx11-dev libxtst-dev libxcb1-dev
    ```
*   Fedora/RHEL 系列系统：
    ```bash
    sudo dnf install libX11-devel libXtst-devel libxcb-devel
    ```

#### 2. 使用 Cargo 编译

克隆仓库并使用 Cargo 编译：

```bash
cargo build --release
```

---

## Windows 一键快捷配置 (本地连接远程 AI 场景)

如果您在本地 Windows 电脑上截图，并希望自动粘贴到 VS Code SSH 终端或 Xshell 的远程 AI 终端中：

1. 下载本项目文件夹或 Release 压缩包。
2. 在该项目文件夹下打开 **PowerShell** 窗口。
3. 运行交互式一键配置脚本：
   ```powershell
   powershell -ExecutionPolicy Bypass -File .\setup.ps1
   ```
   *（脚本会自动检测/配置本地 Rust 开发环境，为您编译出 `.exe`，随后通过交互提示引导您输入服务器 IP、用户名和自定义端口，自动在远程服务器上帮您生成并配置好 SSH 免密密钥，最后在本地当前目录下生成一个后台启动器 `run_hidden.vbs`。）*
4. 双击生成的 **`run_hidden.vbs`** 文件，程序即开始在后台完全静默隐形运行。

---

## 配置文件说明

配置文件存储路径：
*   **Windows**: `%USERPROFILE%\.config\img2cli\config.toml`
*   **Linux/Mac**: `~/.config/img2cli/config.toml`

### 配置项详情

*   `save_dir`: 本地存储临时截图的文件夹（默认：系统临时文件夹下的 `img2cli` 目录，软件每小时会自动清理其中超过 24 小时的旧图）。
*   `output_format`: 写回剪贴板用于粘贴的文本格式。可选：
    *   `"raw"`: 绝对路径文本（如 `/tmp/img2cli/img_123.jpg`）。
    *   `"markdown"`: Markdown 图片链接（如 `![image](/tmp/img2cli/img_123.jpg)`）。
    *   `"html"`: HTML 图片标签（如 `<img src="/tmp/img2cli/img_123.jpg" />`）。
    *   `"base64"`: Base64 Data URI 格式（如 `data:image/jpeg;base64,...`）。
*   `compress_quality`: JPEG 压缩质量，范围 `0` 到 `100`（默认：`80`）。
*   `max_dimension`: 截图最大允许的分辨率（宽或高）。超过该尺寸的图片会被自动等比例缩放（默认：`1024`）。
*   `workspace_aware`: 设置为 `true` 时，工具会自动尝试检测当前活跃终端的工作目录并将截图直接存入其下的 `images/` 或 `assets/` 文件夹中并粘贴相对路径（仅 Linux X11 支持，若启用了 SSH 上传则该选项自动失效）。

### `[ssh]` 远程上传配置块

```toml
[ssh]
enabled = true                       # 设置为 true 开启截图自动从本地上传到远程服务器
host = "your-ssh-host-alias-or-ip"   # 远程 SSH 主机 IP 或 ~/.ssh/config 中配置的主机别名
port = 22                            # 远程 SSH 端口（可选，默认 22，也可以自定义如 7525）
username = "your-remote-username"    # 远程服务器用户名
remote_dir = "/tmp/img2cli"          # 图片上传至远程服务器的目标文件夹
```

### `[[ssh_targets]]` 多服务器自动感知路由（高级）

如果您同时在多个远程服务器上工作（例如在不同的 VS Code SSH 窗口或 Xshell 标签页之间切换），您可以在 `config.toml` 中定义多组服务器路由规则。`img2cli` 会自动获取当前最前端的工作窗口标题，并检查其中是否包含您指定的 `match_pattern`。若匹配成功则自动上传到对应服务器；若均未匹配成功，则自动降级使用默认的 `[ssh]` 配置。

```toml
# 目标服务器 1：S90 节点
[[ssh_targets]]
enabled = true
match_pattern = "S90"                 # 当工作窗口标题中包含 "S90" 时匹配（不区分大小写）
host = "172.16.190.90"
port = 22
username = "mengzijun"
remote_dir = "/s1/SHARE/mengzijun/tmp/img2cli"

# 目标服务器 2：开发环境
[[ssh_targets]]
enabled = true
match_pattern = "my-dev-box"          # 当工作窗口标题中包含 "my-dev-box" 时匹配
host = "10.0.0.5"
port = 2222
username = "root"
remote_dir = "/tmp/img2cli"
```

---

## 命令行用法 (Linux/Mac 常用指令)

*   **启动后台守护进程 (Daemon)**:
    ```bash
    img2cli start
    ```
    此命令会将进程分叉至后台运行，日志输出在 `/tmp/img2cli.out` 和 `/tmp/img2cli.err` 中。

*   **在前台直接运行 (一般用于调试或 Windows 手动运行)**:
    ```bash
    img2cli run
    ```

*   **查看后台守护进程状态**:
    ```bash
    img2cli status
    ```

*   **停止后台守护进程**:
    ```bash
    img2cli stop
    ```

*   **手动清理旧临时文件**:
    ```bash
    img2cli clean
    ```
    手动删除保存目录下超过 24 小时的截图（后台守护进程默认每隔一小时会自动执行此清理操作）。
