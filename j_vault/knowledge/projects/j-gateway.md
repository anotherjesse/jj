---
id: mem_01KGAKS027QAMC0YNFF37E2F8F
title: J Gateway
type: project
status: active
tags:
- j
- gateway
- cli
- telegram
- rust
- event-sourcing
- websocket
confidence: 0.82
created_at: 2026-01-31T18:06:52.487514Z
updated_at: 2026-01-31T18:06:52.487514Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKQYA2PRMPRV128VBJMAR2
supersedes: []
summary: 'J Gateway v0.1: local-first daemon plus CLI REPL and Telegram bot routing, with durable replayable sessions and an agent loop.'
---
## Overview
J Gateway v0.1 is a **local-first control-plane daemon** (`j gateway`) plus a **CLI REPL** (`j chat`) and a **Telegram bot adapter**. The daemon owns session storage, agent execution, and routing messages between channels.

## Goals (v0.1)
- Ship a first working “Gateway + Agent Loop”.
- Support two channels:
  - Local CLI REPL (`j chat`) with streaming assistant output.
  - Telegram bot (long polling) routed through the same gateway.
- Ensure **durable, replayable sessions** (crash-only, restart-safe).

## Core design principles
- **Gateway as control plane**: all channel messages flow through the daemon.
- **Session as unit of consistency**: one serialized agent run per `session_key`.
- **Event log as source of truth**: state derived from append-only transcripts.
- **Crash-only + replayable**: restart rehydrates from `sessions.json` + transcript JSONL.
- **Idempotency** for side-effecting ops and inbound message dedupe.

## Architecture (v0.1)
- Gateway core: session manager, run coordinator (per-session lock), model-provider interface (streaming), internal event bus, channel router.
- CLI adapter: connects to gateway over loopback WebSocket JSON protocol.
- Telegram adapter: `getUpdates` long-poll loop; maps `chat_id` to `session_key` (`tg:<chat_id>`); delivers via `sendMessage`.

## Persistence
Default data dir: `~/.j/gateway/`
- `sessions.json`: small index mapping `session_key -> session_id` + metadata.
- `transcripts/<session_id>.jsonl`: append-only per-session event log.
- Optional: `dedupe/` idempotency records, `logs/` raw stream capture.

## Protocol surface (CLI)
- Transport: WebSocket on `127.0.0.1:<port>`.
- Frames: `{type:req|res|event,...}`.
- Methods: `gateway.hello`, `session.open`, `session.send`, `session.subscribe`, `session.history`, dev-only `gateway.shutdown`.
- Events: `assistant.delta`, `assistant.final`, `run.started`, `run.completed`, `error`.

## Configuration
Config file: `~/.j/gateway/config.toml` with sections `[gateway]`, `[model]`, `[telegram]`. Secrets (model API key, Telegram token) are loaded via environment variables and must not be written to transcripts/logs.

## Milestones
M0 scaffold → M1 sessions+persistence → M2 streaming agent loop → M3 Telegram adapter → M4 hardening (idempotency, retry, shutdown).