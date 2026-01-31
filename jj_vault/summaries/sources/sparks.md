---
id: mem_01KGAKMW97ZEA3AK14GBMAJ61T
title: sparks (source summary)
type: source_summary
status: active
tags:
- sparks
- dev-tools
- agentic-compute
- containers
- btrfs
- rust
confidence: 0.88
created_at: 2026-01-31T18:04:37.543321Z
updated_at: 2026-01-31T18:04:37.543321Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKKWB9TDHWX5V9423PZ1BG
supersedes: []
---
## Summary
Sparks is a CLI-first system for creating **instant sandbox environments** (“sparks”) that run in isolated containers with **persistent filesystems**. Each spark has its own root filesystem backed by **btrfs snapshots**, enabling fast lifecycle operations and hard snapshot/restore. The repository is `github.com/loopwork/sparks`, with a local path noted as `~/lw/sparks`. It is written in **Rust** and described as Jesse’s first Rust project (tagline: “if it compiles, it works”). Sparks is positioned as a spiritual successor to **Vibewire**, previously a monolithic Elixir system.

The philosophy emphasizes a **simple CLI that does one thing well**, avoiding an “all-in-one monster.” It distinguishes responsibilities: **coding agents** should own code and tests, while **humans** own verification via automated QA/smoke tests; agent-generated passing tests are not treated as sufficient proof of correctness.

Core commands include lifecycle management (`spark create/delete/stop/resume`), running commands and shells (`spark exec/console`), snapshotting (`spark snapshot create/restore`), templating (`spark base`), persistent cross-spark volumes (`spark data`), and **1Password** integration (`spark secrets`). It supports project-based namespace isolation (auto-detected from git repos) and ephemeral sparks denoted with `-` that auto-delete on exit.

Sparks auto-mounts shared volumes: `/spark/bin` (global executables), `/spark/all` (global shared data), and `/spark/proj` (per-project shared data).

The broader vision frames Sparks as “agentic-first cloud compute,” analogous to OpenStack’s role for VMs. It argues that “frozen” agent-built apps feel “dead” and explores blurred boundaries between dev mode and shipped product, static apps vs. just-in-time interfaces, and human collaboration mediated by agents. A validated capability (2025-01-24) is that unannounced btrfs snapshots work with stateful apps: Postgres survives via WAL crash recovery, supporting “fork anything anytime.” Status as of 2025-01-23: focusing on making Sparks able to develop itself by moving development “onto the inside,” refining git workflows and source/data trust boundaries, and building composable “LEGOs.”
