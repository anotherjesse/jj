pub mod cli_client;
pub mod protocol;
pub mod session;
pub mod ws;

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use fs2::FileExt;
use rand::Rng;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

/// Default gateway port.
const DEFAULT_PORT: u16 = 9123;

/// Returns the gateway data directory, creating it if needed.
pub fn gateway_dir() -> Result<PathBuf> {
    let dir = dirs_gateway();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn dirs_gateway() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".jj").join("gateway")
}

/// Resolve the port from env or default.
pub fn resolve_port() -> u16 {
    std::env::var("JJ_GATEWAY_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}

// ── PID Guard ──────────────────────────────────────────────────────────

pub struct PidGuard {
    _file: File,
    path: PathBuf,
}

impl PidGuard {
    pub fn acquire(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .with_context(|| format!("open pid file {}", path.display()))?;
        file.try_lock_exclusive()
            .map_err(|_| anyhow!("daemon already running (pid file locked)"))?;
        let mut f = &file;
        write!(f, "{}", std::process::id())?;
        f.flush()?;
        Ok(PidGuard {
            _file: file,
            path: path.to_owned(),
        })
    }
}

impl Drop for PidGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

// ── Token Auth ─────────────────────────────────────────────────────────

/// Read or generate the bearer token for this gateway instance.
pub fn ensure_token(dir: &Path) -> Result<String> {
    let token_path = dir.join("token");
    if token_path.exists() {
        let token = fs::read_to_string(&token_path)?.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }
    let token = generate_token();
    fs::write(&token_path, &token)?;
    // Best-effort permissions (unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&token_path, fs::Permissions::from_mode(0o600));
    }
    Ok(token)
}

fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
    hex::encode(bytes)
}

// ── Daemon Status ──────────────────────────────────────────────────────

