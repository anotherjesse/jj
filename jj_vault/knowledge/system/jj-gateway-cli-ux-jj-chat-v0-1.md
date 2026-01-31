---
id: mem_01KGAKT05ZWMD4CRA2KXC39VVA
title: 'JJ Gateway CLI UX: `jj chat` (v0.1)'
type: system
status: active
tags:
- jj
- cli
- repl
- streaming
- websocket
- idempotency
confidence: 0.8
created_at: 2026-01-31T18:07:25.375074Z
updated_at: 2026-01-31T18:07:25.375074Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKQYA2PRMPRV128VBJMAR2
supersedes: []
summary: 'Spec for `jj chat` v0.1: REPL CLI to local Gateway daemon w/ streaming deltas, session mgmt (`main`, flags), and slash cmds (/help,/session,/history,/restart).'
---
## JJ Gateway v0.1 CLI UX (`jj chat`)
- Provides a **REPL-like** chat experience backed by the local Gateway daemon.
- Must support **streaming** assistant output (via `assistant.delta` events).

### Sessions
- Default session key: `main`.
- CLI options:
  - `jj chat` (opens `main`)
  - `jj chat --session <key>`
  - `jj chat --new <key>` (optional convenience)

### In-REPL slash commands
- `/help`
- `/session <key>`: switch sessions
- `/history [n]`
- `/restart`: calls `gateway.shutdown` then reconnects
- `/model <name>`: optional; store as metadata/no-op initially

### Disconnect behavior
- Auto-reconnect and re-subscribe to the session.
- If `session.send` may have failed mid-flight, retry using the same `idempotency_key` to avoid duplicates.