use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, Utc};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

/// Events emitted during an agent run for live streaming to clients.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Tool call started — includes the arguments so callers can show what's happening
    ToolCallStart {
        tool_name: String,
        arguments: Value,
    },
    /// Tool call finished — includes the result
    ToolCallResult {
        tool_name: String,
        result: Value,
    },
    /// Final assistant message content
    FinalContent { content: String },
    /// Deep think background task completed
    DeepThinkComplete { monologue: String },
}

pub struct AgentConfig {
    pub vault_path: PathBuf,
    pub thread_path: PathBuf,
    pub max_turns: usize,
    pub allow_commit: bool,
    /// If set, only expose these tools (by name). If None, expose all.
    pub tool_filter: Option<Vec<String>>,
    /// Optional channel for streaming events to gateway clients.
    pub event_sink: Option<std::sync::mpsc::Sender<AgentEvent>>,
    /// Flag indicating whether a deep_think background task is running.
    pub deep_think_running: Arc<AtomicBool>,
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
            if let Some(ref sink) = config.event_sink {
                // Streaming to gateway — don't print locally
                let _ = sink.send(AgentEvent::FinalContent { content: content.clone() });
            } else if !content.is_empty() {
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
            if let Some(ref sink) = config.event_sink {
                let _ = sink.send(AgentEvent::ToolCallStart {
                    tool_name: call.name.clone(),
                    arguments: call.arguments.clone(),
                });
            } else {
                // Debug: show tool calls in direct/CLI mode
                let detail = call.arguments.get("query").and_then(|v| v.as_str())
                    .or_else(|| call.arguments.get("doc_path").and_then(|v| v.as_str()))
                    .or_else(|| call.arguments.get("prompt").and_then(|v| v.as_str()).map(|s| &s[..s.len().min(80)]));
                match detail {
                    Some(d) => eprintln!("[{}: {}]", call.name, d),
                    None => eprintln!("[{}]", call.name),
                }
            }
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
                config,
            );
            let result_value = match result {
                Ok(data) => json!({"status": "ok", "data": data}),
                Err(err) => json!({"status": "error", "error": err.to_string()}),
            };

            if let Some(ref sink) = config.event_sink {
                let _ = sink.send(AgentEvent::ToolCallResult {
                    tool_name: call.name.clone(),
                    result: result_value.clone(),
                });
            }

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
        json!({
            "type": "function",
            "function": {
                "name": "draw",
                "description": "Draw an image onto the user's canvas using rcast. Accepts a URL or local file path. Use this to show images, diagrams, or visual content to the user.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "source": { "type": "string", "description": "URL or local file path of the image to display." },
                        "overlay": { "type": "boolean", "description": "If true, overlay on existing canvas instead of clearing." },
                        "reason": { "type": "string" }
                    },
                    "required": ["source", "reason"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "generate_image",
                "description": "Generate an image using flux2 and store it in the vault media directory. Path should use descriptive folders for uniqueness (e.g. 'diagrams/arch-v2.png', 'food/pepperoni-pizza.png'). Returns error if path already exists.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "prompt": { "type": "string", "description": "Text description of the image to generate." },
                        "path": { "type": "string", "description": "Relative path within media/ (e.g. 'food/pizza.png'). Must end with .png. Intermediate directories are created automatically." },
                        "reason": { "type": "string" }
                    },
                    "required": ["prompt", "path", "reason"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "deep_think",
                "description": "Trigger deep thinking. Calls a slower model to reflect on the conversation, search knowledge, and produce inner monologue. The result is appended to the thread as internal context visible on your next turn. Use when the conversation would benefit from deeper analysis, pattern recognition, or knowledge retrieval. The inner monologue will NOT be shown to the user.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "string",
                            "description": "What to think about. If omitted, reflects on the overall conversation."
                        },
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
    config: &AgentConfig,
) -> Result<Value> {
    let vault = &config.vault_path;
    let thread_path = &config.thread_path;
    let allow_commit = config.allow_commit;
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
        "draw" => {
            let source = args
                .get("source")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("draw requires 'source' (URL or file path)"))?;

            // If source looks like a URL, download to a temp file first.
            let file_path = if source.starts_with("http://") || source.starts_with("https://") {
                let resp = reqwest::blocking::get(source)?;
                if !resp.status().is_success() {
                    return Err(anyhow!("failed to download {source}: HTTP {}", resp.status()));
                }
                let bytes = resp.bytes()?;
                let ext = source.rsplit('.').next().unwrap_or("png");
                let tmp = std::env::temp_dir().join(format!("jj_draw.{ext}"));
                fs::write(&tmp, &bytes)?;
                tmp
            } else if Path::new(source).is_absolute() {
                PathBuf::from(source)
            } else {
                // Resolve relative paths against the vault root
                vault.join(source)
            };

            let mut cmd = std::process::Command::new("rcast");
            cmd.arg("draw").arg(&file_path);

            if let Some(true) = args.get("overlay").and_then(|v| v.as_bool()) {
                cmd.arg("--overlay");
            }

            let output = cmd.output()?;
            if output.status.success() {
                Ok(json!({ "drawn": source }))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow!("rcast draw failed: {stderr}"))
            }
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
        "generate_image" => {
            let prompt = args
                .get("prompt")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("generate_image requires 'prompt'"))?;
            let rel_path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("generate_image requires 'path'"))?;

