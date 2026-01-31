# Context-Aware Chat Startup

**Date:** 2026-01-31
**Status:** Ready for planning

## What We're Building

When `jay chat` starts, the agent should already know the shape of its own memory. Two things get injected into the system prompt automatically at chat startup:

1. **Vault TOC** — A compact tree of ALL knowledge docs with one-line summaries. The agent can see everything it knows at a glance and choose to `knowledge_read` specific docs.

2. **Last 24h mutation log** — A list of recent mutations (creates, updates) pulled from the audit ledger, each with a one-line change description. Gives the agent recency awareness without searching.

## Two New Required Fields on `knowledge_apply`

- **`summary`** — One-line description of the entire document (not the change). Powers the TOC. Stored in document frontmatter.
- **`change_summary`** — One-line description of what this specific mutation did. Written to the audit ledger. Powers the 24h digest.

## Why This Approach

**Problem:** Today the agent wakes up with only a ~46-line system prompt and thread history. It has no ambient awareness of what it knows. It must proactively call `knowledge_search` — but it doesn't know what to search for because it doesn't know what exists.

**Solution:** Build-time injection. At chat startup, Rust code scans `knowledge/` frontmatter for summaries (TOC) and reads the audit ledger for last-24h mutations (digest). Both get prepended to the system prompt. No new tools, no agent cooperation needed.

**Rejected alternatives:**
- *New `memory_digest` tool* — relies on agent choosing to call it (the exact problem we're solving)
- *Pre-generated context file* — more moving parts, staleness risk
- *Scanning frontmatter timestamps for recency* — audit ledger is the single source of truth for mutations

## Key Decisions

- **Approach:** Build-time injection in Rust at chat startup
- **TOC source:** `summary` field in knowledge doc frontmatter
- **Digest source:** Audit ledger entries from last 24h, each with `change_summary`
- **Backfill:** Existing docs without `summary` show title only in the TOC
- **Both fields required** on `knowledge_apply` going forward

## Open Questions

- Token budget: how large can the TOC get before it hurts context? May need truncation later.
- TOC format: flat list vs nested tree by directory?
- Should the digest include the actor/author of each mutation?
