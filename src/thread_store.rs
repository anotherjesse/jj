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
    InnerMonologue,
    TitleGenerated,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadHeader {
    pub jj_thread: bool,
    pub thread_id: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct ThreadMeta {
    pub kind: String,
    pub agent: Option<String>,
    pub model: Option<String>,
}

pub fn create_thread(
    vault_path: &Path,
    thread_id: Option<String>,
    date: Option<NaiveDate>,
    meta: Option<ThreadMeta>,
) -> Result<PathBuf> {
    let thread_id = thread_id.unwrap_or_else(|| format!("thr_{}", Ulid::new()));
    let date = date.unwrap_or_else(|| Utc::now().date_naive());
    let dir = vault_path
        .join("threads")
        .join(format!("{:04}", date.year()))
        .join(format!("{:02}", date.month()))
        .join(format!("{:02}", date.day()));
    fs::create_dir_all(&dir).with_context(|| format!("create thread dir {}", dir.display()))?;
    let path = dir.join(format!("{thread_id}.jsonl"));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .with_context(|| format!("create thread {}", path.display()))?;
    let meta = meta.unwrap_or(ThreadMeta {
        kind: "chat".into(),
        agent: None,
        model: None,
    });
    let header = ThreadHeader {
        jj_thread: true,
        thread_id: thread_id.clone(),
        kind: meta.kind,
        agent: meta.agent,
        model: meta.model,
        created_at: Utc::now(),
    };
    let header_json = serde_json::to_string(&header)?;
    file.write_all(header_json.as_bytes())?;
    file.write_all(b"\n")?;
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

pub fn read_header(thread_path: &Path) -> Result<Option<ThreadHeader>> {
    let file = File::open(thread_path)
        .with_context(|| format!("open thread {}", thread_path.display()))?;
    let reader = BufReader::new(file);
    if let Some(Ok(first_line)) = reader.lines().next() {
        if let Ok(header) = serde_json::from_str::<ThreadHeader>(&first_line) {
            if header.jj_thread {
                return Ok(Some(header));
            }
        }
    }
    Ok(None)
}

pub fn read_thread(thread_path: &Path, offset: Option<usize>, limit: Option<usize>) -> Result<Vec<String>> {
    let file = File::open(thread_path)
        .with_context(|| format!("open thread {}", thread_path.display()))?;
    let reader = BufReader::new(file);
    let offset = offset.unwrap_or(0);
    let mut lines = Vec::new();
    let mut event_idx = 0usize;
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        // Skip header line
        if idx == 0 && serde_json::from_str::<ThreadHeader>(&line).map(|h| h.jj_thread).unwrap_or(false) {
            continue;
        }
        if event_idx < offset {
            event_idx += 1;
            continue;
        }
        event_idx += 1;
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

#[derive(Debug, Clone)]
pub struct ThreadSummary {
    pub path: PathBuf,
    pub thread_id: String,
    pub kind: String,
    pub agent: Option<String>,
    pub modified: std::time::SystemTime,
    pub first_user_line: Option<String>,
    pub last_line: Option<String>,
}

pub fn list_threads(vault_path: &Path, limit: Option<usize>, kind_filter: Option<&str>) -> Result<Vec<ThreadSummary>> {
    let threads_dir = vault_path.join("threads");
    if !threads_dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
    collect_thread_files(&threads_dir, &mut entries)?;
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    let limit = limit.unwrap_or(10);
    let mut summaries = Vec::new();
    for (path, mtime) in entries {
        if summaries.len() >= limit {
            break;
        }
        let header = read_header(&path).ok().flatten();
        let kind = header.as_ref().map(|h| h.kind.clone()).unwrap_or_else(|| "chat".into());
        let agent = header.as_ref().and_then(|h| h.agent.clone());
        if let Some(filter) = kind_filter {
            if kind != filter {
                continue;
            }
        }
        let thread_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let lines = read_thread(&path, None, None).unwrap_or_default();
        let mut first_user_line = None;
        let mut last_line = None;
        for line in &lines {
            if let Ok(event) = serde_json::from_str::<ThreadEvent>(line) {
                let content_str = event.content.as_ref().and_then(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    other => Some(other.to_string()),
                });
                if first_user_line.is_none() && matches!(event.event_type, EventType::UserMessage) {
                    first_user_line = content_str.clone();
                }
                if content_str.is_some() {
                    last_line = content_str;
                }
            }
        }
        summaries.push(ThreadSummary {
            path,
            thread_id,
            kind,
            agent,
            modified: mtime,
            first_user_line,
            last_line,
        });
    }
    Ok(summaries)
}

fn collect_thread_files(dir: &Path, out: &mut Vec<(PathBuf, std::time::SystemTime)>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if ft.is_dir() {
            collect_thread_files(&entry.path(), out)?;
        } else if ft.is_file() {
            let name = entry.file_name();
            let name = name.to_str().unwrap_or("");
            if name.starts_with("thr_") && name.ends_with(".jsonl") {
                let mtime = entry.metadata()?.modified()?;
                out.push((entry.path(), mtime));
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
