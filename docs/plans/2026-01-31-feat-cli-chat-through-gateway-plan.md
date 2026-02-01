---
title: CLI Chat Through Gateway
type: feat
date: 2026-01-31
---

# CLI Chat Through Gateway

## Overview

Validate and fix the existing `run_chat_daemon()` path so `cargo run -- chat` works end-to-end through the gateway daemon. The scaffolding exists but has never been tested — there are known architectural gaps that will prevent it from working.

## Problem Statement

The daemon chat code path (`src/chat.rs:134-280`) and CLI WebSocket client (`src/gateway/cli_client.rs`) exist but have never been validated. Key issues identified through code review:

1. **No streaming** — `run_agent_blocking()` (`src/gateway/session.rs:330-400`) runs the full agent loop synchronously and only broadcasts a single `final` event when done. No `delta` or `tool_activity` events are emitted during the run, so the CLI client's streaming handlers are dead code.
2. **Event/response split** — The CLI client splits the WS connection: `request()` reads until it gets its response (dropping events), then the event task takes over. But after `session.send`, the response comes back immediately ("accepted") and the agent runs in background — so the REPL loop fires `session.send`, gets the ack, then waits for user input while the event task handles streaming. This part should actually work, but needs testing.
3. **`/sessions` display** — The `/sessions` command sends a raw frame and hopes the event task prints the response, but the event task only handles event frames, not response frames. The response gets silently dropped.

## Proposed Solution

Validate end-to-end, fix what's broken, keep it simple.

### Phase 0: Fix Nested Runtime Panic

**Blocker.** `main()` is `#[tokio::main]` async (`src/main.rs:237`), but `run_chat_daemon()` (`src/chat.rs:135-139`) creates a new `tokio::runtime::Runtime::new()` — this panics with "Cannot start a runtime from within a runtime."

**Fix:** Make `run_chat_daemon` async and call it with `.await` from the already-running tokio runtime. This means `run_chat()` also needs to be async (or at minimum the daemon path). The simplest approach: make `run_chat` async, await `run_chat_daemon_async` directly, and remove the manual `Runtime::new()` wrapper.

- [x] Make `run_chat()` async, remove `Runtime::new()` from `run_chat_daemon()`
- [x] Update the call site in `main.rs:408` to `.await` the result

### Phase 1: Smoke Test & Basic Fixes

- [x] Start the gateway daemon, run `cargo run -- chat`, send a message, verify connection + auth works
- [x] Fix `/sessions` command to display response frames in the event task

### Phase 2: Add Streaming Events from Agent Run

The big gap. `run_agent_blocking` needs to emit `delta` and `tool_activity` events as the agent loop progresses.

**Approach:** Pass an event callback (or channel sender) into the agent loop so it can emit events during execution.

- [x] Add `AgentEvent` enum and `event_sink: Option<std::sync::mpsc::Sender<AgentEvent>>` to `AgentConfig`
- [x] In the agent loop, emit `FinalContent` event (suppresses local println when sink present)
- [x] Emit `ToolActivity` events when tool calls start
- [x] In `run_agent_blocking`, accept event sink param and pass to `AgentConfig`
- [x] In `run_agent`, create sync channel + async bridge task that forwards `AgentEvent`s to WS broadcast subscribers

### Phase 3: Polish

- [x] Handle the case where daemon isn't running gracefully (already done — falls through to direct mode with a message)
- [x] Implement `/session <key>` switch command
- [x] Web UI benefits from same event pipeline (tool_activity + final events now emitted during agent run)
- [ ] Test multiple messages in sequence (manual testing needed)

## Acceptance Criteria

- [ ] `cargo run -- gateway start` then `cargo run -- chat` connects and authenticates
- [ ] Sending a message produces streaming output (deltas appear as the LLM responds)
- [ ] Tool calls show `[tool: name]` indicators during execution
- [ ] `/sessions` lists sessions correctly
- [ ] `/exit` cleanly disconnects
- [ ] Direct mode fallback still works when daemon isn't running
- [ ] Web UI streaming also works with the same event pipeline

## Technical Considerations

- The agent loop (`src/agent.rs:run_agent_loop`) is synchronous and uses `reqwest::blocking`. The event sink will need to be a `std::sync::mpsc::Sender` (not tokio) since it runs in `spawn_blocking`. The receiving end in the async context bridges with a small forwarding task.
- The `cli_client::request()` helper drops events while waiting for a response. This is fine for `session.open` but means any events arriving during `request()` calls are lost. Since `session.send` returns immediately, this should be okay — events only flow after the response.
- The write half of the WS is currently owned by the REPL loop closure. `/sessions` bypasses `cli_client::request()` and sends a raw frame. Fixing this properly means either wrapping `write` in `Arc<Mutex<>>` or restructuring so slash commands can use `request()`.

## References

- CLI daemon chat: `src/chat.rs:134-280`
- CLI WS client: `src/gateway/cli_client.rs:11-126`
- Session manager: `src/gateway/session.rs:41-313`
- Agent run in session: `src/gateway/session.rs:241-303`
- Blocking agent runner: `src/gateway/session.rs:330-400`
- Agent loop: `src/agent.rs:27-132`
- WS handler: `src/gateway/ws.rs:61-157`
