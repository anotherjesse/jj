---
id: mem_01KGAHB6MZ6CGJNQKQSTC1JQTY
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
- websocket
confidence: 0.9
created_at: 2026-01-31T17:24:23.327903Z
updated_at: 2026-01-31T17:24:23.327903Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAH9QWXDTVYBKPNG42FDGDE
supersedes: []
---
## Summary

JJ Gateway v0.1 specifies a first shippable “Gateway + Agent Loop” system with two message channels—local CLI (`jj chat`) and Telegram bot—both routed through a single local daemon (`jj gateway`). The Gateway is the control plane: it manages sessions, persists transcripts, coordinates a per-session serialized agent run loop (LLM streaming), and routes outbound responses back to the originating channel.

Core design principles: session key is the unit of consistency; all state is derived from an append-only event log (JSONL transcript per session); the daemon is crash-only and restart-safe via replay/rehydration from `sessions.json` + transcripts; idempotency is required to make retries safe and prevent duplicate side effects (notably for Telegram update delivery and client reconnect retries).

Architecture components: Gateway Core (session manager, run coordinator, model provider interface, internal event bus/router), CLI Adapter (WebSocket client with streaming display and reconnect/resubscribe), and Telegram Adapter (long polling via `getUpdates`, mapping `tg:<chat_id>` to session keys, dedupe via durable offset/update tracking, and outbound `sendMessage`). Concurrency model: one active run per session key; cross-session concurrency allowed with a configurable global cap.

Persistence layout defaults to `~/.jj/gateway/` with `sessions.json` as a small index (metadata + channel bindings) and `transcripts/<session_id>.jsonl` as source-of-truth logs (header + typed entries: `message`, `assistant_delta` optional, `assistant_final`, `error`). Idempotency can be implemented with a dedicated `dedupe/` store or via scanning recent transcript lines.

Gateway API for CLI: JSON frames over loopback WebSocket (`gateway.hello`, `session.open`, `session.send`, `session.subscribe`, `session.history`, dev-only `gateway.shutdown`), and events (`run.started`, `assistant.delta`, `assistant.final`, `run.completed`, `error`). Config lives at `~/.jj/gateway/config.toml` with sections for gateway, model provider, and Telegram; secrets must be env-var only. Deliverables include Rust workspace layout, tests (persistence, idempotency, WS happy path, Telegram dedupe with mocks), and docs (quickstart/protocol/storage).
