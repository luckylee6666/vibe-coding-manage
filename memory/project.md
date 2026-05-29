# Vibe Coding Manager 项目记忆

## 技术栈
- **前端**: Vanilla HTML/CSS/JS
- **后端**: Rust + Tauri v2
- **包管理**: pnpm
- **数据存储**: JSON 文件 (`~/Library/Application Support/vibe-coding-manage/projects.json`)
- **Excel 导出**: rust_xlsxwriter

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
```

## 数据模型 (Project)
- id: String (UUID)
- name: String
- local_path: String
- remote_url: String (可选)
- description: String (可选)
- machine: String (local/ssh-dev/ssh-prod/docker/wsl/other)
- group: String (可选，如：前端、后端、工具)
- created_at: String
- updated_at: String

## 已完成功能
- [x] 项目 CRUD（新增/编辑/删除/查询）
- [x] 项目分组（侧边栏筛选）
- [x] 搜索功能
- [x] 导出 Excel
- [x] 本地数据持久化
- [x] Ant Design 风格 UI

## 已知修复
- 旧数据兼容：所有可选字段加了 `#[serde(default)]`
- XSS 修复：卡片按钮改用事件绑定，不用 onclick 内联
- 表单校验：checkValidity() + reportValidity()
- submit 只绑 form.onsubmit，不绑按钮 onclick（避免重复触发）

## 启动命令
```bash
cd /Users/lucky/git/smalltree/self/vibe-coding-manage
pnpm tauri dev
```

## 注意事项
- Tauri 构建脚本在 Claude Code sandbox 里被 SIGKILL，需要用 `dangerouslyDisableSandbox: true`
- `pnpm tauri dev` 关窗口后报 ELIFECYCLE exit 144 是正常退出，不是崩溃
