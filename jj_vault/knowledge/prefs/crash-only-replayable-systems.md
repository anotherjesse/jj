---
id: mem_01KGAKT6CQ3MP4DKVMQNQADMWW
title: 'Preference: crash-only + replayable (event-sourced) systems'
type: preference
status: active
tags:
- jj
- preference
- reliability
- event-sourcing
- idempotency
confidence: 0.78
created_at: 2026-01-31T18:07:31.735561Z
updated_at: 2026-01-31T18:07:31.735561Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKQYA2PRMPRV128VBJMAR2
supersedes: []
summary: JJ Gateway v0.1 should be crash-only and replayable via append-only per-session JSONL event logs, plus idempotency keys for retries/dedupe.
---
## Statement
For JJ Gateway v0.1, the intended operating model is **crash-only** and **replayable**:
- The gateway daemon can be killed/restarted at any time.
- State continuity comes from an **append-only event log** (per-session transcript JSONL) plus a small session index.
- The system should be safe under retries and partial failures via **idempotency keys** and dedupe.

## Implications
- Prefer designs where restart is the recovery mechanism.
- Keep state derivable from persisted logs rather than in-memory mutations.
- Ensure client reconnect/retry does not create duplicate messages or side effects.