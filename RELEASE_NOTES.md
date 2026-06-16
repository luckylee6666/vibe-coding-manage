Cross-platform desktop app: macOS (Apple Silicon) + Windows (x64 / ARM64)
跨平台桌面版：macOS (Apple Silicon) + Windows (x64 / ARM64)

## What's new in v1.2.4 / 本版更新

**English**
- **Claude usage panel** (5-hour window): a new clock icon in the terminal toolbar shows your current 5-hour billing window — live countdown to reset, cost + projected cost, burn rate, total/output tokens, and active models. Read locally via the community `ccusage` tool from `~/.claude`; nothing is uploaded.
- **Auto-hello on window reset**: optional toggle that fires a tiny `claude -p hello` once the 5-hour window resets/goes idle, so a fresh window starts immediately. Includes a "send hello now" button.

**中文**
- **Claude 用量面板**（5 小时窗口）：终端工具栏新增时钟图标，显示当前 5 小时计费窗口——实时倒计时、花费 + 预计花费、燃烧速率、总/输出 token、活跃模型。经社区工具 `ccusage` 读取本机 `~/.claude` 日志，不上传任何数据。
- **窗口重置后自动 hello**：可选开关，5 小时窗口重置/空闲后自动发一句极小的 `claude -p hello`，立刻开新窗口重新计时；另有「立刻发一次 hello」按钮。

## Install / 安装

**macOS** — the app is unsigned / 应用未签名：
1. Open the `.dmg`, drag the app into Applications / 打开 `.dmg`，把应用拖入「应用程序」
2. First launch is blocked → **System Settings → Privacy & Security** → scroll down → **Open Anyway** / 首次打开被拦 → **系统设置 → 隐私与安全性** → 滚到底 → **仍要打开**

**Windows** — unsigned, SmartScreen will warn / 未签名，会弹 SmartScreen：
1. Download the `*-setup.exe` for your architecture and run it / 下载对应架构的 `*-setup.exe` 安装
2. On the SmartScreen prompt → **More info → Run anyway** / SmartScreen 提示 → **更多信息 → 仍要运行**
3. Pick **x64** for Intel/AMD, **arm64** for Snapdragon/Surface-ARM machines / Intel/AMD 选 **x64**，骁龙/ARM Surface 选 **arm64**
