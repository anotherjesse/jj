use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path as AxumPath, State,
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{any, get},
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use super::protocol::{self, InboundFrame};
use super::session::SessionManager;

#[derive(Clone)]
pub struct AppState {
    token: Arc<String>,
    pub sessions: Arc<SessionManager>,
}

impl AppState {
    pub fn new(token: String, sessions: SessionManager) -> Self {
        Self {
            token: Arc::new(token),
            sessions: Arc::new(sessions),
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/ws", any(ws_handler))
        .route("/health", get(health))
        .route("/media/{*path}", get(serve_media))
        .route("/", get(index_html))
        .with_state(state)
}

async fn index_html(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    match params.get("token") {
        Some(t) if t == state.token.as_str() => {
            axum::response::Html(include_str!("../../web/index.html")).into_response()
        }
        _ => {
            (StatusCode::UNAUTHORIZED, "Unauthorized. Add ?token=<bearer> to URL.").into_response()
        }
    }
}

async fn serve_media(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
) -> Response {
    // Reject path traversal
    if path.contains("..") || path.starts_with('/') || path.contains('\\') {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let vault_path = state.sessions.vault_path();
    let file_path = vault_path.join("media").join(&path);

    // Verify resolved path is within media/
    let media_dir = vault_path.join("media");
    if let (Ok(canonical_file), Ok(canonical_media)) =
        (file_path.canonicalize(), media_dir.canonicalize())
    {
        if !canonical_file.starts_with(&canonical_media) {
            return StatusCode::FORBIDDEN.into_response();
        }
    }

    match tokio::fs::read(&file_path).await {
        Ok(bytes) => {
            let content_type = if path.ends_with(".png") {
                "image/png"
            } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
                "image/jpeg"
            } else if path.ends_with(".gif") {
                "image/gif"
            } else if path.ends_with(".webp") {
                "image/webp"
            } else {
                "application/octet-stream"
            };
            ([(axum::http::header::CONTENT_TYPE, content_type)], bytes).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn health() -> &'static str {
    "ok"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    // Validate Origin header for browser connections
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        let port = super::resolve_port();
        let allowed_ip = format!("http://127.0.0.1:{port}");
        let allowed_localhost = format!("http://localhost:{port}");
        if origin != allowed_ip && origin != allowed_localhost && origin != "null" {
            warn!(origin, "rejected WebSocket connection: invalid origin");
            return StatusCode::FORBIDDEN.into_response();
        }
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    info!("new WebSocket connection");

    // First message must be auth: {"token": "<bearer>"}
    let authenticated = match receiver.next().await {
        Some(Ok(Message::Text(text))) => match serde_json::from_str::<Value>(&text) {
            Ok(val) => val
                .get("token")
                .and_then(|t| t.as_str())
                .map(|t| t == state.token.as_str())
                .unwrap_or(false),
            Err(_) => false,
        },
        _ => false,
    };

    if !authenticated {
        let err = protocol::Response::err("0", "auth.failed", "invalid or missing token");
        let msg = serde_json::to_string(&err).unwrap_or_default();
        let _ = sender.send(Message::Text(msg.into())).await;
        let _ = sender.close().await;
        warn!("rejected unauthenticated connection");
        return;
    }

    // Send auth success
    let ok = protocol::Response::ok("0", json!({"status": "authenticated"}));
    let msg = serde_json::to_string(&ok).unwrap_or_default();
    if sender.send(Message::Text(msg.into())).await.is_err() {
        return;
    }

    info!("client authenticated");

    // This client may have event subscriptions from session.open calls.
    // We collect all subscription receivers here and forward them to the WS sender.
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Value>();

    // Spawn a task that forwards events from subscriptions to the WebSocket sender.
    // We use a channel aggregator pattern: session subscriptions feed into event_tx,
    // and this task drains event_rx to the WS.
    let sender = Arc::new(tokio::sync::Mutex::new(sender));
    let sender_clone = Arc::clone(&sender);
    let forward_task = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            let out = serde_json::to_string(&event).unwrap_or_default();
            let mut s = sender_clone.lock().await;
            if s.send(Message::Text(out.into())).await.is_err() {
                break;
            }
        }
    });

    // Main message loop
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let response =
                    handle_message(&text, &state, &event_tx).await;
                let out = serde_json::to_string(&response).unwrap_or_default();
                let mut s = sender.lock().await;
                if s.send(Message::Text(out.into())).await.is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {} // ignore binary, ping, pong
            Err(e) => {
                debug!("ws error: {e}");
                break;
            }
        }
    }

    forward_task.abort();
    info!("client disconnected");
}

async fn handle_message(
    text: &str,
    state: &AppState,
    event_tx: &mpsc::UnboundedSender<Value>,
) -> protocol::Response {
    let frame: InboundFrame = match serde_json::from_str(text) {
        Ok(f) => f,
        Err(e) => {
            return protocol::Response::err("0", "parse.error", format!("invalid frame: {e}"));
        }
    };

    match frame {
        InboundFrame::Request { id, method, params } => {
            dispatch_method(&id, &method, &params, state, event_tx).await
        }
    }
}

async fn dispatch_method(
    id: &str,
    method: &str,
    params: &Value,
    state: &AppState,
    event_tx: &mpsc::UnboundedSender<Value>,
) -> protocol::Response {
    match method {
        "session.open" => {
            let session_key = params
                .get("session_key")
                .and_then(|v| v.as_str())
                .unwrap_or("main");

            match state.sessions.open(session_key).await {
                Ok((entry, mut rx)) => {
                    // Forward this session's events to the client's event channel
                    let tx = event_tx.clone();
                    tokio::spawn(async move {
                        while let Some(event) = rx.recv().await {
                            if tx.send(event).is_err() {
                                break;
                            }
                        }
                    });

                    protocol::Response::ok(
                        id,
                        json!({
                            "session_key": entry.session_key,
                            "thread_id": entry.thread_id,
                            "created_at": entry.created_at,
                        }),
                    )
                }
                Err(e) => protocol::Response::err(id, "session.open.failed", e.to_string()),
            }
        }

        "session.list" => {
            let entries = state.sessions.list().await;
            let items: Vec<Value> = entries
                .into_iter()
                .map(|e| {
                    json!({
                        "session_key": e.session_key,
                        "thread_id": e.thread_id,
                        "created_at": e.created_at,
                    })
                })
                .collect();
            protocol::Response::ok(id, json!({ "sessions": items }))
        }

        "session.history" => {
            let session_key = params
                .get("session_key")
                .and_then(|v| v.as_str())
                .unwrap_or("main");
            let limit = params
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(50) as usize;

            match state.sessions.history(session_key, limit).await {
                Ok(events) => {
                    protocol::Response::ok(id, json!({ "events": events, "count": events.len() }))
                }
                Err(e) => protocol::Response::err(id, "session.history.failed", e.to_string()),
            }
        }

        "session.send" => {
            let session_key = params
                .get("session_key")
                .and_then(|v| v.as_str())
                .unwrap_or("main");
            let content = params
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if content.is_empty() {
                return protocol::Response::err(id, "invalid_params", "content is required");
            }

            match state.sessions.send(session_key, content).await {
                Ok(()) => protocol::Response::ok(id, json!({ "status": "accepted" })),
                Err(e) => {
                    let code = if e.to_string().contains("busy") {
                        "session.busy"
                    } else {
                        "session.send.failed"
                    };
                    protocol::Response::err(id, code, e.to_string())
                }
            }
        }

        _ => protocol::Response::err(
            id,
            "unknown_method",
            format!("unknown method: {method}"),
        ),
    }
}