pub fn daemon_status() -> Result<bool> {
    let port = resolve_port();
    let addr = format!("127.0.0.1:{port}");
    match TcpStream::connect_timeout(
        &addr.parse().unwrap(),
        std::time::Duration::from_secs(1),
    ) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub fn read_pid(dir: &Path) -> Option<u32> {
    let path = dir.join("daemon.pid");
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

// ── Start Daemon ───────────────────────────────────────────────────────

pub async fn run_daemon() -> Result<()> {
    let dir = gateway_dir()?;
    let pid_path = dir.join("daemon.pid");
    let _guard = PidGuard::acquire(&pid_path)?;
    let token = ensure_token(&dir)?;
    let port = resolve_port();

    info!(port, "starting jj gateway daemon");
    info!("token written to {}", dir.join("token").display());

    // Resolve vault path from env or default
    let vault_path = crate::vault::resolve_vault(
        std::env::var("JJ_VAULT").ok().map(std::path::PathBuf::from),
    );
    let sessions = session::SessionManager::new(vault_path)?;
    let state = ws::AppState::new(token.clone(), sessions);

    // Backfill titles for existing sessions that have messages but no title
    state.sessions.backfill_titles().await;

    let app = ws::router(state);

    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    info!("serving at http://localhost:{port}/?token={token}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("daemon stopped");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut sigterm =
            signal::unix::signal(signal::unix::SignalKind::terminate()).expect("sigterm handler");
        tokio::select! {
            _ = ctrl_c => { info!("received SIGINT"); }
            _ = sigterm.recv() => { info!("received SIGTERM"); }
        }
    }
    #[cfg(not(unix))]
    {
        ctrl_c.await.ok();
        info!("received ctrl-c");
    }
}

// ── Stop Daemon ────────────────────────────────────────────────────────

// ── CLI Subcommand Handlers ───────────────────────────────────────────

/// Connect to daemon, send one request, return payload. Prints JSON error to
/// stderr and returns Err on failure (daemon down, auth fail, etc.).
async fn oneshot_request(method: &str, params: Value) -> Result<Value> {
    let (mut write, mut read) = cli_client::connect().await.map_err(|e| {
        eprintln!(
            "{}",
            serde_json::json!({"error": "daemon_not_running", "message": format!("Gateway daemon is not running. Start it with: jj gateway start ({})", e)})
        );
        e
    })?;
    cli_client::request(&mut write, &mut read, method, params).await
}

pub async fn handle_list() -> Result<()> {
    let payload = oneshot_request("session.list", serde_json::json!({})).await?;
    let sessions = payload.get("sessions").cloned().unwrap_or(serde_json::json!([]));
    println!("{}", serde_json::to_string(&sessions)?);
    Ok(())
}

pub async fn handle_open(session_key: &str) -> Result<()> {
    let payload = oneshot_request(
        "session.open",
        serde_json::json!({"session_key": session_key}),
    )
    .await?;
    println!("{}", serde_json::to_string(&payload)?);
    Ok(())
}

pub async fn handle_history(session_key: &str, limit: usize) -> Result<()> {
    let limit = limit.min(500);
    let payload = oneshot_request(
        "session.history",
        serde_json::json!({"session_key": session_key, "limit": limit}),
    )
    .await?;
    println!("{}", serde_json::to_string(&payload)?);
    Ok(())
}

pub async fn handle_send(session_key: &str, message: &str, wait: Option<Option<u64>>) -> Result<()> {
    use futures_util::StreamExt;
    use tokio_tungstenite::tungstenite::Message;

    let (mut write, mut read) = cli_client::connect().await.map_err(|e| {
        eprintln!(
            "{}",
            serde_json::json!({"error": "daemon_not_running", "message": format!("Gateway daemon is not running. Start it with: jj gateway start ({})", e)})
        );
        e
    })?;

    // Subscribe to session events first (so we receive events from the send)
    cli_client::request(
        &mut write,
        &mut read,
        "session.open",
        serde_json::json!({"session_key": session_key}),
    )
    .await?;

    // Send the message
    let send_payload = cli_client::request(
        &mut write,
        &mut read,
        "session.send",
        serde_json::json!({"session_key": session_key, "content": message}),
    )
    .await?;

    // If not waiting, print accepted and exit
    let timeout_secs = match wait {
        None => {
            println!("{}", serde_json::to_string(&send_payload)?);
            return Ok(());
        }
        Some(None) => 120u64,
        Some(Some(s)) => s,
    };

    // Block and stream events until final/error or timeout
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);

    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            eprintln!(
                "{}",
                serde_json::json!({"error": "timeout", "message": format!("Timed out after {}s waiting for agent response", timeout_secs)})
            );
            std::process::exit(1);
        }

        let msg = tokio::time::timeout(remaining, read.next()).await;
        let msg = match msg {
            Ok(Some(Ok(msg))) => msg,
            Ok(Some(Err(e))) => {
                eprintln!(
                    "{}",
                    serde_json::json!({"error": "connection_lost", "message": format!("{}", e)})
                );
                std::process::exit(1);
            }
            Ok(None) => {
                eprintln!(
                    "{}",
                    serde_json::json!({"error": "connection_lost", "message": "Connection closed"})
                );
                std::process::exit(1);
            }
            Err(_) => {
                eprintln!(
                    "{}",
                    serde_json::json!({"error": "timeout", "message": format!("Timed out after {}s waiting for agent response", timeout_secs)})
                );
                std::process::exit(1);
            }
        };

        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => {
                eprintln!(
                    "{}",
                    serde_json::json!({"error": "connection_lost", "message": "Connection closed"})
                );
                std::process::exit(1);
            }
            _ => continue,
        };

        let val: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Only process events (skip res frames from our earlier requests)
        if val.get("type").and_then(|v| v.as_str()) != Some("event") {
            continue;
        }

        let event_name = val.get("event").and_then(|v| v.as_str()).unwrap_or("");

        // Print every event as a JSON line
        println!("{}", serde_json::to_string(&val)?);

        match event_name {
            "final" => return Ok(()),
            "error" => std::process::exit(1),
            _ => {}
        }
    }
}

pub fn stop_daemon() -> Result<()> {
    let dir = dirs_gateway();
    match read_pid(&dir) {
        Some(pid) => {
            #[cfg(unix)]
            {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
                println!("Sent SIGTERM to pid {pid}");
            }
            #[cfg(not(unix))]
            {
                println!("Cannot send signal on this platform (pid {pid})");
            }
            Ok(())
        }
        None => {
            if daemon_status()? {
                Err(anyhow!("daemon appears running but no pid file found"))
            } else {
                println!("daemon is not running");
                Ok(())
            }
        }
    }
}
