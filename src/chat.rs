use anyhow::{anyhow, Context, Result};
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::audit::LedgerEntry;
use crate::knowledge::read_doc;

use crate::agent::{run_agent_loop, tool_schemas, with_datetime, AgentConfig};
use crate::openai::OpenAIClient;
use crate::thread_store::{
    append_event, build_event, create_thread, read_thread, EventType, Role, ThreadEvent, ThreadMeta,
};
use crate::vault::{init_vault, resolve_vault};

pub struct ChatOptions {
    pub vault: Option<PathBuf>,
    pub thread: Option<PathBuf>,
    pub model: Option<String>,
    pub allow_commit: bool,
    pub history: usize,
    pub direct: bool,
}

pub fn run_chat(options: ChatOptions) -> Result<()> {
    dotenv().ok();

    // If not --direct, try daemon mode
    if !options.direct {
        if crate::gateway::daemon_status()? {
            return run_chat_daemon(options);
        }
        // Daemon not running — fall through to direct mode
        eprintln!("(daemon not running, using direct mode)");
    }

    run_chat_direct(options)
}

fn run_chat_direct(options: ChatOptions) -> Result<()> {
    let vault = resolve_vault(options.vault);
    if !vault.exists() {
        init_vault(&vault)?;
    }

    let thread_path = match options.thread {
        Some(path) => {
            if !path.exists() {
                return Err(anyhow!("thread file does not exist: {}", path.display()));
            }
            path
        }
        None => create_thread(&vault, None, None, Some(ThreadMeta {
            kind: "chat".into(),
            agent: Some("jj".into()),
            model: None,
        }))?,
    };

    let system_prompt = load_system_prompt(&vault)?;
    let mut messages = Vec::new();
    messages.push(json!({"role":"system","content": system_prompt}));
    if options.history > 0 {
        load_history(&thread_path, options.history, &mut messages)?;
    }

    let api_key = env::var("OPENAI_API_KEY").context("OPENAI_API_KEY is not set")?;
    let base_url = env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".to_string());
    let model = options
        .model
        .or_else(|| env::var("OPENAI_MODEL").ok())
        .unwrap_or_else(|| "gpt-5.2-2025-12-11".to_string());

    let mut client = OpenAIClient::new(api_key, base_url, model.clone());
    let tools = tool_schemas();

    println!("JJ REPL ready. Thread: {}", thread_path.display());
    println!("Model: {model}. Type /help for commands.");

    let mut rl = DefaultEditor::new()?;
    loop {
        let line = match rl.readline("jj> ") {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => return Err(err.into()),
        };
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        rl.add_history_entry(input)?;
        if input.starts_with('/') {
            if !handle_command(input, &mut client, &thread_path, &vault, &tools)? {
                break;
            }
            continue;
        }

        let user_event = build_event(
            None,
            EventType::UserMessage,
            Role::User,
            Some(Value::String(input.to_string())),
            None,
            None,
            None,
            None,
        );
        let user_content = with_datetime(user_event.ts, input);
        append_event(&thread_path, user_event)?;
        messages.push(json!({"role":"user","content": user_content}));

        let config = AgentConfig {
            vault_path: vault.clone(),
            thread_path: thread_path.clone(),
            max_turns: 20,
            allow_commit: options.allow_commit,
            tool_filter: None,
        };
        messages = run_agent_loop(&config, messages, &client)?;
    }

    Ok(())
}

/// Run chat via the gateway daemon over WebSocket.
fn run_chat_daemon(options: ChatOptions) -> Result<()> {
    // Build a tokio runtime for the async WS client
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_chat_daemon_async(options))
}

