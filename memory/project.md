# Vibe Coding Manager 项目记忆

## 技术栈
- **前端**: Vanilla HTML/CSS/JS
- **后端**: Rust + Tauri v2
- **包管理**: pnpm
- **数据存储**: JSON 文件
  - 项目: `~/Library/Application Support/vibe-coding-manage/projects.json`
  - 服务器: `~/Library/Application Support/vibe-coding-manage/servers.json`
- **Excel 导出**: rust_xlsxwriter
- **内置终端**: Rust `portable-pty`（真实 PTY）+ 前端 `xterm.js` v6 + `addon-fit`（vendor 进 `src/vendor/`，不走 CDN）

## 远端仓库
- `git@github.com:luckylee6666/vibe-coding-manage.git`
- 当前版本: v1.1.0（内置终端 + 服务器管理修复 + 新 logo）
- GitHub Actions 自动构建 macOS ARM64 dmg（推 `v*` 标签触发）

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
- note: String
- created_at: String → createdAt

**注意**: **只记录登录方式（password/key），不存密码、不存秘钥路径**。表单里登录方式下拉框下面没有任何额外输入框（密码框、秘钥路径框、提示框都已删除）。Rust 侧 `Server.key_path` 字段、`add_server`/`update_server` 的 `key_path` 参数、`pick_ssh_key` 命令均已移除；旧 `servers.json` 残留的 `keyPath` 会被 serde 自动忽略。

## 内置终端（底部抽屉，多标签）
- **后端命令**（`src-tauri/src/lib.rs`）: `terminal_create(id, cwd, cols, rows)` 起登录 shell `$SHELL -l`（加载 PATH，claude 才找得到）/ `terminal_write(id, data)` / `terminal_resize(id, cols, rows)` / `terminal_close(id)`
- **输出流**: 后台线程读 PTY → base64（避免切断转义序列/多字节）→ Tauri 事件 `terminal-output`；进程退出推 `terminal-exit`
- **前端**（`main.js` 终端模块 + `index.html` 底部抽屉 + `styles.css` `.terminal-dock`）: 每会话一个 xterm 实例，`Map` 管理；标签横向滚动、运行/已结束状态点、切换/关闭、拖拽调高度、浮动开关（角标=会话数）
- **入口**: 项目卡片「终端」按钮 → 新标签 + cwd + 自动注入 `claude\r`；面板「＋」→ 空白 shell（home，不跑 claude）
- 全局名: `window.Terminal` / `window.FitAddon.FitAddon`；事件用 `window.__TAURI__.event.listen`
- **旧 `open_terminal`（跳系统终端）保留但前端不再调用**，可作未来「外部终端」备选

## 已完成功能
- [x] 项目 CRUD
- [x] 运行环境：本地电脑 / 服务器
- [x] 服务器管理（IP、端口、用户名、密码/秘钥登录方式；仅记录方式不存凭据）
- [x] 项目分组（侧边栏展开/折叠，点击子项目定位高亮）
- [x] 搜索功能
- [x] 导出 Excel（含服务器列）
- [x] 本地数据持久化
- [x] Ant Design 风格 UI
- [x] 扫描目录批量导入 git 项目（按本地路径去重，跳过已存在）
- [x] **内置终端**：底部抽屉多标签，集中管理所有 claude 会话（输入输出双向打通，已 Playwright 实测）
- [x] 侧边栏服务器管理入口 + 项目表单内管理入口（多入口）

## 关键修复
- **Tauri v2 camelCase**: Rust snake_case 参数/字段自动转 camelCase，前端必须用 camelCase
- **serde 兼容**: Project 加 `#[serde(rename_all = "camelCase")]` + `alias` 兼容旧 snake_case 数据
- **submit 双重触发**: 确定按钮设 `type="button"` 避免触发 form 原生 submit
- **服务器删除保护**: 检查是否有项目引用该 server_id，有则阻止删除
- **不存凭据**: Server 不含 password/key_path，只记登录方式；表单仅留「登录方式」下拉
- **终端路径含空格/注入**: `open_terminal` 与 PTY cwd 用 `shell_quote()` 单引号包裹，macOS 再做 AppleScript 两层转义
- **扫描重复导入**: 取消时按 localPath 去重
- **WKWebView 不支持原生弹窗**: `window.confirm/alert/prompt` 在 Tauri WKWebView 里直接返回 false（删除服务器曾因此「点了没反应」）→ 必须用应用内弹窗。通用确认走 `askConfirm(kind, name, onConfirm)` + `pendingConfirm`，确认弹窗 `z-index:1100` 叠在服务器列表之上
- **卡片操作按钮**: 服务器卡片编辑/删除按钮原 `opacity:0` 仅 hover 显示，难发现，已改常显

## 启动命令
```bash
cd /Users/lucky/git/smalltree/self/vibe-coding-manage
pnpm tauri dev
```

## 注意事项
- Tauri 构建脚本在 Claude Code sandbox 里被 SIGKILL，需要用 `dangerouslyDisableSandbox: true`
- `pnpm tauri dev` 关窗口后报 ELIFECYCLE exit 144 是正常退出，不是崩溃
- macOS ARM64 打包：推送 `v*` 标签触发 GitHub Actions 自动构建 dmg
- **xterm vendor 配方**: `pnpm add @xterm/xterm @xterm/addon-fit` → 拷 `node_modules/@xterm/xterm/lib/xterm.js`、`css/xterm.css`、`@xterm/addon-fit/lib/addon-fit.js` 到 `src/vendor/`，`<script>` 经典标签在 `main.js`(module) 前引入
- **PTY/前端联调验证**: 原生窗口没法自动点；用 mock `window.__TAURI__`（回显桩）把真实前端跑进浏览器 + Playwright 驱动，PTY 机制单独用 Rust test 起真 shell 验证（见 lib.rs `test_pty_spawn_echo`）
- **dev 模式程序坞图标不反映 `icon.icns`**: `pnpm tauri dev` 跑的是裸二进制，macOS 程序坞只显示通用/缓存图标；新图标只在打包的 `.app`/dmg 里生效。验证图标要 `pnpm tauri build` 看 `target/release/bundle/macos/*.app`
- **Logo/图标生成**: logo 用 codex 的 `image2` 生图工具生成（`codex exec --sandbox workspace-write -C <repo> "用 image2 ..." < /dev/null`，**后台跑必须 `< /dev/null` 否则卡在读 stdin**）→ 存 `src/assets/logo.png`（侧边栏引用）→ `pnpm tauri icon src/assets/logo.png` 生成全套图标（会多出 ios/android 目录，macOS 桌面应用可删）
