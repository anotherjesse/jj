---
id: mem_01KGAKR3971QDEQ7QC49S3KBQZ
title: spark-pg service
type: project
status: active
tags:
- sparks
- postgres
- service
- cli
confidence: 0.78
created_at: 2026-01-31T18:06:23.015970Z
updated_at: 2026-01-31T18:06:23.015970Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKP9BVDCXHC4JF6841GQZS
supersedes: []
---
## Summary
`spark-pg` is a Sparks-adjacent service/CLI for managing Postgres databases in the Sparks ecosystem.

## Status
- Working as of **2025-01-25**.

## Implemented commands
- `spark-pg new`
- `spark-pg fork`
- `spark-pg connect`
- `spark-pg status`

## Implementation notes
- Uses the `spark-client` crate directly.
- Intended to establish a repeatable pattern for additional services (e.g., `spark-redis`).

## Open question
- Architectural boundary: how much complexity should live **inside Spark** vs. **outside** Spark in separate services.
