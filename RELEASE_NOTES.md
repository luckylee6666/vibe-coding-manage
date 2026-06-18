Cross-platform desktop app: macOS (Apple Silicon) + Windows (x64 / ARM64)
跨平台桌面版：macOS (Apple Silicon) + Windows (x64 / ARM64)

## What's new in v1.2.5 / 本版更新

**English**
- **Session attention + notifications**: get a desktop notification + chime (and an amber pulsing tab dot) when an AI CLI goes quiet after working — i.e. it finished or is waiting for you. Toggle via the toolbar bell.
- **Rate-limit usage (no Node needed)**: the Claude usage panel and a new **menu-bar tray** show your real 5h / 7d limit utilization (% + reset countdown) — same source as `/usage`, read from the Keychain token (first read prompts a Keychain authorization).
- **Context % on terminal tabs**: each Claude tab shows its context-window fill; window size is read from Claude Code's startup banner so it's right on 1M and 200K plans.
- **Git status badges** on project cards (branch / changes / ahead-behind) and a **"restore context"** card (git overview + recent commits + changed files + CLAUDE.md summary).
- **Session restore** (re-open last tab layout; Claude with `--continue`) and a **Prompt/snippet library** to inject reusable prompts.
- Cost stats / Codex / OpenCode views still use `ccusage` (via `npx`); without Node they degrade gracefully with a one-click install hint — the rate-limit usage keeps working.

**中文**
- **会话状态感知 + 通知**：AI CLI 干完活后突然安静（跑完/在等你）时，弹桌面通知 + 提示音 + 标签琥珀呼吸点；工具栏铃铛可开关。
- **限流用量（无需 Node）**：Claude 用量面板和新增的**菜单栏托盘**显示真实 5h / 7d 限流使用率（百分比 + 重置倒计时）——和 `/usage` 同源，读钥匙串 token（首次弹钥匙串授权）。
- **终端标签上下文 %**：每个 Claude 标签显示上下文窗口占用；窗口大小读 Claude Code 启动横幅，1M 和 200K 套餐都准。
- 项目卡片 **Git 状态徽标**（分支/改动/领先落后）和**「恢复现场」**卡片（git 概览 + 最近提交 + 改动文件 + CLAUDE.md 摘要）。
- **会话恢复**（重开上次标签布局；Claude 用 `--continue`）和 **Prompt 片段库**（一键注入常用指令）。
- 花费统计 / Codex / OpenCode 仍走 `ccusage`（经 `npx`）；没装 Node 时优雅降级 + 一键安装引导，限流用量照常工作。

## Install / 安装

**macOS** — the app is unsigned / 应用未签名：
1. Open the `.dmg`, drag the app into Applications / 打开 `.dmg`，把应用拖入「应用程序」
2. First launch is blocked → **System Settings → Privacy & Security** → scroll down → **Open Anyway** / 首次打开被拦 → **系统设置 → 隐私与安全性** → 滚到底 → **仍要打开**

**Windows** — unsigned, SmartScreen will warn / 未签名，会弹 SmartScreen：
1. Download the `*-setup.exe` for your architecture and run it / 下载对应架构的 `*-setup.exe` 安装
2. On the SmartScreen prompt → **More info → Run anyway** / SmartScreen 提示 → **更多信息 → 仍要运行**
3. Pick **x64** for Intel/AMD, **arm64** for Snapdragon/Surface-ARM machines / Intel/AMD 选 **x64**，骁龙/ARM Surface 选 **arm64**
