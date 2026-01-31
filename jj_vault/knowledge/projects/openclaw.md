---
id: mem_01KGAKMBWCEZTTJG80RZC52GFN
title: OpenClaw
type: project
status: active
tags:
- ai-agent-framework
- gateway
- local-first
- nodejs
- typescript
confidence: 0.88
created_at: 2026-01-31T18:04:20.748717Z
updated_at: 2026-01-31T18:04:20.748717Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKJXVY906XCK7PZ4K4TCT8
supersedes: []
---
## Overview

OpenClaw is a **gateway-centric AI agent framework** for an always-on, local-first personal assistant. A single long-running **Gateway daemon** is the central control plane for messaging channel connections, session routing/serialization, scheduling (cron + heartbeats), and tool execution.

## Key architectural components

- **Gateway daemon (single control plane)**
  - Runs on **Node.js ≥22** (TypeScript).
  - Exposes a WebSocket control plane: `ws://127.0.0.1:18789`.
  - Enforces **one Gateway per host** (e.g., due to WhatsApp Web session constraints).
  - Owns channel connectivity (examples mentioned): WhatsApp (Baileys), Telegram (grammY), Discord (discord.js), Slack (Bolt).

- **Typed WebSocket protocol**
  - Frame types: `req`, `res`, `event`.
  - First frame must be authenticated `connect` handshake.
  - Side-effecting calls (e.g., `agent`, `send`) require **idempotency keys** for safe retries.

- **Session-based serialization**
  - Agent runs are queued **per session key** to avoid tool/session race conditions (GenServer-per-session analogue).
  - Example keys: `agent:<agentId>:<channel>:dm:<peerId>`, `agent:<agentId>:<channel>:group:<id>`.

- **Scheduling**
  - Cron job schedules: `at`, `every`, `cron`.
  - Job session modes:
    - `main`: inject system event into existing main session.
    - `isolated`: runs in fresh session `cron:<jobId>` and posts results back.
  - Heartbeats: periodic proactive turns in main session (default stated: every 30 minutes) driven by `HEARTBEAT.md`.
  - Cron persistence: `~/.openclaw/cron/jobs.json`; concurrency limited by `maxConcurrentRuns`.

- **Durable memory and isolation**
  - File-based memory: `~/.openclaw/workspace/memory/YYYY-MM-DD.md` (daily append-only) and `~/.openclaw/workspace/MEMORY.md` (curated).
  - Sessions persisted as JSONL transcripts.
  - Each agent has isolated workspace/session store/auth profile directories; auth profiles are not shared automatically.

## Tools and skills

- Tools use **TypeBox** (`@sinclair/typebox`) schemas with runtime validation.
- Skills are defined in `SKILL.md` files with YAML frontmatter; loaded with precedence **workspace > managed > bundled** and gated by required binaries/env/config/OS.
- Tool access is controlled per agent via allow/deny lists and presets (`minimal`, `coding`, `messaging`, `full`) plus group shorthands (`group:fs`, `group:web`, etc.).

## Security model (layering)

Described as layered controls: DM policy → group policy (mention gating) → tool policy (allow/deny) → sandboxing (Docker isolation) → exec approvals; defaults are fail-closed with explicit escape hatches.
