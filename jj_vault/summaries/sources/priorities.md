---
id: mem_01KGAKR395681CT4B2BK72DT2C
title: CTO Priorities (2025-01-23 sync)
type: source_summary
status: active
tags:
- priorities
- cto
- roadmap
- sparks
- loopwork
confidence: 0.86
created_at: 2026-01-31T18:06:23.013310Z
updated_at: 2026-01-31T18:06:23.013310Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKP9BVDCXHC4JF6841GQZS
supersedes: []
---
## Summary (2005-01-24 last updated; content from 2025-01-23 Carl sync)
This document captures CTO-level priorities and near-term execution notes for LoopWork/Sparks.

**Immediate deliverables (late Jan 2025):**
- Completed/validated items:
  - TMUX sizing fix (done).
  - Postgres-on-btrfs snapshot patterns validated: hard snapshots require no Postgres coordination; WAL crash recovery tolerates unannounced snapshots; enables forking prod DB for testing/fixtures and rollbacking migrations.
  - SSH improvements: multi-connection and SFTP working; tests pass; Zed terminal issues appear not Sparks-specific.
- **spark-pg service** is working: implements `spark-pg new/fork/connect/status`, uses `spark-client` crate directly, and establishes a pattern for additional services (e.g., redis). Open design question: how much complexity should live inside Spark vs. outside services.
- Remaining immediate work: dev Spark containers with KVM; and **dev/prod unification** (blocking) so deploy/setup/verify share a single flow with flag differences.

**Current top priorities:**
1. **Sparks self-hosting development**: make Sparks capable of developing itself, with clear trust boundaries, git workflows, and separation of source vs. data; goal is composable primitives ("LEGOs") that outpace alternatives.
2. **Internal tooling (LoopWork)**: build an internal chat tool ("Loop chat" as first Spark-native app), support voice call/session logging, publish a shared skills repo at `loop.work/skills`, and crucially **record conversations for AI context** (Claude Code sessions, chats, calls) so future AIs can browse organizational history.
3. **Agentic imagery exploration**: investigate what "Cursor for images" means and how agenticness applies to media; Q1 focus on images then video; includes inspiration from MattF’s Prologue CAD approach (variation generation, rapid evaluation, genetic algorithms for creativity).

**Big-picture framing:** AI tools feel "dead" when frozen into static apps; the agentic nature is the product. Envision "Spark Native Apps" that spin up dedicated Spark instances per collaborative context (e.g., per chat channel). Strategic options with ~$2M runway: build toward revenue vs. build awareness/reputation for potential acquihire.
