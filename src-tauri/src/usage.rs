//! Claude 用量查询 + 5 小时窗口「自动 hello」重置计时。
//!
//! 数据源：社区工具 `ccusage`（读 ~/.claude/projects 下的 JSONL 本地日志，不联网传数据），
//! 取 `ccusage blocks --json` 里 `isActive` 的那一块——即当前 5 小时计费窗口，
//! 含起止时间、花费、token、燃烧速率、预测。
//!
//! 自动 hello：后台轮询，发现「无活跃窗口」（即上一个 5h 窗口已重置 / 长时间没用）时，
//! 自动跑一次 `claude -p hello` 触发一次极小请求，立刻开一个新的 5h 窗口——
//! 保证计时从你想要的时刻重新开始，不浪费窗口。

use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

/// 当前 5 小时窗口的用量快照，推给前端展示。
#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeUsage {
    /// ccusage 是否成功跑通（没装 / 没日志时为 false）
    pub ok: bool,
    pub error: Option<String>,
    /// 是否存在活跃的 5h 窗口（false = 已重置 / 空闲）
    pub active: bool,
    /// 窗口起始（ISO，UTC）
    pub start_time: String,
    /// 窗口重置时刻（ISO，UTC）——前端据此倒计时
    pub end_time: String,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub models: Vec<String>,
    /// 按当前速率预测到窗口结束的总花费
    pub projected_cost: Option<f64>,
    /// 燃烧速率（美元/小时）
    pub cost_per_hour: Option<f64>,
    /// 自动 hello 开关当前状态（顺带回传，省一次 IPC）
    pub auto_hello: bool,
}

/// 自动 hello 设置 + 运行态。
pub struct UsageState {
    pub auto_hello: AtomicBool,
    settings_path: PathBuf,
    /// 上次触发 hello 的时刻，去重用（同一空闲期只触发一次）
    last_fired: Mutex<Option<Instant>>,
}

impl UsageState {
    pub fn load() -> Self {
        let path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("vibe-coding-manage")
            .join("usage-settings.json");
        let auto = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .and_then(|v| v.get("autoHello").and_then(|b| b.as_bool()))
            .unwrap_or(false);
        Self {
            auto_hello: AtomicBool::new(auto),
            settings_path: path,
            last_fired: Mutex::new(None),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let v = serde_json::json!({ "autoHello": self.auto_hello.load(Ordering::Relaxed) });
        std::fs::write(
            &self.settings_path,
            serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())
    }
}

/// 经登录 shell 跑一条命令，继承用户 PATH（GUI 启动的进程默认拿不到 nvm/npx/claude）。
/// 带超时，避免 ccusage / claude 卡死拖住调用线程。
fn run_shell(script: &str, timeout_secs: u64) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let script = script.to_string();
    std::thread::spawn(move || {
        #[cfg(not(target_os = "windows"))]
        let out = {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
            std::process::Command::new(shell)
                .arg("-lc")
                .arg(&script)
                .output()
        };
        #[cfg(target_os = "windows")]
        let out = std::process::Command::new("powershell.exe")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(&script)
            .output();
        let _ = tx.send(out);
    });
    match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
        Ok(Ok(o)) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            if !stdout.trim().is_empty() {
                Ok(stdout)
            } else {
                Err(String::from_utf8_lossy(&o.stderr).trim().to_string())
            }
        }
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("命令超时（ccusage/claude 未响应）".to_string()),
    }
}

/// 拉取当前 5 小时窗口用量。优先全局 `ccusage`，否则 `npx -y ccusage@latest`。
pub fn fetch_usage() -> ClaudeUsage {
    // stderr 丢弃（npm 的 warn 不影响 stdout 的 JSON）
    // 优先全局 ccusage；否则走 npx --prefer-offline 用本地缓存（不每次查 registry，省几秒）
    let script = "ccusage blocks --json 2>/dev/null || npx -y --prefer-offline ccusage blocks --json 2>/dev/null";
    match run_shell(script, 60) {
        Ok(json) => match parse_blocks(&json) {
            Ok(mut u) => {
                u.ok = true;
                u
            }
            Err(e) => ClaudeUsage {
                ok: false,
                error: Some(format!("解析 ccusage 输出失败：{e}")),
                ..Default::default()
            },
        },
        Err(e) => ClaudeUsage {
            ok: false,
            error: Some(if e.is_empty() {
                "无法运行 ccusage（确认已装 Node/npx，且用过 Claude Code）".to_string()
            } else {
                e
            }),
            ..Default::default()
        },
    }
}

