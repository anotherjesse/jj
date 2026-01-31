use anyhow::{anyhow, Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::{ensure_token, gateway_dir, resolve_port};

/// Connect to the gateway daemon, authenticate, and return a split WS connection.
pub async fn connect() -> Result<(
    futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
)> {
    let port = resolve_port();
    let dir = gateway_dir()?;
    let token = read_token(&dir)?;

    let url = format!("ws://127.0.0.1:{port}/ws");
    let (ws, _) = connect_async(&url)
        .await
        .with_context(|| format!("connect to gateway at {url}"))?;

    let (mut write, mut read) = ws.split();

    // Send auth
    let auth = json!({"token": token});
    write
        .send(Message::Text(serde_json::to_string(&auth)?.into()))
        .await?;

    // Read auth response
    let resp = read
        .next()
        .await
        .ok_or_else(|| anyhow!("gateway closed during auth"))??;
    let resp: Value = match resp {
        Message::Text(t) => serde_json::from_str(&t)?,
        _ => return Err(anyhow!("unexpected message type during auth")),
    };
    if !resp.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        let msg = resp
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("auth failed");
        return Err(anyhow!("gateway auth failed: {msg}"));
    }

    Ok((write, read))
}

/// Send a JSON-RPC request and wait for the response.
pub async fn request(
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    read: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    method: &str,
    params: Value,
) -> Result<Value> {
    let id = ulid::Ulid::new().to_string();
    let frame = json!({
        "type": "req",
        "id": id,
        "method": method,
        "params": params,
    });
    write
        .send(Message::Text(serde_json::to_string(&frame)?.into()))
        .await?;

    // Read messages until we get the matching response
    // (events may arrive in between)
    loop {
        let msg = read
            .next()
            .await
            .ok_or_else(|| anyhow!("connection closed"))??;
        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => return Err(anyhow!("connection closed")),
            _ => continue,
        };
        let val: Value = serde_json::from_str(&text)?;
        // Check if this is our response
        if val.get("type").and_then(|v| v.as_str()) == Some("res")
            && val.get("id").and_then(|v| v.as_str()) == Some(&id)
        {
            if val.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                return Ok(val.get("payload").cloned().unwrap_or(json!(null)));
            } else {
                let msg = val
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown error");
                return Err(anyhow!("{msg}"));
            }
        }
        // Otherwise it's an event — ignore for now in this simple request helper
    }
}

fn read_token(dir: &PathBuf) -> Result<String> {
    let path = dir.join("token");
    fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .with_context(|| format!("read gateway token from {}", path.display()))
}
