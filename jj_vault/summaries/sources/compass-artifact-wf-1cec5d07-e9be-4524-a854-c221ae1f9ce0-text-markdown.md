---
id: mem_01KGAKMBWBN9T7YVXKMJ9X442P
title: OpenClaw architecture deep-dive (gateway-centric agent framework)
type: source_summary
status: active
tags:
- openclaw
- architecture
- agents
- gateway
- websocket
- scheduling
- tools
confidence: 0.9
created_at: 2026-01-31T18:04:20.747897Z
updated_at: 2026-01-31T18:04:20.747897Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKJXVY906XCK7PZ4K4TCT8
supersedes: []
---
## Summary

OpenClaw is a **gateway-centric AI agent framework** designed for an “always-on”, local-first personal assistant. A single long-running **Gateway daemon** (Node.js ≥22, TypeScript) serves as the **control plane** for channel connectivity, session management, routing, scheduling, and tool execution. All messaging channels (e.g., WhatsApp via Baileys, Telegram via grammY, Discord via discord.js, Slack via Bolt) terminate in the Gateway, which exposes a typed **WebSocket control plane** at `ws://127.0.0.1:18789`. The protocol uses three frame types—`req`, `res`, `event`—and requires an initial authenticated `connect` handshake; side-effecting methods require **idempotency keys** for safe retries.

Incoming messages are routed to agents using **binding rules** (most-specific match by channel/account/peer/guild). The core **agent loop** returns early acceptance (`{runId, acceptedAt}`), then serializes work through a **per-session queue**, preventing concurrency races (analogous to GenServer-per-session). Sessions are hierarchically keyed, e.g., `agent:<agentId>:<channel>:dm:<peerId>` and `agent:<agentId>:<channel>:group:<id>`. The run pipeline includes context assembly from workspace files (AGENTS.md, SOUL.md), LLM inference with streaming, sandboxed tool execution with sanitization, event streaming of deltas, and persistence to JSONL transcripts with token accounting.

Scheduling runs inside the Gateway via **cron jobs** and **heartbeats**. Cron supports `at`, `every`, and `cron` expression schedules, with two modes: **main session** jobs inject system events into the ongoing main session, while **isolated** jobs create fresh sessions (`cron:<jobId>`) whose results are posted back. Heartbeats (default every 30 minutes) read `HEARTBEAT.md` to decide between emitting alerts or returning `HEARTBEAT_OK` (dropped).

Durable memory is file-based (`~/.openclaw/workspace/memory/YYYY-MM-DD.md`, `MEMORY.md`, JSONL session transcripts). Each agent has an isolated workspace/session store/auth profile directory to prevent credential leakage.

Tools are defined with **TypeBox** schemas and governed by a layered **skill** system (`SKILL.md` with YAML frontmatter). Skills are loaded with precedence (workspace > managed > bundled) and gated by environment/binaries/config/OS. Tool access is controlled via per-agent allow/deny lists and presets (`minimal`, `coding`, `messaging`, `full`), plus groups (e.g., `group:fs`, `group:web`). Coordination primarily occurs through Gateway RPC; optional agent-to-agent tools (`sessions_send`, `sessions_spawn`) are disabled by default and allowlisted when enabled, with recursion prevented for spawned sub-agents.
