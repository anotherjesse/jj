---
id: mem_01KGAKS7SB7NC86FA45N57WQP0
title: JJ Gateway v0.1 persistence model
type: system
status: active
tags:
- jj
- gateway
- persistence
- event-sourcing
- jsonl
- idempotency
confidence: 0.83
created_at: 2026-01-31T18:07:00.395903Z
updated_at: 2026-01-31T18:07:00.395903Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKQYA2PRMPRV128VBJMAR2
supersedes: []
summary: 'JJ Gateway v0.1 uses event-sourced storage at ~/.jj/gateway/: per-session transcripts/<session_id>.jsonl (truth) + sessions.json index.'
---
## JJ Gateway v0.1 persistence model
- Data root (default): `~/.jj/gateway/`
- Primary persistence is **event-sourced**:
  - `transcripts/<session_id>.jsonl`: append-only per-session log; treated as source of truth.
  - `sessions.json`: small index mapping stable `session_key` (e.g., `main`, `tg:<chat_id>`) to internal `session_id` plus minimal metadata.

### Transcript JSONL format (v0.1)
- First line is a `header` entry.
- Minimal entry types:
  - `header`
  - `message` (user)
  - `assistant_final`
  - optional debug: `assistant_delta`
  - `error`

### Restart safety + idempotency
- Daemon is **crash-only**; restart rehydrates from `sessions.json` + transcripts.
- Use idempotency keys on side-effecting RPCs; dedupe inbound messages (Telegram polling retries, CLI reconnect retries).
- Dedupe storage options:
  - durable `{key -> result_ref}` records in `dedupe/` with TTL pruning, or
  - embed `idempotency_key` in transcript and scan last N lines (acceptable for v0.1).