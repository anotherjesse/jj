use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock, Semaphore};
use tracing::{info, warn};

use crate::thread_store::{
    append_event, build_event, create_thread, read_thread, EventType, Role,
    ThreadEvent, ThreadMeta,
};

/// Persistent mapping of session_key -> session metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    pub session_key: String,
    pub thread_id: String,
    pub thread_path: String,
    pub created_at: String,
}

/// Runtime state for a single session.
struct SessionState {
    entry: SessionEntry,
    /// Limits to 1 concurrent agent run per session.
    run_semaphore: Arc<Semaphore>,
    /// All subscribed clients for this session.
    subscribers: Mutex<Vec<mpsc::UnboundedSender<Value>>>,
}

/// Manages all sessions, backed by sessions.json in the vault.
pub struct SessionManager {
    vault_path: PathBuf,
    sessions: RwLock<HashMap<String, Arc<SessionState>>>,
    index_path: PathBuf,
}

impl SessionManager {
    /// Load or create the session manager for a vault.
    pub fn new(vault_path: PathBuf) -> Result<Self> {
        let index_path = vault_path.join("gateway").join("sessions.json");
        if let Some(parent) = index_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut sessions = HashMap::new();
        if index_path.exists() {
            let content = fs::read_to_string(&index_path)
                .with_context(|| format!("read {}", index_path.display()))?;
            let entries: Vec<SessionEntry> = serde_json::from_str(&content)
                .with_context(|| "parse sessions.json")?;
            for entry in entries {
                let key = entry.session_key.clone();
                sessions.insert(
                    key,
                    Arc::new(SessionState {
                        entry,
                        run_semaphore: Arc::new(Semaphore::new(1)),
                        subscribers: Mutex::new(Vec::new()),
                    }),
                );
            }
        }

        info!(sessions = sessions.len(), "loaded sessions index");

        Ok(Self {
            vault_path,
            sessions: RwLock::new(sessions),
            index_path,
        })
    }

    #[allow(dead_code)]
    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    /// Open (create-if-missing) a session. Returns session metadata and subscribes the client.
    pub async fn open(
        &self,
        session_key: &str,
    ) -> Result<(SessionEntry, mpsc::UnboundedReceiver<Value>)> {
        // Check if exists
        {
            let sessions = self.sessions.read().await;
            if let Some(state) = sessions.get(session_key) {
                let (tx, rx) = mpsc::unbounded_channel();
                state.subscribers.lock().await.push(tx);
                return Ok((state.entry.clone(), rx));
            }
        }

        // Create new session
        let thread_path = create_thread(
            &self.vault_path,
            None,
            None,
            Some(ThreadMeta {
                kind: "chat".into(),
                agent: Some("jj".into()),
                model: None,
            }),
        )?;
        let thread_id = thread_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let entry = SessionEntry {
            session_key: session_key.to_string(),
            thread_id,
            thread_path: thread_path.to_string_lossy().to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let (tx, rx) = mpsc::unbounded_channel();
        let state = Arc::new(SessionState {
            entry: entry.clone(),
            run_semaphore: Arc::new(Semaphore::new(1)),
            subscribers: Mutex::new(vec![tx]),
        });

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_key.to_string(), state);
        }
        self.persist_index().await?;

