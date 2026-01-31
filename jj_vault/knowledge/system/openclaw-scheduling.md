---
id: mem_01KGAKMBWDSWT4DRET861KA07T
title: OpenClaw scheduling (cron jobs and heartbeats)
type: system
status: active
tags:
- openclaw
- cron
- heartbeat
- scheduling
confidence: 0.85
created_at: 2026-01-31T18:04:20.749408Z
updated_at: 2026-01-31T18:04:20.749408Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKJXVY906XCK7PZ4K4TCT8
supersedes: []
summary: OpenClaw schedules jobs in the Gateway process via at/every/cron triggers, running in main session or isolated cron:<jobId> sessions.
---
## Scheduling mechanisms

OpenClaw scheduling runs **inside the Gateway process** (no separate scheduler daemons).

### Cron jobs

- Supported schedule types:
  - `at` (one-shot)
  - `every` (fixed interval)
  - `cron` (cron expression)
- **Session modes**:
  - `main`: injects a system event into the existing main session (handled during the agent’s regular cycles with full context; no new instance).
  - `isolated`: runs a dedicated agent turn in a fresh session keyed as `cron:<jobId>`; each run has no prior conversation carry-over and posts results back to the main session.
- Persistence: jobs are stored in `~/.openclaw/cron/jobs.json` across Gateway restarts.
- Concurrency control: `maxConcurrentRuns`.

### Heartbeats

- Heartbeats are periodic agent turns in the **main session** (default mentioned: every 30 minutes).
- The agent reads `HEARTBEAT.md` from the workspace and either:
  - returns `HEARTBEAT_OK` (message dropped), or
  - surfaces an alert to the user.