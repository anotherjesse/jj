---
id: mem_01KGAKSGFMCKSNNVRD7WE6Y05D
title: JJ Gateway WebSocket protocol (v0.1)
type: system
status: active
tags:
- jj
- gateway
- websocket
- protocol
- rpc
- idempotency
confidence: 0.8
created_at: 2026-01-31T18:07:09.300065Z
updated_at: 2026-01-31T18:07:09.300065Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKQYA2PRMPRV128VBJMAR2
supersedes: []
---
## Transport
- WebSocket server bound to loopback: `127.0.0.1:<port>`.
- Messages are JSON frames.

## Framing
- Client request:
  - `{"type":"req","id":"uuid","method":"...","params":{...},"idempotency_key":"..."}`
- Server response:
  - `{"type":"res","id":"uuid","ok":true,"payload":{...}}`
- Server event:
  - `{"type":"event","event":"assistant.delta","payload":{...},"seq":123}`

## Required methods (v0.1)
- `gateway.hello`: handshake + protocol version/time.
- `session.open`: create-if-missing; returns `session_id`.
- `session.send`: append user message to transcript; schedule a run; returns ack + `message_id`.
- `session.subscribe`: register client for events for a session.
- `session.history`: return last N `message` + `assistant_final` entries.
- `gateway.shutdown`: dev-only graceful shutdown.

## Events (v0.1)
- `assistant.delta` (stream chunk)
- `assistant.final`
- `run.started`
- `run.completed`
- `error`

## Idempotency requirement
- Side-effecting requests (notably `session.send`) must accept an `idempotency_key` so client retries do not duplicate transcript entries.
