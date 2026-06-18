# Changelog

All notable changes to this project are documented here. 本项目的更新记录如下。

## Unreleased

### English

**Added**
- **Session attention awareness**: the built-in terminal now detects when a session has been actively producing output and then goes quiet — i.e. an AI CLI (Claude/Codex/…) likely finished or is waiting for your input — and pings you. The tab shows an amber pulsing dot; if the window is unfocused or you're on another tab, you get a native desktop notification plus a chime. A bell icon in the terminal toolbar toggles notifications (on by default; the choice is remembered). One-shot prompt prints (a plain shell sitting idle) are filtered out so you only get pinged for real work. Exiting sessions notify too.
- **Git status badges on project cards**: each local project card now shows its current branch, working-tree changes (● tracked / + untracked), and ahead/behind counts vs upstream (↑/↓), or a green ✓ when clean and in sync. Scanned in the background (parallel `git status`), refreshed on launch and whenever the window regains focus.
- **Session restore**: the built-in terminal remembers your open tab layout (working dir + which CLI per tab). On the next launch it offers to restore them — re-launching each CLI in the same directory; Claude tabs come back with `--continue` to pick up the previous conversation.
- **Prompt/snippet library**: a new bookmark icon in the terminal toolbar opens a library of reusable prompts/commands; click one to inject it into the current terminal (text only, no auto-Enter, so you can review before sending — a blank terminal is opened first if none is active). A management dialog lets you add/edit/delete snippets, stored in `snippets.json` alongside your other data.
- **"Restore context" card**: each project card gets a history icon that opens a one-glance snapshot to help you pick up where you left off — git overview (branch / changes / ahead-behind), the 5 most recent commits, the changed-files list, the project's CLAUDE.md summary, and when you last launched a CLI there. Footer buttons jump straight back in (open terminal / open Claude).

### 中文

**新增**
- **会话状态感知 + 通知**：内置终端现在能识别某会话"持续输出了一阵后突然安静"——即 AI CLI（Claude/Codex/…）可能跑完了或在等你输入——并提醒你。标签上出现琥珀色呼吸点；若窗口失焦或你正看着别的标签，会弹原生桌面通知 + 提示音。终端工具栏新增铃铛图标可开关提醒（默认开，选择会记住）。瞬时的提示符打印（空闲的普通 shell）已被过滤，只在真正干活时才提醒。会话退出也会通知。
- **项目卡片 Git 状态徽标**：每个本地项目卡片现在显示当前分支、工作区改动（● 已追踪 / + 未追踪）、相对上游的领先/落后提交数（↑/↓），干净且与上游同步时显示绿色 ✓。后台并行 `git status` 扫描，启动时及窗口重新聚焦时刷新。
- **会话恢复**：内置终端记住你打开的标签布局（每个标签的工作目录 + 所用 CLI）。下次启动时询问是否恢复——在同目录重新拉起对应 CLI；Claude 标签用 `--continue` 接上次对话。
- **Prompt/片段库**：终端工具栏新增书签图标，打开可复用的 Prompt/命令库；点一条即注入当前终端（仅文本、不自动回车，可先检查再发送；无活动终端会先开一个空白的）。管理弹窗可增删改片段，数据存于 `snippets.json`（与其他数据放一起）。
- **"恢复现场"卡片**：每个项目卡片新增历史图标，打开一张速览帮你接回上次的工作——git 概览（分支/改动/领先落后）、最近 5 条提交、改动文件列表、项目 CLAUDE.md 摘要、以及上次在该项目启动了哪个 CLI、多久前。底部按钮可一键接回（打开终端 / 打开 Claude）。

## v1.2.4

### English

**Added**
- **Claude usage panel** (5-hour window): a new clock icon in the terminal toolbar opens a panel showing the current 5-hour billing window — a live countdown to reset, cost so far + projected cost, burn rate ($/hr), total/output tokens, and the active models. Data comes from the community `ccusage` tool reading your local `~/.claude` logs; nothing is uploaded.
- **Auto-hello on window reset**: an optional toggle that, once your 5-hour window has reset / gone idle, automatically fires a tiny `claude -p hello` to immediately open a fresh window so the clock restarts when you want it. A "send hello now" button is also provided for manual triggering.

### 中文

**新增**
- **Claude 用量面板**（5 小时窗口）：终端工具栏新增时钟图标，打开后显示当前 5 小时计费窗口——实时倒计时、本窗口花费 + 预计花费、燃烧速率（美元/小时）、总/输出 token、活跃模型。数据来自社区工具 `ccusage` 读取本机 `~/.claude` 日志，不上传任何数据。
- **窗口重置后自动 hello**：可选开关，当 5 小时窗口重置 / 空闲后，自动发一句极小的 `claude -p hello` 立刻开新窗口，让计时从你想要的时刻重新开始；另有「立刻发一次 hello」按钮可手动触发。

## v1.2.3

### English

**Added**
- AI CLI launch menu now includes **Gemini** (`gemini`) and **agy** (`agy`), in addition to Claude / Codex / opencode. Each gets its own tab badge color (gemini purple, agy cyan).

