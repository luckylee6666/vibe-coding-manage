# Vibe Coding Manager

一个用于管理 Vibe Coding 项目的桌面应用，基于 Tauri v2 构建。

## 功能特性

- **项目管理** — 添加、编辑、删除项目
- **运行环境** — 本地电脑 / 服务器
- **服务器管理** — 配置 SSH 服务器（IP、端口、用户名、密码/秘钥登录方式）
- **分组管理** — 项目分组，侧边栏展开/折叠，点击定位
- **内置终端** — 应用内底部抽屉多标签终端，一键在项目目录启动 Claude Code，所有会话集中管理
- **扫描导入** — 扫描目录批量导入 git 项目（自动读取 remote、按路径去重）
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

### macOS 安装说明

由于应用未签名，首次打开会被 macOS 拦截，请按以下步骤授权：

1. 双击 `.dmg` 文件，将 `Vibe Coding Manager.app` 拖入 Applications
2. 首次打开时会提示"已取消"或"移到废纸篓"
3. 打开 **系统设置 → 隐私与安全性**，滚动到底部
4. 在"安全性"区域会看到 `"Vibe Coding Manager"已被阻止打开，因为它来自未验证的开发者`
5. 点击 **仍要打开**，输入密码确认即可
