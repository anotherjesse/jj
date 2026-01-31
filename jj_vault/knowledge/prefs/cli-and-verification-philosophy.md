---
id: mem_01KGAKNA2VHTV4A139VZ621H4F
title: CLI and verification philosophy (agents vs humans)
type: preference
status: active
tags:
- philosophy
- verification
- cli
- agents
confidence: 0.78
created_at: 2026-01-31T18:04:51.675384Z
updated_at: 2026-01-31T18:04:51.675384Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKKWB9TDHWX5V9423PZ1BG
supersedes: []
summary: 'Sparks prefers a simple, single-purpose CLI and splits responsibility: agents write code/tests, humans do QA/smoke verification; agent tests aren’t proof.'
---
## CLI/product philosophy (Sparks)
- Prefer a **simple CLI that does one thing well**, avoiding an all-in-one “monster.”
- Division of responsibility: **agents** own code/tests; **humans** own verification (QA/smoke tests). Passing agent tests isn’t treated as proof of correctness.

Source context: stated as Sparks philosophy.