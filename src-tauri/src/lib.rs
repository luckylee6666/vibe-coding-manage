use base64::Engine;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

mod remote;
use remote::{PtySession, RemoteHub, SessionMeta};
mod usage;

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

/// Prompt/Snippet 库：可复用的指令片段，一键注入当前终端。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub id: String,
    pub title: String,
    pub content: String,
    #[serde(default, alias = "created_at")]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppState {
    projects: Vec<Project>,
    servers: Vec<Server>,
    #[serde(default)]
    snippets: Vec<Snippet>,
    data_path: PathBuf,
    server_path: PathBuf,
    snippet_path: PathBuf,
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

        let snippet_path = data_dir.join("snippets.json");
        let snippets = if snippet_path.exists() {
            let data = fs::read_to_string(&snippet_path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };

        Self { projects, servers, snippets, data_path, server_path, snippet_path }
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

    fn save_snippets(&self) -> Result<(), String> {
        let data = serde_json::to_string_pretty(&self.snippets)
            .map_err(|e| e.to_string())?;
        fs::write(&self.snippet_path, data).map_err(|e| e.to_string())
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

#[tauri::command]
fn get_snippets(state: State<Mutex<AppState>>) -> Result<Vec<Snippet>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.snippets.clone())
}

/// 整表保存（前端管理增删改后回写）。给缺 id / created_at 的项补齐。
#[tauri::command]
fn save_snippets(
    state: State<Mutex<AppState>>,
    snippets: Vec<Snippet>,
) -> Result<Vec<Snippet>, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let snippets: Vec<Snippet> = snippets
        .into_iter()
        .map(|mut s| {
            if s.id.is_empty() {
                s.id = Uuid::new_v4().to_string();
            }
            if s.created_at.is_empty() {
                s.created_at = now.clone();
            }
            s
        })
        .collect();
    state.snippets = snippets.clone();
    state.save_snippets()?;
    Ok(snippets)
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

// ========== 项目 Git 状态徽标 ==========

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GitStatus {
    path: String,
    /// 是否是 git 仓库
    is_repo: bool,
    /// 当前分支名（detached 时为 "(detached)"）
    branch: String,
    /// 相对上游领先 / 落后的提交数
    ahead: u32,
    behind: u32,
    /// 已追踪文件的改动数（暂存 + 未暂存 + 冲突）
    changed: u32,
    /// 未追踪文件数
    untracked: u32,
    /// 工作区是否有改动
    dirty: bool,
    /// 执行 git 出错（如 git 不在 PATH）
    error: bool,
}

impl GitStatus {
    fn empty(path: String) -> Self {
        GitStatus {
            path,
            is_repo: false,
            branch: String::new(),
            ahead: 0,
            behind: 0,
            changed: 0,
            untracked: 0,
            dirty: false,
            error: false,
        }
    }
}

/// 扫描单个仓库的 git 状态。用 `status --porcelain=v2 --branch` 一条命令拿全：
/// 分支 / 上游领先落后 / 各文件状态。非仓库直接返回 is_repo=false。
fn git_status_one(path: &str) -> GitStatus {
    let mut st = GitStatus::empty(path.to_string());
    let p = std::path::Path::new(path);
    if !p.is_dir() || !p.join(".git").exists() {
        return st;
    }
    st.is_repo = true;
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["status", "--porcelain=v2", "--branch"])
        .output();
    let out = match out {
        Ok(o) if o.status.success() => o,
        _ => {
            st.error = true;
            return st;
        }
    };
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            st.branch = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
            for tok in rest.split_whitespace() {
                if let Some(n) = tok.strip_prefix('+') {
                    st.ahead = n.parse().unwrap_or(0);
                } else if let Some(n) = tok.strip_prefix('-') {
                    st.behind = n.parse().unwrap_or(0);
                }
            }
        } else if line.starts_with("1 ") || line.starts_with("2 ") || line.starts_with("u ") {
            st.changed += 1;
        } else if line.starts_with("? ") {
            st.untracked += 1;
        }
    }
    st.dirty = st.changed > 0 || st.untracked > 0;
    st
}

