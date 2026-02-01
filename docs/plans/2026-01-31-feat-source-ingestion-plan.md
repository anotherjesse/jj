---
title: "Source Ingestion CLI"
type: feat
date: 2026-01-31
brainstorm: docs/brainstorms/2026-01-31-source-ingestion-brainstorm.md
---

# feat: Source Ingestion — `j ingest <file>`

## Overview

Add a `j ingest <file.md>` CLI command that imports external markdown documents into the vault. The original is preserved verbatim in `sources/`. An agent flow (reusing the orchestrator loop from `repl.rs`) processes the document to produce a summary and knowledge extraction proposals through the existing governance pipeline.

## Proposed Solution

### CLI Interface

```
j ingest <file_path> --vault <vault_path> [--source <provenance>] [--tags tag1,tag2] [--title "Title"]
```

- `file_path`: Path to a markdown file (must be valid UTF-8, `.md` extension)
- `--vault`: Required (or `JJ_VAULT_PATH` env, same as existing commands)
- `--source`: Free-text provenance string (e.g., "chatgpt-export", "notion", "manual")
- `--tags`: Comma-separated tags
- `--title`: Override title (default: derived from filename)

### Processing Flow

1. **Validate** file exists, is UTF-8, vault exists
2. **Copy** original into `sources/YYYY/MM/DD/{slug}.md` with frontmatter prepended
3. **Create ingestion thread** via `thread_store::create_thread()`
4. **Run agent loop** (extracted from `repl.rs`) with an ingestion-specific system prompt
   - Agent receives the source document content as the initial "user message"
   - Agent has access to all existing tools: `knowledge_read`, `knowledge_search`, `knowledge_apply`, `knowledge_index`
   - Agent writes summary to `summaries/sources/{slug}.md`
   - Agent creates proposals in `inbox/proposals/`
5. **Embed** summary chunks into the vector index
6. **Git commit** the new source + summary + any auto-applied knowledge
7. **Print** results to stdout

### Vault Layout

```
sources/
  YYYY/MM/DD/
    {slug}.md          # verbatim original + j frontmatter prepended
summaries/
  sources/
    {slug}.md          # agent-generated summary with frontmatter
```

### Source File Frontmatter

Prepended to the verbatim copy:

```yaml
---
id: src_01KG...           # ULID with src_ prefix
title: "Planning Conversation"
ingested_at: 2026-01-31T14:00:00Z
original_path: /Users/jesse/docs/planning.md
source: "chatgpt-export"  # from --source
tags: [planning, j]       # from --tags
processing_status: complete # pending|processing|complete|failed
content_hash: sha256:abc...
---
```

### Summary Frontmatter

```yaml
---
id: sum_01KG...
title: "Summary: Planning Conversation"
source_id: src_01KG...     # links back to source
type: source_summary
status: active
created_at: 2026-01-31T14:01:00Z
---
```

## Technical Approach

### Phase 1: Plumbing — CLI + source storage

**Files to modify:**

- `src/main.rs` — Add `Ingest` subcommand to `Commands` enum (~15 lines)
- `src/vault.rs` — Add `sources/` and `summaries/sources/` to `init_vault()` dirs array (~2 lines)

**Files to create:**

- `src/ingest.rs` — Core ingestion logic

**`src/ingest.rs` responsibilities:**

```
pub fn ingest_file(vault_path, file_path, source, tags, title) -> Result<IngestResult>
```

1. Read and validate input file (UTF-8, exists, reasonable size)
2. Generate slug from `--title` or filename (lowercase, hyphens, strip special chars)
3. Build frontmatter struct, compute content SHA256
4. Write to `sources/YYYY/MM/DD/{slug}.md` (frontmatter + original content)
5. Create ingestion thread
6. Call agent loop (Phase 2)
7. Return paths to created files

**Slug generation:** filename without extension → lowercase → replace non-alphanumeric with hyphens → collapse consecutive hyphens → trim. If collision at target path, append ULID suffix.

**Acceptance criteria:**

- [x] `j ingest plan.md --vault j_vault` copies file to `sources/2026/01/31/plan.md`
- [x] Frontmatter is prepended with correct fields
- [x] Content hash matches original file
- [x] `sources/` directory created by `vault init`

