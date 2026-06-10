// 内嵌远程服务：手机端（局域网）通过浏览器访问，镜像并控制桌面已开的终端会话。
//
// 数据流：PTY ←→ RemoteHub（会话表 + 滚动缓存 + 广播通道）←→ WebSocket ←→ 手机 xterm.js
// 桌面窗口仍走 Tauri 事件，手机走这里的 WS，两边订阅同一批会话，互不影响。
//
// 安全：服务绑定 0.0.0.0 但要求 6 位 PIN（启动时随机生成，桌面 UI 展示）。
// 这一层暴露的是「在本机跑 shell」的能力，PIN 是最低门槛，远程场景（Tailscale）务必保留。

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use base64::Engine;
use portable_pty::{Child, MasterPty};
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// 单个会话的滚动缓存上限（字节）。手机连上时回放最近这么多输出，避免黑屏。
const SCROLLBACK_CAP: usize = 256 * 1024;

/// 一个活跃的伪终端会话：保留 master（resize）、writer（写入键入）、child（kill）。
pub struct PtySession {
    pub master: Box<dyn MasterPty + Send>,
    pub writer: Box<dyn Write + Send>,
    pub child: Box<dyn Child + Send + Sync>,
}

/// 会话元信息，供手机端列表展示（桌面那边叫「人物」/标签名）。
#[derive(Clone, Serialize)]
pub struct SessionMeta {
    pub id: String,
    pub name: String,
    pub tool: String,
}

/// 一段终端输出（base64，避免切断转义序列 / 多字节字符）。
#[derive(Clone)]
pub struct OutputMsg {
    pub id: String,
    pub data: String,
}

/// 所有终端状态的单一持有者，被 Tauri 托管的 TerminalState 与 axum 服务共享（克隆即共享 Arc）。
#[derive(Clone)]
pub struct RemoteHub {
    pub sessions: Arc<Mutex<HashMap<String, PtySession>>>,
    pub metas: Arc<Mutex<HashMap<String, SessionMeta>>>,
    pub scrollback: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    pub output_tx: broadcast::Sender<OutputMsg>,
    pub exit_tx: broadcast::Sender<String>,
    pub token: Arc<Mutex<String>>,
    pub port: u16,
}

impl RemoteHub {
    pub fn new(port: u16) -> Self {
        let (output_tx, _) = broadcast::channel(2048);
        let (exit_tx, _) = broadcast::channel(64);
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            metas: Arc::new(Mutex::new(HashMap::new())),
            scrollback: Arc::new(Mutex::new(HashMap::new())),
            output_tx,
            exit_tx,
            token: Arc::new(Mutex::new(String::new())),
            port,
        }
    }

    /// 由 reader 线程调用：把一段输出同时广播给 WS 客户端并追加进滚动缓存。
    pub fn publish(&self, id: &str, raw: &[u8]) {
        let data = base64::engine::general_purpose::STANDARD.encode(raw);
        // 没有订阅者时 send 返回 Err，忽略即可。
        let _ = self.output_tx.send(OutputMsg {
            id: id.to_string(),
            data,
        });
        if let Ok(mut sb) = self.scrollback.lock() {
            let buf = sb.entry(id.to_string()).or_default();
            buf.extend_from_slice(raw);
            if buf.len() > SCROLLBACK_CAP {
                let drop = buf.len() - SCROLLBACK_CAP;
                buf.drain(0..drop);
            }
        }
    }

    /// 由 reader 线程在 EOF 时调用：广播退出并清理该会话的缓存/元信息。
    pub fn mark_exit(&self, id: &str) {
        let _ = self.exit_tx.send(id.to_string());
        if let Ok(mut sb) = self.scrollback.lock() {
            sb.remove(id);
        }
        if let Ok(mut m) = self.metas.lock() {
            m.remove(id);
        }
    }
}

/// 在独立线程里起一个 tokio 运行时跑 axum 服务（不依赖 Tauri 的异步运行时）。
pub fn spawn_server(hub: RemoteHub) {
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("[remote] 运行时启动失败: {e}");
                return;
            }
        };
        rt.block_on(async move {
            let port = hub.port;
            let app = Router::new()
                .route("/", get(serve_index))
                .route("/vendor/xterm.css", get(serve_xterm_css))
                .route("/vendor/xterm.js", get(serve_xterm_js))
                .route("/vendor/addon-fit.js", get(serve_fit_js))
                .route("/api/sessions", get(list_sessions))
                .route("/ws", get(ws_handler))
                .with_state(hub);
            match tokio::net::TcpListener::bind(("0.0.0.0", port)).await {
                Ok(listener) => {
                    println!("[remote] 手机端服务监听 0.0.0.0:{port}");
                    if let Err(e) = axum::serve(listener, app).await {
                        eprintln!("[remote] 服务退出: {e}");
                    }
                }
                Err(e) => eprintln!("[remote] 端口 {port} 绑定失败: {e}"),
            }
        });
    });
}

// ===== 静态资源（编译期嵌入二进制，离线可用，不走 CDN）=====

fn asset(content_type: &'static str, body: &'static str) -> Response {
    ([(header::CONTENT_TYPE, content_type)], body).into_response()
}

