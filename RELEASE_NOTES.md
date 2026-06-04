Cross-platform desktop app: macOS (Apple Silicon) + Windows (x64 / ARM64)
跨平台桌面版：macOS (Apple Silicon) + Windows (x64 / ARM64)

## What's new in v1.2.2 / 本版更新

**English**
- **Rename groups inline**: hover a group, click the pencil, type the new name — all projects in that group move together.
- **Close-terminal confirmation**: closing a terminal tab now asks first, and reminds you to let the AI "update its memory" before the session ends.
- **Title bar shows the version**; server management moved to the top; redundant project count removed.
- **Fixes**: confirm dialogs no longer hide behind the open terminal panel; the DMG now has a "drag to Applications" layout.

**中文**
- **分组就地重命名**：hover 分组点铅笔即可改名，组内项目一起迁移。
- **关闭终端前确认**：关终端标签会先弹确认，并提醒你先让 AI「更新记忆」再结束会话。
- **标题栏显示版本号**；服务器管理移到顶部；去掉冗余的项目数。
- **修复**：终端面板开着时确认弹窗不再被压在底下；DMG 带「拖到 Applications」布局。

## Install / 安装

**macOS** — the app is unsigned / 应用未签名：
1. Open the `.dmg`, drag the app into Applications / 打开 `.dmg`，把应用拖入「应用程序」
2. First launch is blocked → **System Settings → Privacy & Security** → scroll down → **Open Anyway** / 首次打开被拦 → **系统设置 → 隐私与安全性** → 滚到底 → **仍要打开**

**Windows** — unsigned, SmartScreen will warn / 未签名，会弹 SmartScreen：
1. Download the `*-setup.exe` for your architecture and run it / 下载对应架构的 `*-setup.exe` 安装
2. On the SmartScreen prompt → **More info → Run anyway** / SmartScreen 提示 → **更多信息 → 仍要运行**
3. Pick **x64** for Intel/AMD, **arm64** for Snapdragon/Surface-ARM machines / Intel/AMD 选 **x64**，骁龙/ARM Surface 选 **arm64**
