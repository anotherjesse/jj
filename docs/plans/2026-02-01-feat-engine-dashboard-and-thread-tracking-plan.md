---
title: "Engine Dashboard, Session Override & Per-Message Tracking"
type: feat
date: 2026-02-01
---

# Engine Dashboard, Session Override & Per-Message Tracking

## Overview

Replace the empty chat window shown on initial web UI load with an engine dashboard displaying available engines, their configuration, and current defaults. Allow users to change the active engine per-session (runtime only). Record engine provenance at both thread and message level for auditability and analysis.

## Problem Statement / Motivation

After the multi-engine abstraction landed, there's no way to:
- See which engines are available and configured
- Know which engine is currently active
- Change engines without restarting with different env vars
- Know which engine produced a given response in history

The web UI also shows an empty chat window before any session is opened, which is confusing.

## Proposed Solution

### 1. Engine Dashboard (Web UI landing page)

When no session is active, render a dashboard showing:

| Engine | Status | Model | Base URL |
|--------|--------|-------|----------|
| OpenAI | Available | gpt-5-mini-2025-08-07 | https://api.openai.com |
| Anthropic | No API key | — | — |
| Gemini | Available | gemini-2.0-flash | https://generativelanguage.googleapis.com |

- **Active engine** highlighted with a visual indicator
- Deep think engine shown separately if overridden
- Custom base URLs shown as "(custom)" badge (never expose API keys)

Backed by a new `engine.list` RPC method that checks env var presence (no network calls).

### 2. Per-Session Engine Override

Add `engine.set` RPC method. Changes the active engine for one session until changed again or the session ends. Not persisted to config files — lost on server restart.

```
→ { type: "req", method: "engine.set", params: { session_key: "...", engine: "anthropic" } }
← { type: "res", ok: true, payload: { engine: "anthropic", model: "claude-sonnet-4-20250514" } }
```

Validation: reject if the requested engine has no API key configured.

When switching sessions, the UI snaps to whichever engine that session is using.

### 3. Thread-Level Engine Recording

Extend `ThreadHeader` to record the engine at thread creation:

```rust
pub struct ThreadHeader {
    // ... existing fields ...
    pub engine: Option<String>,     // "openai" | "anthropic" | "gemini"
    pub base_url: Option<String>,   // for custom endpoints
    // model already exists
}
```

### 4. Per-Message Engine Attribution

Extend `ThreadEvent` to record which engine produced each assistant response:

```rust
pub struct ThreadEvent {
    // ... existing fields ...
    pub engine: Option<String>,
    pub model: Option<String>,
}
```

Only populated on `AssistantMessage` and `ToolCall` events (where the LLM is the actor).

### 5. Cross-Engine History Replay

When engine changes mid-thread, tool call/result message formats may be incompatible. Strategy: strip tool call/result events from replayed history when sending to a different engine than the one that generated them. The raw thread JSONL retains everything.

## Technical Considerations

**Data model changes (backward-compatible):**
- All new fields are `Option<T>` with `#[serde(default)]` — old threads parse fine
- `SessionEntry` gets `engine: Option<String>`, `model: Option<String>` for sidebar display
- `SessionState` gets `engine_override: Option<EngineKind>` (runtime only)

**Gateway changes:**
- `engine.list` RPC: enumerate engines, check env var presence, return config (minus keys)
- `engine.set` RPC: validate + update `SessionState.engine_override`
- `run_agent_blocking` (`src/gateway/session.rs:553`): accept engine override from session state instead of always calling `create_engine()` from env
- `SessionManager::open` (`src/gateway/session.rs:146-148`): resolve engine kind correctly instead of hardcoding OpenAI model fallback

**Web UI changes (`web/index.html`):**
- New dashboard view rendered when no session is active
- Engine selector in the UI (dropdown or card-based)
- Per-message engine badge in chat history (small label like "gpt-5-mini" or "claude-sonnet-4")
- `session.list` response shows engine per session in sidebar

**Multi-interface design:**
- RPC methods work for any client (web, CLI, future TUI)
- CLI: add `--engine` flag to `cargo run -- chat` as convenience override
- All engine state flows through the same `SessionState` / `ThreadEvent` structs

**Security:**
- Never expose API keys over WebSocket. Only report `available: bool`
- `base_url` and `model` are safe to display

## Acceptance Criteria

- [x] Opening web UI with no active session shows engine dashboard (not empty chat)
- [x] Dashboard lists all 3 engines with availability status, model, and base URL
- [x] User can change active engine for current session via UI
- [x] Engine change takes effect on next message send (not retroactive)
- [x] `ThreadHeader` records `engine`, `model`, `base_url` at creation
- [x] `AssistantMessage` and `ToolCall` events record `engine` and `model`
- [x] Switching sessions snaps the engine selector to that session's engine
- [x] `session.list` returns engine info per session for sidebar badges
- [x] Old threads without engine fields still parse and display correctly
- [x] API keys are never exposed to the web client
- [x] CLI `--engine` flag works as override for direct chat mode

## Success Metrics

- Every assistant response in a thread can be attributed to a specific engine + model
- Users can switch engines without restarting the server
- New users see useful information (not a blank chat) on first load

## Dependencies & Risks

**Dependencies:**
- Multi-engine abstraction (already merged on this branch)
- Gateway WebSocket protocol (already working)

**Risks:**
- **Cross-engine replay**: Switching engines mid-thread with tool calls in history may cause malformed requests. Mitigation: strip tool events when replaying to different engine.
- **Session state loss on restart**: Engine overrides are runtime-only. Acceptable per user requirement, but `SessionEntry` records last-used engine for informational display.
- **Deep think engine**: When primary engine changes, `DEEP_THINK_ENGINE` remains independent (governed by its own env var). Document this clearly.

## References & Research

### Internal References
- Engine trait + factory: `src/engine.rs`
- Thread storage: `src/thread_store.rs` (ThreadHeader, ThreadEvent structs)
- Session management: `src/gateway/session.rs` (SessionEntry, SessionState, run_agent_blocking)
- WebSocket handler: `src/gateway/ws.rs` (RPC dispatch)
- Web UI: `web/index.html`
- Multi-engine plan: `docs/plans/2026-02-01-feat-multi-llm-provider-support-plan.md`
