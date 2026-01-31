---
id: mem_01KGAN18WPTWGXKTDPAHS25J2Y
title: 'Preference: include reading activity in AI context (via periodic sync)'
type: preference
status: active
tags:
- context
- reading
- readwise
- sync
confidence: 0.76
created_at: 2026-01-31T18:28:52.246423Z
updated_at: 2026-01-31T18:28:52.246423Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAMZXKVG13ZFCF7ET71Z06P
supersedes: []
summary: Preference to periodically sync Jesse’s reading activity (current reads + highlights/notes) into local Markdown for offline, searchable AI context without APIs.
---
## Preference
Treat Jesse’s **reading activity** (what’s being read + highlights/notes) as part of the assistant’s broader context/corpus.

## Rationale (from plan)
- Keep data **locally available** for context loading without live API calls.
- Enable offline use and simple grep/search.

## Default approach
Prefer **Option 2: periodic sync to local Markdown** first; optionally add on-demand API tools later if real-time queries become necessary.

## Guardrails / concerns
- Avoid context bloat (limit history window; consider tag-based filtering).
- Accept some staleness between sync runs.