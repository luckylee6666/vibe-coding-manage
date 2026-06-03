Cross-platform desktop app: macOS (Apple Silicon) + Windows (x64 / ARM64)
跨平台桌面版：macOS (Apple Silicon) + Windows (x64 / ARM64)

## What's new in v1.2.1 / 本版更新

**English**
- **File tree → terminal**: drag a file/folder onto the terminal to insert its path.
- **Right-click menu**: Open folder · Insert path · Copy path · Move to Trash (recoverable).
- **Richer preview**: images, PDF, rendered Markdown (Source/Rendered toggle), CSV/TSV tables.
- **Security**: Markdown preview sanitized (DOMPurify); links open in the system browser.

**中文**
- **文件树 → 终端**：拖文件/文件夹到终端插入路径。
- **右键菜单**：打开文件夹 · 插入路径 · 复制路径 · 移到废纸篓（可恢复）。
- **更丰富的预览**：图片、PDF、Markdown 渲染（源码/渲染切换）、CSV/TSV 表格。
- **安全**：Markdown 预览经 DOMPurify 净化；链接走系统浏览器。

## Install / 安装

**macOS** — the app is unsigned / 应用未签名：
1. Open the `.dmg`, drag the app into Applications / 打开 `.dmg`，把应用拖入「应用程序」
2. First launch is blocked → **System Settings → Privacy & Security** → scroll down → **Open Anyway** / 首次打开被拦 → **系统设置 → 隐私与安全性** → 滚到底 → **仍要打开**

**Windows** — unsigned, SmartScreen will warn / 未签名，会弹 SmartScreen：
1. Download the `*-setup.exe` for your architecture and run it / 下载对应架构的 `*-setup.exe` 安装
2. On the SmartScreen prompt → **More info → Run anyway** / SmartScreen 提示 → **更多信息 → 仍要运行**
3. Pick **x64** for Intel/AMD, **arm64** for Snapdragon/Surface-ARM machines / Intel/AMD 选 **x64**，骁龙/ARM Surface 选 **arm64**
