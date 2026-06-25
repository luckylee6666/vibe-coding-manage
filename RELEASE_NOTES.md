Cross-platform desktop app: macOS (Apple Silicon) + Windows (x64 / ARM64)
跨平台桌面版：macOS (Apple Silicon) + Windows (x64 / ARM64)

## What's new in v1.2.6 / 本版更新

**English**
- **Requirements list**: a lightweight inbox for the stray feature ideas you jot down while coding — a sidebar "Requirements" entry (with an open-count badge) opens a quick-capture box (type + Enter to save), with To-do / Doing / Done filters, inline editing, priority, and optional project tagging.
- **Floating snippet quick-panel**: a collapsible card panel in the bottom-right of the terminal — a single click injects a snippet **and presses Enter** (one-click send, no dropdown step, no manual Enter).
- **Rate-limit usage fix**: the tray/panel could freeze on an hours-old value when a refresh silently failed. It now marks stale data (`⚠`), shows "updated X min ago" + the real failure reason, surfaces the actual error (curl stderr + HTTP status), and calls `curl` by absolute path.
- **Global app log** (`logs/app.log`) for easy troubleshooting — startup, usage refreshes, data writes, terminal/remote lifecycle, uncaught front-end errors (never tokens/PINs); open it from the tray's **"Open log"**.
- **Hardened storage**: atomic data writes (temp + rename, no half-written corruption) with `*.bad` backup of unparseable files; serialized saves so rapid edits can't drop an item.

**中文**
- **需求清单**：写代码时随口冒出的碎片需求/想法的轻量收集箱——侧栏「需求清单」入口（带未完成角标）打开速记框（输入回车即存），含 待办/进行中/已完成 过滤、行内编辑、优先级、可选关联项目。
- **片段快捷悬浮面板**：终端右下角可收起的卡片浮层——**单击一条 = 注入并自动回车**（一次点击直接发送，免开下拉、免手按回车）。
- **限流用量修复**：刷新静默失败时，托盘/面板会冻在几小时前的旧值。现在对过期数据加 `⚠`、显示「X 分钟前更新」+ 真实失败原因、抓出实际错误（curl stderr + HTTP 状态码），并用绝对路径调用 `curl`。
- **全局应用日志**（`logs/app.log`）便于排查——记录启动、用量刷新、数据写入、终端/手机服务生命周期、前端未捕获异常（绝不记 token/PIN）；从托盘**「打开日志」**打开。
- **存储加固**：数据原子写（临时文件 + rename，杜绝半截文件损坏）+ 解析失败文件备份为 `*.bad`；保存串行化，连续快速增改不丢条目。

## Install / 安装

**macOS** — the app is unsigned / 应用未签名：
1. Open the `.dmg`, drag the app into Applications / 打开 `.dmg`，把应用拖入「应用程序」
2. First launch is blocked → **System Settings → Privacy & Security** → scroll down → **Open Anyway** / 首次打开被拦 → **系统设置 → 隐私与安全性** → 滚到底 → **仍要打开**

**Windows** — unsigned, SmartScreen will warn / 未签名，会弹 SmartScreen：
1. Download the `*-setup.exe` for your architecture and run it / 下载对应架构的 `*-setup.exe` 安装
2. On the SmartScreen prompt → **More info → Run anyway** / SmartScreen 提示 → **更多信息 → 仍要运行**
3. Pick **x64** for Intel/AMD, **arm64** for Snapdragon/Surface-ARM machines / Intel/AMD 选 **x64**，骁龙/ARM Surface 选 **arm64**
