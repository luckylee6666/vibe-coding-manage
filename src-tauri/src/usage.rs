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

/// 经「交互式登录」shell 跑一条命令，继承用户完整 PATH
/// （GUI 启动的进程默认拿不到 nvm/npx/claude）。
/// 必须是交互式（-i）：nvm 等对 PATH 的设置几乎都写在 .zshrc/.bashrc 里，
/// 而这些 rc 只在交互式 shell 加载；只用 -l（登录非交互）只读 .zprofile/.zlogin，
/// 拿不到 nvm 的 node/npx → ccusage 跑不起来。内置终端是真交互 PTY 所以一直正常。
/// 带超时，避免 ccusage / claude 卡死拖住调用线程。
fn run_shell(script: &str, timeout_secs: u64) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let script = script.to_string();
    std::thread::spawn(move || {
        #[cfg(not(target_os = "windows"))]
        let out = {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
            std::process::Command::new(shell)
                .arg("-ilc")
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
    // 优先全局 ccusage；否则 npx 拉 @latest。不用 --prefer-offline：缓存里的旧版可能缺
    // darwin-arm64 原生依赖（报 "native binary is not available"），@latest 会装齐 optional deps。
    let script = "ccusage blocks --json 2>/dev/null || npx -y ccusage@latest blocks --json 2>/dev/null";
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

/// 从可能带 shell 启动噪声（交互式 zsh 的 .zshrc 偶尔往 stdout 打印 "Restored session:" 等）
/// 的输出里截出 JSON 主体——从第一个 `{` 或 `[` 开始。
fn slice_json(s: &str) -> &str {
    match s.find(|c| c == '{' || c == '[') {
        Some(i) => &s[i..],
        None => s,
    }
}

fn parse_blocks(json: &str) -> Result<ClaudeUsage, String> {
    let v: Value = serde_json::from_str(slice_json(json)).map_err(|e| e.to_string())?;
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

// ============================================================================
// 多 CLI 周用量统计（claude / codex / opencode），同样走 ccusage 读本地日志。
// ============================================================================

/// 单个 CLI 的周用量统计。
#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentWeekly {
    pub ok: bool,
    pub error: Option<String>,
    /// "claude" | "codex" | "opencode"
    pub agent: String,
    /// 累计总花费（USD）
    pub total_cost: f64,
    /// 累计总 token
    pub total_tokens: u64,
    /// 近若干周，按时间倒序（最新在前）
    pub weeks: Vec<WeekRow>,
}

/// 一周的用量。
#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct WeekRow {
    /// 周起始日（周一）YYYY-MM-DD
    pub period: String,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub models: Vec<String>,
}

/// 最多展示几周。
const MAX_WEEKS: usize = 8;

// ccusage 不同子命令字段名不统一：weekly 用 totalCost / period / modelsUsed，
// codex daily 用 costUSD / date / models(对象)。下面几个取值器吸收差异。
fn row_cost(o: &Value) -> f64 {
    o.get("totalCost")
        .or_else(|| o.get("costUSD"))
        .and_then(|x| x.as_f64())
        .unwrap_or(0.0)
}
fn row_tokens(o: &Value) -> u64 {
    o.get("totalTokens").and_then(|x| x.as_u64()).unwrap_or(0)
}
fn row_period(o: &Value) -> String {
    o.get("period")
        .or_else(|| o.get("date"))
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}
fn row_models(o: &Value) -> Vec<String> {
    if let Some(arr) = o.get("modelsUsed").and_then(|x| x.as_array()) {
        return arr.iter().filter_map(|x| x.as_str().map(String::from)).collect();
    }
    if let Some(obj) = o.get("models").and_then(|x| x.as_object()) {
        return obj.keys().cloned().collect();
    }
    Vec::new()
}

/// 拉某个 agent 的周用量。claude/opencode 有原生 `weekly`；codex 只有 `daily`，
/// 在此按 ISO 周（周一为起）聚合成周。
pub fn fetch_agent_weekly(agent: &str) -> AgentWeekly {
    // 白名单：agent 会拼进 shell 命令，杜绝注入
    let agent = match agent {
        "claude" | "opencode" | "codex" => agent,
        other => {
            return AgentWeekly {
                ok: false,
                agent: other.to_string(),
                error: Some(format!("不支持的 agent：{other}")),
                ..Default::default()
            }
        }
    };
    let sub = if agent == "codex" { "daily" } else { "weekly" };
    let script = format!(
        "ccusage {agent} {sub} --json 2>/dev/null || npx -y ccusage@latest {agent} {sub} --json 2>/dev/null"
    );
    match run_shell(&script, 90) {
        Ok(json) => match parse_agent_weekly(&json, agent) {
            Ok(mut w) => {
                w.ok = true;
                w.agent = agent.to_string();
                w
            }
            Err(e) => AgentWeekly {
                ok: false,
                agent: agent.to_string(),
                error: Some(format!("解析 ccusage 输出失败：{e}")),
                ..Default::default()
            },
        },
        Err(e) => AgentWeekly {
            ok: false,
            agent: agent.to_string(),
            error: Some(if e.is_empty() {
                format!("无法运行 ccusage（确认装了 Node/npx，且用过 {agent}）")
            } else {
                e
            }),
            ..Default::default()
        },
    }
}

fn parse_agent_weekly(json: &str, agent: &str) -> Result<AgentWeekly, String> {
    let v: Value = serde_json::from_str(slice_json(json)).map_err(|e| e.to_string())?;
    let totals = v.get("totals");
    let total_cost = totals.map(row_cost).unwrap_or(0.0);
    let total_tokens = totals.map(row_tokens).unwrap_or(0);

    let mut weeks: Vec<WeekRow> = if agent == "codex" {
        aggregate_daily_to_weeks(&v)?
    } else {
        let arr = v
            .get("weekly")
            .and_then(|x| x.as_array())
            .ok_or("缺少 weekly 字段")?;
        arr.iter()
            .map(|o| WeekRow {
                period: row_period(o),
                cost_usd: row_cost(o),
                total_tokens: row_tokens(o),
                models: row_models(o),
            })
            .collect()
    };
    // 按周起始日倒序（最新在前），只留最近 MAX_WEEKS 周
    weeks.sort_by(|a, b| b.period.cmp(&a.period));
    weeks.truncate(MAX_WEEKS);
    Ok(AgentWeekly {
        total_cost,
        total_tokens,
        weeks,
        ..Default::default()
    })
}

/// 把 codex 的 daily 数组按周一为起的周聚合。
fn aggregate_daily_to_weeks(v: &Value) -> Result<Vec<WeekRow>, String> {
    use chrono::{Datelike, Duration as Dur, NaiveDate};
    let arr = v
        .get("daily")
        .and_then(|x| x.as_array())
        .ok_or("缺少 daily 字段")?;
    let mut map: std::collections::BTreeMap<String, WeekRow> = std::collections::BTreeMap::new();
    for o in arr {
        let date = row_period(o);
        // 该日所在周的周一；解析失败就退化成按当日分组（不丢数据）
        let monday = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
            .map(|d| d - Dur::days(d.weekday().num_days_from_monday() as i64))
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|_| date.clone());
        let entry = map.entry(monday.clone()).or_insert_with(|| WeekRow {
            period: monday.clone(),
            ..Default::default()
        });
        entry.cost_usd += row_cost(o);
        entry.total_tokens += row_tokens(o);
        for m in row_models(o) {
            if !entry.models.contains(&m) {
                entry.models.push(m);
            }
        }
    }
    Ok(map.into_values().collect())
}

/// 触发一次 `claude -p hello`，开一个新 5h 窗口。返回 claude 的输出（成功时）。
pub fn fire_hello() -> Result<String, String> {
    // 进 HOME 跑，避免落在某个奇怪 cwd；输入接 /dev/null 防止它等输入
    let script = "cd \"$HOME\"; claude -p \"hello\" </dev/null 2>&1";
    run_shell(script, 120).map(|out| {
        // 滤掉交互式 shell 启动噪声行（如 .zshrc 打印的 "Restored session:"）
        out.lines()
            .filter(|l| !l.trim_start().starts_with("Restored session"))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    })
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

    #[test]
    fn parse_native_weekly() {
        // claude/opencode 原生 weekly 形态
        let json = r#"{
            "totals":{"totalCost":99.5,"totalTokens":1000},
            "weekly":[
                {"period":"2026-06-08","totalCost":40.0,"totalTokens":400,"modelsUsed":["claude-opus-4-8"]},
                {"period":"2026-06-15","totalCost":59.5,"totalTokens":600,"modelsUsed":["claude-opus-4-8","claude-haiku-4-5"]}
            ]
        }"#;
        let w = parse_agent_weekly(json, "claude").expect("应解析成功");
        assert_eq!(w.total_cost, 99.5);
        assert_eq!(w.total_tokens, 1000);
        // 倒序：最新的 2026-06-15 在前
        assert_eq!(w.weeks[0].period, "2026-06-15");
        assert_eq!(w.weeks[0].cost_usd, 59.5);
        assert_eq!(w.weeks[0].models.len(), 2);
        assert_eq!(w.weeks[1].period, "2026-06-08");
    }

    #[test]
    fn codex_daily_aggregates_into_weeks() {
        // codex 只有 daily（字段名 costUSD / date / models 对象），按周聚合
        // 2026-06-08 周一、2026-06-10 周三 → 同一周；2026-06-15 → 下一周
        let json = r#"{
            "totals":{"costUSD":12.0,"totalTokens":300},
            "daily":[
                {"date":"2026-06-08","costUSD":2.0,"totalTokens":50,"models":{"gpt-5.4":{}}},
                {"date":"2026-06-10","costUSD":3.0,"totalTokens":70,"models":{"gpt-5.3-codex":{}}},
                {"date":"2026-06-15","costUSD":7.0,"totalTokens":180,"models":{"gpt-5.4":{}}}
            ]
        }"#;
        let w = parse_agent_weekly(json, "codex").expect("应解析成功");
        assert_eq!(w.total_cost, 12.0);
        assert_eq!(w.weeks.len(), 2);
        // 倒序：本周（06-15 起）在前
        assert_eq!(w.weeks[0].period, "2026-06-15");
        assert_eq!(w.weeks[0].total_tokens, 180);
        // 上一周：06-08 + 06-10 合并，周起为周一 06-08
        assert_eq!(w.weeks[1].period, "2026-06-08");
        assert!((w.weeks[1].cost_usd - 5.0).abs() < 1e-9);
        assert_eq!(w.weeks[1].total_tokens, 120);
        assert_eq!(w.weeks[1].models.len(), 2);
    }

    #[test]
    fn unsupported_agent_errors() {
        let w = fetch_agent_weekly("agy");
        assert!(!w.ok);
        assert_eq!(w.agent, "agy");
    }
}
