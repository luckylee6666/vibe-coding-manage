use base64::Engine;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

mod remote;
use remote::{PtySession, RemoteHub, SessionMeta};

/// 手机端远程服务监听端口（局域网）。
const REMOTE_PORT: u16 = 8787;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(alias = "local_path")]
    pub local_path: String,
    #[serde(default, alias = "remote_url")]
    pub remote_url: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub machine: String,
    #[serde(default)]
    pub server_id: String,
    #[serde(default)]
    pub group: String,
    #[serde(default, alias = "created_at")]
    pub created_at: String,
    #[serde(default, alias = "updated_at")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: String,
    pub name: String,
    pub host: String,
    #[serde(default)]
    pub port: u16,
    pub user: String,
    #[serde(default)]
    pub auth_type: String,
    #[serde(default)]
    pub note: String,
    #[serde(default, alias = "created_at")]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppState {
    projects: Vec<Project>,
    servers: Vec<Server>,
    data_path: PathBuf,
    server_path: PathBuf,
}

impl AppState {
    fn new() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("vibe-coding-manage");
        
        fs::create_dir_all(&data_dir).ok();
        
        let data_path = data_dir.join("projects.json");
        let projects = if data_path.exists() {
            let data = fs::read_to_string(&data_path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };

        let server_path = data_dir.join("servers.json");
        let servers = if server_path.exists() {
            let data = fs::read_to_string(&server_path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };

        Self { projects, servers, data_path, server_path }
    }

    fn save_projects(&self) -> Result<(), String> {
        let data = serde_json::to_string_pretty(&self.projects)
            .map_err(|e| e.to_string())?;
        fs::write(&self.data_path, data).map_err(|e| e.to_string())
    }

    fn save_servers(&self) -> Result<(), String> {
        let data = serde_json::to_string_pretty(&self.servers)
            .map_err(|e| e.to_string())?;
        fs::write(&self.server_path, data).map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn get_projects(state: State<Mutex<AppState>>) -> Result<Vec<Project>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.projects.clone())
}

#[tauri::command]
fn add_project(
    state: State<Mutex<AppState>>,
    name: String,
    local_path: String,
    remote_url: String,
    description: String,
    machine: String,
    server_id: String,
    group: String,
) -> Result<Project, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let project = Project {
        id: Uuid::new_v4().to_string(),
        name,
        local_path,
        remote_url,
        description,
        machine,
        server_id,
        group,
        created_at: now.clone(),
        updated_at: now,
    };
    
    state.projects.push(project.clone());
    state.save_projects()?;
    
    Ok(project)
}

#[tauri::command]
fn update_project(
    state: State<Mutex<AppState>>,
    id: String,
    name: String,
    local_path: String,
    remote_url: String,
    description: String,
    machine: String,
    server_id: String,
    group: String,
) -> Result<Project, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    
    let index = state.projects.iter().position(|p| p.id == id)
        .ok_or_else(|| "Project not found".to_string())?;
    
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let project = Project {
        id: id.clone(),
        name,
        local_path,
        remote_url,
        description,
        machine,
        server_id,
        group,
        created_at: state.projects[index].created_at.clone(),
        updated_at: now,
    };
    
    state.projects[index] = project.clone();
    state.save_projects()?;
    
    Ok(project)
}

#[tauri::command]
fn delete_project(state: State<Mutex<AppState>>, id: String) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.projects.retain(|p| p.id != id);
    state.save_projects()?;
    Ok(())
}

/// 重命名分组：把该组下所有项目的 group 字段批量改名（分组无独立实体，靠 group 字段聚合）。
#[tauri::command]
fn rename_group(state: State<Mutex<AppState>>, old: String, new: String) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    for p in state.projects.iter_mut() {
        if p.group == old {
            p.group = new.clone();
        }
    }
    state.save_projects()?;
    Ok(())
}

