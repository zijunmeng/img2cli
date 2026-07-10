***

# AGENTS.md - Developer Agent Guide for `image2cli`

欢迎阅读 `image2cli` 项目。本指南旨在为开发此项目的 AI Agent 提供必要的上下文、架构蓝图和编码规范，以确保代码质量与系统设计的一致性。

## 1. 项目概述 (Project Overview)

`image2cli` 是一个使用 Rust 开发的轻量级后台工具。
*   **核心功能**：监听全局快捷键（如 `Ctrl + Shift + V`），当用户触发时，检查系统剪贴板。如果剪贴板中存在图片，将其保存到本地临时目录，并将生成的图片路径（或 Markdown 格式路径）自动键入到当前处于激活状态的终端光标处。
*   **核心痛点**：解决多模态 AI Agent 在 CLI 模式下运行时，用户无法方便地通过“截图-复制-粘贴”传递视觉信息的问题。

## 2. 核心技术栈 (Technology Stack)

在修改或添加代码时，请优先使用以下核心库：
*   **剪贴板操作 (Clipboard)**: `arboard` (跨平台的剪贴板读写)
*   **全局按键监听 (Global Hotkeys)**: `rdev` (全局监听) 或 `device_query`
*   **按键模拟 (Input Simulation)**: `enigo` (用于向当前焦点窗口发送输入)
*   **配置管理 (Configuration)**: `serde` & `toml` (解析 `config.toml`)
*   **时间与文件名 (Utils)**: `chrono` 或 `uuid` (用于生成不冲突的临时文件名)
*   **日志系统 (Logging)**: `log` & `env_logger` (便于调试)

## 3. 架构与模块划分 (Architecture & Modules)

项目采用模块化设计，避免将逻辑堆积在 `main.rs` 中。模块结构如下：

```text
src/
├── main.rs         # 业务入口，解析 CLI 参数，启动/停止后台进程
├── config.rs       # 读取与解析 config.toml（如路径、格式、自定义快捷键）
├── clipboard.rs    # 与 arboard 交互，检测和提取剪贴板中的图像数据
├── keyboard.rs     # 全局热键监听（rdev）与字符输入模拟（enigo）
├── daemon.rs       # 后台服务循环，管理通道 (mpsc) 传递的触发事件
└── utils.rs        # 辅助功能（如：临时目录获取、过期图片自动清理逻辑）
```

### 数据流向 (Data Flow):
```text
[用户按下快捷键] 
       │
       ▼
(keyboard::监听模块) ──[发送事件]──> (daemon::事件循环)
                                         │
                                         ▼
(keyboard::模拟输入) <──[写入路径]── (clipboard::保存图片)
```

## 4. 开发与编码规范 (Development Guidelines)

在为此项目编写 Rust 代码时，请遵循以下原则：

### 4.1 错误处理 (Error Handling)
*   避免滥用 `unwrap()`。所有可能失败的操作（I/O、剪贴板读取、按键模拟）应返回 `Result<T, E>`。
*   在各模块中，定义清晰的错误类型。可以考虑使用 `thiserror` 库来减少模板代码。

### 4.2 跨平台兼容性 (Cross-Platform)
*   图片保存路径：使用标准库中的 `std::env::temp_dir()` 或第三方库 `directories` 获取符合 OS 规范的路径。
*   不同操作系统下的路径分隔符：优先使用 `PathBuf` 进行路径拼接，避免硬编码 `/` 或 `\`。

### 4.3 性能与资源控制 (Resource Control)
*   这是一个常驻后台的守护进程，**内存控制**至关重要。
*   在读取剪贴板大图后，及时释放图片内存，避免内存泄漏。
*   如果使用图片压缩功能，应在保存文件阶段流式处理，避免长时间占用高额内存。

## 5. 平台已知限制与边界 (Known Constraints)

在处理系统级事件时，请注意以下平台限制：

*   **macOS 安全限制**：
    *   全局键盘监听（`rdev`）和按键模拟（`enigo`）需要系统“辅助功能 (Accessibility)”权限。
    *   在引导用户使用时，应在控制台打印友好的权限提示。
*   **Linux (Wayland)**：
    *   Wayland 环境下全局热键和剪贴板操作限制较多。如果检测到 `XDG_SESSION_TYPE=wayland`，需要注意兼容性，部分情况下可能需要回退到 X11 或提示用户相关限制。
*   **Windows**：
    *   注意模拟输入时的输入法状态影响。如果用户处于中文输入法状态，模拟键入路径可能会触发输入法候选框。应考虑使用 Clipboard 覆盖或 Enigo 的安全文本输入模式。

## 6. 后续待开发任务列表 (Roadmap for Agent)

如果您被要求添加新功能，可以参考以下规划：
1.  **图片压缩模块**：在 `utils.rs` 或独立模块中，引入 `image` 库对截图进行尺寸调整和 JPEG 压缩，以节省 AI Token。
2.  **清理机制**：在 `utils.rs` 中实现 `clean_expired_images` 函数，允许程序每次启动时删除超过 24 小时的临时文件。
3.  **格式配置项**：在 `config.rs` 中支持 `output_template`（例如可以配置为 `![img]({path})` 或原生的 `{path}`）。