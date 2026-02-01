use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, Utc};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::audit::append_ledger;
use crate::embedding_index::{build_knowledge_index, search_knowledge_index};
use crate::embeddings::EmbeddingClient;
use crate::git_utils::git_commit;
use crate::knowledge::{apply_patch, read_doc, KnowledgePatch};
use crate::openai::{ChatResponse, OpenAIClient};
use crate::thread_store::{
    append_event, build_event, create_thread, read_thread, EventType, Role,
};
use crate::vault::init_vault;

pub struct AgentConfig {
    pub vault_path: PathBuf,
    pub thread_path: PathBuf,
    pub max_turns: usize,
    pub allow_commit: bool,
    /// If set, only expose these tools (by name). If None, expose all.
    pub tool_filter: Option<Vec<String>>,
}

pub fn run_agent_loop(
    config: &AgentConfig,
    initial_messages: Vec<Value>,
    client: &OpenAIClient,
) -> Result<Vec<Value>> {
    let all_tools = tool_schemas();
    let tools: Vec<Value> = match &config.tool_filter {
        Some(names) => all_tools
            .into_iter()
            .filter(|t| {
                t.get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                    .map(|n| names.iter().any(|allowed| allowed == n))
                    .unwrap_or(false)
            })
            .collect(),
        None => all_tools,
    };
    let mut messages = initial_messages;

    for turn in 0..config.max_turns {
        let response = client.chat(&messages, &tools)?;

        if response.tool_calls.is_empty() {
            let content = response.content.unwrap_or_default();
            if !content.is_empty() {
                println!("{content}");
            }
            let event = build_event(
                None,
                EventType::AssistantMessage,
                Role::Assistant,
                Some(Value::String(content.clone())),
                None,
                None,
                None,
                None,
            );
            append_event(&config.thread_path, event)?;
            messages.push(json!({"role": "assistant", "content": content}));
            break;
        }

        let tool_call_payload = tool_calls_payload(&response)?;
        messages.push(json!({"role": "assistant", "tool_calls": tool_call_payload}));

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
            append_event(&config.thread_path, tool_call_event)?;

            let result = execute_tool(
                &call.name,
                &call.arguments,
                &config.vault_path,
                &config.thread_path,
                config.allow_commit,
            );
            let result_value = match result {
                Ok(data) => json!({"status": "ok", "data": data}),
                Err(err) => json!({"status": "error", "error": err.to_string()}),
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
            append_event(&config.thread_path, tool_result_event)?;

            let tool_output = serde_json::to_string(&result_value)?;
            messages.push(json!({
                "role": "tool",
                "tool_call_id": call.id,
                "content": tool_output
            }));
        }

        if turn == config.max_turns - 1 {
            eprintln!("Warning: agent reached max turns ({})", config.max_turns);
        }
    }

    Ok(messages)
}

