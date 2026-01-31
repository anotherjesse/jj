mod agent;
mod audit;
mod embedding_index;
mod embeddings;
mod git_utils;
mod ingest;
mod knowledge;
mod openai;
mod chat;
mod thread_store;
mod vault;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;

use crate::audit::append_ledger;
use crate::git_utils::git_commit;
use crate::ingest::{run_ingest, IngestOptions};
use crate::knowledge::{apply_patch, KnowledgePatch};
use crate::chat::{run_chat, ChatOptions};
use crate::thread_store::{append_event, build_event, create_thread, list_threads, read_thread, EventType, Role};
use crate::vault::{init_vault, resolve_vault};

#[derive(Parser)]
#[command(name = "jay")]
#[command(about = "JJ memory-first agent vault tools", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize or manage a vault
    Vault {
        #[command(subcommand)]
        command: VaultCommand,
    },
    /// Create, append to, or read conversation threads
    Thread {
        #[command(subcommand)]
        command: ThreadCommand,
    },
    /// Apply knowledge patches to the vault
    Knowledge {
        #[command(subcommand)]
        command: KnowledgeCommand,
    },
    /// Build the embedding index for knowledge search
    Index {
        /// Vault path (default: jj_vault)
        #[arg(long)]
        vault: Option<PathBuf>,
    },
    /// Start an interactive chat session with the agent
    Chat {
        /// Vault path (default: jj_vault)
        #[arg(long)]
        vault: Option<PathBuf>,
        /// Resume an existing thread
        #[arg(long)]
        thread: Option<PathBuf>,
        /// Resume the most recent thread
        #[arg(long, default_value_t = false)]
        last: bool,
        /// Override the LLM model
        #[arg(long)]
        model: Option<String>,
        /// Allow the agent to commit changes to git
        #[arg(long, default_value_t = false)]
        allow_commit: bool,
        /// Number of thread history events to load
        #[arg(long, default_value_t = 50)]
        history: usize,
    },
    /// Backfill one-line summaries for knowledge docs missing them
    BackfillSummaries {
        /// Vault path (default: jj_vault)
        #[arg(long)]
        vault: Option<PathBuf>,
        /// Override the LLM model
        #[arg(long)]
        model: Option<String>,
        /// Dry run: show what would be updated without writing
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Ingest a markdown document into the vault as a source
    Ingest {
        /// Path to the markdown file to ingest
        file: PathBuf,
        /// Vault path (default: jj_vault)
        #[arg(long)]
        vault: Option<PathBuf>,
        /// Provenance string (e.g., "chatgpt-export", "notion", "manual")
        #[arg(long)]
        source: Option<String>,
        /// Comma-separated tags
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        /// Override document title (default: derived from filename)
        #[arg(long)]
        title: Option<String>,
        /// Override the LLM model
        #[arg(long)]
        model: Option<String>,
    },
}