**Fixed**
- Built-in terminal switched to the WebGL renderer, fixing selection "ghosting" — a blue block smearing across consecutive lines — when scrolling on a macOS trackpad. The default DOM renderer failed to reposition the selection layer on scroll. WebGL falls back to the default renderer gracefully when unavailable.

### 中文

**新增**
- AI CLI 启动菜单新增 **Gemini**（`gemini`）和 **agy**（`agy`），与 Claude / Codex / opencode 并列。各有独立 tab 色标（gemini 紫、agy 青）。

**修复**
- 内置终端改用 WebGL 渲染器，修复 macOS 触控板滚动时选区「ghosting」——一块蓝色高亮糊在连续多行上。默认 DOM 渲染器在滚动时没有重新定位选区层。WebGL 不可用时安全降级回默认渲染器。

## v1.2.2

### English

**Added**
- Rename a group inline: hover a group in the sidebar, click the pencil icon, edit the name, press Enter. All projects in that group are re-assigned in one batch (groups have no standalone entity — they aggregate from each project's `group` field).
- Close-terminal confirmation: closing a terminal tab now prompts first. If the session was started with an AI CLI (claude/codex/…), it reminds you to let the tool "update its memory" before closing, so context isn't lost.

**Changed**
- Title bar now shows the app version.
- Server management moved to the top of the sidebar; the redundant project count was removed.
- The confirm dialog was generalized (title / message / button text / danger style) and now supports multi-line messages.
- DMG installer now uses a "drag to Applications" layout.

**Fixed**
- Confirm dialogs were hidden behind the built-in terminal panel when it was open — their z-index is now raised above it.

### 中文

**新增**
- 分组就地重命名：在侧栏 hover 分组、点铅笔图标、改名后回车。组内所有项目一次性批量迁移（分组没有独立实体，靠各项目的 `group` 字段聚合）。
- 关闭终端前确认：关终端标签会先弹确认。若该会话起的是某个 AI CLI（claude/codex/…），会提醒你先让它「更新记忆」再关，避免上下文丢失。

**变更**
- 标题栏显示应用版本号。
- 服务器管理移到侧栏顶部；去掉冗余的项目数显示。
- 确认弹窗抽象为通用组件（标题 / 内容 / 按钮文案 / 危险样式），支持多行内容。
- DMG 安装界面改为「拖到 Applications」布局。

**修复**
- 内置终端面板打开时确认弹窗会被压在底下——已把弹窗层级提到终端之上。

## v1.2.1

### English

**Added**
- File tree: drag a file or folder onto the terminal to insert its path — handy for pointing an AI session at a specific directory.
- File tree right-click menu: **Open folder** (folder → open in system file manager; file → open its containing folder), **Insert path into terminal**, **Copy path**, **Move to Trash** (recoverable, with confirmation).
- File preview now supports more formats:
  - **Images** (png/jpg/gif/webp/svg/ico/avif) rendered on a checkerboard transparency background.
  - **PDF** rendered inline.
  - **Markdown** rendered as a formatted page, with a Source / Rendered toggle.
  - **CSV / TSV** rendered as a table, switchable back to source.

**Security**
- Markdown preview is sanitized with DOMPurify, and links open in the system browser instead of navigating the app — any untrusted file can be previewed safely.

### 中文

**新增**
- 文件树：把文件/文件夹拖到终端即可插入路径——跟 AI 对话时指定某个目录很方便。
- 文件树右键菜单：**打开文件夹**（文件夹 → 在系统文件管理器打开；文件 → 打开所在文件夹）、**插入路径到终端**、**复制路径**、**移到废纸篓**（可恢复，删除前有确认）。
- 文件预览支持更多格式：
  - **图片**（png/jpg/gif/webp/svg/ico/avif），棋盘格透明底。
  - **PDF** 内嵌渲染。
  - **Markdown** 渲染成排版页面，支持「源码 / 渲染」切换。
  - **CSV / TSV** 渲染成表格，可切回源码。

**安全**
- Markdown 预览经 DOMPurify 净化，链接走系统浏览器而非劫持应用导航——任意来源的文件都能安全预览。

## v1.2.0

### English

- Cross-platform: added **Windows x64 / ARM64** builds alongside macOS (Apple Silicon).
- Built-in terminal: color themes (Default Dark / Homebrew), font size shortcuts (`⌘/Ctrl +/-/0`, `⌘/Ctrl + wheel`), drag a file from the OS into the terminal to insert its path.
- Launch **Claude / Codex / opencode** from a project card, with a tool badge on the tab.
- File tree with lazy loading and read-only syntax-highlighted preview.

### 中文

- 跨平台：在 macOS（Apple Silicon）之外新增 **Windows x64 / ARM64** 构建。
- 内置终端：配色主题（默认深色 / Homebrew）、字号快捷键（`⌘/Ctrl +/-/0`、`⌘/Ctrl + 滚轮`）、从系统拖文件进终端插入路径。
- 项目卡片一键启动 **Claude / Codex / opencode**，标签上有工具色标。
- 文件树（懒加载）+ 只读语法高亮预览。
