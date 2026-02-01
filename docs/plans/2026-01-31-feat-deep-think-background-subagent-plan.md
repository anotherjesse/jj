---
title: "feat: Add deep_think background subagent tool"
type: feat
date: 2026-01-31
---

# feat: Add deep_think background subagent tool

## Overview

Add a `deep_think` tool that spawns a background LLM call on a slower/more powerful model. The background job reads the recent conversation, optionally calls tools (knowledge_search), and returns "inner monologue" text. This text gets queued as an internal event and silently incorporated by the fast model on its next turn — the user never sees it directly.

This is the first step toward a dual-model "thinking fast and slow" architecture where the fast model (mouth) keeps conversation flowing while a slow model (subconscious) periodically deepens understanding.

## Problem Statement / Motivation

The fast model is great for low-latency voice/text chat but sometimes lacks depth. A background process that periodically reflects on the conversation — searching knowledge, noticing patterns, forming hypotheses — would make the fast model's responses richer without adding latency to the main loop.

## Proposed Solution

### Tool: `deep_think`

The fast model calls `deep_think` with an optional prompt. This spawns a background job that:

1. Builds a context window from the recent thread transcript
2. Calls a slower model with a "you are the inner voice" system prompt
3. Optionally runs an agentic sub-loop (knowledge_search, thread_read)
4. Returns inner monologue text
5. Result is appended to the thread as a `SystemNote` with a `deep_think` marker
6. Fast model sees it on its next turn as context (not shown to user)

### Concurrency

- **One in-flight deep_think per session.** If a new one is triggered while one is running, set a `rerun_needed` flag and re-run after completion.
- Deep think runs on a **separate semaphore** from the main agent run — it must not block the fast model from responding to user messages.

### Auto-triggering (v0.2)

After N user turns without an explicit `deep_think` call, the system auto-triggers one. For v0.1, only manual triggering via the tool.

## Technical Considerations

### Architecture

The main agent loop (`run_agent_loop` in `src/agent.rs`) is synchronous and runs inside `spawn_blocking`. Deep think needs its own execution path:

```
User message → fast model responds (may call deep_think tool)
                                          ↓
                              deep_think returns immediately: {"status": "started"}
                                          ↓
                              Background: spawn_blocking runs slow model
                                          ↓
                              On completion: append SystemNote to thread
                                          ↓
                              Next fast-model turn picks it up from history
```

### New EventType variant

Add `InnerMonologue` to `EventType` enum (or reuse `SystemNote` with a convention like content starting with `[deep_think]`). Using a dedicated variant is cleaner for filtering.

### Thread event format

```json
{
  "type": "inner_monologue",
  "role": "system",
  "content": "hmm: The user seems to be exploring how to structure their API layer...\n\nthoughts: Based on the knowledge docs about their REST conventions and the recent discussion about authentication, they likely want...",
  "tool_name": "deep_think",
  "reason": "periodic reflection after 5 user turns"
}
```

### Message building

In `load_history` / message construction (`src/chat.rs`), `InnerMonologue` events get injected as system messages with a prefix like `[inner thoughts — not spoken aloud]`. The fast model's system prompt tells it to incorporate these silently.

### Model selection

Deep think should use a configurable model, defaulting to the same model or a stronger one. Read from `OPENAI_DEEP_THINK_MODEL` env var, falling back to the session's model.

### Files to modify

| File | Change |
|------|--------|
| `src/thread_store.rs:12` | Add `InnerMonologue` to `EventType` enum |
| `src/agent.rs:176` | Add `deep_think` tool schema in `tool_schemas()` |
| `src/agent.rs:350` | Add `deep_think` handler in `execute_tool()` |
| `src/chat.rs:70` | Include `InnerMonologue` events in message building |
| `src/gateway/session.rs` | Add deep-think semaphore + background spawn logic |
| `src/openai.rs` | No changes — reuse `OpenAIClient` with different model |

## Acceptance Criteria

- [x] `deep_think` tool schema registered in `tool_schemas()`
- [x] Fast model can call `deep_think` with optional `prompt` and `reason`
- [ ] Tool returns immediately with `{"status": "started"}` (non-blocking) — v0.1 is synchronous within agent loop
- [x] Background job calls slow model with recent transcript + inner-voice system prompt
- [ ] Background job can call `knowledge_search` in a sub-loop (1-2 tool turns max) — deferred to v0.2
- [x] Result appended to thread as `InnerMonologue` event
- [x] Fast model incorporates inner monologue on next turn (visible in message array as system context)
- [x] Inner monologue NOT shown to user in chat output
- [ ] Only 1 deep_think job in-flight per session — deferred to v0.2 (async)
- [x] Errors in deep_think logged but don't crash the session