            // Validate path: no .., no leading /, no backslash, must end with .png
            if rel_path.contains("..")
                || rel_path.starts_with('/')
                || rel_path.contains('\\')
                || rel_path.contains('\0')
                || !rel_path.ends_with(".png")
            {
                return Err(anyhow!(
                    "invalid path: must be relative, no '..', and end with .png"
                ));
            }
            // Additional check: only allow alphanumeric, dash, underscore, slash, dot
            if !rel_path
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '/' | '.'))
            {
                return Err(anyhow!(
                    "invalid path: only alphanumeric, dash, underscore, slash allowed"
                ));
            }

            let media_dir = vault.join("media");
            let full_path = media_dir.join(rel_path);

            // Verify resolved path is within media/
            if let Ok(canonical_parent) = full_path.parent().unwrap_or(&media_dir).canonicalize() {
                let canonical_media = media_dir.canonicalize().unwrap_or_else(|_| media_dir.clone());
                if !canonical_parent.starts_with(&canonical_media) {
                    return Err(anyhow!("path escapes media directory"));
                }
            }

            // Check existence
            if full_path.exists() {
                return Err(anyhow!("exists: media/{rel_path}"));
            }

            // Create parent directories
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Run flux2
            let output = std::process::Command::new("flux2")
                .arg(prompt)
                .arg(&full_path)
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    if full_path.exists() {
                        Ok(json!({ "path": format!("media/{rel_path}") }))
                    } else {
                        Err(anyhow!("flux2 completed but output file not found"))
                    }
                }
                Ok(out) => {
                    // Clean up partial file
                    let _ = fs::remove_file(&full_path);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    Err(anyhow!("flux2 failed: {stderr}"))
                }
                Err(e) => {
                    let _ = fs::remove_file(&full_path);
                    if e.kind() == std::io::ErrorKind::NotFound {
                        Err(anyhow!("flux2 not found"))
                    } else {
                        Err(anyhow!("flux2 error: {e}"))
                    }
                }
            }
        }
        "deep_think" => {
            let prompt = args.get("prompt").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let reason = args
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("deep_think")
                .to_string();

            // Check if already running
            if config.deep_think_running.compare_exchange(
                false, true, Ordering::SeqCst, Ordering::SeqCst,
            ).is_err() {
                return Ok(json!({ "status": "already_running" }));
            }

            let vault_path = vault.to_path_buf();
            let thread_path = thread_path.to_path_buf();
            let running = Arc::clone(&config.deep_think_running);
            let event_sink = config.event_sink.clone();

            std::thread::spawn(move || {
                let result = deep_think_background(
                    &vault_path,
                    &thread_path,
                    &prompt,
                    &reason,
                );
                match result {
                    Ok(monologue) => {
                        if let Some(ref sink) = event_sink {
                            let _ = sink.send(AgentEvent::DeepThinkComplete { monologue });
                        }
                    }
                    Err(e) => {
                        eprintln!("deep_think background error: {e}");
                    }
                }
                running.store(false, Ordering::SeqCst);
            });

            Ok(json!({ "status": "queued" }))
        }
        _ => Err(anyhow!("unknown tool: {name}")),
    }
}

