# img2cli

English | [简体中文](./README_zh.md)

A fast, lightweight, and cross-platform command-line utility written in Rust that allows you to easily copy/cut screenshots from your local system clipboard directly into your command-line interface (CLI) tools (especially useful for multi-modal AI Agent CLIs running remotely over SSH).

## How it Works

When the tool is running in the background, copying an image (e.g., via standard screenshot tools) will automatically trigger `img2cli` to:
1. Retrieve the screenshot image from the system clipboard.
2. Compress and optimize the image (e.g. resizing and converting to JPEG) to save network transmission time and API token costs.
3. Save the image locally.
4. If configured, automatically upload the image to your remote Linux server via `scp`.
5. Format the output path (e.g., as an absolute path, Markdown image link, or HTML tag).
6. Update your clipboard with the formatted remote path text, and send a notification.
7. You simply press paste (Ctrl+V, Shift+Insert, or Right-Click) to enter the path into your terminal.

## Installation

### Method A: Download Pre-compiled Binaries (Recommended)

You can download the pre-compiled executables for **Windows**, **macOS**, and **Linux** directly from the **GitHub Releases** page of your repository. 

* **Windows**: Download `img2cli-windows.exe`.
* **Linux**: Download `img2cli-linux`. (See Prerequisites below for Linux dependencies).
* **macOS**: Download `img2cli-macos`.

### Method B: Build from Source

#### Prerequisites (Linux only)

On Linux (X11), you will need X11 runtime and development libraries.
* Debian/Ubuntu based systems:
  ```bash
  sudo apt-get install libx11-dev libxtst-dev libxcb1-dev
  ```
* Fedora/RHEL systems:
  ```bash
  sudo dnf install libX11-devel libXtst-devel libxcb-devel
  ```

#### Compile using Cargo

Clone this repository and build using Cargo:

```bash
cargo build --release
```

---

## Windows Quick Setup

If you are running the tool locally on Windows and want to connect to a remote AI Agent (like `agy cli` running via VS Code SSH or Xshell):

1. Download the project folder or release zip.
2. Open **PowerShell** in the project directory.
3. Run the interactive setup script:
   ```powershell
   powershell -ExecutionPolicy Bypass -File .\setup.ps1
   ```
   *The script will verify Rust/Cargo, compile the binary, prompt you for SSH upload parameters, and write the VBS background launcher `run_hidden.vbs` in the current folder.*
4. Double-click the generated `run_hidden.vbs` to launch the listener in the background invisibly.

---

## Configuration

The tool reads its configuration from:
* **Windows**: `%USERPROFILE%\.config\img2cli\config.toml`
* **Linux/Mac**: `~/.config/img2cli/config.toml`

### Configuration Options

* `save_dir`: Directory to store saved images (default: system temporary directory under `img2cli`).
* `output_format`: Format typed into the active CLI. Options:
  * `"raw"`: Raw file path (e.g. `/tmp/img2cli/img_123.jpg`).
  * `"markdown"`: Markdown image link (e.g. `![image](/tmp/img2cli/img_123.jpg)`).
  * `"html"`: HTML image tag (e.g. `<img src="/tmp/img2cli/img_123.jpg" />`).
  * `"base64"`: Base64 data URI (e.g. `data:image/jpeg;base64,...`).
* `compress_quality`: JPEG quality, from `0` to `100` (default: `80`).
* `max_dimension`: Max width or height of saved image. Larger images are scaled down (default: `1024`).
* `workspace_aware`: If `true`, the tool attempts to detect the active terminal window's current working directory. (Linux X11 only. Disabled if SSH upload is active).

### `[ssh]` Upload Block

```toml
[ssh]
enabled = true                       # Set to true to upload screenshots from local to remote
host = "your-ssh-host-alias-or-ip"   # Remote SSH Host IP or ~/.ssh/config alias (e.g., S91)
port = 22                            # Remote SSH port (optional, defaults to 22 or inherits from SSH config)
username = "your-remote-username"    # Remote username (optional if using SSH alias)
remote_dir = "/tmp/img2cli"          # Directory on the remote server to store the uploaded image
```

### `[[ssh_targets]]` Multi-Server Routing (Advanced)

If you work on multiple remote servers simultaneously (e.g. multiple VS Code SSH windows or Xshell tabs), you can define matching rules in `config.toml`. `img2cli` will automatically detect the active window's title, check if it contains your `match_pattern`, and upload to the corresponding server. If no pattern matches, it will fallback to the default `[ssh]` target.

```toml
# Target 1: S90 Server
[[ssh_targets]]
enabled = true
match_pattern = "S90"                 # Match if active window title contains "S90" (case-insensitive)
host = "172.16.190.90"
port = 22
username = "mengzijun"
remote_dir = "/s1/SHARE/mengzijun/tmp/img2cli"

# Target 2: Dev Box
[[ssh_targets]]
enabled = true
match_pattern = "my-dev-box"          # Match if active window title contains "my-dev-box"
host = "10.0.0.5"
port = 2222
username = "root"
remote_dir = "/tmp/img2cli"
```

---

## Usage (Linux/Mac commands)

* **Start the background daemon**:
  ```bash
  img2cli start
  ```
  This forks the process into the background and writes logs to `/tmp/img2cli.out` and `/tmp/img2cli.err`.

* **Run in foreground (for debugging or Windows run)**:
  ```bash
  img2cli run
  ```

* **Check daemon status**:
  ```bash
  img2cli status
  ```

* **Stop the daemon**:
  ```bash
  img2cli stop
  ```

* **Manually clean temporary files**:
  ```bash
  img2cli clean
  ```
  Deletes image files in the saving directory that are older than 24 hours. (The daemon also runs this check automatically every hour).
