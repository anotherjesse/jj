---
id: mem_01KGAM6QBHQN22PNVECVZC0W9G
title: Weekly CTO Review prompt
type: source_summary
status: active
tags:
- cto
- process
- prompts
- weekly-review
confidence: 0.74
created_at: 2026-01-31T18:14:22.321140Z
updated_at: 2026-01-31T18:14:22.321140Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAM69X9EWDQ54A2SB15GTCY
supersedes: []
---
## Summary
This document defines a **Weekly CTO Review** session-starter prompt to be run at the end of each week (Friday) or start of the next (Monday). The assistant is instructed to read CTO system context files under `context/`, the current `priorities.md`, and any weekly logs in `logs/`, then help conduct a structured review and planning session.

The prompt is organized into four sections:
- **Retrospective**: assess what shipped/accomplished, what didn’t get done but should have, key learnings, and what to stop/start/continue.
- **Planning**: identify top 3 priorities for next week; evaluate roadmap status and needed adjustments; surface accumulating technical debt; and list decisions that must be made.
- **Team & Communication**: determine what to sync with **Carl** about and which external stakeholders require updates.
- **Self-Care**: evaluate sustainability/pace and choose one improvement for personal effectiveness.

A key behavioral instruction is to **be direct and challenge JJ’s thinking**, with an emphasis on staying focused on what matters most **at pre-product stage**.

After completing the session, the operator should:
- update `context/roadmap.md`
- update `priorities.md`
- archive important decisions to `decisions/`
