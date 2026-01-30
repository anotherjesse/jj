use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    UserMessage,
    AssistantMessage,
    ToolCall,
    ToolResult,
    SystemNote,
    AttachmentAdded,
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    Tool,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadEvent {
    pub thread_id: String,
    pub event_id: String,
    pub ts: DateTime<Utc>,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_args: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

pub fn create_thread(vault_path: &Path, thread_id: Option<String>, date: Option<NaiveDate>) -> Result<PathBuf> {
    let thread_id = thread_id.unwrap_or_else(|| format!("thr_{}", Ulid::new()));
    let date = date.unwrap_or_else(|| Utc::now().date_naive());
    let dir = vault_path
        .join("threads")
        .join(format!("{:04}", date.year()))
        .join(format!("{:02}", date.month()))
        .join(format!("{:02}", date.day()));
    fs::create_dir_all(&dir).with_context(|| format!("create thread dir {}", dir.display()))?;
    let path = dir.join(format!("{thread_id}.jsonl"));
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .with_context(|| format!("create thread {}", path.display()))?;
    Ok(path)
}

pub fn append_event(thread_path: &Path, mut event: ThreadEvent) -> Result<()> {
    if event.thread_id.is_empty() {
        event.thread_id = derive_thread_id(thread_path)?
            .ok_or_else(|| anyhow!("thread_id missing and could not be derived"))?;
    }
    validate_event(&event)?;
    let serialized = serde_json::to_string(&event)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(thread_path)
        .with_context(|| format!("open thread {}", thread_path.display()))?;
    file.write_all(serialized.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

pub fn read_thread(thread_path: &Path, offset: Option<usize>, limit: Option<usize>) -> Result<Vec<String>> {
    let file = File::open(thread_path)
        .with_context(|| format!("open thread {}", thread_path.display()))?;
    let reader = BufReader::new(file);
    let offset = offset.unwrap_or(0);
    let mut lines = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        if idx < offset {
            continue;
        }
        lines.push(line);
        if let Some(limit) = limit {
            if lines.len() >= limit {
                break;
            }
        }
    }
    Ok(lines)
}

pub fn build_event(
    thread_id: Option<String>,
    event_type: EventType,
    role: Role,
    content: Option<Value>,
    tool_name: Option<String>,
    tool_args: Option<Value>,
    tool_result: Option<Value>,
    reason: Option<String>,
) -> ThreadEvent {
    ThreadEvent {
        thread_id: thread_id.unwrap_or_default(),
        event_id: format!("evt_{}", Ulid::new()),
        ts: Utc::now(),
        event_type,
        role,
        content,
        tool_name,
        tool_args,
        tool_result,
        reason,
    }
}

fn validate_event(event: &ThreadEvent) -> Result<()> {
    match event.event_type {
        EventType::ToolCall => {
            if event.tool_name.is_none() {
                return Err(anyhow!("tool_call requires tool_name"));
            }
            if event.tool_args.is_none() {
                return Err(anyhow!("tool_call requires tool_args"));
            }
            if event.reason.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                return Err(anyhow!("tool_call requires reason"));
            }
        }
        EventType::ToolResult => {
            if event.tool_name.is_none() {
                return Err(anyhow!("tool_result requires tool_name"));
            }
            if event.tool_result.is_none() {
                return Err(anyhow!("tool_result requires tool_result"));
            }
        }
        _ => {
            if event.content.is_none() {
                return Err(anyhow!("event requires content"));
            }
        }
    }
    Ok(())
}

fn derive_thread_id(thread_path: &Path) -> Result<Option<String>> {
    let file_name = thread_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if file_name.starts_with("thr_") {
        return Ok(Some(file_name.to_string()));
    }
    Ok(None)
}
