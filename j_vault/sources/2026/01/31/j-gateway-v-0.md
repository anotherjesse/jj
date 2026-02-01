---
id: src_01KGAKQYA2PRMPRV128VBJMAR2
title: j_gateway_v_0
ingested_at: 2026-01-31T18:06:17.922964+00:00
original_path: /Users/jesse/Downloads/j_gateway_v_0.md
tags: []
processing_status: complete
content_hash: sha256:f9db0d7594fef75d4d67b81acfdeaafab511b323dde9d02d46667327c5846b7e
---
# J Gateway v0.1 — Formal Plan / Spec (CLI + Telegram)

**Audience:** implementation / coding agents

**Primary goal:** ship a first working “Gateway + Agent Loop” that supports:
- a local CLI REPL (`j chat`) that talks to the Gateway
- Telegram integration (bot) as an additional channel that routes messages through the same Gateway
- durable, replayable sessions (crash-only daemon; restart-safe)

---

## 0. Scope

### 0.1 In-scope (v0.1)
1) **Gateway daemon**: one long-running process that owns:
   - session storage + transcripts
   - message intake from channels (CLI + Telegram)
   - agent execution loop (LLM call + streaming)
   - outbound delivery back to channels
2) **CLI client**: `j chat` provides a REPL with streaming assistant responses.
3) **Telegram bot**: `j gateway` can also run a Telegram adapter using long polling.
4) **Persistence**:
   - append-only per-session transcripts (`.jsonl`)
   - session index (`sessions.json`)
5) **Restart-safe iteration**:
   - idempotency keys for side-effecting ops
   - the daemon can be killed/restarted at any time without losing state

### 0.2 Not in scope (future)
- multi-node device capabilities (camera/screen/etc)
- deep tool system / plugin marketplace
- subagents / lanes (beyond minimal per-session serialization)
- cron scheduler (add after v0.1)
- full web UI

---

## 1. Key Ideas (Design Principles)

1) **Gateway is the control plane**: all channel messages flow through it.
2) **Session is the unit of consistency**: one serialized agent run per session key.
3) **Event log is source of truth**: all state is derived from append-only transcript(s).
4) **Crash-only + replayable**: restart = rehydrate from `sessions.json` + transcripts.
5) **Idempotency everywhere**: client retries safe after restarts/network failures.

---

## 2. Terminology

- **Gateway**: daemon process `j gateway`.
- **Channel adapter**: component that converts external messages to internal events (CLI, Telegram).
- **Session key**: human-meaningful stable identifier (e.g., `main`, `tg:<chat_id>`).
- **Session ID**: internal UUID for transcript file naming.
- **Transcript**: append-only JSONL log per session.
- **Run**: one agent turn (user message → assistant streaming → final assistant message).

---

## 3. System Architecture

### 3.1 Components

1) **Gateway Core**
- session manager (index + transcripts)
- run coordinator (per-session serialization)
- model provider interface
- event bus (internal)
- channel router (deliver messages/events)

2) **CLI Adapter**
- `j chat` connects to Gateway, opens a session, sends user messages, displays streaming events.

3) **Telegram Adapter**
- long-poll Telegram Bot API `getUpdates`
- maps Telegram chats to session keys
- injects inbound messages into Gateway
- delivers assistant responses back via `sendMessage`

### 3.2 Concurrency model (v0.1)
- **Per-session serialization**: only one active run per session key.
- **Cross-session concurrency**: allowed; cap with a global semaphore (configurable).
- No multi-step workflows or subagents in v0.1.

### 3.3 High-level data flow

Inbound:
1) Channel adapter receives a user message
2) Adapter creates an internal `InboundMessage` with:
   - `session_key`
   - `channel`
   - `channel_message_id` (if available)
   - `idempotency_key`
3) Gateway appends message to transcript
4) Gateway enqueues an agent `Run`

Agent loop:
5) Gateway builds prompt context (recent transcript, optional summary)
6) Calls model provider; streams deltas as `AssistantDelta` events
7) Writes final assistant message to transcript

