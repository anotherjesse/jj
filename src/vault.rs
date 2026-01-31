use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn init_vault(path: &Path) -> Result<()> {
    let dirs = [
        path.to_path_buf(),
        path.join("config"),
        path.join("prompts"),
        path.join("threads"),
        path.join("summaries/threads"),
        path.join("summaries/daily"),
        path.join("summaries/weekly"),
        path.join("knowledge/people"),
        path.join("knowledge/projects"),
        path.join("knowledge/prefs"),
        path.join("knowledge/system"),
        path.join("index"),
        path.join("inbox/proposals"),
        path.join("inbox/questions"),
        path.join("sources"),
        path.join("summaries/sources"),
        path.join("artifacts"),
        path.join("audit"),
    ];

    for dir in dirs {
        fs::create_dir_all(&dir).with_context(|| format!("create directory {}", dir.display()))?;
    }

    write_new_file(
        &path.join("agents.md"),
        r#"# JJ Agents

This vault is the source of truth for JJ's memory and operating rules.

## Pointers
- See `invariants.md` for hard rules.
- Prompts live in `prompts/`.
- Runtime config lives in `config/`.
"#,
    )?;

    write_new_file(
        &path.join("invariants.md"),
        r#"# JJ Invariants

1. Raw threads are append-only.
2. Durable memory changes are reversible and attributable.
3. No silent overwrites of beliefs (supersede or contradict instead).
4. Model suggests; system governs.
5. Retrieval uses tiers by default.
6. Tools are explicit and discoverable.
"#,
    )?;

    write_new_file(
        &path.join("config/jj.runtime.yml"),
        r#"# JJ runtime configuration
providers:
  default: "openai"

logging:
  level: "info"
"#,
    )?;

    write_new_file(
        &path.join("config/memory.policy.yml"),
        r#"# Memory governance policy defaults
auto_apply:
  confidence_threshold: 0.8
  risk_levels: ["low"]

queue_for_review:
  risk_levels: ["medium", "high"]
"#,
    )?;

    write_new_file(
        &path.join("prompts/jj.system.md"),
        r#"# JJ System Prompt

You are JJ, a memory-first assistant. Follow the invariants and log all tool use.
"#,
    )?;

    write_new_file(
        &path.join("prompts/curator.system.md"),
        r#"# Curator System Prompt

Propose memory updates as JSON. Include sources and risk level.
"#,
    )?;

    write_new_file(
        &path.join("prompts/daily_review.system.md"),
        r#"# Daily Review Prompt

Summarize recent threads and propose memory hygiene improvements.
"#,
    )?;

    write_new_file(
        &path.join("prompts/weekly_review.system.md"),
        r#"# Weekly Review Prompt

Consolidate knowledge, detect contradictions, and propose updates.
"#,
    )?;

    Ok(())
}

fn write_new_file(path: &Path, content: &str) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("create file {}", path.display()))?;
    file.write_all(content.as_bytes())
        .with_context(|| format!("write file {}", path.display()))?;
    Ok(())
}

pub fn resolve_vault(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| PathBuf::from("jj_vault"))
}
