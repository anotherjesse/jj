use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local, Utc};
use dotenvy::dotenv;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::audit::append_ledger;
use crate::git_utils::git_commit;
use crate::knowledge::{apply_patch, KnowledgePatch};
use crate::openai::{ChatResponse, OpenAIClient};
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
        .unwrap_or_else(|| "gpt-4.1-mini".to_string());

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

        loop {
            let response = client.chat(&messages, &tools)?;
            if response.tool_calls.is_empty() {
                let content = response.content.unwrap_or_default();
                if !content.is_empty() {
                    println!("{content}");
                }
                let assistant_event = build_event(
                    None,
                    EventType::AssistantMessage,
                    Role::Assistant,
                    Some(Value::String(content.clone())),
                    None,
                    None,
                    None,
                    None,
                );
                let assistant_content = with_datetime(assistant_event.ts, &content);
                append_event(&thread_path, assistant_event)?;
                messages.push(json!({"role":"assistant","content": assistant_content}));
                break;
            }

            let tool_call_payload = tool_calls_payload(&response)?;
            messages.push(json!({"role":"assistant","tool_calls": tool_call_payload}));

            for call in response.tool_calls {
                let reason = call
                    .arguments
                    .get("reason")
                    .and_then(|val| val.as_str())
                    .unwrap_or("llm_tool_call")
                    .to_string();

                let tool_call_event = build_event(
                    None,
                    EventType::ToolCall,
                    Role::Assistant,
                    None,
                    Some(call.name.clone()),
                    Some(call.arguments.clone()),
                    None,
                    Some(reason),
                );
                append_event(&thread_path, tool_call_event)?;

                let result = execute_tool(
                    &call.name,
                    &call.arguments,
                    &vault,
                    &thread_path,
                    options.allow_commit,
                );
                let result_value = match result {
                    Ok(data) => json!({"status":"ok","data": data}),
                    Err(err) => json!({"status":"error","error": err.to_string()}),
                };

                let tool_result_event = build_event(
                    None,
                    EventType::ToolResult,
                    Role::Tool,
                    None,
                    Some(call.name.clone()),
                    None,
                    Some(result_value.clone()),
                    None,
                );
                append_event(&thread_path, tool_result_event)?;

                let tool_output = serde_json::to_string(&result_value)?;
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call.id,
                    "content": tool_output
                }));
            }
        }
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

fn tool_calls_payload(response: &ChatResponse) -> Result<Vec<Value>> {
    let mut payload = Vec::new();
    for call in &response.tool_calls {
        let args = serde_json::to_string(&call.arguments)?;
        payload.push(json!({
            "id": call.id,
            "type": "function",
            "function": {
                "name": call.name,
                "arguments": args
            }
        }));
    }
    Ok(payload)
}

fn with_datetime(ts: DateTime<Utc>, content: &str) -> String {
    let local = ts.with_timezone(&Local).to_rfc3339();
    if content.is_empty() {
        format!("[datetime]: {local}")
    } else {
        format!("[datetime]: {local}\n{content}")
    }
}

fn tool_schemas() -> Vec<Value> {
    vec![
        json!({
            "type": "function",
            "function": {
                "name": "vault_init",
                "description": "Initialize a JJ vault directory with required structure.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Vault directory path." },
                        "reason": { "type": "string" }
                    },
                    "required": ["reason"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "thread_create",
                "description": "Create a new thread file in the vault.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "vault": { "type": "string", "description": "Vault path." },
                        "thread_id": { "type": "string" },
                        "date": { "type": "string", "description": "YYYY-MM-DD" },
                        "reason": { "type": "string" }
                    },
                    "required": ["reason"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "thread_read",
                "description": "Read events from a thread file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "thread": { "type": "string", "description": "Thread file path." },
                        "offset": { "type": "integer" },
                        "limit": { "type": "integer" },
                        "reason": { "type": "string" }
                    },
                    "required": ["reason"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "thread_append",
                "description": "Append an event to a thread file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "thread": { "type": "string", "description": "Thread file path." },
                        "event_type": { "type": "string", "description": "user_message|assistant_message|tool_call|tool_result|system_note|attachment_added" },
                        "role": { "type": "string", "description": "user|assistant|tool|system" },
                        "content": { "type": "string" },
                        "content_json": { "type": "object" },
                        "tool_name": { "type": "string" },
                        "tool_args": { "type": "object" },
                        "tool_result": { "type": "object" },
                        "reason": { "type": "string" }
                    },
                    "required": ["thread", "event_type", "role", "reason"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "knowledge_apply",
                "description": "Apply a knowledge patch (front matter + body) and append to audit ledger.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "patch": { "type": "object" },
                        "author": { "type": "string" },
                        "reason": { "type": "string" },
                        "proposal_id": { "type": "string" },
                        "commit": { "type": "boolean" }
                    },
                    "required": ["patch", "author", "reason"]
                }
            }
        })
    ]
}

