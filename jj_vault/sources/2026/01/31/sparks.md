---
id: src_01KGAHQM5C1W0DZ2N3XS49GQPP
title: sparks
ingested_at: 2026-01-31T17:31:10.380342+00:00
original_path: /Users/jesse/cto/context/sparks.md
tags: []
processing_status: complete
content_hash: sha256:0233f67c15b39e8303034750f0713485eea2105fe69455bae1021ba4660246f2
---
# Sparks

## What It Is
Instant sandbox environments with persistent filesystems. Each spark runs in an isolated container with its own root filesystem backed by btrfs snapshots.

- **Repo**: github.com/loopwork/sparks
- **Local**: ~/lw/sparks
- **Language**: Rust (Jesse's first Rust project - "if it compiles, it works")
- **Lineage**: Spiritual child of Vibewire (which was a monolithic Elixir system)

## Philosophy
- Simple CLI that does one thing well (vs. all-in-one monster)
- Coding agents own the code and tests
- Humans own the verification flow (automated QA / smoke tests)
- Think of coding agents like outsourced dev - their tests passing doesn't mean it's right for you

## Core Features
- `spark create/delete/stop/resume` - lifecycle
- `spark exec/console` - run commands or get shell
- `spark snapshot create/restore` - btrfs snapshots
- `spark base` - save spark as template
- `spark data` - persistent volumes across sparks
- `spark secrets` - 1Password integration
- Projects for namespace isolation (auto-detects from git repo)
- Ephemeral sparks with `-` (auto-delete on exit)

## System Volumes (auto-mounted)
- `/spark/bin` - shared executables (global)
- `/spark/all` - shared data (global)
- `/spark/proj` - shared data (per project)

## The Vision: Agentic-First Cloud Compute

Sparks is to agentic workloads what OpenStack was to VMs - the underlying physics.

### The "Deadness" Problem
Previous experiments (ZOMG) let you chat with an agent to build experiences, then "freeze" them into apps/widgets. But frozen apps feel dead - even with a chatbot, the capabilities are stuck in time.

**The question**: What does it really mean to be agentic execution? Where do lines blur between:
- Dev mode (iterating with agent) vs. shipped product
- Just-in-time interfaces vs. static apps
- Human collaboration vs. agent-mediated communication

### Multi-User, But Different
Google Docs showed collaboration is powerful, but current practices have problems:
- Groupthink, loudest person wins
- Comments become bikeshedding, lose forest for trees
- Still humans collaborating with humans

**Idea**: Maybe humans can't see each other until mediated by agents. Agents as facilitators of human communication, not just tools.

### Cloud Thinking Applied
Jesse's OpenStack insight: cloud isn't just cost savings, it's magical capabilities you couldn't have before:
- Shift time/space
- Scale to 2000, back to zero
- What took 2 years on hidden hardware takes a day

Sparks provides these "magical capabilities" for agentic workloads.

## Validated Capabilities

- **Hard snapshots work with stateful apps** (2025-01-24): Postgres survives unannounced btrfs snapshots. WAL crash recovery handles it. No need to coordinate with apps before forking - "fork anything anytime" is real.

## Current Status (Week of 2025-01-23)

Getting to where Sparks can develop itself:
- Moving development "onto the inside"
- Working out: git workflows, source vs data, trust/canonical truth
- Goal: LEGOs that snap together, faster to use Sparks than alternatives
- Transitioning from "verification passes" to "accelerant for dev work"

---
*Last updated: 2025-01-23*
