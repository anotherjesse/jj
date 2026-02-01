---
id: mem_01KGAKMBWDGVFAH74HN8TZR9WM
title: OpenClaw Gateway control plane (WebSocket RPC)
type: system
status: active
tags:
- openclaw
- websocket
- rpc
- idempotency
confidence: 0.86
created_at: 2026-01-31T18:04:20.749115Z
updated_at: 2026-01-31T18:04:20.749115Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKJXVY906XCK7PZ4K4TCT8
supersedes: []
summary: Defines OpenClaw's single-per-host Gateway daemon control plane exposing ws://127.0.0.1:18789 with req/res/event framing and auth connect handshake.
---
## Gateway control plane

- OpenClaw uses a single long-running **Gateway daemon** as the control plane.
- The Gateway exposes a WebSocket endpoint: `ws://127.0.0.1:18789`.
- The Gateway hard-enforces **one Gateway per host**.

## WebSocket framing protocol

- Frame types:
  - `req`: client request with `method` and `params`.
  - `res`: server response.
  - `event`: server-push messages used for streaming/deltas.
- The first frame must be a `connect` handshake with authentication; the Gateway closes the connection otherwise.
- Side-effecting RPC methods (e.g., `agent`, `send`) require **idempotency keys** so clients can safely retry.

## Session routing and serialization

- Routing uses binding rules with “most-specific-wins” matching by channel/account/peer/guild.
- Agent runs are **serialized per session key** via a per-session queue to prevent concurrent state/tool races.
- Example session key formats:
  - DMs: `agent:<agentId>:<channel>:dm:<peerId>`
  - Groups: `agent:<agentId>:<channel>:group:<id>`