/// 批量扫描多个项目路径的 git 状态（并行，每仓库一线程）。
/// git status 单仓库很快，整体在 spawn_blocking 里跑，绝不阻塞主线程。
#[tauri::command]
async fn git_status_batch(paths: Vec<String>) -> Result<Vec<GitStatus>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let handles: Vec<_> = paths
            .into_iter()
            .map(|path| std::thread::spawn(move || git_status_one(&path)))
            .collect();
        handles
            .into_iter()
            .map(|h| {
                h.join().unwrap_or_else(|_| {
                    let mut s = GitStatus::empty(String::new());
                    s.error = true;
                    s
                })
            })
            .collect()
    })
    .await
    .map_err(|e| e.to_string())
}

// ========== 项目"恢复现场" ==========

#[derive(Clone, Serialize)]
struct Commit {
    hash: String,
    subject: String,
    /// 相对时间，如 "2 hours ago"（git 自带）
    rel: String,
}

#[derive(Clone, Serialize)]
struct ChangedFile {
    /// porcelain 两字符状态码去空格后的值（M / A / D / R / ?? 等）
    status: String,
    path: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectContext {
    is_repo: bool,
    branch: String,
    ahead: u32,
    behind: u32,
    changed: u32,
    untracked: u32,
    dirty: bool,
    /// 最近提交（最多 5 条）
    commits: Vec<Commit>,
    /// 改动文件（最多 20 条）
    files: Vec<ChangedFile>,
    /// 还有多少改动文件未列出（files 截断后的剩余数）
    files_more: u32,
    /// CLAUDE.md 摘要（前若干字符；无则空）
    claude_md: String,
    /// 项目目录是否存在
    exists: bool,
}

fn read_claude_md(dir: &std::path::Path) -> String {
    let candidates = [dir.join("CLAUDE.md"), dir.join(".claude").join("CLAUDE.md")];
    for c in candidates {
        if let Ok(text) = fs::read_to_string(&c) {
            // 取前若干非空行，拼成摘要，最长 ~500 字符
            let mut out = String::new();
            for line in text.lines() {
                let l = line.trim_end();
                if out.is_empty() && l.trim().is_empty() {
                    continue; // 跳过开头空行
                }
                out.push_str(l);
                out.push('\n');
                if out.chars().count() >= 500 {
                    break;
                }
            }
            return out.trim_end().to_string();
        }
    }
    String::new()
}

/// 聚合一个项目的"现场"：git 概览 + 最近提交 + 改动文件 + CLAUDE.md 摘要。
#[tauri::command]
async fn project_context(path: String) -> Result<ProjectContext, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = std::path::Path::new(&path);
        let mut ctx = ProjectContext {
            is_repo: false,
            branch: String::new(),
            ahead: 0,
            behind: 0,
            changed: 0,
            untracked: 0,
            dirty: false,
            commits: Vec::new(),
            files: Vec::new(),
            files_more: 0,
            claude_md: String::new(),
            exists: dir.is_dir(),
        };
        if !ctx.exists {
            return ctx;
        }
        ctx.claude_md = read_claude_md(dir);

        // git 概览复用 git_status_one
        let st = git_status_one(&path);
        ctx.is_repo = st.is_repo;
        ctx.branch = st.branch;
        ctx.ahead = st.ahead;
        ctx.behind = st.behind;
        ctx.changed = st.changed;
        ctx.untracked = st.untracked;
        ctx.dirty = st.dirty;