#[derive(Subcommand)]
enum VaultCommand {
    /// Create a new vault directory structure
    Init {
        /// Where to create the vault (default: jj_vault)
        #[arg(long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ThreadCommand {
    /// Create a new conversation thread
    Create {
        /// Vault path (default: jj_vault)
        #[arg(long)]
        vault: Option<PathBuf>,
        /// Custom thread ID (default: auto-generated)
        #[arg(long)]
        thread_id: Option<String>,
        /// Date override as YYYY-MM-DD
        #[arg(long)]
        date: Option<String>,
    },
    /// Append an event to an existing thread
    Append {
        /// Path to the thread JSONL file
        #[arg(long)]
        thread: PathBuf,
        /// Event type (e.g., message, tool_call, tool_result)
        #[arg(long)]
        event_type: EventType,
        /// Role (e.g., user, assistant, system, tool)
        #[arg(long)]
        role: Role,
        #[arg(long)]
        thread_id: Option<String>,
        #[arg(long)]
        content: Option<String>,
        /// Content as raw JSON
        #[arg(long)]
        content_json: Option<String>,
        #[arg(long)]
        tool_name: Option<String>,
        /// Tool arguments as JSON
        #[arg(long)]
        tool_args: Option<String>,
        /// Tool result as JSON
        #[arg(long)]
        tool_result: Option<String>,
        #[arg(long)]
        reason: Option<String>,
    },
    /// List recent threads with previews
    List {
        /// Vault path (default: jj_vault)
        #[arg(long)]
        vault: Option<PathBuf>,
        /// Maximum number of threads to show
        #[arg(long)]
        limit: Option<usize>,
        /// Filter by thread kind (default: chat). Use "all" to show everything.
        #[arg(long, default_value = "chat")]
        kind: String,
    },
    /// Read events from a thread
    Read {
        /// Path to the thread JSONL file
        #[arg(long)]
        thread: PathBuf,
        /// Skip this many events from the start
        #[arg(long)]
        offset: Option<usize>,
        /// Maximum number of events to return
        #[arg(long)]
        limit: Option<usize>,
    },
}

#[derive(Subcommand)]
enum KnowledgeCommand {
    /// Apply a JSON knowledge patch to the vault
    Apply {
        /// Vault path (default: jj_vault)
        #[arg(long)]
        vault: Option<PathBuf>,
        /// Path to the patch JSON file
        #[arg(long)]
        patch: PathBuf,
        /// Author attribution for the change
        #[arg(long)]
        author: String,
        /// Human-readable reason for the change
        #[arg(long)]
        reason: String,
        /// Link to the originating proposal
        #[arg(long)]
        proposal_id: Option<String>,
        /// Commit the change to git after applying
        #[arg(long, default_value_t = false)]
        commit: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Vault { command } => match command {
            VaultCommand::Init { path } => {
                let vault = resolve_vault(path);
                init_vault(&vault)?;
                println!("Initialized vault at {}", vault.display());
            }
        },
        Commands::Thread { command } => match command {
            ThreadCommand::Create { vault, thread_id, date } => {
                let vault = resolve_vault(vault);
                let date = match date {
                    Some(value) => Some(chrono::NaiveDate::parse_from_str(&value, "%Y-%m-%d")?),
                    None => None,
                };
                let path = create_thread(&vault, thread_id, date, None)?;
                println!("{}", path.display());
            }
            ThreadCommand::Append {
                thread,
                event_type,
                role,
                thread_id,
                content,
                content_json,
                tool_name,
                tool_args,
                tool_result,
                reason,
            } => {
                let content_value = match (content_json, content) {
                    (Some(json), _) => Some(parse_json(&json, "content_json")?),
                    (None, Some(text)) => Some(Value::String(text)),
                    (None, None) => None,
                };
                let tool_args_value = match tool_args {
                    Some(json) => Some(parse_json(&json, "tool_args")?),
                    None => None,
                };
                let tool_result_value = match tool_result {
                    Some(json) => Some(parse_json(&json, "tool_result")?),
                    None => None,
                };
                let event = build_event(
                    thread_id,
                    event_type,
                    role,
                    content_value,
                    tool_name,
                    tool_args_value,
                    tool_result_value,
                    reason,
                );
                append_event(&thread, event)?;
            }
            ThreadCommand::List { vault, limit, kind } => {
                let vault = resolve_vault(vault);
                let kind_filter = if kind == "all" { None } else { Some(kind.as_str()) };
                let summaries = list_threads(&vault, limit, kind_filter)?;
                if summaries.is_empty() {
                    println!("No threads found.");
                } else {
                    let show_kind = kind == "all";
                    for s in &summaries {
                        let first = truncate_preview(s.first_user_line.as_deref().unwrap_or("(empty)"), 60);
                        let last = truncate_preview(s.last_line.as_deref().unwrap_or("(empty)"), 60);
                        let time: chrono::DateTime<chrono::Utc> = s.modified.into();
                        let label = if show_kind {
                            let agent = s.agent.as_deref().unwrap_or(&s.kind);
                            format!("  [{}]", agent)
                        } else {
                            String::new()
                        };
                        println!("{}  {}{label}  {:?}  →  {:?}", s.thread_id, time.format("%Y-%m-%d %H:%M"), first, last);
                    }
                }
            }
            ThreadCommand::Read { thread, offset, limit } => {
                let lines = read_thread(&thread, offset, limit)?;
                for line in lines {
                    println!("{line}");
                }
            }
        },
        Commands::Knowledge { command } => match command {
            KnowledgeCommand::Apply {
                vault,
                patch,
                author,
                reason,
                proposal_id,
                commit,
            } => {
                let vault = resolve_vault(vault);
                let patch_content = fs::read_to_string(&patch)
                    .with_context(|| format!("read patch {}", patch.display()))?;
                let patch: KnowledgePatch = serde_json::from_str(&patch_content)
                    .with_context(|| "parse patch json")?;
                let result = apply_patch(&vault, patch, &author, &reason, proposal_id.clone(), &reason)?;
                let ledger_path = vault.join("audit/ledger.jsonl");
                append_ledger(&ledger_path, &result.ledger_entry)?;
                if commit {
                    let repo_root = PathBuf::from(".");
                    let message = match &proposal_id {
                        Some(id) => format!("{id}: {reason}"),
                        None => format!("memory: {reason}"),
                    };
                    git_commit(&repo_root, &[result.doc_path, ledger_path], &message)?;
                }
            }
        },
        Commands::Index { vault } => {
            use crate::embedding_index::build_knowledge_index;
            use crate::embeddings::EmbeddingClient;
            let vault = resolve_vault(vault);
            let client = EmbeddingClient::from_env()?;
            let stats = build_knowledge_index(&vault, &client)?;
            println!(
                "Indexed {} docs / {} chunks ({} {})",
                stats.doc_count, stats.chunk_count, stats.provider, stats.model
            );
            println!("Index: {}", stats.index_path.display());
        }
        Commands::Chat {
            vault,
            thread,
            last,
            model,
            allow_commit,
            history,
        } => {
            let thread = if last {
                let v = resolve_vault(vault.clone());
                let summaries = list_threads(&v, Some(1), Some("chat"))?;
                let s = summaries.into_iter().next()
                    .ok_or_else(|| anyhow::anyhow!("no threads found in vault"))?;
                Some(s.path)
            } else {
                thread
            };
            run_chat(ChatOptions {
                vault,
                thread,
                model,
                allow_commit,
                history,
            })?;
        }
        Commands::BackfillSummaries { vault, model, dry_run } => {
            use crate::knowledge::read_doc;
            use crate::openai::OpenAIClient;

            dotenvy::dotenv().ok();
            let vault = resolve_vault(vault);
            let api_key = env::var("OPENAI_API_KEY").context("OPENAI_API_KEY is not set")?;
            let base_url = env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".to_string());
            let model = model
                .or_else(|| env::var("OPENAI_MODEL").ok())
                .unwrap_or_else(|| "gpt-5.2-2025-12-11".to_string());
            let client = OpenAIClient::new(api_key, base_url, model);

            let root = vault.join("knowledge");
            let mut stack = vec![root.clone()];
            let mut docs_to_update: Vec<PathBuf> = Vec::new();
            while let Some(dir) = stack.pop() {
                if !dir.exists() { continue; }
                for entry in fs::read_dir(&dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                        if let Ok(doc) = read_doc(&path) {
                            if doc.front_matter.summary.is_empty() {
                                docs_to_update.push(path);
                            }
                        }
                    }
                }
            }

            println!("Found {} docs without summaries.", docs_to_update.len());

            for path in &docs_to_update {
                let doc = read_doc(path)?;
                let body_excerpt: String = doc.body.chars().take(500).collect();
                let prompt = format!(
                    "Write a single-line summary (max 150 chars) describing this entire document. Be specific and concrete. Return ONLY the summary line, nothing else.\n\nTitle: {}\nType: {}\nContent:\n{}",
                    doc.front_matter.title, doc.front_matter.doc_type, body_excerpt
                );
                let messages = vec![
                    serde_json::json!({"role": "user", "content": prompt}),
                ];
                let response = client.chat(&messages, &[])?;
                let summary = response.content.unwrap_or_default().trim().to_string();

                let rel = path.strip_prefix(&vault).unwrap_or(path).to_string_lossy();
                if dry_run {
                    println!("[dry-run] {} → {}", rel, summary);
                } else {
                    // Update frontmatter and write back
                    let mut new_fm = doc.front_matter.clone();
                    new_fm.summary = summary.clone();
                    let rendered = crate::knowledge::render_markdown_pub(&new_fm, &doc.body)?;
                    fs::write(path, rendered.as_bytes())?;
                    println!("{} → {}", rel, summary);
                }
            }

            if !dry_run && !docs_to_update.is_empty() {
                println!("\nDone. Review changes and commit when ready.");
            }
        }
        Commands::Ingest {
            file,
            vault,
            source,
            tags,
            title,
            model,
        } => {
            let result = run_ingest(IngestOptions {
                vault,
                file,
                source,
                tags,
                title,
                model,
            })?;
            println!("\nIngested: {}", result.source_path.display());
            println!("Summary:  {}", result.summary_path.display());
            println!("Thread:   {}", result.thread_path.display());
            println!("Proposals: {}", result.proposal_count);
        }
    }
    Ok(())
}

fn parse_json(value: &str, label: &str) -> Result<Value> {
    serde_json::from_str(value).with_context(|| format!("parse {label} JSON"))
}

fn truncate_preview(s: &str, max: usize) -> String {
    let s = s.replace('\n', " ");
    if s.len() <= max {
        s
    } else {
        format!("{}…", &s[..max])
    }
}

// ledger + git helpers live in modules