Outbound:
8) Router delivers final message (and optionally streamed deltas) to the originating channel

---

## 4. Persistence

### 4.1 Directory layout
Default root: `~/.j/gateway/`

- `sessions.json`
- `transcripts/`
  - `<session_id>.jsonl`
- `dedupe/` (optional; idempotency records)
- `logs/` (optional; raw streaming logs)

### 4.2 `sessions.json` schema
A single JSON file mapping `session_key` → session entry.

**Schema (conceptual):**
```json
{
  "version": 1,
  "updated_at": "2026-01-31T12:34:56Z",
  "sessions": {
    "main": {
      "session_id": "uuid",
      "created_at": "...",
      "updated_at": "...",
      "channel_bindings": [
        {"type": "cli", "client_id": "..."},
        {"type": "telegram", "chat_id": 123456}
      ],
      "model": {"provider": "openai", "model": "...", "thinking": "..."},
      "flags": {"stream": true}
    }
  }
}
```

Notes:
- Keep this small; it is an index + metadata.
- Avoid storing full conversation content here.

### 4.3 Transcript JSONL format
One file per session. Append-only.

First line: **session header**

Then entries. Each line is a single JSON object.

**Entry types (v0.1 minimal):**
- `header`
- `message`
- `assistant_delta` (optional; for debug only)
- `assistant_final`
- `error`

**Example:**
```jsonl
{"type":"header","version":1,"session_id":"...","session_key":"main","created_at":"..."}
{"type":"message","id":"...","role":"user","text":"hello","ts":"...","channel":{"type":"cli","client_id":"..."}}
{"type":"assistant_final","id":"...","role":"assistant","text":"hi!","ts":"..."}
```

### 4.4 Idempotency (dedupe)
Two levels:
1) **Protocol idempotency** for side-effecting Gateway methods.
2) **Inbound message dedupe** for Telegram updates (and potentially CLI reconnect retries).

Implementation options:
- store a small `{key → result_ref}` record in a `dedupe/` directory with TTL pruning
- or embed `idempotency_key` entries into transcript and scan last N lines (fast enough for v0.1)

---

## 5. Gateway API (for CLI)

### 5.1 Transport
Preferred: **WebSocket** on loopback (`127.0.0.1:<port>`), JSON messages.
Alternative: Unix domain socket (later).

### 5.2 Message framing
All frames are JSON objects.

**Client → Gateway request:**
```json
{"type":"req","id":"uuid","method":"session.send","params":{...},"idempotency_key":"..."}
```

**Gateway → Client response:**
```json
{"type":"res","id":"uuid","ok":true,"payload":{...}}
```

**Gateway → Client event:**
```json
{"type":"event","event":"assistant.delta","payload":{...},"seq":123}
```

### 5.3 Required methods (v0.1)

1) `gateway.hello`
- Purpose: handshake + protocol version.
- Response includes server version, protocol range, time.

2) `session.open`
- Params: `session_key` (string)
- Behavior: create if missing; return `session_id`.

3) `session.send`
- Params: `session_key`, `text`, `client_message_id?`
- Behavior: append user message to transcript; schedule a run.
- Response: ack + `message_id`.

4) `session.subscribe`
- Params: `session_key`
- Behavior: register client for events for this session.

5) `session.history`
- Params: `session_key`, `limit`, `before?`
- Behavior: returns last N `assistant_final` + `message` entries.

6) `gateway.shutdown` (dev-only)
- Behavior: graceful shutdown (flush + stop).

### 5.4 Events (v0.1)
- `assistant.delta` (stream chunk)
- `assistant.final`
- `run.started`
- `run.completed`
- `error`

---

## 6. CLI UX: `j chat`

### 6.1 Requirements
- REPL-like experience
- streaming output
- persistent default session (`main`)
- can select/create sessions

### 6.2 CLI commands
- `j chat` — opens `main`
- `j chat --session <key>`
- `j chat --new <key>` (optional convenience)