async fn run_chat_daemon_async(options: ChatOptions) -> Result<()> {
    use crate::gateway::cli_client;
    use futures_util::SinkExt;
    use tokio_tungstenite::tungstenite::Message;

    let (mut write, mut read) = cli_client::connect().await?;

    // Determine session key
    let session_key = options
        .thread
        .as_ref()
        .and_then(|p| p.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("main")
        .to_string();

    // Open/join the session
    let payload = cli_client::request(
        &mut write,
        &mut read,
        "session.open",
        json!({"session_key": session_key}),
    )
    .await?;
    let thread_id = payload
        .get("thread_id")
        .and_then(|v| v.as_str())
        .unwrap_or("?");

    println!("JJ REPL (daemon). Session: {session_key}, Thread: {thread_id}");
    println!("Type /help for commands.");

    // Spawn a background task to print incoming events
    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
    let event_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = futures_util::StreamExt::next(&mut read) => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(val) = serde_json::from_str::<Value>(&text) {
                                let event_type = val.get("event").and_then(|v| v.as_str()).unwrap_or("");
                                match event_type {
                                    "delta" => {
                                        if let Some(text) = val.get("payload").and_then(|p| p.get("text")).and_then(|t| t.as_str()) {
                                            print!("{text}");
                                            use std::io::Write;
                                            let _ = std::io::stdout().flush();
                                        }
                                    }
                                    "final" => {
                                        if let Some(content) = val.get("payload").and_then(|p| p.get("content")).and_then(|c| c.as_str()) {
                                            println!("{content}");
                                        }
                                    }
                                    "tool_activity" => {
                                        if let Some(name) = val.get("payload").and_then(|p| p.get("tool_name")).and_then(|n| n.as_str()) {
                                            eprintln!("[tool: {name}]");
                                        }
                                    }
                                    "error" => {
                                        if let Some(msg) = val.get("payload").and_then(|p| p.get("message")).and_then(|m| m.as_str()) {
                                            eprintln!("Error: {msg}");
                                        }
                                    }
                                    "user_message" => {} // we sent this, ignore
                                    _ => {
                                        // Response frames (type: "res") — ignore, handled by request()
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) | None => break,
                        _ => {}
                    }
                }
                _ = &mut stop_rx => break,
            }
        }
    });

    // REPL loop on the main thread (rustyline blocks)
    let mut rl = rustyline::DefaultEditor::new()?;
    loop {
        let line = match rl.readline("jj> ") {
            Ok(line) => line,
            Err(rustyline::error::ReadlineError::Interrupted) => continue,
            Err(rustyline::error::ReadlineError::Eof) => break,
            Err(err) => return Err(err.into()),
        };
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        rl.add_history_entry(input)?;

        if input == "/exit" || input == "/quit" {
            break;
        }
        if input == "/help" {
            println!("Commands:");
            println!("  /help           Show this help");
            println!("  /exit, /quit    Exit the REPL");
            println!("  /sessions       List all sessions");
            println!("  /session <key>  Switch to a different session");
            continue;
        }
        if input == "/sessions" {
            let frame = json!({
                "type": "req",
                "id": ulid::Ulid::new().to_string(),
                "method": "session.list",
                "params": {},
            });
            write
                .send(Message::Text(serde_json::to_string(&frame)?.into()))
                .await?;
            // Response will be printed by event task (not ideal, but works for now)
            continue;
        }

        // Send user message
        let frame = json!({
            "type": "req",
            "id": ulid::Ulid::new().to_string(),
            "method": "session.send",
            "params": {
                "session_key": session_key,
                "content": input,
            },
        });
        write
            .send(Message::Text(serde_json::to_string(&frame)?.into()))
            .await?;
    }

    let _ = stop_tx.send(());
    event_task.abort();
    Ok(())
}

fn handle_command(
    input: &str,
    client: &mut OpenAIClient,
    thread_path: &Path,
    vault: &Path,
    tools: &[Value],
) -> Result<bool> {
    let mut parts = input.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    match cmd {
        "/exit" | "/quit" => return Ok(false),
        "/help" => {
            println!("Commands:");
            println!("  /help           Show this help");
            println!("  /exit, /quit    Exit the REPL");
            println!("  /model <name>   Set model for this session");
            println!("  /thread         Show thread path");
            println!("  /vault          Show vault path");
            println!("  /tools          List tool names");
        }
        "/model" => {
            if let Some(model) = parts.next() {
                client.set_model(model.to_string());
                println!("Model set to {model}");
            } else {
                println!("Usage: /model <name>");
            }
        }
        "/thread" => println!("{}", thread_path.display()),
        "/vault" => println!("{}", vault.display()),
        "/tools" => {
            let names: Vec<String> = tools
                .iter()
                .filter_map(|tool| {
                    tool.get("function")
                        .and_then(|func| func.get("name"))
                        .and_then(|name| name.as_str())
                        .map(|s| s.to_string())
                })
                .collect();
            println!("{}", names.join(", "));
        }
        _ => println!("Unknown command. Type /help."),
    }
    Ok(true)
}