        info!(session_key, "created new session");
        Ok((entry, rx))
    }

    /// List all sessions with metadata.
    pub async fn list(&self) -> Vec<SessionEntry> {
        let sessions = self.sessions.read().await;
        sessions.values().map(|s| s.entry.clone()).collect()
    }

    /// Fetch last N events from a session's thread transcript.
    pub async fn history(&self, session_key: &str, limit: usize) -> Result<Vec<String>> {
        let sessions = self.sessions.read().await;
        let state = sessions
            .get(session_key)
            .ok_or_else(|| anyhow!("session not found: {session_key}"))?;
        let thread_path = PathBuf::from(&state.entry.thread_path);
        // Read all events then take last N
        let all = read_thread(&thread_path, None, None)?;
        let start = all.len().saturating_sub(limit);
        Ok(all.into_iter().skip(start).collect())
    }

    /// Send a user message and trigger an agent run.
    /// Returns immediately after enqueuing; the agent run happens in background.
    pub async fn send(
        self: &Arc<Self>,
        session_key: &str,
        content: &str,
    ) -> Result<()> {
        let state = {
            let sessions = self.sessions.read().await;
            sessions
                .get(session_key)
                .cloned()
                .ok_or_else(|| anyhow!("session not found: {session_key}"))?
        };

        let thread_path = PathBuf::from(&state.entry.thread_path);

        // Append user message to thread
        let user_event = build_event(
            None,
            EventType::UserMessage,
            Role::User,
            Some(Value::String(content.to_string())),
            None,
            None,
            None,
            None,
        );
        append_event(&thread_path, user_event)?;

        // Broadcast the user message to all subscribers
        let user_msg = json!({
            "type": "event",
            "event": "user_message",
            "session_id": state.entry.session_key,
            "payload": { "content": content }
        });
        broadcast(&state.subscribers, &user_msg).await;

        // Try to acquire the run semaphore (non-blocking, owned)
        let permit = match Arc::clone(&state.run_semaphore).try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                return Err(anyhow!("session is busy (agent run in progress)"));
            }
        };

        // Spawn agent run in background
        let manager = Arc::clone(self);
        let session_key = session_key.to_string();
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            let _permit = permit; // held until this block completes
            if let Err(e) = manager.run_agent(&session_key, &state).await {
                warn!(session_key, error = %e, "agent run failed");
                let err_event = json!({
                    "type": "event",
                    "event": "error",
                    "session_id": session_key,
                    "payload": { "message": e.to_string() }
                });
                broadcast(&state.subscribers, &err_event).await;
            }
        });

        Ok(())
    }

    /// Subscribe to a session's events (without sending a message).
    #[allow(dead_code)]
    pub async fn subscribe(
        &self,
        session_key: &str,
    ) -> Result<mpsc::UnboundedReceiver<Value>> {
        let sessions = self.sessions.read().await;
        let state = sessions
            .get(session_key)
            .ok_or_else(|| anyhow!("session not found: {session_key}"))?;
        let (tx, rx) = mpsc::unbounded_channel();
        state.subscribers.lock().await.push(tx);
        Ok(rx)
    }

    /// Run the agent loop for a session (blocking work wrapped in spawn_blocking).
    async fn run_agent(&self, session_key: &str, state: &SessionState) -> Result<()> {
        let thread_path = PathBuf::from(&state.entry.thread_path);
        let vault_path = self.vault_path.clone();
        let session_key_owned = session_key.to_string();
        let subscribers = state.subscribers.lock().await.clone();

        // Write run.started marker
        let started = build_event(
            None,
            EventType::SystemNote,
            Role::System,
            Some(Value::String("run.started".to_string())),
            None,
            None,
            None,
            None,
        );
        append_event(&thread_path, started)?;

        // Create a sync channel for the agent loop to emit events
        let (event_tx, event_rx) = std::sync::mpsc::channel::<crate::agent::AgentEvent>();

        // Spawn a task that bridges sync events to async broadcast
        let subs_clone = subscribers.clone();
        let sk = session_key.to_string();
        let bridge_task = tokio::spawn(async move {
            // Wrap the blocking recv in spawn_blocking to avoid blocking the async runtime
            loop {
                let rx_ref = &event_rx;
                // We can't move event_rx into spawn_blocking, so poll with try_recv + sleep
                match rx_ref.try_recv() {
                    Ok(event) => {
                        use crate::agent::AgentEvent;
                        let ws_event = match event {
                            AgentEvent::ToolActivity { tool_name } => json!({
                                "type": "event",
                                "event": "tool_activity",
                                "session_id": sk,
                                "payload": { "tool_name": tool_name }
                            }),
                            AgentEvent::FinalContent { content } => json!({
                                "type": "event",
                                "event": "final",
                                "session_id": sk,
                                "payload": { "content": content }
                            }),
                        };
                        broadcast_to(&subs_clone, &ws_event);
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                }
            }
        });

        // Run the sync agent loop in a blocking thread
        let result = tokio::task::spawn_blocking(move || {
            run_agent_blocking(&vault_path, &thread_path, &session_key_owned, event_tx)
        })
        .await
        .context("agent task panicked")?;

        // Wait for bridge to drain remaining events
        let _ = bridge_task.await;

        if let Err(ref e) = result {
            let err_event = json!({
                "type": "event",
                "event": "error",
                "session_id": session_key,
                "payload": { "message": e.to_string() }
            });
            broadcast_to(&subscribers, &err_event);
        }

        // Write run.completed marker
        let completed = build_event(
            None,
            EventType::SystemNote,
            Role::System,
            Some(Value::String("run.completed".to_string())),
            None,
            None,
            None,
            None,
        );
        append_event(&PathBuf::from(&state.entry.thread_path), completed)?;

        result.map(|_| ())
    }

    /// Persist the sessions index to disk.
    async fn persist_index(&self) -> Result<()> {
        let sessions = self.sessions.read().await;
        let entries: Vec<&SessionEntry> = sessions.values().map(|s| &s.entry).collect();
        let json = serde_json::to_string_pretty(&entries)?;
        fs::write(&self.index_path, json.as_bytes())
            .with_context(|| format!("write {}", self.index_path.display()))?;
        Ok(())
    }
}

