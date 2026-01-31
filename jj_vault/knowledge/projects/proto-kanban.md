---
id: mem_01KGAMMVZZ3TA37DKV44YMYDZP
title: Proto Kanban (Rust app)
type: project
status: active
tags:
- kanban
- rust
- agentic-dev
- workflow
confidence: 0.7
created_at: 2026-01-31T18:22:05.822987Z
updated_at: 2026-01-31T18:22:05.822987Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAMJZD7RGD1SHMHS18R8A2G
supersedes: []
summary: 'Rust proto Kanban: 5-column board where moving tasks to “ready” auto-starts Claude coding in a new Sparks fork via RPC triggers.'
---
## Summary
A lightweight **kanban management** tool (described as a Rust app) used to drive agentic coding workflows.

## Workflow described (2026-01-24)
- Prototype kanban board with **5 columns**.
- Dragging a task to **“ready”** automatically kicks off a **Claude coding session** in a **new fork** (via Sparks).
- Uses an **RPC-style architecture** for state management.

## Rationale
Treat task state transitions as automation triggers, enabling long autonomous agent runs while keeping the human in a verification/QA role.

## Source
- `src_01KGAMJZD7RGD1SHMHS18R8A2G`