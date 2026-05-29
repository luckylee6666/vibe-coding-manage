# Vibe Coding Manager

一个用于管理 Vibe Coding 项目的桌面应用，基于 Tauri v2 构建。

## 功能特性

- **项目管理** — 添加、编辑、删除项目
- **运行环境** — 本地电脑 / 服务器
- **服务器管理** — 配置 SSH 服务器（IP、端口、用户名、密码/秘钥登录方式）
- **分组管理** — 项目分组，侧边栏展开/折叠，点击定位
- **一键终端** — 打开终端并启动 Claude Code
- **搜索过滤** — 快速搜索项目名称、路径、描述
- **数据导出** — 导出项目数据为 Excel 文件
- **本地存储** — 数据自动保存到本地 JSON 文件

## 项目信息字段

- 项目名称
- 本地路径
- 远端仓库地址
- 项目分组
- 运行环境（本地电脑 / 服务器）
- 服务器关联
- 项目描述

## 截图

> 深色主题 UI，Ant Design 风格

## 开发环境要求

- Node.js 18+
- pnpm
- Rust 1.70+

## 安装依赖

```bash
pnpm install
```

## 开发运行

```bash
pnpm tauri dev
```

## 构建打包

```bash
pnpm tauri build
```

## 技术栈

- **前端**: HTML/CSS/JavaScript (Vanilla)
- **后端**: Rust + Tauri v2
- **数据存储**: JSON 文件
- **Excel 导出**: rust_xlsxwriter

## 数据存储位置

- macOS: `~/Library/Application Support/vibe-coding-manage/`
  - `projects.json` — 项目数据
  - `servers.json` — 服务器配置
- Windows: `%APPDATA%/vibe-coding-manage/`
- Linux: `~/.local/share/vibe-coding-manage/`

## 下载

前往 [Releases](https://github.com/luckylee6666/vibe-coding-manage/releases) 下载 macOS ARM64 (Apple Silicon) 版本。
