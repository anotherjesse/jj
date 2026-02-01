---
topic: Auto Thread Titling
date: 2026-02-01
status: decided
---

# Auto Thread Titling

## What We're Building

Auto-generate human-readable titles for threads so the sidebar is navigable instead of showing raw session keys like `session-1769930789753` and thread IDs.

**Approach: Progressive enhancement.** Show a truncated first-user-message preview immediately, then replace it with an LLM-generated title once the background call completes.

## Why This Approach

- Zero-latency feedback: the preview appears instantly when the thread is created
- LLM titles are more scannable than raw message previews (e.g., "Refactor auth flow" vs "hey can you help me refactor the auth...")
- Background call avoids blocking the chat experience

## Key Decisions

1. **Progressive title**: first-message preview -> LLM-generated title (replaces preview when ready)
2. **Storage**: title is source of truth in the thread JSONL header; session entry caches it for fast sidebar rendering
3. **No manual rename yet** — keep scope small, add click-to-edit later
4. **LLM call**: fire-and-forget after the first human message, using the cheapest available model
5. **Title event**: append a `title` event to the thread JSONL rather than rewriting the header (append-only invariant)

## Existing Infrastructure

- `ThreadSummary` already extracts `first_user_line` and `last_line` — not yet wired to `SessionEntry`
- `SessionEntry` needs a `title: Option<String>` field
- Web UI has `.session-key` and `.session-meta` slots ready for richer display
- Gateway plan already mentions "last message preview" as a planned feature

## Open Questions

- Which model for titling? Cheapest option (gpt-4o-mini or equivalent). Could also be local/offline.
- Max title length? ~60 chars feels right for sidebar display.
- Should we re-title if the conversation pivots significantly? Probably not for v1.

## Next Steps

Run `/workflows:plan` to spec out the implementation.
