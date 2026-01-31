---
id: mem_01KGAHTRP7MB7THJY9XXFJVQZH
title: CTO Priorities (Carl sync 2025-01-23)
type: source_summary
status: active
tags:
- priorities
- loopwork
- sparks
- roadmap
confidence: 0.82
created_at: 2026-01-31T17:32:53.319582Z
updated_at: 2026-01-31T17:32:53.319582Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAHSVDN8HRDMTRPVTHV8MMW
supersedes: []
---
## Summary
This document captures LoopWork/Sparks CTO priorities as of a Carl sync on **2025-01-23** (last updated **2025-01-24**). The immediate track shows several infrastructure items completed/validated: a **TMUX sizing fix** (done), **Postgres snapshot patterns** validated (hard btrfs snapshots require no Postgres coordination; WAL crash recovery tolerates unannounced snapshots; enables fast DB forking, test fixtures, and migration rollback), and **SSH improvements** with tests passing (multi-connection + SFTP; a Zed hang appears to be a general Linux issue, not Sparks-specific).

A key near-term deliverable is the **`spark-pg` service** (working as of 2025-01-25), implementing `spark-pg new/fork/connect/status` using the `spark-client` crate directly. This establishes a reusable pattern for additional services (e.g., Redis), with an open design question around how much complexity should live “inside” Spark vs outside.

Two urgent platform priorities remain: **Dev Spark containers with KVM** and **Dev/Prod unification** (explicitly labeled new + blocking). The latter aims to eliminate divergent dev vs prod scripts/flows/settings in favor of a single pathway with a flag difference, spanning deploy/setup/verify.

The **current top 3 priorities** are: (1) **Sparks self-hosting development**—make Sparks able to develop itself (trust, git workflows, and source vs data boundaries) with a goal of composable “LEGOs” that snap together faster than alternatives; (2) **internal tooling for LoopWork**, including moving from iMessage to a first Spark-native app (“Loop chat”), building voice call/session logging, a shared skills repo at `loop.work/skills`, and broad **conversation recording** (chats/calls/Claude sessions) so future AI can browse history for context; and (3) **agentic imagery exploration** (“Cursor for images”), with Q1 focus on images then video, referencing patterns like Prologue CAD (generate variations, rapid human eval, genetic algorithms).

Open questions include a port **8080 documentation gap** for standardized HTTP service patterns, skills repo structure (manual sync → CLI), and external vs internal separation for Picnic. Parking-lot integrations include Granola transcripts, Readwise Reader sync, and an automated changelog service.