#[tauri::command]
fn export_excel(state: State<Mutex<AppState>>) -> Result<String, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    
    let file_path = rfd::FileDialog::new()
        .set_title("导出Excel文件")
        .set_file_name("vibe-coding-projects.xlsx")
        .add_filter("Excel文件", &["xlsx"])
        .save_file()
        .ok_or_else(|| "未选择保存位置".to_string())?;

    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();
    
    let header_format = rust_xlsxwriter::Format::new()
        .set_bold()
        .set_background_color(rust_xlsxwriter::Color::RGB(0x4472C4))
        .set_font_color(rust_xlsxwriter::Color::White)
        .set_border(rust_xlsxwriter::FormatBorder::Thin);
    
    worksheet.write_string_with_format(0, 0, "项目名称", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_string_with_format(0, 1, "分组", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_string_with_format(0, 2, "本地路径", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_string_with_format(0, 3, "远端仓库", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_string_with_format(0, 4, "项目描述", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_string_with_format(0, 5, "运行环境", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_string_with_format(0, 6, "服务器", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_string_with_format(0, 7, "创建时间", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_string_with_format(0, 8, "更新时间", &header_format).map_err(|e| e.to_string())?;
    
    worksheet.set_column_width(0, 20).map_err(|e| e.to_string())?;
    worksheet.set_column_width(1, 15).map_err(|e| e.to_string())?;
    worksheet.set_column_width(2, 40).map_err(|e| e.to_string())?;
    worksheet.set_column_width(3, 40).map_err(|e| e.to_string())?;
    worksheet.set_column_width(4, 30).map_err(|e| e.to_string())?;
    worksheet.set_column_width(5, 12).map_err(|e| e.to_string())?;
    worksheet.set_column_width(6, 15).map_err(|e| e.to_string())?;
    worksheet.set_column_width(7, 20).map_err(|e| e.to_string())?;
    worksheet.set_column_width(8, 20).map_err(|e| e.to_string())?;
    
    let data_format = rust_xlsxwriter::Format::new()
        .set_border(rust_xlsxwriter::FormatBorder::Thin);
    
    for (i, project) in state.projects.iter().enumerate() {
        let row = (i + 1) as u32;
        let server_name = if project.server_id.is_empty() {
            String::new()
        } else {
            state.servers.iter()
                .find(|s| s.id == project.server_id)
                .map(|s| s.name.clone())
                .unwrap_or_default()
        };
        let machine_label = match project.machine.as_str() {
            "local" => "本地电脑",
            "server" => "服务器",
            other => other,
        };
        worksheet.write_string_with_format(row, 0, &project.name, &data_format).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(row, 1, &project.group, &data_format).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(row, 2, &project.local_path, &data_format).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(row, 3, &project.remote_url, &data_format).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(row, 4, &project.description, &data_format).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(row, 5, machine_label, &data_format).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(row, 6, &server_name, &data_format).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(row, 7, &project.created_at, &data_format).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(row, 8, &project.updated_at, &data_format).map_err(|e| e.to_string())?;
    }
    
    workbook.save(&file_path).map_err(|e| e.to_string())?;
    
    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    opener::open(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_folder_dialog() -> Result<String, String> {
    let folder = rfd::FileDialog::new()
        .set_title("选择项目文件夹")
        .pick_folder()
        .ok_or_else(|| "未选择文件夹".to_string())?;
    
    Ok(folder.to_string_lossy().to_string())
}

/// 把路径包成单引号 shell 字面量（转义内部单引号），
/// 防止空格、`;`、反引号、`$()` 等破坏命令或被注入执行。
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[tauri::command]
fn open_terminal(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // 两层转义：先 shell 单引号包路径，再为 AppleScript 字符串转义 \ 和 "
        let cmd = format!("cd {} && claude", shell_quote(&path));
        let as_escaped = cmd.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            "tell application \"Terminal\"\n\tactivate\n\tdo script \"{}\"\nend tell",
            as_escaped
        );
        std::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        // bash -c 收到独立 argv，路径再用单引号包裹，无嵌套引号问题
        std::process::Command::new("x-terminal-emulator")
            .args([
                "-e",
                "bash",
                "-c",
                &format!("cd {} && claude; exec bash", shell_quote(&path)),
            ])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        // Windows 路径用双引号包裹（路径通常不含 "）
        std::process::Command::new("cmd")
            .args(["/C", "start", "cmd", "/K", &format!("cd /d \"{}\" && claude", path)])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_servers(state: State<Mutex<AppState>>) -> Result<Vec<Server>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.servers.clone())
}

#[tauri::command]
fn add_server(
    state: State<Mutex<AppState>>,
    name: String,
    host: String,
    port: u16,
    user: String,
    auth_type: String,
    note: String,
) -> Result<Server, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let server = Server {
        id: Uuid::new_v4().to_string(),
        name,
        host,
        port,
        user,
        auth_type,
        note,
        created_at: now,
    };
    state.servers.push(server.clone());
    state.save_servers()?;
    Ok(server)
}

#[tauri::command]
fn update_server(
    state: State<Mutex<AppState>>,
    id: String,
    name: String,
    host: String,
    port: u16,
    user: String,
    auth_type: String,
    note: String,
) -> Result<Server, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let index = state.servers.iter().position(|s| s.id == id)
        .ok_or_else(|| "Server not found".to_string())?;
    let server = Server {
        id: id.clone(),
        name,
        host,
        port,
        user,
        auth_type,
        note,
        created_at: state.servers[index].created_at.clone(),
    };
    state.servers[index] = server.clone();
    state.save_servers()?;
    Ok(server)
}

#[tauri::command]
fn delete_server(state: State<Mutex<AppState>>, id: String) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let count = state.projects.iter().filter(|p| p.server_id == id).count();
    if count > 0 {
        return Err(format!("有 {} 个项目引用了该服务器，请先修改项目", count));
    }
    state.servers.retain(|s| s.id != id);
    state.save_servers()?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedProject {
    pub name: String,
    pub path: String,
    pub remote_url: String,
    pub group: String,
}

#[tauri::command]
fn scan_directory(path: String) -> Result<Vec<ScannedProject>, String> {
    let dir = std::path::Path::new(&path);
    if !dir.is_dir() {
        return Err("路径不是目录".to_string());
    }

    let dir_name = dir.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut results = Vec::new();

    // 检查当前目录是否本身就是 git 仓库
    if dir.join(".git").exists() {
        let remote = read_git_remote(dir);
        results.push(ScannedProject {
            name: dir_name.clone(),
            path: dir.to_string_lossy().to_string(),
            remote_url: remote,
            group: String::new(),
        });
        return Ok(results);
    }

    // 扫描子目录
    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }
        // 跳过隐藏目录
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if entry_path.join(".git").exists() {
            let remote = read_git_remote(&entry_path);
            results.push(ScannedProject {
                name,
                path: entry_path.to_string_lossy().to_string(),
                remote_url: remote,
                group: dir_name.clone(),
            });
        }
    }

    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

fn read_git_remote(repo_path: &std::path::Path) -> String {
    let config_path = repo_path.join(".git").join("config");
    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    // 简单解析 git config 找 remote "origin" 的 url
    let mut in_origin = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[remote \"origin\"]" {
            in_origin = true;
            continue;
        }
        if in_origin && trimmed.starts_with('[') {
            break;
        }
        if in_origin {
            if let Some(url) = trimmed.strip_prefix("url = ") {
                return url.trim().to_string();
            }
            if let Some(url) = trimmed.strip_prefix("url=") {
                return url.trim().to_string();
            }
        }
    }
    String::new()
}

#[tauri::command]
fn open_pick_directory() -> Result<String, String> {
    let dir = rfd::FileDialog::new()
        .set_title("选择要扫描的目录")
        .pick_folder()
        .ok_or_else(|| "未选择目录".to_string())?;
    Ok(dir.to_string_lossy().to_string())
}

// ========== 文件树 / 文件预览 ==========

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DirEntryInfo {
    name: String,
    path: String,
    is_dir: bool,
}

/// 列出目录直接子项（懒加载用），目录在前、再按名称不区分大小写排序。
#[tauri::command]
fn list_dir(path: String) -> Result<Vec<DirEntryInfo>, String> {
    let dir = std::path::Path::new(&path);
    if !dir.is_dir() {
        return Err("路径不是目录".to_string());
    }
    let mut result = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let p = entry.path();
        result.push(DirEntryInfo {
            name: entry.file_name().to_string_lossy().to_string(),
            path: p.to_string_lossy().to_string(),
            is_dir: p.is_dir(),
        });
    }
    result.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(result)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileContent {
    content: String,
    truncated: bool,
    size: u64,
}

/// 读取文本文件内容预览：>1MB 截断，含 NUL 字节判为二进制拒绝。
#[tauri::command]
fn read_file(path: String) -> Result<FileContent, String> {
    let p = std::path::Path::new(&path);
    if !p.is_file() {
        return Err("不是文件".to_string());
    }
    const MAX: u64 = 1024 * 1024; // 1MB
    let size = fs::metadata(p).map_err(|e| e.to_string())?.len();
    // 只读到上限，避免把超大文件整个读进内存
    let mut bytes = Vec::new();
    fs::File::open(p)
        .map_err(|e| e.to_string())?
        .take(MAX)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let probe = &bytes[..bytes.len().min(8000)];
    if probe.contains(&0) {
        return Err("二进制文件，无法预览".to_string());
    }
    let truncated = size > MAX;
    Ok(FileContent {
        content: String::from_utf8_lossy(&bytes).to_string(),
        truncated,
        size,
    })
}

/// 读取图片文件 → base64 data URL（供 <img> 直接显示）。>16MB 拒绝。
#[tauri::command]
fn read_image(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);
    if !p.is_file() {
        return Err("不是文件".to_string());
    }
    const MAX: u64 = 16 * 1024 * 1024;
    let size = fs::metadata(p).map_err(|e| e.to_string())?.len();
    if size > MAX {
        return Err("图片过大（>16MB）".to_string());
    }
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "svg" => "image/svg+xml",
        "avif" => "image/avif",
        _ => "application/octet-stream",
    };
    let bytes = fs::read(p).map_err(|e| e.to_string())?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

/// 读取任意文件 → base64（供前端转 Blob 显示，如 PDF）。>32MB 拒绝。
#[tauri::command]
fn read_binary_base64(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);
    if !p.is_file() {
        return Err("不是文件".to_string());
    }
    const MAX: u64 = 32 * 1024 * 1024;
    let size = fs::metadata(p).map_err(|e| e.to_string())?.len();
    if size > MAX {
        return Err("文件过大（>32MB）".to_string());
    }
    let bytes = fs::read(p).map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

/// 把文件/文件夹移到系统废纸篓（可恢复，不永久删除）。
#[tauri::command]
fn trash_path(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err("路径不存在".to_string());
    }
    trash::delete(p).map_err(|e| e.to_string())
}

// ========== 内置终端（PTY）==========

/// 终端状态：持有共享的 RemoteHub（会话表 / 滚动缓存 / 广播通道 / PIN）。
/// 桌面命令与内嵌的手机端服务都操作同一个 hub（克隆即共享 Arc）。
struct TerminalState {
    hub: RemoteHub,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            hub: RemoteHub::new(REMOTE_PORT),
        }
    }
}