### 6.3 In-REPL slash commands
- `/help`
- `/session <key>` (switch)
- `/history [n]`
- `/restart` (calls `gateway.shutdown` then waits for reconnect)
- `/model <name>` (optional: just store as metadata; no-op initially)

### 6.4 Behavior on disconnect
- auto-reconnect
- resubscribe to session
- if a `session.send` might have failed mid-flight, retry with same `idempotency_key`

---

## 7. Telegram Integration (v0.1)

### 7.1 Approach
Start with **long polling** (simplest dev story):
- call `getUpdates` in a loop
- track `offset` to avoid duplicates
- for each message update, extract `chat.id`, `message_id`, `text`

### 7.2 Session mapping
- `session_key = "tg:<chat_id>"`
- (optional future) include topic/thread: `tg:<chat_id>:t:<message_thread_id>`

### 7.3 Dedupe
- Deduplicate by Telegram `update_id` and/or `(chat_id, message_id)`.
- Store last processed `update_id` durably (in `sessions.json` or a `telegram_state.json`).

### 7.4 Inbound message rules
- Ignore non-text messages (v0.1)
- Handle `/start` and `/help` specially
- Optional allowlist:
  - Only accept messages from allowed chat IDs or user IDs

### 7.5 Outbound delivery
- Send assistant final response with `sendMessage(chat_id, text)`.
- Optional: include typing action `sendChatAction`.
- For streaming: v0.1 can skip; just send final.

### 7.6 Failure handling
- If Telegram API fails, backoff and continue.
- If Gateway is shutting down, stop polling loop.

---

## 8. Agent Loop (v0.1)

### 8.1 Prompt construction
Inputs:
- last N transcript messages (user + assistant)
- system prompt template
- (future) summary/compaction

Rules:
- Keep prompt assembly deterministic.
- Always include session metadata (session key, channel, etc.) in non-user-visible context.

### 8.2 Model provider interface
Trait-like abstraction:
- `stream_chat(messages, settings) -> Stream<Delta>`
- provider config includes API key, base URL, model name

### 8.3 Run lifecycle
1) Acquire per-session lock
2) Append user message
3) Stream model deltas to subscribed clients
4) Finalize message; append `assistant_final`
5) Release lock

### 8.4 Timeouts / cancellation
- Set max run time (config)
- Support client cancellation (optional)

---

## 9. Configuration

### 9.1 Config file
`~/.j/gateway/config.toml`

Sections:
- `[gateway]` port, data_dir, log_level, max_concurrency
- `[model]` provider, model, api_key env var name
- `[telegram]` enabled, bot_token env var name, allowlist

Example:
```toml
[gateway]
port = 9123
data_dir = "~/.j/gateway"
max_concurrency = 4
log_level = "info"

[model]
provider = "openai"
model = "gpt-4.1"
api_key_env = "OPENAI_API_KEY"

[telegram]
enabled = true
bot_token_env = "TELEGRAM_BOT_TOKEN"
allow_chat_ids = [123456789]
```

### 9.2 Secrets
- never store bot tokens in transcripts
- load via environment variables

---

## 10. Observability + Debuggability

### 10.1 Logs
- structured logs (JSON) for:
  - inbound messages
  - run start/end
  - errors

### 10.2 Raw stream capture (optional but recommended)
- write `logs/stream/<session_id>-<run_id>.jsonl` with timestamps and deltas

### 10.3 Inspectability
- transcripts are human-readable JSONL
- provide `j sessions` and `j tail <session_key>` later

---

## 11. Security + Safety (minimal v0.1)

- Gateway listens on loopback only
- Telegram adapter uses allowlist (chat IDs) by default
- No remote admin endpoints
- Idempotency prevents duplicate tool side effects (future tools)

---

## 12. Development / Iteration Model

### 12.1 Crash-only daemon
- Prefer restart over hot-code upgrades.
- State continuity comes from transcripts + session index.

### 12.2 Safe restart sequence
1) Stop intake (close WS accept loop; stop Telegram poll)
2) Finish in-flight writes
3) Flush transcript buffers
4) Exit

