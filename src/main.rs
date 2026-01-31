mod agent;
mod audit;
mod embedding_index;
mod embeddings;
mod git_utils;
mod ingest;
mod knowledge;
mod openai;
mod repl;
mod thread_store;
mod vault;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::audit::append_ledger;
use crate::git_utils::git_commit;
use crate::ingest::{run_ingest, IngestOptions};
use crate::knowledge::{apply_patch, KnowledgePatch};
use crate::repl::{run_repl, ReplOptions};
use crate::thread_store::{append_event, build_event, create_thread, read_thread, EventType, Role};
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
                let path = create_thread(&vault, thread_id, date)?;
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
                let result = apply_patch(&vault, patch, &author, &reason, proposal_id.clone())?;
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
            model,
            allow_commit,
            history,
        } => {
            run_repl(ReplOptions {
                vault,
                thread,
                model,
                allow_commit,
                history,
            })?;
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

// ledger + git helpers live in modules