/// Run deep_think work in a background thread: read transcript, call slow model
/// with knowledge tool access, append InnerMonologue to thread.
fn deep_think_background(
    vault_path: &Path,
    thread_path: &Path,
    prompt: &str,
    reason: &str,
) -> Result<String> {
    // 1. Read recent thread events for context
    let lines = read_thread(thread_path, None, Some(50))?;
    let mut transcript = String::new();
    for line in &lines {
        if let Ok(event) = serde_json::from_str::<crate::thread_store::ThreadEvent>(line) {
            let role_label = match event.role {
                Role::User => "User",
                Role::Assistant => "Assistant",
                Role::System => "System",
                Role::Tool => "Tool",
            };
            if let Some(content) = &event.content {
                let text = match content {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                if !text.is_empty() {
                    transcript.push_str(&format!("{role_label}: {text}\n"));
                }
            }
        }
    }

    // 2. Build inner-voice system prompt
    let system_msg = "You are the inner voice of an AI assistant named JJ. \
        You are thinking privately — nothing you say will be shown to the user. \
        Reflect on the conversation so far. Note patterns, form hypotheses, \
        identify what you know vs don't know, and suggest what to explore or say next. \
        You have access to knowledge_search and knowledge_read tools to look up information. \
        Be concise but thorough. Write in first person as inner thoughts.";

    let mut deep_messages = vec![json!({"role": "system", "content": system_msg})];
    if !transcript.is_empty() {
        deep_messages.push(json!({"role": "user", "content": format!("Here is the conversation so far:\n\n{transcript}")}));
    }
    if !prompt.is_empty() {
        deep_messages.push(json!({"role": "user", "content": format!("Focus your thinking on: {prompt}")}));
    } else {
        deep_messages.push(json!({"role": "user", "content": "Reflect on the overall conversation. What patterns do you see? What should I consider next?"}));
    }

    // 3. Create client with deep think model
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow!("OPENAI_API_KEY not set"))?;
    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com".to_string());
    let deep_model = std::env::var("OPENAI_DEEP_THINK_MODEL")
        .or_else(|_| std::env::var("OPENAI_MODEL"))
        .unwrap_or_else(|_| "gpt-5.2-2025-12-11".to_string());

    let client = OpenAIClient::new(api_key, base_url, deep_model);

    // 4. Run agent loop with tool filter for knowledge tools, max 3 turns
    let inner_config = AgentConfig {
        vault_path: vault_path.to_path_buf(),
        thread_path: thread_path.to_path_buf(),
        max_turns: 3,
        allow_commit: false,
        tool_filter: Some(vec![
            "knowledge_search".into(),
            "knowledge_read".into(),
        ]),
        event_sink: None,
        deep_think_running: Arc::new(AtomicBool::new(false)),
    };

    let final_messages = run_agent_loop(&inner_config, deep_messages, &client)?;

    // 5. Extract final content from returned messages
    let monologue = final_messages
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

    // 6. Append InnerMonologue event to thread
    let event = build_event(
        None,
        EventType::InnerMonologue,
        Role::System,
        Some(Value::String(monologue.clone())),
        Some("deep_think".to_string()),
        None,
        None,
        Some(reason.to_string()),
    );
    append_event(thread_path, event)?;

    Ok(monologue)
}

fn parse_event_type(value: &str) -> Result<EventType> {
    match value {
        "user_message" => Ok(EventType::UserMessage),
        "assistant_message" => Ok(EventType::AssistantMessage),
        "tool_call" => Ok(EventType::ToolCall),
        "tool_result" => Ok(EventType::ToolResult),
        "system_note" => Ok(EventType::SystemNote),
        "attachment_added" => Ok(EventType::AttachmentAdded),
        "inner_monologue" => Ok(EventType::InnerMonologue),
        "title_generated" => Ok(EventType::TitleGenerated),
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
