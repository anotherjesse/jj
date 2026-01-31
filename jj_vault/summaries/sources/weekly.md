---
id: mem_01KGAJ6Y2ACJARBNZE3WVRK46Y
title: 'Weekly CTO Review prompt (source: weekly)'
type: source_summary
status: active
tags:
- weekly
- cto
- workflow
- review
confidence: 0.9
created_at: 2026-01-31T17:39:32.042571Z
updated_at: 2026-01-31T17:39:32.042571Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAJ6E9V1ARFTBFG0RPMVP8P
supersedes: []
---
## Summary
This document defines a **Weekly CTO Review** ritual to run at the **end of each week (Friday)** or **start of week (Monday)**. The assistant is instructed to first read existing system context from `context/`, `priorities.md`, and any weekly logs in `logs/`, then guide a structured review across four areas:

- **Retrospective**: what shipped/accomplished; what missed; learnings; and stop/start/continue behavior changes.
- **Planning**: identify top 3 priorities for next week; check roadmap alignment and adjust; flag technical debt; and list decisions that must be made.
- **Team & Communication**: define what to sync with **Carl**; identify external stakeholders to update.
- **Self-Care**: assess sustainability of pace and one improvement for personal effectiveness.

The assistant should be **direct**, challenge the user’s thinking, and keep focus on what matters most at a **pre-product stage**.

## Post-session hygiene
After the session, the process requires updating:
- `context/roadmap.md`
- `priorities.md`
And archiving important decisions into `decisions/`.

## Notes / overlaps
This weekly ritual complements (but is distinct from) the existing **daily CTO check-in** workflow and aligns with the broader assistant operating style: read context first, be direct, and capture decisions in the repo.
