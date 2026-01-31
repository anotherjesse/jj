---
id: mem_01KGAH88MYGZT011MPWNRFZ7MQ
title: JJ Gateway v0.1 — Formal Plan / Spec (CLI + Telegram)
type: source_summary
status: active
tags:
- jj
- gateway
- spec
- cli
- telegram
- event-sourcing
- rust
confidence: 0.88
created_at: 2026-01-31T17:22:47.070283Z
updated_at: 2026-01-31T17:22:47.070283Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAH6T2E37N2SRV9HAVZX7GV
supersedes: []
---
## Summary
JJ Gateway v0.1 defines a crash-only, restart-safe local “control plane” daemon (`jj gateway`) that centralizes message intake from multiple channels (initially a local CLI REPL `jj chat` and a Telegram bot) and runs a serialized per-session agent loop with streaming responses. The system treats a **session** (identified by a stable `session_key` like `main` or `tg:<chat_id>`) as the unit of consistency: only one active run per session at a time, while multiple sessions can run concurrently with a global concurrency cap.

The core durability model is **event sourcing** via append-only per-session **transcripts** stored as JSONL files under `~/.jj/gateway/transcripts/<session_id>.jsonl`, with a small `sessions.json` index mapping `session_key → session_id` plus metadata (bindings, model config, flags). On restart, the daemon rehydrates state from `sessions.json` + transcripts. Transcript entries include a header plus minimal event types: user `message`, optional `assistant_delta` (debug), `assistant_final`, and `error`.

CLI and other clients talk to the Gateway over a loopback WebSocket protocol. Frames are JSON request/response/event objects. Required methods: `gateway.hello`, `session.open`, `session.send`, `session.subscribe`, `session.history`, and dev-only `gateway.shutdown`. Events include `run.started`, `assistant.delta`, `assistant.final`, `run.completed`, and `error`. **Idempotency keys** are required for side-effecting operations and for inbound message dedupe (particularly Telegram long-poll `getUpdates` replay). A dedupe store may live in `dedupe/` or be derived from transcript scanning.

Telegram integration is via long polling, mapping chats to `session_key = tg:<chat_id>`, ignoring non-text messages in v0.1, and using an allowlist by default. Outbound delivery uses `sendMessage` with final assistant text (streaming to Telegram is optional/deferred). Configuration lives in `~/.jj/gateway/config.toml` (or `--config`), with secrets loaded only from environment variables. The spec includes milestones (M0–M4), minimum test plan (restart safety, idempotency, Telegram dedupe, concurrency), and deliverables including a Rust workspace layout (`crates/jj-gateway`, `crates/jj-cli`, `crates/jj-proto`).