        if ctx.is_repo {
            // 最近提交
            if let Ok(out) = std::process::Command::new("git")
                .arg("-C")
                .arg(&path)
                .args(["log", "-n", "5", "--pretty=format:%h%x1f%s%x1f%cr"])
                .output()
            {
                if out.status.success() {
                    for line in String::from_utf8_lossy(&out.stdout).lines() {
                        let mut parts = line.split('\u{1f}');
                        if let (Some(h), Some(s), Some(r)) =
                            (parts.next(), parts.next(), parts.next())
                        {
                            ctx.commits.push(Commit {
                                hash: h.to_string(),
                                subject: s.to_string(),
                                rel: r.to_string(),
                            });
                        }
                    }
                }
            }
            // 改动文件
            if let Ok(out) = std::process::Command::new("git")
                .arg("-C")
                .arg(&path)
                .args(["status", "--porcelain"])
                .output()
            {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    let lines: Vec<&str> = text.lines().filter(|l| l.len() > 3).collect();
                    let total = lines.len();
                    for l in lines.iter().take(20) {
                        ctx.files.push(ChangedFile {
                            status: l[..2].trim().to_string(),
                            path: l[3..].trim().to_string(),
                        });
                    }
                    if total > 20 {
                        ctx.files_more = (total - 20) as u32;
                    }
                }
            }
        }
        ctx
    })
    .await
    .map_err(|e| e.to_string())
}

// ========== 终端会话上下文用量（Claude 会话当前上下文占比）==========

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ContextUsage {
    /// 是否找到该项目的 Claude 会话 transcript
    ok: bool,
    /// 当前上下文占比 0-100
    percent: u32,
    /// 当前上下文 token 数（input + cache_read + cache_creation）
    tokens: u64,
    /// 模型上下文上限
    limit: u64,
}

/// 估算模型上下文窗口上限。
/// 难点：transcript 里 model id 是裸的 `claude-opus-4-8`（不带 `[1m]` 后缀，1M beta 是
/// Claude Code 通过请求头开的、不落盘），磁盘上无从确知窗口大小。
/// 折中：Opus 4.x 是 1M 上下文型号、本工具用户基本是 Max → 默认 1M；其余 200k；
/// 另外只要实测用量已超基准就抬到 1M 兜底。和 Claude Code /usage(claude-hud) 的口径对齐。
fn context_limit_for(model: &str, tokens: u64) -> u64 {
    let base = if model.contains("[1m]") || model.contains("opus-4") {
        1_000_000
    } else {
        200_000
    };
    if tokens > base {
        1_000_000
    } else {
        base
    }
}

/// Claude Code 把项目路径编码成 ~/.claude/projects 下的目录名：`/` 和 `.` 都替换为 `-`。
fn encode_claude_project_dir(cwd: &str) -> String {
    cwd.chars()
        .map(|c| if c == '/' || c == '.' { '-' } else { c })
        .collect()
}

/// 找某项目对应的 Claude transcript 目录：先按编码规则猜，猜不中再扫 projects 下
/// 各目录、读首行的 `cwd` 字段匹配。
fn find_claude_project_dir(cwd: &str) -> Option<PathBuf> {
    let projects = dirs::home_dir()?.join(".claude").join("projects");
    let cand = projects.join(encode_claude_project_dir(cwd));
    if cand.is_dir() {
        return Some(cand);
    }
    for entry in fs::read_dir(&projects).ok()?.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        // 读该目录任一 jsonl 的首行，比对 cwd 字段
        if let Some(j) = newest_jsonl(&p) {
            if let Ok(content) = fs::read_to_string(&j) {
                if let Some(line) = content.lines().next() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                        if v.get("cwd").and_then(|x| x.as_str()) == Some(cwd) {
                            return Some(p);
                        }
                    }
                }
            }
        }
    }
    None
}

fn newest_jsonl(dir: &std::path::Path) -> Option<PathBuf> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let mtime = entry.metadata().and_then(|m| m.modified()).ok()?;
        if best.as_ref().map(|(t, _)| mtime > *t).unwrap_or(true) {
            best = Some((mtime, p));
        }
    }
    best.map(|(_, p)| p)
}

