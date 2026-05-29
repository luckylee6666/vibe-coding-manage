# Vibe Coding Manager 项目记忆

## 技术栈
- **前端**: Vanilla HTML/CSS/JS
- **后端**: Rust + Tauri v2
- **包管理**: pnpm
- **数据存储**: JSON 文件
  - 项目: `~/Library/Application Support/vibe-coding-manage/projects.json`
  - 服务器: `~/Library/Application Support/vibe-coding-manage/servers.json`
- **Excel 导出**: rust_xlsxwriter

## 远端仓库
- `git@github.com:luckylee6666/vibe-coding-manage.git`
- 当前版本: v1.0.0
- GitHub Actions 自动构建 macOS ARM64 dmg

## 项目结构
```
src/
  index.html      - 主页面（Ant Design 风格）
  main.js         - 前端逻辑
  styles.css      - 样式
src-tauri/
  src/lib.rs      - Rust 后端（Tauri commands）
  src/main.rs     - 入口
  Cargo.toml      - Rust 依赖
  tauri.conf.json - Tauri 配置
.github/
  workflows/
    build.yml     - GitHub Actions macOS ARM64 构建
```

## 数据模型

### Project
- id: String (UUID)
- name: String
- local_path: String → 序列化为 localPath（camelCase）
- remote_url: String → remoteUrl
- description: String
- machine: String (local/server)
- server_id: String → serverId（关联 Server）
- group: String（如：前端、后端、工具）
- created_at: String → createdAt
- updated_at: String → updatedAt

### Server
- id: String (UUID)
- name: String
- host: String
- port: u16 (默认 22)
- user: String
- auth_type: String → authType (password/key)
- key_path: String → keyPath（秘钥路径）
- note: String
- created_at: String → createdAt

**注意**: 不存储密码，只记录登录方式。密码连接时手动输入。

## 已完成功能
- [x] 项目 CRUD
- [x] 运行环境：本地电脑 / 服务器
- [x] 服务器管理（IP、端口、用户名、密码/秘钥方式、秘钥路径）
- [x] 项目分组（侧边栏展开/折叠，点击子项目定位高亮）
- [x] 搜索功能
- [x] 导出 Excel（含服务器列）
- [x] 本地数据持久化
- [x] Ant Design 风格 UI
- [x] 一键打开终端并启动 claude（支持 macOS/Linux/Windows）
- [x] 侧边栏服务器管理入口 + 项目表单内管理入口（多入口）

## 关键修复
- **Tauri v2 camelCase**: Rust snake_case 参数/字段自动转 camelCase，前端必须用 camelCase
- **serde 兼容**: Project 加 `#[serde(rename_all = "camelCase")]` + `alias` 兼容旧 snake_case 数据
- **submit 双重触发**: 确定按钮设 `type="button"` 避免触发 form 原生 submit
- **服务器删除保护**: 检查是否有项目引用该 server_id，有则阻止删除
- **密码不存储**: Server 模型不含 password 字段

## 启动命令
```bash
cd /Users/lucky/git/smalltree/self/vibe-coding-manage
pnpm tauri dev
```

## 注意事项
- Tauri 构建脚本在 Claude Code sandbox 里被 SIGKILL，需要用 `dangerouslyDisableSandbox: true`
- `pnpm tauri dev` 关窗口后报 ELIFECYCLE exit 144 是正常退出，不是崩溃
- macOS ARM64 打包：推送 `v*` 标签触发 GitHub Actions 自动构建 dmg
