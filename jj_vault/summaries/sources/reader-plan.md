---
id: mem_01KGAN0HVQT6Q5N0BY0AFFGZR2
title: Readwise Reader Integration Plan (source summary)
type: source_summary
status: active
tags:
- readwise
- reader
- integration
- sync
- api
confidence: 0.9
created_at: 2026-01-31T18:28:28.663651Z
updated_at: 2026-01-31T18:28:28.663651Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAMZXKVG13ZFCF7ET71Z06P
supersedes: []
---
## Summary
This document proposes integrating **Readwise Reader** data (saved documents, reading progress, highlights, notes) into Jesse’s broader “conversation corpus” so an AI assistant can use reading activity as context.

It outlines the **Readwise Reader API** (base URL `https://readwise.io/api/v3/`) with **token auth** (`Authorization: Token …`) and notes a **rate limit of 20 requests/min** for list endpoints. The key v3 endpoint is `GET /api/v3/list/`, which supports filtering/pagination via `location` (`new`, `later`, `archive`, `feed`, `inbox`, `shortlist`), `category`, `tag`, `updatedAfter` (ISO 8601; intended for incremental sync), and `pageCursor`. Responses include document metadata (title/author/URLs), timestamps (saved/first opened/last opened), `reading_progress` (0–100; suggested threshold **≥85% = “read”**), plus `summary`, `notes`, `tags`, and `location`.

For highlights/notes, two approaches are described: (1) via v3 list where highlights are returned as documents with `parent_id`, or (2) via the v2 export endpoint (`GET /api/v2/export/`) for bulk highlight export including `text`, `note`, `location`, `color`, `highlighted_at`, and tags.

Three integration options are evaluated:
1) **MCP server** using an existing repo (Readwise-Reader-MCP) for on-demand queries, but with added latency and possible mismatch to needs.
2) **Periodic sync to local markdown files** (recommended to start): scheduled script (hourly/daily) pulls updated documents since last sync, fetches highlights, and writes to a `reading/` directory (e.g., `inbox.md`, `in-progress.md`, `archive.md`, plus `reading/highlights/{doc-id}.md`). Benefits: always-available context, offline use, grep/search, version control; downside: staleness and data growth.
3) **On-demand tool** providing functions like `readwise_search`, `readwise_recent(days)`, and `readwise_highlights(url)`, with freshness but implementation and rate-limit considerations.

Implementation steps: store token as `READWISE_TOKEN`, write a sync script (Python/Rust pseudocode provided), schedule via cron/launchd, and update `CLAUDE.md` to document where reading data lives.

Open questions include how much history to sync, whether to store full text vs metadata+highlights, handling large lists to avoid bloating context, and possible tag-based filtering (e.g., only sync items tagged “work”).
