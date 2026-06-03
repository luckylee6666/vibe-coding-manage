# Changelog

All notable changes to this project are documented here. 本项目的更新记录如下。

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
