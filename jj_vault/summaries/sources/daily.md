---
id: mem_01KGAM4MWVM8V1GNZ3PRYVVCVA
title: 'Source summary: Daily CTO Check-in prompt'
type: source_summary
status: active
tags:
- process
- cto
- daily
- prompts
confidence: 0.86
created_at: 2026-01-31T18:13:14.267188Z
updated_at: 2026-01-31T18:13:14.267188Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAM3XVP0HWCMX3EZEKBCEYN
supersedes: []
---
## Summary
This document defines a **Daily CTO Check-in** routine intended to be run every morning to keep work focused for a **2-person, pre-product startup** where speed matters. The core asset is a reusable prompt for an AI assistant.

The prompt instructs the assistant to first read the CTO system context (files under `context/`) and the current `priorities.md`, then guide a lightweight daily planning conversation across five areas:
1. **Yesterday Review**: Jesse shares what was accomplished.
2. **Today’s Focus**: The assistant recommends what to focus on today based on priorities/roadmap.
3. **Blockers**: Identify blockers and propose strategies to resolve them.
4. **Quick Wins**: Suggest small, high-leverage tasks.
5. **Carl Sync**: Call out anything that should be synced with Carl today.

Operationally, it specifies how to run the workflow locally (from `~/cto` using `claude`) and sets an **after-session discipline**: update `logs/YYYY-MM-DD.md` with key decisions and update `priorities.md` if priorities shift.

## Notes
- Emphasizes responses should be **concise and actionable**.
- Reinforces use of the existing CTO System file structure (`context/`, `priorities.md`, `logs/`).
