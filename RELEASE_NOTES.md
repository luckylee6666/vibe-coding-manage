Cross-platform desktop app: macOS (Apple Silicon) + Windows (x64 / ARM64)
跨平台桌面版：macOS (Apple Silicon) + Windows (x64 / ARM64)

## What's new in v1.2.3 / 本版更新

**English**
- **More AI CLIs**: the launch menu now adds **Gemini** and **agy** (alongside Claude / Codex / opencode), each with its own tab badge color.
- **Terminal scrolling fix**: switched the built-in terminal to the WebGL renderer, fixing the selection "ghosting" (a blue block smearing across the screen) when scrolling on a trackpad. Falls back to the default renderer where WebGL is unavailable.

**中文**
- **更多 AI CLI**：启动菜单新增 **Gemini** 和 **agy**（与 Claude / Codex / opencode 并列），各有独立 tab 色标。
- **终端滚动修复**：内置终端改用 WebGL 渲染器，修复触控板滚动时选区「糊成一大片蓝」的 ghosting；不支持 WebGL 时自动降级回默认渲染器。

## Install / 安装

**macOS** — the app is unsigned / 应用未签名：
1. Open the `.dmg`, drag the app into Applications / 打开 `.dmg`，把应用拖入「应用程序」
2. First launch is blocked → **System Settings → Privacy & Security** → scroll down → **Open Anyway** / 首次打开被拦 → **系统设置 → 隐私与安全性** → 滚到底 → **仍要打开**

**Windows** — unsigned, SmartScreen will warn / 未签名，会弹 SmartScreen：
1. Download the `*-setup.exe` for your architecture and run it / 下载对应架构的 `*-setup.exe` 安装
2. On the SmartScreen prompt → **More info → Run anyway** / SmartScreen 提示 → **更多信息 → 仍要运行**
3. Pick **x64** for Intel/AMD, **arm64** for Snapdragon/Surface-ARM machines / Intel/AMD 选 **x64**，骁龙/ARM Surface 选 **arm64**
