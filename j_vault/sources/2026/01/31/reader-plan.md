---
id: src_01KGAMZXKVG13ZFCF7ET71Z06P
title: reader-plan
ingested_at: 2026-01-31T18:28:07.931510+00:00
original_path: /Users/jesse/cto/ideas/reader-plan.md
tags: []
processing_status: complete
content_hash: sha256:fc1d7a3bbe3d6da3d7f67e693e91235da6bc009f2a60ee54e374c29eee88bf4f
---
# Readwise Reader Integration Plan

## Goal

Give Claude access to Jesse's reading activity - what's being read, highlights, notes - as part of the broader "conversation corpus" for AI context.

## Readwise Reader API Overview

**Auth:** Token-based, get at https://readwise.io/access_token
```
Authorization: Token YOUR_ACCESS_TOKEN
```

**Base URL:** `https://readwise.io/api/v3/`

**Rate Limits:** 20 requests/min for list endpoints

## Key Endpoints

### Fetch Documents
```
GET /api/v3/list/
```

Parameters:
- `location` - `new`, `later`, `archive`, `feed`, `inbox`, `shortlist`
- `category` - document type
- `tag` - filter by tag
- `updatedAfter` - ISO 8601 date (for incremental sync)
- `pageCursor` - pagination

Response includes:
- `title`, `author`, `url`, `source_url`
- `reading_progress` (0-100) - use >= 85 as "read"
- `saved_at`, `first_opened_at`, `last_opened_at`
- `summary`, `notes`
- `tags`, `location`

### Fetch Highlights/Notes
Two approaches:

1. **Via v3 list** - Highlights are documents with `parent_id` set
2. **Via v2 export** - `GET /api/v2/export/` for bulk highlights with notes

Highlight fields: `text`, `note`, `location`, `color`, `highlighted_at`, `tags`

## Integration Options

### Option 1: MCP Server (Quickest)

Already exists: [Readwise-Reader-MCP](https://github.com/edricgsh/Readwise-Reader-MCP)

Pros:
- Direct Claude integration
- Already built
- Query on-demand

Cons:
- Adds latency to queries
- May not match our exact needs

### Option 2: Periodic Sync to Local Files

Script that runs daily/hourly:
1. Fetch recent documents (`updatedAfter` for incremental)
2. Fetch highlights for those documents
3. Write to `reading/` as markdown files

```
reading/
├── inbox.md          # Currently saving to read
├── in-progress.md    # Started but not finished
├── archive.md        # Finished reading
└── highlights/
    └── {doc-id}.md   # Highlights + notes per article
```

Pros:
- Always available in context
- Can be version controlled
- Works offline

Cons:
- Stale until next sync
- Storage grows over time

### Option 3: On-Demand Tool

Custom tool that Claude can call:
- `readwise_search(query)` - search saved articles
- `readwise_recent(days=7)` - what's been read recently
- `readwise_highlights(url)` - get highlights for specific article

Pros:
- Fresh data
- Only fetches what's needed

Cons:
- Requires tool implementation
- Rate limits could be hit

## Recommended Approach

**Start with Option 2 (periodic sync)** because:
1. Simplest to implement
2. Data is always available for context loading
3. Can grep/search without API calls
4. Natural fit with how this system already works

Later, add Option 3 for real-time queries if needed.

## Implementation Steps

1. **Store API token** - Add `READWISE_TOKEN` to environment or secure storage

2. **Write sync script** (Python or Rust)
   ```python
   # Pseudocode
   def sync_readwise():
       last_sync = read_last_sync_time()
       docs = fetch_documents(updated_after=last_sync)
       for doc in docs:
           highlights = fetch_highlights(doc.id)
           write_markdown(doc, highlights)
       write_last_sync_time(now())
   ```

3. **Schedule** - cron job or launchd, hourly or on-demand

4. **Update CLAUDE.md** - Tell Claude where reading data lives and how to use it

## Data We'd Capture

For each article/document:
- Title, author, source URL
- When saved, when read, reading progress
- All highlights with:
  - Highlighted text
  - Any notes added
  - Color (if meaningful)
  - Position in document

## Example Output

```markdown
# In Progress Reading

## The Incorruptible - Eric Ries
- **Saved:** 2025-01-20
- **Progress:** 45%
- **Last opened:** 2025-01-23

### Highlights
> "Financial gravity is the invisible force that pulls every company toward short-term shareholder primacy."

Note: This connects to the Boeing/Jack Welch discussion

> "The governance structure IS the strategy."

---
```

## Questions to Resolve

- How much history to sync? (Last 30 days? All time?)
- Include full article text or just metadata + highlights?
- How to handle large reading lists without bloating context?
- Tag-based filtering? (e.g., only sync items tagged "work")

## Related

- Broader goal: [Conversation recording for AI context](../priorities.md)
- Similar to Granola integration for meeting transcripts
- Complements the `yt` tool for video consumption tracking

---
*Created: 2025-01-24*
