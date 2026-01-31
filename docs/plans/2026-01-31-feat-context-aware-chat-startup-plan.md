---
title: "feat: Context-Aware Chat Startup"
type: feat
date: 2026-01-31
brainstorm: docs/brainstorms/2026-01-31-context-aware-startup-brainstorm.md
---

# feat: Context-Aware Chat Startup

## Overview

When `jay chat` starts, the agent currently wakes up with only a static 46-line system prompt and thread history — no awareness of what it knows or what changed recently. This feature injects two dynamic sections into the system prompt at startup:

1. **Vault TOC** — compact tree of all knowledge docs with one-line summaries
2. **Last 24h mutation digest** — recent audit ledger entries with change descriptions

## Problem Statement

The agent must proactively call `knowledge_search` to discover what it knows, but it doesn't know *what to search for* because it has no index of its own memory. Users experience this as the agent seeming uninformed about context that was just ingested.

## Proposed Solution

**Build-time injection in Rust.** At chat startup, scan knowledge doc frontmatter for summaries (TOC) and read the audit ledger for last-24h mutations (digest). Append both to the system prompt before the first LLM call.

No new tools. No agent cooperation needed. The context is always there.

## Technical Approach

### Phase 1: Schema Changes

**Add `summary` field to FrontMatter** (`src/knowledge.rs:22-34`)

```rust
pub struct FrontMatter {
    // ... existing fields ...
    #[serde(default)]
    pub summary: String,  // one-line description of entire doc
}
```

- Required on new docs via `knowledge_apply`
- Existing docs without it fall back to title-only in TOC
- Plain text, max ~150 chars, no markdown
- Update `parse_markdown()` and `render_markdown()` to handle the field

**Add `change_summary` field to LedgerEntry** (`src/audit.rs:11-26`)

```rust
pub struct LedgerEntry {
    // ... existing fields ...
    #[serde(default)]
    pub change_summary: String,  // one-line: what this mutation did
}
```

- Required on new mutations via `knowledge_apply`
- Distinct from `reason` (intent) — this describes the actual change
- Example reason: "Processing ingested CTO doc"
- Example change_summary: "Created project doc for JJ Gateway with tech stack and architecture"

**Update `knowledge_apply` tool schema** (`src/agent.rs:233-265`)

Add two new required params to the tool schema:
- `summary`: "One-line description of the entire document (not the change). Max 150 chars."
- `change_summary`: "One-line description of what this specific mutation does. Max 150 chars."

Update the tool description to explain both fields. Follow the learning from `docs/solutions/` — schema fields must be fully described or the LLM ignores them.

### Phase 2: TOC Builder

**New function in `src/chat.rs`** (or a small `src/context.rs` module):

```rust
fn build_vault_toc(vault: &Path) -> Result<String>
```

- Walk `knowledge/` recursively using existing `walk_markdown` pattern (`src/agent.rs:543`)
- For each `.md` file, parse frontmatter only (skip body for speed)
- Group by subdirectory (people/, projects/, prefs/, system/)
- Format as compact tree:

```
## Your Knowledge

### projects/ (12 docs)
- jj-gateway.md — Event-sourced CLI + Telegram gateway for JJ agent
- openclaw.md — AI agent framework with local-first architecture
- ...

### people/ (8 docs)
- jeremy-howard.md — Fast.ai founder, deep learning researcher
- ...

### system/ (5 docs)
- cto-system-loopwork.md — CTO folder structure and usage guide
- ...

### prefs/ (3 docs)
- coding-style.md — Rust, minimal deps, small commits
- ...
```

- Docs without `summary`: show title only (from frontmatter `title` field)
- Skip docs with malformed frontmatter, log warning
- Sort alphabetically within each group

### Phase 3: Mutation Digest

**New function:**

```rust
fn build_mutation_digest(vault: &Path) -> Result<String>
```

- Read `audit/ledger.jsonl` line by line
- Filter entries where `ts >= now - 24h` (UTC)
- For each matching entry, emit one line:

```
## Recent Changes (last 24h)

- [15:23] Created knowledge/projects/jj-gateway.md — Event-sourced CLI + Telegram gateway
- [14:01] Updated knowledge/system/priorities.md — Added Q1 infrastructure goals
- [13:45] Created knowledge/people/jeremy-howard.md — Fast.ai founder profile
```

- If no mutations in 24h: `## Recent Changes (last 24h)\n\nNo changes.`
- Read ledger from the end for efficiency (recent entries are at the bottom)
- Old entries without `change_summary` field: fall back to `reason`
- Show all entries (no deduplication) — the audit log is the truth

### Phase 4: Inject into System Prompt

**Modify `load_system_prompt`** (`src/chat.rs:161-170`):

```rust
fn load_system_prompt(vault: &Path) -> Result<String> {
    let path = vault.join("prompts/jj.system.md");
    let base = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        "You are JJ, a memory-first assistant.".to_string()
    };

    let toc = build_vault_toc(vault).unwrap_or_default();
    let digest = build_mutation_digest(vault).unwrap_or_default();

    Ok(format!("{base}\n\n{toc}\n\n{digest}"))
}
```

- TOC and digest appended after the static prompt
- Errors in TOC/digest building are non-fatal (empty string fallback)
- Built once at startup, frozen for the session

### Phase 5: Backfill Script

**One-time CLI command: `cargo run -- backfill-summaries --vault jj_vault`**

- Scans all knowledge docs without a `summary` field
- For each doc, sends title + first 500 chars of body to LLM with prompt:
  "Write a single-line summary (max 150 chars) describing this entire document. Be specific and concrete."
- Updates frontmatter with the generated `summary`
- Commits changes to git
- Also writes a `change_summary` to the audit ledger for each update

This is simpler than re-importing 65 docs and preserves all existing metadata.

## Acceptance Criteria

- [x] `summary` field added to FrontMatter struct, parsed and rendered
- [x] `change_summary` field added to LedgerEntry struct
- [x] `knowledge_apply` tool schema requires both `summary` and `change_summary`
- [x] System prompt includes vault TOC at chat startup
- [x] System prompt includes 24h mutation digest at chat startup
- [x] Docs without `summary` show title-only in TOC (no crash)
- [x] Empty vault or no recent mutations handled gracefully
- [x] Backfill command generates summaries for all existing docs
- [x] `scripts/verify.sh` passes

## Dependencies & Risks

- **Token budget**: 65 docs × ~80 chars = ~5K chars for TOC. Manageable now, may need truncation at 200+ docs.
- **Ledger size**: Reading from end is fast. At 10K+ entries, consider an index or reverse-read optimization.
- **LLM backfill quality**: Summaries may need manual review. Run once, spot-check, commit.

## References

- Brainstorm: `docs/brainstorms/2026-01-31-context-aware-startup-brainstorm.md`
- System prompt: `jj_vault/prompts/jj.system.md`
- Chat startup: `src/chat.rs:47-49` (prompt load + inject)
- FrontMatter struct: `src/knowledge.rs:22-34`
- LedgerEntry struct: `src/audit.rs:11-26`
- Tool schema: `src/agent.rs:233-265`
- Audit ledger: `jj_vault/audit/ledger.jsonl`
- Learning on tool schemas: `docs/solutions/integration-issues/untyped-tool-schemas-cause-empty-llm-output.md`
