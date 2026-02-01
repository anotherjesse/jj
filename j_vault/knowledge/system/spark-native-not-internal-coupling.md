---
id: mem_01KGAMGXPMQ76AYS3M9EE4ENF8
title: 'Spark-native apps: agentic on Spark, not coupled to internal systems'
type: system
status: active
tags:
- sparks
- architecture
- coupling
- principles
confidence: 0.77
created_at: 2026-01-31T18:19:56.500282Z
updated_at: 2026-01-31T18:19:56.500282Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAMF16BHX3QRAJSV7DV8F6K
supersedes: []
summary: Defines LoopWork “Spark-native apps” as agentic frameworks on Spark while keeping external apps standalone and avoiding coupling to internal-only systems.
---
## Definition / principle
Within LoopWork, a **“Spark native app”** is an application built as an **agentic computing framework on Spark/Sparks**.

Critically, being Spark-native does **not** mean the app is coupled to internal-only systems.

## Implications
- Maintain **external vs. internal tool separation**:
  - External-facing apps (e.g., Picnic) must have standalone capabilities.
  - Internal tools may reuse the same underlying skills, but should avoid tight coupling that blocks external use cases.