#[derive(Clone, Serialize)]
struct TerminalOutput {
    id: String,
    /// base64 编码的原始字节（避免 UTF-8 切断转义序列 / 多字节字符）
    data: String,
}

/// 一个可访问地址（带网络类型标注 + 扫码用二维码）。
#[derive(Clone, Serialize)]
struct RemoteAddr {
    /// 网络类型："局域网" / "其他"
    kind: String,
    ip: String,
    url: String,
    /// 二维码 SVG（编码 url?k=PIN，扫码即自动带 PIN 登录）
    qr: String,
}

/// 手机端连接信息，桌面 UI 展示给用户。
#[derive(Clone, Serialize)]
struct RemoteInfo {
    addrs: Vec<RemoteAddr>,
    port: u16,
    pin: String,
}

/// 创建一个新的终端会话，在 `cwd` 起一个登录 shell，并把输出流式推到前端。
#[tauri::command]
fn terminal_create(
    app: AppHandle,
    state: State<TerminalState>,
    id: String,
    cwd: String,
    cols: u16,
    rows: u16,
    name: Option<String>,
    tool: Option<String>,
) -> Result<(), String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: rows.max(1),
            cols: cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    // 选 shell：Unix 用用户默认 shell 的登录交互模式（加载 PATH/别名）；Windows 用 PowerShell
    #[cfg(not(target_os = "windows"))]
    let mut cmd = {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let mut c = CommandBuilder::new(&shell);
        c.arg("-l");
        c
    };
    #[cfg(target_os = "windows")]
    let mut cmd = CommandBuilder::new("powershell.exe");

    if !cwd.is_empty() && std::path::Path::new(&cwd).is_dir() {
        cmd.cwd(&cwd);
    }
    cmd.env("TERM", "xterm-256color");

    let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    // slave 句柄在 spawn 后即可释放，否则子进程退出时读端不会收到 EOF
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

    // 后台线程持续读 PTY 输出：base64 后同时推给桌面窗口（Tauri 事件）和手机端（WS 广播 + 滚动缓存）
    let app_evt = app.clone();
    let sid = id.clone();
    let hub_evt = state.hub.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = &buf[..n];
                    // 同一块只 base64 一次，桌面事件与手机端广播共用（手机没连也只编这一次）
                    let data = base64::engine::general_purpose::STANDARD.encode(chunk);
                    let _ = app_evt.emit(
                        "terminal-output",
                        TerminalOutput {
                            id: sid.clone(),
                            data: data.clone(),
                        },
                    );
                    hub_evt.publish(&sid, chunk, data);
                }
                Err(_) => break,
            }
        }
        let _ = app_evt.emit("terminal-exit", &sid);
        hub_evt.mark_exit(&sid);
    });

    // 注册会话元信息（供手机端列表展示）
    state.hub.metas.lock().map_err(|e| e.to_string())?.insert(
        id.clone(),
        SessionMeta {
            id: id.clone(),
            name: name.unwrap_or_else(|| id.clone()),
            tool: tool.unwrap_or_default(),
        },
    );

    state
        .hub
        .sessions
        .lock()
        .map_err(|e| e.to_string())?
        .insert(
            id,
            PtySession {
                master: pair.master,
                writer,
                child,
            },
        );
    Ok(())
}