pub fn load_system_prompt(vault: &Path) -> Result<String> {
    let path = vault.join("prompts/jj.system.md");
    let base = if path.exists() {
        fs::read_to_string(&path)
            .with_context(|| format!("read system prompt {}", path.display()))?
    } else {
        "You are JJ, a memory-first assistant.".to_string()
    };

    let toc = build_vault_toc(vault).unwrap_or_default();
    let digest = build_mutation_digest(vault).unwrap_or_default();

    if toc.is_empty() && digest.is_empty() {
        Ok(base)
    } else {
        Ok(format!("{}\n\n{}\n\n{}", base, toc, digest))
    }
}

fn build_vault_toc(vault: &Path) -> Result<String> {
    let root = vault.join("knowledge");
    if !root.exists() {
        return Ok(String::new());
    }

    // Collect docs grouped by subdirectory
    let mut groups: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let group = path
                    .parent()
                    .and_then(|p| p.strip_prefix(&root).ok())
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                let filename = path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                let label = match read_doc(&path) {
                    Ok(doc) => {
                        if doc.front_matter.summary.is_empty() {
                            doc.front_matter.title.clone()
                        } else {
                            doc.front_matter.summary.clone()
                        }
                    }
                    Err(_) => filename.trim_end_matches(".md").to_string(),
                };
                groups.entry(group).or_default().push((filename, label));
            }
        }
    }

    if groups.is_empty() {
        return Ok(String::new());
    }

    let mut out = String::from("## Your Knowledge\n");
    for (group, mut docs) in groups {
        docs.sort_by(|a, b| a.0.cmp(&b.0));
        let dir_name = if group.is_empty() { "root" } else { &group };
        out.push_str(&format!("\n### {}/ ({} docs)\n", dir_name, docs.len()));
        for (filename, label) in &docs {
            out.push_str(&format!("- {} — {}\n", filename, label));
        }
    }
    Ok(out)
}

fn build_mutation_digest(vault: &Path) -> Result<String> {
    let ledger_path = vault.join("audit/ledger.jsonl");
    if !ledger_path.exists() {
        return Ok(String::new());
    }

    let cutoff = Utc::now() - Duration::hours(24);
    let file = fs::File::open(&ledger_path)?;
    let reader = BufReader::new(file);

    let mut recent: Vec<LedgerEntry> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<LedgerEntry>(&line) {
            if entry.ts >= cutoff {
                recent.push(entry);
            }
        }
    }

    if recent.is_empty() {
        return Ok("## Recent Changes (last 24h)\n\nNo changes.\n".to_string());
    }

    let mut out = String::from("## Recent Changes (last 24h)\n\n");
    for entry in &recent {
        let time = entry.ts.format("%H:%M");
        let desc = if entry.change_summary.is_empty() {
            &entry.reason
        } else {
            &entry.change_summary
        };
        let op = if entry.prev_hash.is_some() {
            "Updated"
        } else {
            "Created"
        };
        out.push_str(&format!("- [{}] {} {} — {}\n", time, op, entry.doc_path, desc));
    }
    Ok(out)
}

fn load_history(thread_path: &Path, history: usize, messages: &mut Vec<Value>) -> Result<()> {
    let lines = read_thread(thread_path, None, None)?;
    let start = lines.len().saturating_sub(history);
    for line in lines.into_iter().skip(start) {
        if let Ok(event) = serde_json::from_str::<ThreadEvent>(&line) {
            if let Some(value) = event.content {
                let content = match value {
                    Value::String(text) => text,
                    other => other.to_string(),
                };
                match event.event_type {
                    EventType::UserMessage => {
                        let content = with_datetime(event.ts, &content);
                        messages.push(json!({"role":"user","content": content}));
                    }
                    EventType::AssistantMessage => {
                        let content = with_datetime(event.ts, &content);
                        messages.push(json!({"role":"assistant","content": content}));
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

