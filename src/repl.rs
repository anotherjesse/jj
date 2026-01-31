use anyhow::{anyhow, Context, Result};
use dotenvy::dotenv;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::agent::{run_agent_loop, tool_schemas, with_datetime, AgentConfig};
use crate::openai::OpenAIClient;
use crate::thread_store::{
    append_event, build_event, create_thread, read_thread, EventType, Role, ThreadEvent,
};
use crate::vault::{init_vault, resolve_vault};

pub struct ReplOptions {
    pub vault: Option<PathBuf>,
    pub thread: Option<PathBuf>,
    pub model: Option<String>,
    pub allow_commit: bool,
    pub history: usize,
}

pub fn run_repl(options: ReplOptions) -> Result<()> {
    dotenv().ok();

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
        None => create_thread(&vault, None, None)?,
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
        };
        messages = run_agent_loop(&config, messages, &client)?;
    }

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

fn load_system_prompt(vault: &Path) -> Result<String> {
    let path = vault.join("prompts/jj.system.md");
    if path.exists() {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("read system prompt {}", path.display()))?;
        Ok(content)
    } else {
        Ok("You are JJ, a memory-first assistant.".to_string())
    }
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