/// 把前端的键入（已是 UTF-8 文本）写进对应会话的 PTY。
#[tauri::command]
fn terminal_write(state: State<TerminalState>, id: String, data: String) -> Result<(), String> {
    let mut sessions = state.hub.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions.get_mut(&id).ok_or("会话不存在")?;
    session
        .writer
        .write_all(data.as_bytes())
        .map_err(|e| e.to_string())?;
    session.writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// 终端尺寸变化时同步 PTY 窗口大小（让 TUI 正确换行）。
#[tauri::command]
fn terminal_resize(
    state: State<TerminalState>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let sessions = state.hub.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions.get(&id).ok_or("会话不存在")?;
    session
        .master
        .resize(PtySize {
            rows: rows.max(1),
            cols: cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 关闭并清理一个会话（杀掉子进程）。
#[tauri::command]
fn terminal_close(state: State<TerminalState>, id: String) -> Result<(), String> {
    // 三个表（sessions/metas/scrollback）统一在 cleanup_session 里清，
    // 与 reader 线程 EOF 路径（mark_exit）共用同一处逻辑，避免漏删某个表泄漏。
    if let Some(mut session) = state.hub.cleanup_session(&id) {
        let _ = session.child.kill();
    }
    Ok(())
}

/// 把一个 IPv4 分类为「局域网」/「其他」；返回 None 表示该地址不适合展示
/// （回环、链路本地，以及 Tailscale/CGNAT 100.64.0.0/10——先不接 Tailscale，直接排除）。
fn classify_ipv4(ip: std::net::Ipv4Addr) -> Option<&'static str> {
    if ip.is_loopback() || ip.is_link_local() || ip.is_unspecified() {
        return None;
    }
    let o = ip.octets();
    // Tailscale / CGNAT 100.64.0.0/10：本期不展示
    if o[0] == 100 && (64..=127).contains(&o[1]) {
        return None;
    }
    if o[0] == 10
        || (o[0] == 172 && (16..=31).contains(&o[1]))
        || (o[0] == 192 && o[1] == 168)
    {
        return Some("局域网");
    }
    Some("其他")
}

/// 把字符串编码成二维码 SVG（深蓝点 + 白底，留白边便于扫描）。
fn make_qr_svg(data: &str) -> String {
    use qrcode::render::svg;
    match qrcode::QrCode::new(data.as_bytes()) {
        Ok(code) => code
            .render::<svg::Color>()
            .min_dimensions(168, 168)
            .quiet_zone(true)
            .dark_color(svg::Color("#0f172a"))
            .light_color(svg::Color("#ffffff"))
            .build(),
        Err(_) => String::new(),
    }
}

/// 生成 6 位随机 PIN（取 UUID 前 4 字节 mod 1_000_000，左补零）。
fn random_pin() -> String {
    let b = Uuid::new_v4().into_bytes();
    format!("{:06}", u32::from_le_bytes([b[0], b[1], b[2], b[3]]) % 1_000_000)
}

/// 按需启动手机端服务：用户首次打开「手机远程」面板时才生成随机 PIN 并监听端口。
/// 不打开就永不对外暴露；幂等（compare_exchange 保证只起一次）。
fn ensure_remote_started(hub: &remote::RemoteHub) {
    if hub.start_if_needed() {
        let pin = random_pin();
        if let Ok(mut t) = hub.token.lock() {
            *t = pin;
        }
        remote::spawn_server(hub.clone());
    }
}

/// 返回手机端连接信息（局域网地址 + PIN），桌面 UI 展示。
/// 枚举所有网卡，局域网地址排前面；多网卡（有线/WiFi）全部列出供选择。
#[tauri::command]
fn terminal_remote_info(state: State<TerminalState>) -> RemoteInfo {
    ensure_remote_started(&state.hub);
    let port = state.hub.port;
    let pin = state.hub.token.lock().map(|t| t.clone()).unwrap_or_default();

    let mut addrs: Vec<RemoteAddr> = Vec::new();
    if let Ok(ifaces) = local_ip_address::list_afinet_netifas() {
        for (_name, ip) in ifaces {
            if let std::net::IpAddr::V4(v4) = ip {
                if let Some(kind) = classify_ipv4(v4) {
                    let s = v4.to_string();
                    if addrs.iter().any(|a| a.ip == s) {
                        continue; // 去重
                    }
                    let url = format!("http://{s}:{port}");
                    // 二维码编码 url?k=PIN，手机扫码打开即自动登录
                    let qr = make_qr_svg(&format!("{url}/?k={pin}"));
                    addrs.push(RemoteAddr {
                        kind: kind.to_string(),
                        url,
                        ip: s,
                        qr,
                    });
                }
            }
        }
    }
    // 局域网优先
    addrs.sort_by_key(|a| if a.kind == "局域网" { 0 } else { 1 });

    RemoteInfo { addrs, port, pin }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Mutex::new(AppState::new());
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .manage(TerminalState::default())
        .setup(|app| {
            // 版本号显示在原生标题栏（来自 Cargo.toml，单一来源）
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_title(&format!(
                    "Vibe Coding Manager v{}",
                    env!("CARGO_PKG_VERSION")
                ));
            }
            // 手机端服务不在启动时常驻：PIN 随机化 + 端口监听都推迟到用户首次打开
            // 「手机远程」面板（terminal_remote_info → ensure_remote_started）。
            // 不用该功能就永远不对外暴露端口。
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_projects,
            add_project,
            update_project,
            delete_project,
            rename_group,
            export_excel,
            open_folder,
            open_folder_dialog,
            open_terminal,
            get_servers,
            add_server,
            update_server,
            delete_server,
            scan_directory,
            open_pick_directory,
            list_dir,
            read_file,
            read_image,
            read_binary_base64,
            trash_path,
            terminal_create,
            terminal_write,
            terminal_resize,
            terminal_close,
            terminal_remote_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_serde_roundtrip() {
        let p = Project {
            id: "test-id".into(),
            name: "测试项目".into(),
            local_path: "/Users/test/project".into(),
            remote_url: "https://github.com/test/repo".into(),
            description: "desc".into(),
            machine: "local".into(),
            server_id: String::new(),
            group: "前端".into(),
            created_at: "2025-01-01 00:00:00".into(),
            updated_at: "2025-01-01 00:00:00".into(),
        };
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("localPath"), "JSON should use camelCase: {}", json);
        assert!(json.contains("remoteUrl"), "JSON should use camelCase: {}", json);
        assert!(json.contains("createdAt"), "JSON should use camelCase: {}", json);

        // 验证能从 camelCase JSON 反序列化
        let p2: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(p2.name, "测试项目");
        assert_eq!(p2.local_path, "/Users/test/project");

        // 验证也能从 snake_case JSON 反序列化（兼容旧数据）
        let snake = r#"{"id":"x","name":"t","local_path":"/tmp","remote_url":"","description":"","machine":"local","group":"","created_at":"","updated_at":""}"#;
        let p3: Project = serde_json::from_str(snake).unwrap();
        assert_eq!(p3.local_path, "/tmp");
    }

    #[test]
    fn test_add_and_save() {
        let dir = std::env::temp_dir().join("vibe-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let data_path = dir.join("projects.json");

        let mut state = AppState {
            projects: vec![],
            servers: vec![],
            data_path: data_path.clone(),
            server_path: dir.join("servers.json"),
        };

        let now = "2025-01-01 00:00:00".to_string();
        let project = Project {
            id: Uuid::new_v4().to_string(),
            name: "test".into(),
            local_path: "/tmp".into(),
            remote_url: String::new(),
            description: String::new(),
            machine: "local".into(),
            server_id: String::new(),
            group: String::new(),
            created_at: now.clone(),
            updated_at: now,
        };
        state.projects.push(project);
        state.save_projects().unwrap();

        let data = std::fs::read_to_string(&data_path).unwrap();
        let loaded: Vec<Project> = serde_json::from_str(&data).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "test");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 验证 base64 对终端原始字节（含转义序列、UTF-8 多字节、控制字符）能无损往返。
    #[test]
    fn test_terminal_base64_roundtrip() {
        let data: &[u8] = b"\x1b[31m\xe4\xbd\xa0\xe5\xa5\xbd\x07\x1b[0m"; // ESC[31m 你好 BEL ESC[0m
        let enc = base64::engine::general_purpose::STANDARD.encode(data);
        let dec = base64::engine::general_purpose::STANDARD.decode(&enc).unwrap();
        assert_eq!(dec, data, "base64 应无损还原原始终端字节");
    }

    /// 验证本机能真正打开 PTY、起一个 shell、写入命令并读回输出（内置终端的核心机制）。
    #[test]
    fn test_pty_spawn_echo() {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("openpty 失败");

        let cmd = CommandBuilder::new("/bin/sh");
        let mut child = pair.slave.spawn_command(cmd).expect("spawn shell 失败");
        drop(pair.slave);

        let mut reader = pair.master.try_clone_reader().expect("clone reader 失败");
        let mut writer = pair.master.take_writer().expect("take writer 失败");

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut out = Vec::new();
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => out.extend_from_slice(&buf[..n]),
                    Err(_) => break,
                }
            }
            let _ = tx.send(out);
        });

        writer.write_all(b"echo VIBE_TEST_123\n").unwrap();
        writer.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(300));
        writer.write_all(b"exit\n").unwrap();
        writer.flush().unwrap();

        let out = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("读取 PTY 输出超时");
        let _ = child.wait();

        let s = String::from_utf8_lossy(&out);
        assert!(
            s.contains("VIBE_TEST_123"),
            "PTY 输出应包含 echo 的内容，实际收到: {:?}",
            s
        );
    }
}