### 12.3 Compatibility strategy
- protocol version handshake: CLI and Gateway can evolve
- maintain backward compat for at least N minor versions

---

## 13. Implementation Plan (Milestones)

### M0 — Scaffold
- `j gateway` starts
- WS handshake works
- `j chat` connects

**Acceptance:** connect/disconnect works reliably.

### M1 — Sessions + persistence
- `session.open`, `session.send`, `session.history`
- `sessions.json` + transcript JSONL

**Acceptance:** restart gateway; history persists.

### M2 — Agent loop streaming (CLI)
- model provider integrated
- stream deltas to CLI

**Acceptance:** `j chat` shows streaming; transcript has final assistant.

### M3 — Telegram adapter
- long polling + mapping to sessions
- send assistant final to Telegram

**Acceptance:** message in Telegram triggers agent response; restart-safe dedupe.

### M4 — Hardening
- idempotency records
- backoff/retry logic
- better shutdown/restart

---

## 14. Test Plan (Minimum)

1) **Restart safety**
- Send a message; kill gateway mid-stream; restart; confirm transcript integrity.

2) **Idempotency**
- Force client retry of `session.send`; ensure single user message entry.

3) **Telegram dedupe**
- Simulate repeated `getUpdates` delivery; ensure single processing.

4) **Concurrency**
- Two sessions run concurrently; within a session, runs are serialized.

---

## 15. Inspiration / Reference Links

### OpenClaw (design inspiration)
- Architecture: https://docs.openclaw.ai/concepts/architecture
- Gateway protocol: https://docs.openclaw.ai/gateway/protocol
- Agent loop: https://docs.openclaw.ai/concepts/agent-loop
- Sessions + transcripts + compaction: https://docs.openclaw.ai/reference/session-management-compaction
- Debugging (restart patterns): https://docs.openclaw.ai/debugging

### Patterns
- Event Sourcing (Fowler): https://martinfowler.com/eaaDev/EventSourcing.html
- Erlang Release Handling (contrast): https://www.erlang.org/doc/system/release_handling.html

### Telegram Bot API
- Core docs: https://core.telegram.org/bots/api
- getUpdates: https://core.telegram.org/bots/api#getupdates
- sendMessage: https://core.telegram.org/bots/api#sendmessage

### Rust building blocks
- Tokio: https://tokio.rs/
- Axum WebSockets: https://docs.rs/axum/latest/axum/extract/ws/index.html
- clap: https://docs.rs/clap
- reedline: https://docs.rs/reedline/
- schemars (JSON Schema): https://docs.rs/schemars
- tracing: https://docs.rs/tracing

---

## 16. Open Questions (leave answers for you / agent)

1) Which model provider(s) are required in v0.1?
2) How much of the transcript should be included in the prompt (N messages vs token-based)?
3) Should Telegram streaming be implemented (editMessageText) or only final messages?
4) Should per-session compaction be in v0.1 or deferred?
5) How strict should allowlisting be for Telegram (chat ID vs user ID)?

---

## 17. Deliverables (Definition of Done for v0.1)

This section is written as a **ship checklist**: for each deliverable, include what must exist, where it lives, and the acceptance criteria.

### 17.1 Source layout (required)
- **Repository** with a top-level workspace:
  - `crates/j-gateway/` (daemon)
  - `crates/j-cli/` (cli)
  - `crates/j-proto/` (shared protocol types + schemas)
  - `docs/` (operator + user docs)
  - `examples/` (sample configs)

**Acceptance:** `cargo build --workspace` succeeds on a clean machine.

---

### 17.2 Gateway daemon `j gateway` (required)
- Binary: `j` with subcommand `gateway`.
- Runs a local control-plane server (WS) on loopback.
- Loads config from `~/.j/gateway/config.toml` (overridable via `--config`).
- Creates/uses data dir `~/.j/gateway/` (overridable via config).
- Graceful shutdown path exists (`SIGINT`/`SIGTERM` + `gateway.shutdown`).

