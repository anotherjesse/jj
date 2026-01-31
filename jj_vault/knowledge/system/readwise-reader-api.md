---
id: mem_01KGAN18WPJ9C56S47PCTXZMPF
title: Readwise Reader API (auth, endpoints, rate limits)
type: system
status: active
tags:
- readwise
- api
- integration
confidence: 0.84
created_at: 2026-01-31T18:28:52.246056Z
updated_at: 2026-01-31T18:28:52.246056Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAMZXKVG13ZFCF7ET71Z06P
supersedes: []
---
## Authentication
- Token-based auth.
- Header format:
  - `Authorization: Token YOUR_ACCESS_TOKEN`
- Token obtained at: https://readwise.io/access_token

## Base URL
- `https://readwise.io/api/v3/`

## Rate limits
- List endpoints: **20 requests/min**.

## Key endpoints
### List documents (v3)
- `GET /api/v3/list/`

Common parameters:
- `location`: `new`, `later`, `archive`, `feed`, `inbox`, `shortlist`
- `category`: document type
- `tag`: filter by tag
- `updatedAfter`: ISO 8601 timestamp (supports incremental sync)
- `pageCursor`: pagination cursor

Response fields noted in plan:
- `title`, `author`, `url`, `source_url`
- `reading_progress` (00); suggested heuristic: **>= 85% = read**
- `saved_at`, `first_opened_at`, `last_opened_at`
- `summary`, `notes`
- `tags`, `location`

### Highlights / notes
Two approaches described:
1. **Via v3 list**: highlights returned as documents with `parent_id` set.
2. **Via v2 export**: `GET /api/v2/export/` for bulk highlights (includes notes).

Highlight fields:
- `text`, `note`, `location`, `color`, `highlighted_at`, `tags`
