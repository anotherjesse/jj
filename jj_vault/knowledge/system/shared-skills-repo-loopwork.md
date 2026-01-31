---
id: mem_01KGAMGXPK5JHCY0X9DS813MZ8
title: Shared skills repository (loop.work/skills)
type: system
status: active
tags:
- skills
- repo-structure
- workflow
confidence: 0.76
created_at: 2026-01-31T18:19:56.499536Z
updated_at: 2026-01-31T18:19:56.499536Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAMF16BHX3QRAJSV7DV8F6K
supersedes: []
---
## Decision
Create a shared repository at **`loop.work/skills`** to consolidate skill definitions that were previously scattered.

## Intended workflow
- Start with **manual sync** of skills; introduce a dedicated **CLI tool later**.
- Maintain a **skills directory** and use **symlinks** to connect skills into Claude/agent environments.