pub fn with_datetime(ts: DateTime<Utc>, content: &str) -> String {
    let local = ts.with_timezone(&Local).format("%Y-%m-%dT%H:%M:%S");
    if content.is_empty() {
        format!("[{local}]")
    } else {
        format!("[{local}] {content}")
    }
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

pub fn tool_schemas() -> Vec<Value> {
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
                "description": "Create or update a knowledge document. The patch object controls what gets written. IMPORTANT: use body_append to write body content; without it the doc will have an empty body.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "patch": {
                            "type": "object",
                            "properties": {
                                "doc_path": { "type": "string", "description": "Path relative to vault root, e.g. summaries/sources/my-doc.md or knowledge/projects/foo.md" },
                                "title": { "type": "string", "description": "Document title (required for new docs)" },
                                "type": { "type": "string", "description": "Doc type: source_summary, project, person, preference, system (required for new docs)" },
                                "status": { "type": "string", "description": "e.g. active (default)" },
                                "confidence": { "type": "number", "description": "0.0-1.0 confidence score" },
                                "tags_add": { "type": "array", "items": { "type": "string" }, "description": "Tags to add" },
                                "tags_remove": { "type": "array", "items": { "type": "string" }, "description": "Tags to remove" },
                                "body_append": { "type": "string", "description": "Markdown content to write as the document body. THIS IS HOW YOU WRITE CONTENT. Without it the doc will be empty." },
                                "sources_add": { "type": "array", "items": { "type": "object", "properties": { "thread_id": { "type": "string" }, "event_ids": { "type": "array", "items": { "type": "string" } } } }, "description": "Source references (optional)" },
                                "supersedes_add": { "type": "array", "items": { "type": "string" }, "description": "IDs of docs this supersedes" },
                                "summary": { "type": "string", "description": "One-line description of the entire document (not the change). Max 150 chars. Required for new docs, updates the existing summary on existing docs." }
                            },
                            "required": ["doc_path"]
                        },
                        "author": { "type": "string" },
                        "reason": { "type": "string" },
                        "change_summary": { "type": "string", "description": "One-line description of what this specific mutation does. Max 150 chars. Example: 'Created project doc for JJ Gateway with tech stack and architecture'" },
                        "proposal_id": { "type": "string" },
                        "commit": { "type": "boolean" }
                    },
                    "required": ["patch", "author", "reason"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "knowledge_read",
                "description": "Read a knowledge document from the vault.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "doc_path": { "type": "string", "description": "Path relative to the vault root, e.g. knowledge/prefs/interaction.md" },
                        "include_body": { "type": "boolean", "description": "Include body content (default true)." },
                        "reason": { "type": "string" }
                    },
                    "required": ["doc_path", "reason"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "knowledge_search",
                "description": "Search knowledge documents for a substring match.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "mode": { "type": "string", "description": "auto|vector|substring (default auto)" },
                        "limit": { "type": "integer" },
                        "reason": { "type": "string" }
                    },
                    "required": ["query", "reason"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "knowledge_index",
                "description": "Build or rebuild the knowledge embedding index.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "reason": { "type": "string" }
                    },
                    "required": ["reason"]
                }
            }
        }),
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
            let path = create_thread(&vault_path, thread_id, date, None)?;
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

            let change_summary = args
                .get("change_summary")
                .and_then(|val| val.as_str())
                .unwrap_or("");
            let result = apply_patch(vault, patch, author, reason, proposal_id.clone(), change_summary)?;
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
        "knowledge_read" => {
            let doc_path = args
                .get("doc_path")
                .and_then(|val| val.as_str())
                .ok_or_else(|| anyhow!("doc_path required"))?;
            let include_body = args
                .get("include_body")
                .and_then(|val| val.as_bool())
                .unwrap_or(true);
            let full_path = vault.join(doc_path);
            if !full_path.starts_with(vault) {
                return Err(anyhow!("doc_path must be within vault"));
            }
            let doc = read_doc(&full_path)?;
            if include_body {
                Ok(json!({ "doc_path": doc_path, "front_matter": doc.front_matter, "body": doc.body }))
            } else {
                Ok(json!({ "doc_path": doc_path, "front_matter": doc.front_matter }))
            }
        }
        "knowledge_search" => {
            let query = args
                .get("query")
                .and_then(|val| val.as_str())
                .ok_or_else(|| anyhow!("query required"))?
                .to_lowercase();
            let limit = args.get("limit").and_then(|val| val.as_u64()).unwrap_or(10) as usize;
            let mode = args
                .get("mode")
                .and_then(|val| val.as_str())
                .unwrap_or("auto");

            if mode == "vector" || mode == "auto" {
                if let Ok(client) = EmbeddingClient::from_env() {
                    if let Ok(hits) = search_knowledge_index(vault, &client, &query, limit) {
                        let items: Vec<Value> = hits
                            .into_iter()
                            .map(|hit| {
                                json!({
                                    "doc_path": hit.doc_path,
                                    "chunk_id": hit.chunk_id,
                                    "score": hit.score,
                                    "excerpt": hit.excerpt
                                })
                            })
                            .collect();
                        return Ok(json!({ "mode": "vector", "count": items.len(), "matches": items }));
                    }
                }
                if mode == "vector" {
                    return Err(anyhow!("vector search unavailable (missing index or provider)"));
                }
            }

            let root = vault.join("knowledge");
            let mut matches = Vec::new();
            for path in walk_markdown(&root)? {
                let content = fs::read_to_string(&path)?;
                let haystack = content.to_lowercase();
                if let Some(idx) = haystack.find(&query) {
                    let excerpt = excerpt_at(&content, idx, 80);
                    let rel = path.strip_prefix(vault).unwrap_or(&path).to_string_lossy().to_string();
                    matches.push(json!({
                        "doc_path": rel,
                        "excerpt": excerpt
                    }));
                    if matches.len() >= limit {
                        break;
                    }
                }
            }
            Ok(json!({ "mode": "substring", "count": matches.len(), "matches": matches }))
        }
        "knowledge_index" => {
            let client = EmbeddingClient::from_env()?;
            let stats = build_knowledge_index(vault, &client)?;
            Ok(json!({
                "doc_count": stats.doc_count,
                "chunk_count": stats.chunk_count,
                "index_path": stats.index_path,
                "provider": stats.provider,
                "model": stats.model
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

fn walk_markdown(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if !root.exists() {
        return Ok(files);
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                files.push(path);
            }
        }
    }
    Ok(files)
}

fn excerpt_at(content: &str, idx: usize, radius: usize) -> String {
    let mut start = idx.saturating_sub(radius);
    let mut end = usize::min(content.len(), idx + radius);
    while start < content.len() && !content.is_char_boundary(start) {
        start += 1;
    }
    while end > start && !content.is_char_boundary(end) {
        end -= 1;
    }
    content[start..end].replace('\n', " ")
}
