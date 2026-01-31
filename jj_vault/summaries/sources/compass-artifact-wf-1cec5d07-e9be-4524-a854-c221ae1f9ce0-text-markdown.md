---
id: mem_01KGAHMXYWZ19MFDR049SD9KPE
title: OpenClaw technical architecture deep-dive (gateway-centric agent framework)
type: source_summary
status: active
tags:
- openclaw
- gateway
- agents
- architecture
- websocket
- sessions
- serialization
- cron
- heartbeats
- tools
- skills
- typebox
- security
confidence: 0.86
created_at: 2026-01-31T17:29:42.108062Z
updated_at: 2026-01-31T17:29:42.108062Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAHKH7MWX9B27SKEBRWXNJR
supersedes: []
---
## Summary

This document describes **OpenClaw**, a local-first, **gateway-centric AI agent framework** implemented in TypeScript on Node.js (≥22). A single long-running **Gateway daemon** acts as the control plane for **all messaging, session management, scheduling, and tool execution**. The Gateway owns channel connections (WhatsApp via Baileys, Telegram via grammY, Discord via discord.js, Slack via Bolt), enforces *one Gateway per host*, and exposes a loopback WebSocket control plane at `ws://127.0.0.1:18789`.

Clients (agent loop, CLI, WebChat UI, mobile nodes, internal scheduler) communicate with the Gateway using a typed JSON WebSocket protocol with frames `req`, `res`, and server-push `event` (streaming). The first frame must be an authenticated `connect` handshake; side-effecting methods (e.g., `agent`, `send`) require **idempotency keys** for safe retries.

A core invariant is **per-session serialization**: inbound messages are routed by binding rules (channel/account/peer/guild; most-specific-wins) to an agent, then queued so only one run executes at a time per session key—analogous to a GenServer-per-session mailbox. The run pipeline is: intake/ack (`{runId, acceptedAt}`), queueing, context assembly (workspace files like `AGENTS.md`, `SOUL.md`, skills), model inference with tool calls, sandboxed tool execution with sanitization, streaming deltas over WebSocket, and persistence to JSONL transcripts with token accounting. Session keys are hierarchical (e.g., `agent:<agentId>:<channel>:dm:<peerId>`).

Scheduling is built into the Gateway via **cron jobs** and **heartbeats** (no separate scheduler daemon). Cron jobs can inject events into the main session or run in **isolated sessions** (`cron:<jobId>`) that post results back to main; jobs persist at `~/.openclaw/cron/jobs.json` and concurrency is bounded (`maxConcurrentRuns`). Heartbeats (default 30 min) run proactive checks driven by `HEARTBEAT.md`.

Coordination relies on the Gateway control plane plus optional, allowlisted **agent-to-agent tools** (`sessions_send`, `sessions_spawn`). Sub-agents run in isolated sessions (`agent:<agentId>:subagent:<uuid>`) and cannot recursively spawn further subagents by default.

Durable memory is file-based: daily append-only logs under `~/.openclaw/workspace/memory/YYYY-MM-DD.md`, curated `MEMORY.md`, and JSONL session transcripts. Each agent has an isolated workspace/session store/auth profile to prevent credential leakage.

Tools use **TypeBox** schemas for compile-time and runtime validation (with restrictions such as avoiding unions), and higher-level **Skills** are defined in `SKILL.md` with YAML frontmatter and a precedence order (workspace > managed > bundled) gated by environment/binaries/OS requirements. Tool access is controlled by per-agent allow/deny lists, presets, and tool groups.

The piece highlights patterns: single-daemon gateway control plane, provider/channel abstractions, session-based serialization, layered configuration, DI via a factory (`createDefaultDeps`), and security layering (DM/group policies → tool policy → sandbox → exec approvals). OpenClaw optimizes for single-user local deployment; horizontal scaling would require session affinity and shared state, representing a major architectural change.