async fn serve_index() -> Response {
    // 页面禁缓存：开发期频繁改动，手机浏览器缓存旧版会导致「改了没生效」。
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store, must-revalidate"),
        ],
        include_str!("../mobile/index.html"),
    )
        .into_response()
}
async fn serve_xterm_css() -> Response {
    asset("text/css; charset=utf-8", include_str!("../../src/vendor/xterm.css"))
}
async fn serve_xterm_js() -> Response {
    asset(
        "application/javascript; charset=utf-8",
        include_str!("../../src/vendor/xterm.js"),
    )
}
async fn serve_fit_js() -> Response {
    asset(
        "application/javascript; charset=utf-8",
        include_str!("../../src/vendor/addon-fit.js"),
    )
}

// ===== 鉴权 + API =====

fn token_ok(hub: &RemoteHub, q: &HashMap<String, String>) -> bool {
    let want = hub.token.lock().map(|t| t.clone()).unwrap_or_default();
    !want.is_empty() && q.get("token").map(|t| t.as_str()) == Some(want.as_str())
}

/// 列出当前活跃会话，供手机端选「人物」。需 PIN。
async fn list_sessions(
    State(hub): State<RemoteHub>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    if !token_ok(&hub, &q) {
        return (StatusCode::UNAUTHORIZED, "PIN 错误").into_response();
    }
    let metas: Vec<SessionMeta> = hub
        .metas
        .lock()
        .map(|m| m.values().cloned().collect())
        .unwrap_or_default();
    axum::Json(metas).into_response()
}

// ===== WebSocket：双向桥接一个会话 =====

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(hub): State<RemoteHub>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    if !token_ok(&hub, &q) {
        return (StatusCode::UNAUTHORIZED, "PIN 错误").into_response();
    }
    let id = match q.get("id") {
        Some(id) if !id.is_empty() => id.clone(),
        _ => return (StatusCode::BAD_REQUEST, "缺少会话 id").into_response(),
    };
    ws.on_upgrade(move |socket| handle_socket(socket, hub, id))
}

async fn handle_socket(mut socket: WebSocket, hub: RemoteHub, id: String) {
    // 先订阅，再快照滚动缓存——宁可首屏重复一小段，也不丢中间产生的输出。
    let mut out_rx = hub.output_tx.subscribe();
    let mut exit_rx = hub.exit_tx.subscribe();

    // 告知手机端 PTY 的真实尺寸：手机按此 cols/rows 镜像渲染（自动缩字号铺满宽度），
    // 不反过来改 PTY，桌面端尺寸不受影响。
    let size = hub
        .sessions
        .lock()
        .ok()
        .and_then(|s| s.get(&id).and_then(|p| p.master.get_size().ok()));
    if let Some(sz) = size {
        let frame = format!("{{\"t\":\"size\",\"cols\":{},\"rows\":{}}}", sz.cols, sz.rows);
        if socket.send(Message::Text(frame)).await.is_err() {
            return;
        }
    }

    let snapshot = hub
        .scrollback
        .lock()
        .ok()
        .and_then(|sb| sb.get(&id).cloned());
    if let Some(buf) = snapshot {
        if !buf.is_empty() {
            let d = base64::engine::general_purpose::STANDARD.encode(&buf);
            if socket.send(Message::Text(out_frame(&d))).await.is_err() {
                return;
            }
        }
    }

    loop {
        tokio::select! {
            out = out_rx.recv() => match out {
                Ok(msg) if msg.id == id => {
                    if socket.send(Message::Text(out_frame(&msg.data))).await.is_err() {
                        break;
                    }
                }
                Ok(_) => {}
                // Lagged：客户端跟不上，丢了一些消息——继续即可（xterm 会在后续重绘自愈）。
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => break,
            },
            ex = exit_rx.recv() => match ex {
                Ok(eid) if eid == id => {
                    let _ = socket.send(Message::Text("{\"t\":\"exit\"}".into())).await;
                    break;
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => break,
            },
            inbound = socket.recv() => match inbound {
                Some(Ok(Message::Text(txt))) => handle_client_msg(&hub, &id, &txt),
                Some(Ok(Message::Close(_))) | None => break,
                Some(Err(_)) => break,
                _ => {}
            },
        }
    }
}

fn out_frame(b64: &str) -> String {
    // {"t":"o","d":"<base64>"} —— 手裸拼 JSON，data 是 base64（无需转义）。
    format!("{{\"t\":\"o\",\"d\":\"{b64}\"}}")
}

/// 处理手机发来的消息：i=键入，r=resize。
fn handle_client_msg(hub: &RemoteHub, id: &str, txt: &str) {
    let v: serde_json::Value = match serde_json::from_str(txt) {
        Ok(v) => v,
        Err(_) => return,
    };
    match v.get("t").and_then(|t| t.as_str()) {
        Some("i") => {
            if let Some(data) = v.get("d").and_then(|d| d.as_str()) {
                if let Ok(mut sessions) = hub.sessions.lock() {
                    if let Some(s) = sessions.get_mut(id) {
                        let _ = s.writer.write_all(data.as_bytes());
                        let _ = s.writer.flush();
                    }
                }
            }
        }
        Some("r") => {
            // 故意忽略：PTY 尺寸只能有一个，由桌面端权威设定。手机是纯镜像，
            // 绝不反过来 resize PTY——否则会把用户正在看的桌面终端画花。
            let _ = (hub, id);
        }
        _ => {}
    }
}