**Acceptance:**
- `j gateway` starts and logs listening address.
- `j gateway` persists state and can be restarted without losing sessions.

---

### 17.3 WebSocket protocol (required)
- Implement:
  - handshake: `gateway.hello`
  - session methods: `session.open`, `session.send`, `session.subscribe`, `session.history`
  - dev-only: `gateway.shutdown`
- Support `idempotency_key` on side-effecting requests.
- Server emits events:
  - `run.started`, `assistant.delta`, `assistant.final`, `run.completed`, `error`

**Acceptance:**
- CLI can connect, handshake, open/subscribe a session, send a message, receive deltas and final.
- Retrying a `session.send` with the same `idempotency_key` does **not** duplicate the message in transcript.

---

### 17.4 Persistence: sessions index + transcript JSONL (required)
- `sessions.json` created/updated atomically.
- Per-session transcript JSONL created/append-only.
- Transcript header line + typed entries.

**Acceptance:**
- After restart, `session.history` returns prior messages.
- Transcript is readable via `tail -f` and contains exactly one entry per accepted inbound message.

---

### 17.5 Agent loop + model provider (required)
- Minimal “chat completion” agent loop:
  1) acquire per-session lock
  2) append user message
  3) call provider with recent context
  4) stream deltas to subscribers
  5) append final assistant message
- Provider abstraction (trait) so it’s swappable.
- Configuration for model provider + model name.

**Acceptance:**
- `j chat` can have a real conversation with streamed output.
- On provider error, the run ends cleanly and an `error` entry is appended.

---

### 17.6 CLI REPL `j chat` (required)
- `j chat` opens `main` by default; `--session <key>` switches sessions.
- Streaming display of assistant deltas.
- Auto-reconnect with resubscribe.
- In-REPL commands:
  - `/help`, `/session <key>`, `/history [n]`, `/restart`

**Acceptance:**
- Works end-to-end against a running local gateway.
- Disconnect/reconnect doesn’t lose the ability to continue the session.

---

### 17.7 Telegram adapter (required)
- Enabled by config `[telegram].enabled = true`.
- Uses long polling:
  - `getUpdates` loop with durable offset storage
- Session mapping:
  - `session_key = tg:<chat_id>`
- Dedupe:
  - never process the same update twice (even after restart)
- Outbound:
  - `sendMessage` with assistant final text
- Security:
  - allowlist `allow_chat_ids` (default deny unless configured)

**Acceptance:**
- Sending a text message to the bot in an allowed chat triggers an agent response.
- Restart gateway mid-conversation; it does not replay old Telegram updates.

---

### 17.8 Configuration + secrets handling (required)
- `examples/config.toml` provided.
- Secrets (model API key, Telegram bot token) loaded only via env vars.
- Config validation with clear error messages.

**Acceptance:**
- Missing/invalid config fails fast with actionable output.
- No secrets are written to transcripts/logs.

---

### 17.9 Observability + debugging (required)
- Structured logging (tracing).
- Optional raw stream capture per run to `logs/stream/*.jsonl`.

**Acceptance:**
- Logs include run id, session key, channel, and error causes.

---

### 17.10 Test suite (minimum required)
- Unit tests for:
  - transcript append + recovery
  - idempotency handling
  - session open/history semantics
- Integration tests (can be behind a feature flag) for:
  - WS happy path
  - Telegram dedupe state machine (mock HTTP)

**Acceptance:**
- `cargo test --workspace` passes.

---

### 17.11 Documentation (required)
- `docs/quickstart.md`:
  - install/run gateway
  - run `j chat`
  - configure Telegram
- `docs/protocol.md`:
  - request/response/event shapes
  - method list
  - idempotency rules
- `docs/storage.md`:
  - sessions.json format
  - transcript JSONL format

**Acceptance:**
- A new developer can run CLI + Telegram in <10 minutes following docs.

---

### 17.12 Optional (nice-to-have, not required for v0.1)
- systemd/launchd service templates
- socket activation
- message streaming to Telegram via editMessageText
- basic `/reset` command to create a fresh session id for a key

