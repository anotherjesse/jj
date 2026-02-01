---
title: "feat: Gateway CLI subcommands + Claude Code skill"
type: feat
date: 2026-02-01
---

# Gateway CLI Subcommands + Claude Code Skill

## Overview

Add CLI subcommands (`jj gateway list/open/history/send`) that talk to the running gateway daemon via the existing WebSocket client, and a Claude Code skill (SKILL.md) that invokes them. The gateway server already implements all four RPC methods — this is purely a client-side addition.

## Motivation

Claude Code currently cannot interact with a running JJ agent. Adding CLI subcommands gives any tool (including Claude Code via a skill) structured access to JJ sessions: listing, reading history, and sending messages.

## Proposed Solution

### Phase 1: CLI Subcommands (Rust)

Extend the existing `GatewayCommand` enum in `src/main.rs` with four new variants. Each connects to the daemon via `cli_client::connect()`, sends one JSON-RPC request, prints the response as JSON to stdout, and exits.

#### `jj gateway list`

- Calls `session.list`
- Prints JSON array of sessions to stdout
- Output schema:
  ```json
  [
    {
      "session_key": "main",
      "thread_id": "thr_01...",
      "created_at": "2026-02-01T10:00:00Z",
      "title": "Build plan review",
      "first_user_line": "Can you review..."
    }
  ]
  ```

#### `jj gateway open <session_key>`

- Calls `session.open` with the given key
- Defaults to `"main"` if session_key omitted
- Prints session metadata JSON to stdout
- Output schema:
  ```json
  {
    "session_key": "main",
    "thread_id": "thr_01...",
    "created_at": "2026-02-01T10:00:00Z",
    "title": "Build plan review",
    "first_user_line": "Can you review..."
  }
  ```

#### `jj gateway history <session_key> [--limit N]`

- Calls `session.history` with session_key and limit (default 50, max 500)
- Prints JSON array of events to stdout
- Output schema:
  ```json
  {
    "events": [...],
    "count": 42
  }
  ```

#### `jj gateway send <session_key> <message> [--wait [TIMEOUT]]`

Two modes:

**Fire-and-forget (no `--wait`):**
- Calls `session.send`, prints `{"status": "accepted"}`, exits immediately

**Blocking (`--wait`):**
- Calls `session.send`, then subscribes to session events
- Streams each event as a JSON line to stdout (tool_call_start, delta, etc.)
- Exits after receiving `final` or `error` event
- Default timeout: 120s. Configurable: `--wait 60` (seconds)
- On timeout: prints error JSON and exits (does NOT kill agent run)

### Phase 2: Claude Code Skill (SKILL.md)

A skill file that teaches Claude Code when and how to use the CLI subcommands.

**Trigger phrases:** "ask JJ", "check JJ", "send to JJ", "JJ sessions", "talk to JJ"

**Key behaviors:**
- Default to `--wait` when querying JJ for information
- Default to fire-and-forget when delegating long tasks
- Parse JSON output and present results as readable markdown
- If daemon not running, suggest `jj gateway start`
- If session busy, inform user and suggest waiting

## Technical Approach

### Files to modify

| File | Change |
|------|--------|
| `src/main.rs:202-210` | Extend `GatewayCommand` enum with List, Open, History, Send variants |
| `src/main.rs:241-266` | Add dispatch arms for new variants |
| `src/gateway/mod.rs` | Add `handle_list()`, `handle_open()`, `handle_history()`, `handle_send()` functions |
| `src/gateway/cli_client.rs` | Add `subscribe_events()` helper for `--wait` mode (read events after request) |

### New files

| File | Purpose |
|------|---------|
| `.claude/skills/jj-gateway.md` | Claude Code skill definition |

### Implementation details

**CLI subcommand args (clap):**

```rust
#[derive(Subcommand)]
enum GatewayCommand {
    Start,
    Stop,
    Status,
    /// List all sessions
    List,
    /// Open or create a session
    Open {
        /// Session key (default: "main")
        #[arg(default_value = "main")]
        session_key: String,
    },
    /// Fetch session history
    History {
        /// Session key
        session_key: String,
        /// Max events to return
        #[arg(long, default_value = "50")]
        limit: usize,
    },
    /// Send a message to a session
    Send {
        /// Session key
        session_key: String,
        /// Message content
        message: String,
        /// Block until agent responds (optional timeout in seconds)
        #[arg(long)]
        wait: Option<Option<u64>>,
    },
}
```

**`handle_send` with `--wait`:**

1. `connect()` to daemon
2. Send `session.open` to subscribe to events
3. Send `session.send` with message
4. Loop reading frames:
   - If `type == "event"` and `event == "final"` → print final payload, exit 0
   - If `type == "event"` and `event == "error"` → print error, exit 1
   - If `type == "event"` → print as JSON line to stdout (streaming)
   - If timeout reached → print timeout error, exit 1
5. On connection close → exit 1

**Error handling:**

All subcommands print errors as JSON to stderr and exit non-zero:
```json
{"error": "daemon_not_running", "message": "Gateway daemon is not running. Start it with: jj gateway start"}
```

Error codes: `daemon_not_running`, `auth_failed`, `session_busy`, `timeout`, `connection_lost`

## Acceptance Criteria

- [x] `jj gateway list` returns JSON array of sessions
- [x] `jj gateway open main` creates/opens session, returns metadata JSON
- [x] `jj gateway history main --limit 10` returns last 10 events as JSON
- [x] `jj gateway send main "hello"` returns `{"status": "accepted"}` immediately
- [x] `jj gateway send main "hello" --wait` blocks and streams events until final
- [x] `jj gateway send main "hello" --wait 5` times out after 5 seconds
- [x] All commands print JSON errors to stderr when daemon not running
- [x] Claude Code skill file exists and documents trigger phrases + usage patterns
- [x] Skill correctly invokes subcommands and formats output for user

## Dependencies & Risks

- **Requires running daemon** — all subcommands fail if daemon is down. Mitigated by clear error messages.
- **`--wait` requires event subscription** — needs `session.open` before `session.send` to receive events. The existing `cli_client.rs` already supports this pattern (see `src/chat.rs:146-164`).
- **No new crate dependencies** — uses existing `tokio-tungstenite`, `clap`, `serde_json`.

## Out of Scope

- `jj gateway cancel` (stop running agent)
- `jj gateway delete <session>` (cleanup)
- Auto-start daemon from CLI subcommands
- Streaming formatted output (human-readable mode) — JSON only for now
