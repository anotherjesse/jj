---
title: "feat: Auto Thread Titling"
type: feat
date: 2026-02-01
---

# feat: Auto Thread Titling

Auto-generate human-readable titles for threads so the sidebar shows scannable names instead of raw session keys and thread IDs. Uses progressive enhancement: truncated first-message preview appears immediately, LLM-generated title replaces it when ready.

## Acceptance Criteria

- [x] After first user message, sidebar shows truncated preview (~60 chars) of that message
- [x] Background LLM call fires after first user message to generate a short title
- [x] When title arrives, sidebar updates in real-time via WebSocket
- [x] Title persisted as `TitleGenerated` event in thread JSONL (append-only)
- [x] `SessionEntry` caches title for fast sidebar rendering on reload
- [x] Existing threads without titles gracefully fall back to `first_user_line` or session_key
- [x] Title rendered with `esc()` to prevent XSS

## Implementation

### Phase 1: Data Model

#### `src/thread_store.rs`

Add `TitleGenerated` to `EventType` enum:

```rust
pub enum EventType {
    // ... existing variants
    TitleGenerated,
}
```

#### `src/gateway/session.rs`

Add `title` to `SessionEntry`:

```rust
pub struct SessionEntry {
    pub session_key: String,
    pub thread_id: String,
    pub thread_path: String,
    pub created_at: String,
    pub title: Option<String>,          // LLM-generated title
    pub first_user_line: Option<String>, // truncated preview fallback
}
```

On session load, scan thread JSONL for last `TitleGenerated` event and `first_user_line`. Cache both in `SessionEntry`.

### Phase 2: Title Generation

#### `src/agent.rs` (or new `src/title.rs`)

Follow the `deep_think_background` pattern:

1. After first `UserMessage` is appended, check if thread already has a title
2. If no title, spawn background thread:

```rust
std::thread::spawn(move || {
    let client = OpenAIClient::from_env_with_model("gpt-5-mini-2025-08-07");
    let messages = vec![
        system("Generate a concise title (max 8 words) for this conversation. Return only the title, nothing else."),
        user(&first_message),
    ];
    let resp = client.chat(&messages, &[])?;
    let title = resp.content.unwrap_or_default().trim().to_string();
    // Truncate to 100 chars max
    let title = if title.len() > 100 { title[..100].to_string() } else { title };
    // Append TitleGenerated event to thread JSONL
    append_event(&thread_path, &title_event)?;
    // Update SessionEntry cache + broadcast to WebSocket subscribers
    // via event_sink channel
});
```

3. Use `AtomicBool` guard per-session to prevent duplicate generation
4. On failure: log warning, leave preview in place, no retry

### Phase 3: Gateway Protocol

#### `src/gateway/ws.rs`

- `session.list` response: include `title` and `first_user_line` fields in each entry
- Add `title_generated` event type to WebSocket broadcasts
- When background thread completes, broadcast: `{ "type": "event", "event": "title_generated", "session_id": "...", "payload": { "title": "..." } }`

### Phase 4: Web UI

#### `web/index.html`

Update `renderSessionList`:

```javascript
// Display priority: title > first_user_line > session_key
const displayName = s.title || s.first_user_line || s.session_key;
div.innerHTML = `<div class="session-key">${esc(displayName)}</div>
                 <div class="session-meta">${esc(s.thread_id)}</div>`;
```

Add WebSocket handler for `title_generated` event:

```javascript
case 'title_generated':
    // Find matching session in sidebar, update display name
    updateSessionTitle(msg.session_id, msg.payload.title);
    break;
```

### Phase 5: Session Load (Backward Compat)

When `SessionManager` loads sessions on startup:

1. For each session, scan thread JSONL for last `TitleGenerated` event → set `title`
2. If no title, extract `first_user_line` from thread → set as fallback
3. Persist updated `sessions.json` with cached titles

Existing threads: show `first_user_line` or `session_key`. No backfill job for v1.

## Edge Cases

| Scenario | Behavior |
|----------|----------|
| LLM call fails | Log warning, preview persists, no retry |
| Empty first message | Skip title generation, show session_key |
| Server restart mid-generation | Title lost, preview persists until next first-message (already titled threads unaffected) |
| Multiple tabs open | All receive `title_generated` WebSocket event simultaneously |
| Multiple `TitleGenerated` events | Last one wins (most recent in JSONL) |
| Title > 100 chars | Truncated server-side before storage |

## Context

- Brainstorm: `docs/brainstorms/2026-02-01-auto-thread-titling-brainstorm.md`
- Background task pattern: `src/agent.rs:739-780` (deep_think)
- Session rendering: `web/index.html:242-253`
- SessionEntry: `src/gateway/session.rs:18-24`
- ThreadSummary (has first_user_line): `src/thread_store.rs:222-287`
- Event appending: `src/thread_store.rs:108-123`
