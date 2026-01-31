# Source Ingestion: Import External Documents into JJ

**Date:** 2026-01-31
**Status:** Brainstorm complete, ready for planning

---

## What We're Building

A `jj ingest <file.md>` CLI command that imports external markdown documents (planning transcripts, reference docs, articles, notes) into the vault. The original is preserved verbatim in a new `sources/` directory. An **agent flow** (not a raw LLM call) processes the document to produce:

1. A **summary** stored in `summaries/sources/` — this is what tiered retrieval hits by default
2. **Knowledge extraction proposals** submitted through the existing governance pipeline — individual facts, preferences, people, projects get proposed as knowledge docs

The agent flow is key: ingest uses the full orchestrator loop with tool calling. The agent can search existing knowledge for context, reason about contradictions, and use tools like `upsert_knowledge` to propose changes naturally.

## Why This Approach

- **Preserves originals:** Verbatim copy into `sources/` respects invariant #1 (append-only raw data) and allows re-processing with better prompts later
- **Governance pipeline reuse:** Knowledge extraction goes through `inbox/proposals/` with risk scoring, auto-apply thresholds, and review queues — no new governance needed
- **Tiered retrieval works naturally:** Summaries are already preferred over raw data in retrieval; source summaries slot right in
- **Agent flow over LLM flow:** The agent can use existing tools, search memory for context, and make informed extraction decisions rather than summarizing in isolation

## Key Decisions

1. **New `sources/` directory** at vault root for verbatim originals (not `artifacts/` which is for binary blobs)
2. **Copy into vault** (not reference-only) so the vault is self-contained and Git-versioned
3. **Agent-driven processing** using the orchestrator loop with tool calling, not a one-shot LLM summarization
4. **Both summary + extraction**: create a summary doc AND propose individual knowledge docs
5. **Single file ingestion** to start — batch/directory support can come later
6. **YAML frontmatter** on stored sources: id, title, type, ingested_at, original_path, processing_status
7. **Agent handles chunking** of large files itself — no pre-chunking by the CLI
8. **Re-ingestion is the agent's call** — the agent decides whether to supersede or create new based on context
9. **CLI accepts optional args** like `--source`, `--tags` for provenance — the system decides what metadata is useful, but where the file came from matters

## Vault Layout Addition

```
jj_vault/
  sources/
    2026/01/31/
      planning-conversation.md    # verbatim original with frontmatter prepended
  summaries/
    sources/
      planning-conversation.md    # agent-generated summary
```

## Open Questions

- Should the agent flow have a specific "ingestion" system prompt, or reuse the curator prompt?