### Phase 2: Agent flow — extract and refactor the orchestrator

**Files to modify:**

- `src/repl.rs` — Extract the inner agent loop into a reusable function

The REPL's inner loop (lines ~102-179) does: send messages to LLM → execute tool calls → append to thread → repeat until no more tool calls. This is exactly what ingestion needs.

**Extract to:**

```rust
// src/agent.rs (new)
pub struct AgentConfig {
    pub system_prompt: String,
    pub tools: Vec<Value>,
    pub vault_path: PathBuf,
    pub thread_path: PathBuf,
    pub max_turns: usize,          // safety limit, e.g. 20
}

pub fn run_agent_loop(
    config: &AgentConfig,
    initial_messages: Vec<Message>,
    client: &OpenAiClient,
) -> Result<Vec<Message>>
```

**Then `repl.rs` calls `run_agent_loop()` instead of inlining the loop.** Same for `ingest.rs`.

**Ingestion system prompt** — new file `j_vault/prompts/ingest.system.md`:

```markdown
You are J's ingestion agent. You have been given an external document to process.

Your tasks:
1. Read and understand the document thoroughly
2. Search existing knowledge for related content using knowledge_search
3. Write a concise summary (200-500 words) using knowledge_apply to summaries/sources/{slug}.md
4. Extract discrete knowledge items and create proposals using knowledge_apply
   - People mentioned → knowledge/people/
   - Projects described → knowledge/projects/
   - Preferences stated → knowledge/prefs/
   - System facts → knowledge/system/
5. For each extraction, search existing knowledge first to avoid duplicates or to supersede existing docs

Follow the invariants. Every knowledge write needs a reason and source references.
```

**Acceptance criteria:**

- [x] Agent loop extracted from `repl.rs` into `agent.rs`
- [x] REPL still works identically (calls extracted function)
- [x] `j ingest` runs the agent loop with ingestion prompt
- [ ] Agent produces summary in `summaries/sources/` (requires live LLM test)
- [ ] Agent produces at least one knowledge proposal (requires live LLM test)
- [x] All agent actions logged in the ingestion thread

### Phase 3: Embed + commit + output

**Files to modify:**

- `src/ingest.rs` — Add post-agent steps
- `src/embedding_index.rs` — May need to support indexing a single doc (currently rebuilds entire index)

**Steps:**

1. After agent completes, update source frontmatter `processing_status: complete`
2. Embed summary chunks into index (append to existing JSONL, or rebuild)
3. Git commit all new/changed files with message: `ingest: {slug} from {source}`
4. Print results:

```
Ingested: sources/2026/01/31/planning-conversation.md
Summary:  summaries/sources/planning-conversation.md
Thread:   threads/2026/01/31/thr_01KG...
Proposals: 3 created in inbox/proposals/
```

**Acceptance criteria:**

- [x] Summary is embedded and searchable via `knowledge_search` (re-index after agent)
- [x] Git commit created with ingestion metadata
- [x] Clear stdout output showing what was created
- [x] If agent fails, `processing_status: failed` is set and error printed

## Edge Cases

| Case | Behavior |
|------|----------|
| File not found | Exit with error before any vault writes |
| Not UTF-8 | Exit with error |
| Vault doesn't exist | Exit with error (use `j vault init` first) |
| Slug collision | Append ULID suffix to slug |
| Agent API failure | Set `processing_status: failed`, source still preserved, print error |
| Large file (>1MB) | Agent handles chunking itself; no CLI limit initially |
| Re-ingestion of same content | Agent's call — it can search for existing source by hash and decide |
| Ctrl+C during processing | Source file persists, thread has partial log, status stays `processing` |

## Dependencies

- Existing: `thread_store`, `knowledge`, `openai`, `embedding_index`, `audit`, `git_utils`
- New crate deps: none expected

## References

- Brainstorm: `docs/brainstorms/2026-01-31-source-ingestion-brainstorm.md`
- Plan spec: `plan.md` (sections 2, 3, 4, 5)
- Agent loop: `src/repl.rs:102-179`
- Tool schemas: `src/repl.rs:295-434`
- Vault scaffold: `src/vault.rs:6-28`
- CLI dispatch: `src/main.rs:24-62`