fn execute_tool(
    name: &str,
    args: &Value,
    vault: &Path,
    thread_path: &Path,
    allow_commit: bool,
) -> Result<Value> {
    match name {
        "vault_init" => {
            let path = args
                .get("path")
                .and_then(|val| val.as_str())
                .map(PathBuf::from)
                .unwrap_or_else(|| vault.to_path_buf());
            init_vault(&path)?;
            Ok(json!({ "vault_path": path }))
        }
        "thread_create" => {
            let vault_path = args
                .get("vault")
                .and_then(|val| val.as_str())
                .map(PathBuf::from)
                .unwrap_or_else(|| vault.to_path_buf());
            let thread_id = args
                .get("thread_id")
                .and_then(|val| val.as_str())
                .map(|s| s.to_string());
            let date = args
                .get("date")
                .and_then(|val| val.as_str())
                .map(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d"))
                .transpose()?;
            let path = create_thread(&vault_path, thread_id, date)?;
            Ok(json!({ "thread_path": path }))
        }
        "thread_read" => {
            let path = args
                .get("thread")
                .and_then(|val| val.as_str())
                .map(PathBuf::from)
                .unwrap_or_else(|| thread_path.to_path_buf());
            let offset = args.get("offset").and_then(|val| val.as_u64()).map(|v| v as usize);
            let limit = args.get("limit").and_then(|val| val.as_u64()).map(|v| v as usize);
            let lines = read_thread(&path, offset, limit)?;
            Ok(json!({ "count": lines.len(), "lines": lines }))
        }
        "thread_append" => {
            let path = args
                .get("thread")
                .and_then(|val| val.as_str())
                .map(PathBuf::from)
                .unwrap_or_else(|| thread_path.to_path_buf());
            let event_type = args
                .get("event_type")
                .and_then(|val| val.as_str())
                .ok_or_else(|| anyhow!("event_type required"))?;
            let role = args
                .get("role")
                .and_then(|val| val.as_str())
                .ok_or_else(|| anyhow!("role required"))?;
            let event_type = parse_event_type(event_type)?;
            let role = parse_role(role)?;

            let content = args.get("content").and_then(|val| val.as_str()).map(|s| Value::String(s.to_string()));
            let content_json = args.get("content_json").cloned();
            let content_value = content_json.or(content);

            let tool_name = args.get("tool_name").and_then(|val| val.as_str()).map(|s| s.to_string());
            let tool_args = args.get("tool_args").cloned();
            let tool_result = args.get("tool_result").cloned();
            let reason = args.get("reason").and_then(|val| val.as_str()).map(|s| s.to_string());

            let event = build_event(None, event_type, role, content_value, tool_name, tool_args, tool_result, reason);
            append_event(&path, event)?;
            Ok(json!({"thread_path": path}))
        }
        "knowledge_apply" => {
            let patch_value = args
                .get("patch")
                .ok_or_else(|| anyhow!("patch required"))?
                .clone();
            let patch: KnowledgePatch = serde_json::from_value(patch_value)?;
            let author = args
                .get("author")
                .and_then(|val| val.as_str())
                .unwrap_or("assistant");
            let reason = args
                .get("reason")
                .and_then(|val| val.as_str())
                .unwrap_or("tool_call");
            let proposal_id = args
                .get("proposal_id")
                .and_then(|val| val.as_str())
                .map(|s| s.to_string());
            let commit = args.get("commit").and_then(|val| val.as_bool()).unwrap_or(false);
            if commit && !allow_commit {
                return Err(anyhow!("commit requested but allow_commit is false"));
            }

            let result = apply_patch(vault, patch, author, reason, proposal_id.clone())?;
            let ledger_path = vault.join("audit/ledger.jsonl");
            append_ledger(&ledger_path, &result.ledger_entry)?;

            if commit {
                let repo_root = PathBuf::from(".");
                let message = match &proposal_id {
                    Some(id) => format!("{id}: {reason}"),
                    None => format!("memory: {reason}"),
                };
                git_commit(&repo_root, &[result.doc_path.clone(), ledger_path.clone()], &message)?;
            }

            Ok(json!({
                "doc_path": result.doc_path,
                "ledger_id": result.ledger_entry.ledger_id
            }))
        }
        _ => Err(anyhow!("unknown tool: {name}")),
    }
}

fn parse_event_type(value: &str) -> Result<EventType> {
    match value {
        "user_message" => Ok(EventType::UserMessage),
        "assistant_message" => Ok(EventType::AssistantMessage),
        "tool_call" => Ok(EventType::ToolCall),
        "tool_result" => Ok(EventType::ToolResult),
        "system_note" => Ok(EventType::SystemNote),
        "attachment_added" => Ok(EventType::AttachmentAdded),
        _ => Err(anyhow!("invalid event_type: {value}")),
    }
}

fn parse_role(value: &str) -> Result<Role> {
    match value {
        "user" => Ok(Role::User),
        "assistant" => Ok(Role::Assistant),
        "tool" => Ok(Role::Tool),
        "system" => Ok(Role::System),
        _ => Err(anyhow!("invalid role: {value}")),
    }
}
