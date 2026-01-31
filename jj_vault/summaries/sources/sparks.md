---
id: mem_01KGAHRHBQQ0VJS66BT0Z4WFB1
title: 'Sparks (source: src_01KGAHQM5C1W0DZ2N3XS49GQPP)'
type: source_summary
status: active
tags:
- loopwork
- sparks
- infrastructure
- agentic-compute
confidence: 0.9
created_at: 2026-01-31T17:31:40.279620Z
updated_at: 2026-01-31T17:31:40.279620Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAHQM5C1W0DZ2N3XS49GQPP
supersedes: []
---
## Summary
Sparks is a LoopWork project that provides **instant sandbox environments** (“sparks”) with **persistent filesystems**. Each spark runs in an **isolated container** with its own root filesystem, backed by **btrfs snapshots** for fast cloning and restore. The repo is `github.com/loopwork/sparks` and the local path is `~/lw/sparks`. It is implemented in **Rust** and is described as Jesse’s first Rust project (with the motto “if it compiles, it works”). Sparks is presented as a spiritual successor to **Vibewire**, a previous monolithic Elixir system.

The product philosophy emphasizes a **simple CLI** that does one thing well, with a clear division of responsibilities: **coding agents** own the code and tests, while **humans own verification** via QA/smoke tests; agent-written tests passing are treated as necessary but not sufficient (“outsourced dev” analogy).

Core CLI capabilities include lifecycle management (`spark create/delete/stop/resume`), command execution and shells (`spark exec/console`), snapshotting (`spark snapshot create/restore`), templating (`spark base`), persistent volumes across sparks (`spark data`), and secrets management via **1Password integration** (`spark secrets`). Sparks supports “projects” for namespace isolation (auto-detected from git repos) and **ephemeral sparks** designated with `-` that auto-delete on exit.

Sparks auto-mounts shared volumes: `/spark/bin` (global executables), `/spark/all` (global shared data), and `/spark/proj` (per-project shared data).

Strategically, Sparks is framed as “**agentic-first cloud compute**”: infrastructure that makes agentic workloads feel like cloud did for VMs (time/space shifting, scale up/down). A key motivation is the “**deadness**” of freezing agent-built experiences into static apps; Sparks explores blurred boundaries between dev mode vs shipped product and human collaboration mediated by agents. A validated capability (2025-01-24) is that **hard btrfs snapshots work with stateful apps** like Postgres without coordination (WAL crash recovery suffices), supporting “fork anything anytime.”

(Last updated in source: 2025-01-23)