/// 估算某项目最近 Claude 会话的当前上下文占比。读最新 transcript 的最后一条带 usage
/// 的 assistant 消息：context ≈ input + cache_read + cache_creation tokens。
#[tauri::command]
async fn context_usage(cwd: String) -> Result<ContextUsage, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut cu = ContextUsage { ok: false, percent: 0, tokens: 0, limit: 200_000 };
        let Some(dir) = find_claude_project_dir(&cwd) else { return cu };
        let Some(jsonl) = newest_jsonl(&dir) else { return cu };
        let Ok(content) = fs::read_to_string(&jsonl) else { return cu };
        // 从后往前找最后一条带 usage 的 assistant 消息
        for line in content.lines().rev() {
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
            let Some(usage) = v.pointer("/message/usage") else { continue };
            let g = |k: &str| usage.get(k).and_then(|x| x.as_u64()).unwrap_or(0);
            let tokens = g("input_tokens")
                + g("cache_read_input_tokens")
                + g("cache_creation_input_tokens");
            if tokens == 0 {
                continue;
            }
            let model = v
                .pointer("/message/model")
                .and_then(|x| x.as_str())
                .unwrap_or("");
            let limit = context_limit_for(model, tokens);
            cu.ok = true;
            cu.tokens = tokens;
            cu.limit = limit;
            cu.percent = ((tokens as f64 / limit as f64) * 100.0).round().min(100.0) as u32;
            break;
        }
        cu
    })
    .await
    .map_err(|e| e.to_string())
}

// ========== 内置终端（PTY）==========

// ===== 会话状态感知（"AI 跑完/在等你"检测）=====
// 思路：reader 线程记录每会话的输出活动；监控线程每秒扫描——
// 一个会话"活跃输出了一阵后突然安静超过阈值"即判定为需要用户关注，emit `terminal-attention`。
// 启发式过滤掉空 shell 打印提示符这类"瞬时单次输出"（不是干活），只在持续输出后变静默才报。

/// 静默多久判定为"等待关注"
const ATTENTION_IDLE_SECS: u64 = 5;
/// 活跃输出至少持续这么久（毫秒）才算"干过活"——过滤瞬时提示符
const ATTENTION_MIN_BURST_MS: u128 = 1500;
/// 或：单段输出累计这么多字节也算干过活（捕捉一次性大输出，如构建日志/长回答）
const ATTENTION_MIN_BYTES: usize = 2000;

/// 单个会话的输出活动追踪。
struct Activity {
    /// 最近一次产生输出的时刻
    last_output: Instant,
    /// 当前这段活跃输出的起点（busy 由 false→true 时重置）
    burst_start: Instant,
    /// 当前活跃段累计字节
    burst_bytes: usize,
    /// 自上次通知后是否有新输出（true = 有待消费的活跃）
    busy: bool,
    /// 本段活跃是否已通知过（避免重复报）
    notified: bool,
    name: String,
    tool: String,
}

/// 推给前端的"需要关注"事件。
#[derive(Clone, Serialize)]
struct AttentionEvent {
    id: String,
    name: String,
    tool: String,
}

type ActivityMap = Arc<Mutex<HashMap<String, Activity>>>;

