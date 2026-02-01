---
title: "Untyped tool schemas cause LLMs to produce empty output"
category: integration-issues
tags: [llm, tool-calling, openai, schema, types]
module: agent
symptoms:
  - "Knowledge docs created with empty bodies"
  - "LLM omits fields that exist but have no schema description"
  - "Tool calls succeed but produce incomplete results"
severity: high
date: 2026-01-31
---

# Untyped tool schemas cause LLMs to produce empty output

## Problem

After ingesting a document, the agent created knowledge docs (summary, people, projects, system facts) that all had correct YAML frontmatter but **completely empty bodies**. The `body_append` field — the only way to write body content — was never included in the LLM's tool calls.

## Root Cause

The `knowledge_apply` tool schema defined the `patch` parameter as:

```json
"patch": { "type": "object" }
```

No `properties`, no `description`, nothing. The LLM had no way to know that `body_append` existed, what fields were available, or how to use them. It guessed at `doc_path` and `title` (obvious names) but skipped `body_append` entirely.

**The principle**: `{ "type": "object" }` without properties is a lie to the LLM. You're telling it "pass an object" without telling it what shape. It will guess, and it will guess wrong on any field that isn't self-evident from its name.

## Solution

Added a detailed `description` to the `patch` field enumerating all available fields:

```json
"patch": {
    "type": "object",
    "description": "Knowledge patch. Fields: doc_path (required, vault-relative path), title (required for new), type (required for new, e.g. source_summary/project/person/preference/system), status, confidence (0-1), tags_add (array), body_append (markdown string - THE BODY CONTENT), sources_add (array of {thread_id, event_ids}), supersedes_add (array of doc IDs)"
}
```

Also updated the system prompt (`j_vault/prompts/ingest.system.md`) with explicit documentation of the patch format and a JSON example showing `body_append` usage.

### Files changed

- `src/agent.rs` — tool schema description for `knowledge_apply`
- `src/ingest.rs` — default prompt with `body_append` documentation
- `j_vault/prompts/ingest.system.md` — detailed patch format docs + example

## Prevention

### Rule: Never use `{ "type": "object" }` without field documentation

Every tool parameter of type `object` must have either:

1. **Full `properties` schema** (ideal) — gives the LLM structured field definitions
2. **Detailed `description`** (acceptable) — lists all fields, their types, and which are required

If you find yourself writing `{ "type": "object" }` bare, ask: **am I doing something wrong?** Usually yes. Either:

- The object has known fields → define `properties`
- The object is truly freeform → you probably need a different design

### Checklist for tool schemas

- [ ] Every `object` parameter has `properties` or a `description` listing fields
- [ ] Required fields are marked as required
- [ ] Fields critical to functionality are called out explicitly (like `body_append`)
- [ ] System prompts reinforce schema with examples for complex tools
- [ ] Test with a fresh conversation (no prior context) to verify the LLM can use the tool correctly from schema alone

### Why descriptions beat properties for dynamic patches

In this case, `KnowledgePatch` has ~10 optional fields and the shape varies by use case. A full `properties` schema would be ideal but verbose in inline JSON. The description approach works because the LLM reads descriptions carefully — but **only if the description actually exists**.

## Key Insight

Tool schemas are the LLM's only API documentation. If a field isn't in the schema (or described), it doesn't exist to the LLM. Treat tool schemas with the same rigor as public API docs — they're the contract between your system and the model.
