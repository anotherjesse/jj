---
id: mem_01KGAN18WNQXG42B1TK10YDWAD
title: Readwise Reader integration (periodic sync + optional tools)
type: project
status: active
tags:
- readwise
- reader
- integration
- sync
- context
confidence: 0.82
created_at: 2026-01-31T18:28:52.245244Z
updated_at: 2026-01-31T18:28:52.245244Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAMZXKVG13ZFCF7ET71Z06P
supersedes: []
---
## Goal
Integrate **Readwise Reader** activity (documents, reading progress, highlights, notes) into Jesse’s AI context / “conversation corpus”.

## Recommended approach
Start with **periodic sync to local Markdown files** (hourly/daily) so reading data is always available for context loading and offline search. Consider adding an on-demand tool later for freshness.

## Proposed data layout
```text
reading/
├── inbox.md
├── in-progress.md
├── archive.md
└── highlights/
    └── {doc-id}.md
```

## Sync behavior (proposed)
- Track `last_sync` timestamp.
- Fetch documents updated since last sync (`updatedAfter` ISO 8601).
- Fetch highlights/notes per document (either via v3 list w/ `parent_id` items or v2 export for bulk).
- Write/overwrite Markdown outputs.

## Open questions
- History window (e.g., last 30 days vs all time).
- Store full text vs metadata + highlights.
- Tag/location filtering to limit context bloat (e.g., only tag `work`).

## Future (optional) on-demand tool interface
- `readwise_search(query)`
- `readwise_recent(days=7)`
- `readwise_highlights(url)`
