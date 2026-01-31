---
id: mem_01KGAKMW982JM8K4E646GW9JYB
title: Sparks
type: project
status: active
tags:
- dev-tools
- agentic-compute
- containers
- btrfs
- rust
- cli
confidence: 0.82
created_at: 2026-01-31T18:04:37.544114Z
updated_at: 2026-01-31T18:06:23.014337Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKKWB9TDHWX5V9423PZ1BG
- thread_id: ''
  event_ids:
  - src_01KGAKP9BVDCXHC4JF6841GQZS
supersedes: []
---
## Summary
Sparks provides instant sandbox environments (“sparks”) in isolated containers with **persistent filesystems** backed by **btrfs snapshots**. It is a CLI tool intended to be fast and composable.

- Repo: https://github.com/loopwork/sparks
- Local path (as noted in source): `~/lw/sparks`
- Language: Rust
- Lineage: spiritual child of **Vibewire** (monolithic Elixir system)

## Philosophy
- Simple CLI that does one thing well (avoid all-in-one systems).
- Coding agents own code + tests.
- Humans own verification flow (automated QA / smoke tests).
- Treat agent output like outsourced dev: passing tests ≠ “right for you.”

## Core features / commands
- Lifecycle: `spark create`, `delete`, `stop`, `resume`
- Execution: `spark exec`, `spark console`
- Snapshots: `spark snapshot create`, `restore` (btrfs)
- Templates: `spark base` (save a spark as a template)
- Persistent volumes: `spark data` (share data across sparks)
- Secrets: `spark secrets` (1Password integration)
- Namespacing: “Projects” for namespace isolation; auto-detect from git repo
- Ephemeral sparks: `-` creates an auto-deleting spark on exit

## System volumes (auto-mounted)
- `/spark/bin` — shared executables (global)
- `/spark/all` — shared data (global)
- `/spark/proj` — shared data (per project)

## Vision
Positioned as “agentic-first cloud compute” (analogous to OpenStack for VMs): infrastructure that enables new “physics” for agentic workloads, including fast scale up/down and time/space shifting of computation.

Key exploration: frozen agent-built artifacts feel “dead,” so Sparks aims toward continuously agentic execution where boundaries blur between dev iteration and shipped product, static apps vs. just-in-time interfaces, and human collaboration vs. agent-mediated communication.

## Validated capability
- 2025-01-24: **Hard snapshots work with stateful apps**—Postgres survives unannounced btrfs snapshots via WAL crash recovery; suggests “fork anything anytime” can be real.

## Status (as of week of 2025-01-23)
Working toward Sparks being able to “develop itself” by moving development “onto the inside,” refining git workflows and trust boundaries between source vs. data, and building composable primitives (“LEGOs”).


## Priorities and near-term work (from CTO priorities, 2025-01-23)
- **Self-hosting development** remains the top goal: get Sparks to a point where it can develop itself (trust boundaries; git workflows; source vs data separation; composable "LEGOs").
- **spark-pg service** (working as of 2025-01-25):
  - Commands implemented: `spark-pg new`, `fork`, `connect`, `status`.
  - Uses `spark-client` crate directly.
  - Establishes a pattern for additional "service" wrappers (e.g., redis).
  - Open question: how much service complexity should live **inside Spark** vs **outside** as separate services.
- **Dev Spark containers with KVM**: priority item to improve local/dev parity.
- **Dev/Prod unification (blocking)**: currently divergent scripts/flows/settings; target is one path with flag differences, affecting deploy/setup/verify.
- Ops/dev UX notes:
  - SSH multi-connection + SFTP reportedly working with tests passing (2025-01-25).
  - Zed terminal "spins on connect" issue is likely not Sparks-specific.