/// Broadcast a JSON event to all subscribers of a session.
async fn broadcast(subscribers: &Mutex<Vec<mpsc::UnboundedSender<Value>>>, event: &Value) {
    let mut subs = subscribers.lock().await;
    subs.retain(|tx| tx.send(event.clone()).is_ok());
}

/// Broadcast to a snapshot of subscribers (no lock needed).
fn broadcast_to(subscribers: &[mpsc::UnboundedSender<Value>], event: &Value) {
    for tx in subscribers {
        let _ = tx.send(event.clone());
    }
}

/// Run the sync agent loop. Called from spawn_blocking.
fn run_agent_blocking(
    vault_path: &Path,
    thread_path: &Path,
    _session_key: &str,
    event_sink: std::sync::mpsc::Sender<crate::agent::AgentEvent>,
) -> Result<String> {
    use crate::agent::{run_agent_loop, AgentConfig};
    use crate::chat::load_system_prompt;
    use crate::openai::OpenAIClient;

    dotenvy::dotenv().ok();

    let api_key = std::env::var("OPENAI_API_KEY").context("OPENAI_API_KEY not set")?;
    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com".to_string());
    let model = std::env::var("OPENAI_MODEL")
        .unwrap_or_else(|_| "gpt-5.2-2025-12-11".to_string());

    let client = OpenAIClient::new(api_key, base_url, model);

    let system_prompt = load_system_prompt(vault_path)?;
    let mut messages = vec![json!({"role": "system", "content": system_prompt})];

    // Load recent history for context
    let history = read_thread(thread_path, None, None)?;
    let start = history.len().saturating_sub(50);
    for line in history.into_iter().skip(start) {
        if let Ok(event) = serde_json::from_str::<ThreadEvent>(&line) {
            if let Some(value) = event.content {
                let content = match value {
                    Value::String(text) => text,
                    other => other.to_string(),
                };
                match event.event_type {
                    EventType::UserMessage => {
                        let content = crate::agent::with_datetime(event.ts, &content);
                        messages.push(json!({"role": "user", "content": content}));
                    }
                    EventType::AssistantMessage => {
                        messages.push(json!({"role": "assistant", "content": content}));
                    }
                    _ => {}
                }
            }
        }
    }

    let config = AgentConfig {
        vault_path: vault_path.to_path_buf(),
        thread_path: thread_path.to_path_buf(),
        max_turns: 20,
        allow_commit: false,
        tool_filter: None,
        event_sink: Some(event_sink),
    };

    let final_messages = run_agent_loop(&config, messages, &client)?;

    // Extract the last assistant message as the final content
    let final_content = final_messages
        .iter()
        .rev()
        .find_map(|m| {
            if m.get("role")?.as_str()? == "assistant" {
                m.get("content")?.as_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();

    Ok(final_content)
}
