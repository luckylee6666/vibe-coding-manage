# Vibe Coding Manager

一个用于管理 Vibe Coding 项目的桌面应用。

## 功能特性

- 📁 项目管理：添加、编辑、删除项目
- 🔍 搜索功能：快速搜索项目
- 📊 数据导出：导出项目数据为 Excel 文件
- 💾 本地存储：数据自动保存到本地
- 🎨 现代 UI：美观的深色主题界面

## 项目信息字段

- 项目名称
- 本地路径
- 远端仓库地址
- 项目描述
- 运行环境（本地电脑、SSH 开发机、SSH 生产机、Docker、WSL 等）

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

- macOS: `~/Library/Application Support/vibe-coding-manage/projects.json`
- Windows: `%APPDATA%/vibe-coding-manage/projects.json`
- Linux: `~/.local/share/vibe-coding-manage/projects.json`
