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
- prompts
- weekly-review
confidence: 0.76
created_at: 2026-01-31T18:09:27.357944Z
updated_at: 2026-01-31T18:14:31.467037Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKX37PMJ4YRJNTWCT6RX3V
- thread_id: ''
  event_ids:
  - src_01KGAM3XVP0HWCMX3EZEKBCEYN
- thread_id: ''
  event_ids:
  - src_01KGAM69X9EWDQ54A2SB15GTCY
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


## Daily CTO check-in prompt
A daily session-starter prompt exists to run each morning. It instructs the assistant to read `context/` and `priorities.md`, then cover: yesterday review, today’s focus, blockers, quick wins, and whether anything should be synced with Carl.

### After-session updates
- Update `logs/YYYY-MM-DD.md` with key decisions
- Update `priorities.md` if priorities shift
## Weekly CTO review prompt
A **weekly** session-starter prompt exists to run **Friday (end of week)** or **Monday (start of week)**.

### Inputs to read
- `context/` (system context files)
- `priorities.md`
- weekly `logs/`

### Sections
- Retrospective (ship/accomplish, missed items, learnings, stop/start/continue)
- Planning (top 3 next-week priorities, roadmap check, tech debt, decisions)
- Team & Communication (sync with Carl; stakeholder updates)
- Self-Care (sustainability; one improvement for effectiveness)

### After-session updates
- Update `context/roadmap.md`
- Update `priorities.md`
- Archive important decisions to `decisions/`
