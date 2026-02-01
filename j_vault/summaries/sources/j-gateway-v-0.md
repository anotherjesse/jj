---
id: mem_01KGAKRM61GAEGM8XQQNKDBZ1V
title: J Gateway v0.1 — Formal Plan / Spec (CLI + Telegram)
type: source_summary
status: active
tags:
- j
- gateway
- spec
- cli
- telegram
- event-sourcing
- websocket
confidence: 0.84
created_at: 2026-01-31T18:06:40.321382Z
updated_at: 2026-01-31T18:06:40.321382Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKQYA2PRMPRV128VBJMAR2
supersedes: []
---
## Summary
This document specifies **J Gateway v0.1**, a first shippable “Gateway + Agent Loop” with two user-facing channels: a local **CLI REPL** (`j chat`) and a **Telegram bot**. The **Gateway daemon** (`j gateway`) is the control plane: all inbound/outbound messages flow through it, and it owns persistence, session coordination, and the agent execution loop.

Core design principles are **event sourcing** and **crash-only** operation: each session has an **append-only JSONL transcript** (source of truth) and a small **sessions index** (`sessions.json`) mapping stable `session_key` (e.g., `main`, `tg:<chat_id>`) to internal `session_id` and metadata. The system must be **restart-safe**, with **idempotency keys** on side-effecting operations and inbound message dedupe (especially for Telegram polling retries).

Architecture includes: (1) Gateway core (session manager, run coordinator with per-session serialization, model-provider interface, internal event bus, channel router), (2) CLI adapter speaking a Gateway API, and (3) Telegram adapter using **long polling** (`getUpdates`) and delivering responses via `sendMessage`. Concurrency is **serialized per session key**; different sessions may run concurrently with a configurable global cap.

The **Gateway API** for CLI is a loopback **WebSocket** JSON protocol with `req/res/event` frames. Required v0.1 methods: `gateway.hello`, `session.open`, `session.send`, `session.subscribe`, `session.history`, and dev-only `gateway.shutdown`. Key events include `assistant.delta`, `assistant.final`, `run.started`, `run.completed`, and `error`. CLI requirements include streaming output, persistent default session, session switching, reconnect/resubscribe, and retrying sends with the same idempotency key.

Telegram v0.1 ignores non-text messages, supports `/start`/`/help`, uses a default-deny allowlist, and stores durable poll offset/update dedupe state. Configuration is via `~/.j/gateway/config.toml` (gateway, model, telegram sections), with secrets loaded from environment variables and never written to transcripts/logs. The plan defines milestones (M0–M4), minimum tests (restart safety, idempotency, telegram dedupe, concurrency), repository layout (Rust workspace crates), and required docs (quickstart/protocol/storage).