/// 终端状态：持有共享的 RemoteHub（会话表 / 滚动缓存 / 广播通道 / PIN）。
/// 桌面命令与内嵌的手机端服务都操作同一个 hub（克隆即共享 Arc）。
struct TerminalState {
    hub: RemoteHub,
    /// 各会话输出活动追踪（reader 线程写、监控线程读）
    activity: ActivityMap,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            hub: RemoteHub::new(REMOTE_PORT),
            activity: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// 监控线程：每秒扫描所有会话，把"活跃后静默超阈值"的会话报给前端。
fn monitor_attention(app: AppHandle, activity: ActivityMap) {
    let idle = Duration::from_secs(ATTENTION_IDLE_SECS);
    loop {
        std::thread::sleep(Duration::from_millis(1000));
        let mut fire: Vec<AttentionEvent> = Vec::new();
        if let Ok(mut map) = activity.lock() {
            let now = Instant::now();
            for (id, a) in map.iter_mut() {
                if !a.busy || a.notified {
                    continue;
                }
                if now.duration_since(a.last_output) < idle {
                    continue;
                }
                let burst_ms = a.last_output.duration_since(a.burst_start).as_millis();
                let qualifies = burst_ms >= ATTENTION_MIN_BURST_MS || a.burst_bytes >= ATTENTION_MIN_BYTES;
                // 不管够不够格，这段活跃都已结束 → 消费掉，等下一段新输出再重新计
                a.busy = false;
                if qualifies {
                    a.notified = true;
                    fire.push(AttentionEvent {
                        id: id.clone(),
                        name: a.name.clone(),
                        tool: a.tool.clone(),
                    });
                }
            }
        }
        for ev in fire {
            let _ = app.emit("terminal-attention", ev);
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
    let sess_name = name.unwrap_or_else(|| id.clone());
    let sess_tool = tool.unwrap_or_default();

    // 登记活动追踪条目（监控线程据此判定"等待关注"）
    {
        let now = Instant::now();
        state
            .activity
            .lock()
            .map_err(|e| e.to_string())?
            .insert(
                id.clone(),
                Activity {
                    last_output: now,
                    burst_start: now,
                    burst_bytes: 0,
                    busy: false,
                    notified: false,
                    name: sess_name.clone(),
                    tool: sess_tool.clone(),
                },
            );
    }

    let app_evt = app.clone();
    let sid = id.clone();
    let hub_evt = state.hub.clone();
    let act_evt = state.activity.clone();
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
                    // 记录输出活动：新一段活跃则重置起点；持续输出则累加
                    if let Ok(mut map) = act_evt.lock() {
                        if let Some(a) = map.get_mut(&sid) {
                            let now = Instant::now();
                            if !a.busy {
                                a.burst_start = now;
                                a.burst_bytes = 0;
                            }
                            a.busy = true;
                            a.notified = false;
                            a.last_output = now;
                            a.burst_bytes += n;
                        }
                    }
                }
                Err(_) => break,
            }
        }
        let _ = app_evt.emit("terminal-exit", &sid);
        hub_evt.mark_exit(&sid);
        if let Ok(mut map) = act_evt.lock() {
            map.remove(&sid);
        }
    });

    // 注册会话元信息（供手机端列表展示）
    state.hub.metas.lock().map_err(|e| e.to_string())?.insert(
        id.clone(),
        SessionMeta {
            id: id.clone(),
            name: sess_name,
            tool: sess_tool,
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
    if let Ok(mut map) = state.activity.lock() {
        map.remove(&id);
    }
    Ok(())
}

/// 发系统级桌面通知（"会话状态感知"用：AI 跑完/在等你时叫回用户）。
/// 由前端在判定窗口失焦/不在当前标签时调用，避免你正盯着看还弹通知。
#[tauri::command]
fn notify(app: AppHandle, title: String, body: String) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())
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

/// 查询当前 5 小时窗口的 Claude 用量（走 ccusage）。
/// async + spawn_blocking：ccusage 要跑几秒，绝不能阻塞主线程（否则 UI 冻住）。
#[tauri::command]
async fn claude_usage(state: State<'_, usage::UsageState>) -> Result<usage::ClaudeUsage, String> {
    let auto = state
        .auto_hello
        .load(std::sync::atomic::Ordering::Relaxed);
    let mut u = tauri::async_runtime::spawn_blocking(usage::fetch_usage_cached)
        .await
        .map_err(|e| e.to_string())?;
    u.auto_hello = auto;
    Ok(u)
}

/// 读「5 小时重置后自动 hello」开关。
#[tauri::command]
fn get_auto_hello(state: State<usage::UsageState>) -> bool {
    state.auto_hello.load(std::sync::atomic::Ordering::Relaxed)
}

/// 设「5 小时重置后自动 hello」开关并持久化。
#[tauri::command]
fn set_auto_hello(state: State<usage::UsageState>, enabled: bool) -> Result<(), String> {
    state
        .auto_hello
        .store(enabled, std::sync::atomic::Ordering::Relaxed);
    state.save()
}