fn parse_blocks(json: &str) -> Result<ClaudeUsage, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let blocks = v
        .get("blocks")
        .and_then(|b| b.as_array())
        .ok_or("缺少 blocks 字段")?;
    let block = blocks
        .iter()
        .find(|b| b.get("isActive").and_then(|x| x.as_bool()).unwrap_or(false));

    let Some(b) = block else {
        // 没有活跃窗口 = 已重置 / 空闲
        return Ok(ClaudeUsage {
            active: false,
            ..Default::default()
        });
    };

    let counts = b.get("tokenCounts");
    let get_u64 = |obj: Option<&Value>, k: &str| -> u64 {
        obj.and_then(|o| o.get(k)).and_then(|x| x.as_u64()).unwrap_or(0)
    };

    Ok(ClaudeUsage {
        active: true,
        start_time: b.get("startTime").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        end_time: b.get("endTime").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        cost_usd: b.get("costUSD").and_then(|x| x.as_f64()).unwrap_or(0.0),
        total_tokens: b.get("totalTokens").and_then(|x| x.as_u64()).unwrap_or(0),
        input_tokens: get_u64(counts, "inputTokens"),
        output_tokens: get_u64(counts, "outputTokens"),
        cache_read_tokens: get_u64(counts, "cacheReadInputTokens"),
        models: b
            .get("models")
            .and_then(|m| m.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        projected_cost: b
            .get("projection")
            .and_then(|p| p.get("totalCost"))
            .and_then(|x| x.as_f64()),
        cost_per_hour: b
            .get("burnRate")
            .and_then(|p| p.get("costPerHour"))
            .and_then(|x| x.as_f64()),
        ..Default::default()
    })
}

/// 触发一次 `claude -p hello`，开一个新 5h 窗口。返回 claude 的输出（成功时）。
pub fn fire_hello() -> Result<String, String> {
    // 进 HOME 跑，避免落在某个奇怪 cwd；输入接 /dev/null 防止它等输入
    let script = "cd \"$HOME\"; claude -p \"hello\" </dev/null 2>&1";
    run_shell(script, 120)
}

/// 后台轮询：推用量给前端 + 在窗口重置时自动 hello。
pub fn poller(app: AppHandle) {
    loop {
        let state = app.state::<UsageState>();
        let enabled = state.auto_hello.load(Ordering::Relaxed);

        // 没开自动 hello 时不必频繁跑 ccusage（前端开面板时会自己按需拉）
        if !enabled {
            std::thread::sleep(Duration::from_secs(60));
            continue;
        }

        let mut usage = fetch_usage();
        usage.auto_hello = true;
        let _ = app.emit("claude-usage", usage.clone());

        if usage.ok && !usage.active {
            // 窗口已重置 / 空闲：同一空闲期只触发一次（10 分钟去重）
            let should = {
                let mut last = state.last_fired.lock().unwrap();
                let ok = last
                    .map(|t| t.elapsed() > Duration::from_secs(600))
                    .unwrap_or(true);
                if ok {
                    *last = Some(Instant::now());
                }
                ok
            };
            if should {
                let _ = app.emit("claude-hello-firing", ());
                let result = fire_hello();
                let _ = app.emit(
                    "claude-hello-fired",
                    serde_json::json!({
                        "ok": result.is_ok(),
                        "detail": result.unwrap_or_else(|e| e),
                    }),
                );
            }
        } else if usage.active {
            // 又有活跃窗口了，清掉去重标记，等下次重置可再触发
            *state.last_fired.lock().unwrap() = None;
        }

        std::thread::sleep(Duration::from_secs(120));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_active_block() {
        let json = r#"{"blocks":[
            {"isActive":false,"isGap":true,"id":"gap"},
            {"isActive":true,"id":"2026-06-15T12:00:00.000Z",
             "startTime":"2026-06-15T12:00:00.000Z","endTime":"2026-06-15T17:00:00.000Z",
             "costUSD":54.677,"totalTokens":77577653,
             "tokenCounts":{"inputTokens":30192,"outputTokens":162249,"cacheReadInputTokens":76145464},
             "models":["claude-opus-4-8"],
             "burnRate":{"costPerHour":27.93},
             "projection":{"remainingMinutes":144,"totalCost":121.71}}
        ]}"#;
        let u = parse_blocks(json).expect("应解析成功");
        assert!(u.active);
        assert_eq!(u.end_time, "2026-06-15T17:00:00.000Z");
        assert_eq!(u.total_tokens, 77577653);
        assert_eq!(u.output_tokens, 162249);
        assert_eq!(u.cache_read_tokens, 76145464);
        assert_eq!(u.models, vec!["claude-opus-4-8".to_string()]);
        assert_eq!(u.projected_cost, Some(121.71));
        assert_eq!(u.cost_per_hour, Some(27.93));
    }

    #[test]
    fn parse_no_active_block() {
        let json = r#"{"blocks":[{"isActive":false,"id":"old"}]}"#;
        let u = parse_blocks(json).expect("应解析成功");
        assert!(!u.active);
        assert_eq!(u.total_tokens, 0);
    }
}
