---
id: mem_01KGAFSQ3XG27NCGW306DYJN6M
title: 'Summary: JJ Gateway v0.1 — Formal Plan / Spec (CLI + Telegram)'
type: source_summary
status: active
tags:
- jj
- gateway
- spec
- cli
- telegram
- event-sourcing
- websocket
confidence: 0.86
created_at: 2026-01-31T16:57:21.789227Z
updated_at: 2026-01-31T17:09:45.339950Z
sources:
- thread_id: src_01KGAG8S2A8JWMC1EC4F390D2H
  event_ids: []
- thread_id: src_01KGAGDYPEJR4348CGAYZDXN0Y
  event_ids: []
supersedes: []
---
## Overview
This document defines **JJ Gateway v0.1**, a first shippable “Gateway + Agent Loop” that supports a local **CLI REPL** (`jj chat`) and a **Telegram bot** channel, both routed through a single **Gateway daemon** (`jj gateway`). The system is designed to be **crash-only, restart-safe**, and **replayable**, with durable sessions and transcripts.

## In-scope (v0.1)
- One long-running Gateway daemon that owns session storage, message intake (CLI + Telegram), agent execution loop (LLM calls with streaming), and outbound delivery.
- CLI client with streaming assistant responses.
- Telegram adapter using long polling (`getUpdates` → `sendMessage`).
- Persistence via `sessions.json` (index/metadata) plus **append-only per-session transcript** files (`transcripts/<session_id>.jsonl`).
- Idempotency and dedupe for side-effecting operations and inbound messages (notably Telegram).

## Core design principles
- **Gateway as control plane**: all channels feed into the same internal event flow.
- **Session = unit of consistency** with **per-session serialization** (only one run active per session key); cross-session concurrency allowed with a configurable global cap.
- **Event log is source of truth**: state is derived from append-only JSONL transcripts.
- **Crash-only + replay**: restart rehydrates from `sessions.json` + transcripts.

## Key data formats & APIs
- Storage layout defaults to `~/.jj/gateway/` with `sessions.json`, `transcripts/`, and optional `dedupe/` and `logs/`.
- Transcript JSONL includes a header line and typed entries: `message`, `assistant_delta` (debug), `assistant_final`, `error`.
- CLI ↔ Gateway transport is **WebSocket on loopback** with JSON frames: `req/res/event`.
- Required methods: `gateway.hello`, `session.open`, `session.send`, `session.subscribe`, `session.history`, and dev-only `gateway.shutdown`.
- Events: `run.started`, `assistant.delta`, `assistant.final`, `run.completed`, `error`.

## Agent loop & configuration
- Run lifecycle: acquire session lock → append user message → stream deltas to subscribers → append final assistant message.
- Provider abstraction (`stream_chat`) configured via `~/.jj/gateway/config.toml`; secrets via env vars.

## Delivery plan
Milestones M0–M4 cover scaffold, persistence, streaming agent loop, Telegram integration with dedupe, then hardening. A minimum test plan focuses on restart safety, idempotency, Telegram dedupe, and concurrency.
## Overview
This document specifies **JJ Gateway v0.1**, a first shippable “Gateway + Agent Loop” that supports two channels—**local CLI REPL (`jj chat`)** and a **Telegram bot**—routing all traffic through a single **Gateway daemon (`jj gateway`)**. The core objective is **durable, replayable sessions** using an **append-only event log** so the daemon can be killed/restarted at any time (“crash-only”) without losing state.

## Architecture & Principles
The Gateway acts as a control plane: channel adapters (CLI, Telegram) translate external messages into internal events, which are appended to a per-session **transcript JSONL**. A **session key** (e.g., `main` or `tg:<chat_id>`) is the unit of consistency: runs are **serialized per session** (one active run at a time), while different sessions can run concurrently under a global concurrency cap. State is derived from transcripts + a small `sessions.json` index.

## Persistence Model
Default data root is `~/.jj/gateway/` containing `sessions.json` and `transcripts/<session_id>.jsonl`. The JSONL transcript begins with a header line and then typed entries (minimal set: `message`, `assistant_final`, optionally `assistant_delta` for debug, plus `error`). Idempotency is required for side-effecting operations and for inbound dedupe (notably Telegram).

## Gateway Protocol (CLI-facing)
The preferred transport is loopback **WebSocket** with JSON frames. Required methods include `gateway.hello`, `session.open`, `session.send`, `session.subscribe`, `session.history`, and dev-only `gateway.shutdown`. The server emits events such as `run.started`, `assistant.delta`, `assistant.final`, `run.completed`, and `error`.

## Channel Requirements
`jj chat` provides a streaming REPL, persistent default session (`main`), session switching, history, and auto-reconnect with idempotent retries. The Telegram adapter uses **long polling** (`getUpdates`), maps chats to `tg:<chat_id>`, ignores non-text messages in v0.1, enforces an allowlist by default, and sends final responses via `sendMessage`, with durable update dedupe/offset storage.
