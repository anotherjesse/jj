mod thread_store;
mod vault;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::path::PathBuf;

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
    Vault {
        #[command(subcommand)]
        command: VaultCommand,
    },
    Thread {
        #[command(subcommand)]
        command: ThreadCommand,
    },
}

#[derive(Subcommand)]
enum VaultCommand {
    Init {
        #[arg(long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ThreadCommand {
    Create {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        thread_id: Option<String>,
        #[arg(long)]
        date: Option<String>,
    },
    Append {
        #[arg(long)]
        thread: PathBuf,
        #[arg(long)]
        event_type: EventType,
        #[arg(long)]
        role: Role,
        #[arg(long)]
        thread_id: Option<String>,
        #[arg(long)]
        content: Option<String>,
        #[arg(long)]
        content_json: Option<String>,
        #[arg(long)]
        tool_name: Option<String>,
        #[arg(long)]
        tool_args: Option<String>,
        #[arg(long)]
        tool_result: Option<String>,
        #[arg(long)]
        reason: Option<String>,
    },
    Read {
        #[arg(long)]
        thread: PathBuf,
        #[arg(long)]
        offset: Option<usize>,
        #[arg(long)]
        limit: Option<usize>,
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
    }
    Ok(())
}

fn parse_json(value: &str, label: &str) -> Result<Value> {
    serde_json::from_str(value).with_context(|| format!("parse {label} JSON"))
}

// ledger + git commit support added in later milestone
