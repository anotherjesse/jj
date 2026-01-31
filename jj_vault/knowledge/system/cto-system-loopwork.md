---
id: mem_01KGAKXQ9XGFXATP2PB6FQS1GT
title: CTO System (LoopWork) knowledge base structure
type: system
status: active
tags:
- process
- documentation
- claude-code
- cto
confidence: 0.85
created_at: 2026-01-31T18:09:27.357944Z
updated_at: 2026-01-31T18:09:27.357944Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKX37PMJ4YRJNTWCT6RX3V
supersedes: []
---
## Statement
LoopWork maintains a local "CTO System" knowledge base (used by Claude Code) to provide session-start context about LoopWork, current priorities, technical decisions/history, and recent activity.

## Interaction contract
- Jesse can ask what’s going on or what’s needed.
- The assistant should read the context files and ask follow-up questions when necessary.

## Files / folders
Located under `cto/`:
- `README.md` — description of the system
- `priorities.md` — current priorities (updated by the assistant)
- `context/` — durable background:
  - `company.md` — team, runway, strategic direction
  - `sparks.md` — Sparks project details (agentic-first compute)
  - `tech-stack.md` — technical choices
  - `roadmap.md` — milestones and plan
- `prompts/` — optional session starters (daily/weekly/architecture/code-review/debug/strategy)
- `decisions/` — Architecture Decision Records
- `logs/` — session logs when useful

## Update discipline
After sessions where important things happen, update:
- `priorities.md` if priorities shift
- relevant `context/` files with new information
- `decisions/` for significant technical decisions

## Dates
- System created: 2025-01-23