/// 查询某个 CLI 的周用量（claude / codex / opencode），走 ccusage。
#[tauri::command]
async fn agent_weekly(agent: String) -> Result<usage::AgentWeekly, String> {
    tauri::async_runtime::spawn_blocking(move || usage::fetch_agent_weekly_cached(&agent))
        .await
        .map_err(|e| e.to_string())
}

/// OAuth 限流用量（Claude 专属，同 /usage 数据源）：5h/7d 使用百分比 + 重置时间，带 60s 缓存。
#[tauri::command]
async fn oauth_usage() -> Result<usage::OAuthUsage, String> {
    tauri::async_runtime::spawn_blocking(usage::fetch_oauth_usage)
        .await
        .map_err(|e| e.to_string())
}

/// 刷新菜单栏托盘标题为「5h X% · 周 Y%」（OAuth 限流用量，走 60s 缓存）。
fn update_tray_usage(app: &AppHandle) {
    let u = usage::fetch_oauth_usage();
    let title = if u.ok {
        format!(
            "5h {}% · 周 {}%",
            u.five_hour.utilization.round() as i64,
            u.seven_day.utilization.round() as i64
        )
    } else {
        "用量 —".to_string()
    };
    if let Some(tray) = app.tray_by_id("usage-tray") {
        let _ = tray.set_title(Some(&title));
    }
}

/// 手动立刻发一次 hello（开新窗口 / 测试）。
#[tauri::command]
async fn claude_hello_now() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(usage::fire_hello)
        .await
        .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Mutex::new(AppState::new());
    let term_state = TerminalState::default();
    let activity_for_monitor = term_state.activity.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(state)
        .manage(term_state)
        .manage(usage::UsageState::load())
        .setup(move |app| {
            // 后台轮询：自动 hello 开启时，5h 窗口重置后自动触发开新窗口
            let usage_app = app.handle().clone();
            std::thread::spawn(move || usage::poller(usage_app));
            // 会话状态感知：监控线程扫描"活跃后静默"的终端，emit terminal-attention
            let mon_app = app.handle().clone();
            std::thread::spawn(move || monitor_attention(mon_app, activity_for_monitor));
            // 版本号显示在原生标题栏（来自 Cargo.toml，单一来源）
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_title(&format!(
                    "Vibe Coding Manager v{}",
                    env!("CARGO_PKG_VERSION")
                ));
            }
            // 菜单栏托盘：常驻显示 5h / 周限流用量，菜单可打开主窗/刷新/退出
            let show_i = MenuItem::with_id(app, "tray_show", "打开 Vibe Coding Manager", true, None::<&str>)?;
            let refresh_i = MenuItem::with_id(app, "tray_refresh", "刷新用量", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "tray_quit", "退出", true, None::<&str>)?;
            let tray_menu = Menu::with_items(
                app,
                &[
                    &show_i as &dyn tauri::menu::IsMenuItem<_>,
                    &refresh_i,
                    &quit_i,
                ],
            )?;
            let mut tray_builder = TrayIconBuilder::with_id("usage-tray")
                .menu(&tray_menu)
                .show_menu_on_left_click(true)
                .title("用量…")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "tray_show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                    "tray_refresh" => update_tray_usage(app),
                    "tray_quit" => app.exit(0),
                    _ => {}
                });
            if let Some(icon) = app.default_window_icon().cloned() {
                tray_builder = tray_builder.icon(icon);
            }
            tray_builder.build(app)?;
            // 后台每 60s 刷新托盘标题（首次会触发钥匙串授权）
            let tray_app = app.handle().clone();
            std::thread::spawn(move || loop {
                update_tray_usage(&tray_app);
                std::thread::sleep(std::time::Duration::from_secs(60));
            });
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
            terminal_remote_info,
            notify,
            git_status_batch,
            project_context,
            context_usage,
            get_snippets,
            save_snippets,
            claude_usage,
            get_auto_hello,
            set_auto_hello,
            agent_weekly,
            oauth_usage,
            claude_hello_now
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