## Success Metrics

- Fast model produces noticeably more contextual responses after deep_think runs
- No added latency to the main chat loop
- Deep think completes within 30s typically

## Dependencies & Risks

**Dependencies:**
- Existing `OpenAIClient` and tool execution infrastructure
- Thread append-only JSONL format

**Risks:**
- **Staleness**: Deep think result may be outdated by the time it completes. Mitigated by: result includes the event_id it was triggered after, and fast model can assess relevance.
- **Cost**: Slow model calls are expensive. Mitigated by: manual-only triggering in v0.1, rate limiting later.
- **Noise**: Bad inner monologue could mislead the fast model. Mitigated by: the fast model's system prompt says to treat inner thoughts as suggestions, not directives.

## MVP

### src/thread_store.rs — EventType addition

```rust
pub enum EventType {
    UserMessage,
    AssistantMessage,
    ToolCall,
    ToolResult,
    SystemNote,
    AttachmentAdded,
    InnerMonologue,  // NEW
}
```

### src/agent.rs — Tool schema

```rust
json!({
    "type": "function",
    "function": {
        "name": "deep_think",
        "description": "Trigger background deep thinking. Spawns a slower model to reflect on the conversation, search knowledge, and produce inner monologue. Results appear as internal context on your next turn. Use when the conversation would benefit from deeper analysis, pattern recognition, or knowledge retrieval.",
        "parameters": {
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "What to think about. If omitted, reflects on the overall conversation."
                },
                "reason": { "type": "string" }
            },
            "required": ["reason"]
        }
    }
})
```

### src/agent.rs — Tool handler (sketch)

```rust
"deep_think" => {
    let prompt = args.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
    let reason = args.get("reason").and_then(|v| v.as_str()).unwrap_or("");

    // Read recent thread events for context
    let events = read_thread(thread_path, None, Some(50))?;

    // Build inner-voice messages
    let system = "You are the inner voice of an AI assistant named JJ. \
        You are thinking privately — nothing you say will be shown to the user. \
        Reflect on the conversation. Note patterns, form hypotheses, \
        identify what you know vs don't know, and suggest what to explore. \
        Be concise but thorough.";

    let mut messages = vec![json!({"role": "system", "content": system})];
    // ... build from events ...
    if !prompt.is_empty() {
        messages.push(json!({"role": "user", "content": format!("Focus on: {prompt}")}));
    }

    // Call slow model (blocking for v0.1, async spawn for v0.2)
    let deep_model = std::env::var("OPENAI_DEEP_THINK_MODEL")
        .unwrap_or_else(|_| "gpt-5.2-2025-12-11".to_string());
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com".to_string());

    let client = OpenAIClient::new(api_key, base_url, deep_model);
    let response = client.chat(&messages, &[])?; // no tools for v0.1

    let monologue = response.content.unwrap_or_default();

    // Append as InnerMonologue event
    let event = build_event(
        None,
        EventType::InnerMonologue,
        Role::System,
        Some(Value::String(monologue.clone())),
        Some("deep_think".to_string()),
        None, None,
        Some(reason.to_string()),
    );
    append_event(thread_path, event)?;

    Ok(json!({ "status": "ok", "length": monologue.len() }))
}
```

### src/chat.rs — Message building (inner monologue injection)

```rust
// In load_history or message building, when event_type is InnerMonologue:
EventType::InnerMonologue => {
    if let Some(content) = &event.content {
        messages.push(json!({
            "role": "system",
            "content": format!("[inner thoughts — not spoken aloud]\n{}", content)
        }));
    }
}
```

## Future Considerations (v0.2+)

- **Auto-triggering**: System triggers deep_think after N user turns without one
- **Async spawn**: Return `{"status": "started"}` immediately, run in background thread
- **Tool access**: Let deep_think's sub-loop call knowledge_search, thread_read
- **Router**: Decide which tools to give deep_think based on conversation topic
- **Condensation**: When inner monologue accumulates too much, run a summarization pass
- **Canvas integration**: Deep think could decide to draw diagrams as part of its reflection

## References

- Tool registration pattern: `src/agent.rs:176-347`
- Tool execution pattern: `src/agent.rs:350-593`
- Thread events: `src/thread_store.rs:12-48`
- Session management: `src/gateway/session.rs:159-258`
- Existing knowledge on this feature: `knowledge/system/dual-model-subconscious-loop.md`
- Documented learning on tool schemas: `docs/solutions/integration-issues/untyped-tool-schemas-cause-empty-llm-output.md`
