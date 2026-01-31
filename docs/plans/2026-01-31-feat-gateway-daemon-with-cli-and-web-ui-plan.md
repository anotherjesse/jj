---
title: "feat: Gateway daemon with CLI and Web UI"
type: feat
date: 2026-01-31
---

# Gateway Daemon with CLI and Web UI

## Enhancement Summary

**Deepened on:** 2026-01-31
**Sections enhanced:** All 6 phases + protocol + architecture
**Review agents used:** architecture-strategist, performance-oracle, security-sentinel, code-simplicity-reviewer, agent-native-reviewer, pattern-recognition-specialist, best-practices-researcher, framework-docs-researcher

### Key Improvements
1. **Security baseline added** — bearer token auth + Origin header validation required before any phase ships
2. **Protocol simplified** — merged `session.subscribe` into `session.open`, reduced events from 7 to 4, dropped premature features (idempotency keys, seq numbers, hello handshake) for v0.1
3. **Agent-native tools added** — agents can create sessions, send cross-session messages, and schedule their own future runs
4. **Async migration staged** — Phase 2 uses `spawn_blocking` bridge first, native async rewrite comes later
5. **Per-client mpsc channels** replace `tokio::broadcast` to prevent silent message loss
6. **Interrupted run detection** — `run.started`/`run.completed` markers in JSONL enable crash recovery awareness

### Institutional Learnings Applied
- **Tool schema rigor** (`docs/solutions/integration-issues/untyped-tool-schemas-cause-empty-llm-output.md`): Every tool parameter exposed over the gateway protocol or to the LLM must have full `properties` schema or detailed `description`. No bare `{ "type": "object" }`.

---

## Overview

Replace the current in-process `jj chat` with a local-first WebSocket daemon (`jj gateway`) that owns session storage, agent execution, and message routing. CLI and web UI are thin clients that connect to the same daemon. Scheduling and sub-agent delegation are just messages into sessions — no separate job runner infrastructure.

## Problem Statement

Today `jj chat` runs the agent loop in-process with blocking I/O. This means:

- Only one frontend (the terminal REPL) can interact with the agent
- No streaming token delivery
- No way to observe or join a running session from another client
- Scheduled jobs (daily review, weekly consolidation) have no execution surface
- No path to web UI or multi-channel access

## Proposed Solution

A single daemon process (`jj gateway start`) that:

1. Listens on `127.0.0.1:<port>` (WebSocket + HTTP for static assets)
2. Manages sessions (1:1 with threads), serializing agent runs per session
3. Persists everything as append-only JSONL transcripts (crash-only, replayable)
4. Serves the CLI adapter (`jj chat`) and web UI as equal clients

## Key Design Decisions

### Session = Thread
A session is the runtime wrapper around a thread. `session_key` (e.g., `main`) maps to a `thread_id` via `sessions.json`. Persistence uses the existing JSONL thread format.

### Research Insights: Session-Thread Coupling

**Architectural constraint to be aware of:** The 1:1 session=thread coupling means cross-session agent delegation (agent in session A sending work to session B) requires careful design. The parent agent's session lock must NOT be held while waiting on the child session's result.

**Pattern:** Agent tool calls that target other sessions should be fire-and-forget: enqueue the message and return immediately. The result arrives later as a cross-session event. This avoids the deadlock-prone "synchronous dependency across two actor mailboxes" anti-pattern.

**Future consideration:** If branching conversations (trying two strategies, picking the better result) becomes needed, the 1:1 mapping will need revisiting. For v0.1, this is acceptable.

### Daemon Discovery
- TCP port on `127.0.0.1`, default `9123`, configurable via env var `JJ_GATEWAY_PORT`
- PID file at `~/.jj/gateway/daemon.pid` with `flock`-based advisory locking (prevents race when two CLI invocations try to auto-start simultaneously)
- CLI auto-starts the daemon if not running (spawn background process, poll port for readiness)

### Research Insights: Daemon Lifecycle

**PID file guard pattern (Rust):**
```rust
use fs2::FileExt; // or fd-lock

struct PidGuard { file: std::fs::File, path: PathBuf }

impl PidGuard {
    fn acquire(path: &Path) -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true).write(true).open(path)?;
        file.try_lock_exclusive()
            .map_err(|_| anyhow!("daemon already running"))?;
        write!(&file, "{}", std::process::id())?;
        Ok(PidGuard { file, path: path.to_owned() })
    }
}
impl Drop for PidGuard {
    fn drop(&mut self) {
        let _ = self.file.unlock();
        let _ = std::fs::remove_file(&self.path);
    }
}
```

**Critical:** Do NOT use `daemonize` crate to fork — tokio's runtime doesn't survive a fork. Either daemonize *before* starting tokio, or (preferred) just run the process in the background via the CLI spawning it detached.

**Alternative considered:** Unix domain sockets avoid port conflicts and are more idiomatic for local daemons. However, TCP is needed for the web UI (browsers can't connect to UDS). Could bind both — UDS for CLI, TCP for web — but this adds complexity. Sticking with TCP-only for v0.1.

### Busy Session Policy
When a message arrives for a session with an in-flight agent run: **reject with `session.busy` error**. No queueing — the client can retry. This is simpler than a 1-deep queue and avoids hidden state where queued messages have no client visibility.

### Async Runtime
Tokio. It's the ecosystem default and required for `axum` + `tokio-tungstenite`. The synchronous CLI fallback (`jj chat --direct`) is preserved for debugging and environments where the daemon is impractical.

### Research Insights: Async Migration Strategy

**Staged approach (reduces risk):**
1. Add `tokio` with `features = ["full"]`, make `main()` async with `#[tokio::main]`
2. Keep ALL existing sync code — wrap calls with `tokio::task::spawn_blocking` at the gateway boundary
3. Gradually convert hot paths (OpenAI client, file I/O) to native async

**Critical pitfall:** Never call `.block_on()` from within a tokio async context — it panics. If sync code needs to call async code, use `tokio::runtime::Handle::current().block_on()` only from within a `spawn_blocking` task.

**Feature gating (recommended):** Gate gateway deps behind a Cargo feature so sync commands like `jj vault init` don't link the full tokio runtime:
```toml
[features]
default = ["gateway"]
gateway = ["tokio", "axum", "tokio-tungstenite", "rust-embed", "axum-embed"]
```

### Streaming
Assistant tokens arrive as `delta` event frames, one per chunk (matching OpenAI SSE chunk boundaries). `final` signals completion with the full assembled text. The `final` event is the authoritative payload — deltas are best-effort (clients that miss deltas can reconstruct from `final`).

### Crash Recovery
- Daemon replays `sessions.json` index on startup (session metadata only — not full transcript contents)
- Transcripts are loaded lazily when a session is opened
- Truncated trailing JSONL lines are skipped with a warning logged
- On replay, if the last transcript event is `run.started` without a matching `run.completed`, mark session as "interrupted" and surface to client

### Research Insights: Crash Recovery

**Interrupted run detection:** Write `run.started` and `run.completed` markers to the JSONL transcript. On restart, scan the tail of each active session's transcript. If the last structural event is `run.started` with no `run.completed`, the session was mid-run when the daemon died. Surface this to the client on `session.open` so they can decide whether to retry.

**JSONL corruption handling:** Use a line-by-line JSON parse on replay. If the last line fails to parse, truncate it and log a warning. This handles partial writes from crash during append. All prior lines are guaranteed valid because JSONL appends are atomic at the OS write() level for lines under ~4KB.

### Web UI Serving
The daemon embeds static assets (HTML/JS/CSS) via `rust-embed` and serves them on the same port over HTTP. No separate dev server in production. During development, use `rust-embed`'s debug mode which reads from disk (no recompile needed).

### Research Insights: Static Asset Serving

**Recommended stack:**
```toml
rust-embed = { version = "8", features = ["compression"] }
axum-embed = "0.2"  # or memory-serve for SPA routing
```

```rust
#[derive(RustEmbed, Clone)]
#[folder = "web/dist"]
struct Assets;

let app = Router::new()
    .route("/ws", any(ws_handler))
    .nest_service("/", ServeEmbed::<Assets>::new());
```

**Debug vs Release:** In debug builds, `rust-embed` reads files from disk at runtime — hot reload without recompiling Rust. In release, files are embedded in the binary. No need for a separate Vite dev server.

### Scheduling
The daemon owns the clock. Cron triggers are internal timers that inject system events into target sessions. Missed jobs run on next startup. This is just another message into a session — no separate infrastructure.

## Security Baseline (Required for All Phases)

These are non-negotiable before any phase ships to regular use:

### 1. Bearer Token Authentication
- Daemon generates a random token on first start, writes to `~/.jj/gateway/token` with `0600` permissions
- All WebSocket connections must include the token (as first message or query param)
- All HTTP requests must include `Authorization: Bearer <token>` header
- CLI reads token from the file automatically

### 2. Origin Header Validation
- On WebSocket upgrade, validate `Origin` header
- Reject connections where Origin is not `http://127.0.0.1:<port>` or `null` (for non-browser clients)
- This prevents cross-origin WebSocket hijacking — any website could otherwise connect to the daemon

### 3. Tool Execution Path Restriction
- File I/O tools confined to the vault directory by default
- Explicit allowlist of permitted tool operations
- Log every tool call with full parameters to the transcript

### Research Insights: Why This Matters

**Cross-origin WebSocket hijacking is the critical threat.** The daemon serves a web UI, so browsers will connect to it. Any malicious website the user visits while the daemon runs can attempt `new WebSocket("ws://127.0.0.1:9123")` — browsers send WebSocket upgrades without CORS preflight. Without Origin validation + token auth, any website can read chat history and trigger file modifications.

**API key protection:** Load `OPENAI_API_KEY` from `~/.jj/gateway/config.toml` (file permissions `0600`), not from environment variables visible in `/proc/<pid>/environ`. Ensure no protocol response or log output includes the key.

## Technical Approach

### Architecture

```
                    +-----------------------+
                    |    jj gateway         |
                    |   (tokio runtime)     |
                    |                       |
  jj chat -------->|  WebSocket server     |
  (CLI REPL)       |    |                  |
                    |  Session manager     |
  Web UI --------->|    |                  |
  (browser)        |  Agent runner        |
                    |    |                  |
  Cron timer ----->|  Thread store (JSONL) |
  (internal)       |                       |
                    +-----------------------+
```

### Concurrency Model

```
SessionManager
  ├─ RwLock<HashMap<SessionKey, Arc<Session>>>  (session registry)
  │   └─ flushes to sessions.json on mutation
  │
  └─ Per Session:
       ├─ RwLock<SessionState>   (transcript cache, subscriber list — read-heavy)
       ├─ Semaphore(1)           (agent run exclusion — held during LLM calls)
       └─ Vec<mpsc::Sender>      (per-client event channels)
```

**Locking invariant:** The agent-run semaphore is held only during agent execution within that session. Tool implementations must never acquire another session's semaphore. Cross-session reads (e.g., reading another thread's transcript) use a read-only `VaultReader` that bypasses session locks.

**Per-client mpsc channels** instead of `tokio::broadcast`: Each `session.open` creates a bounded `mpsc::Sender` (capacity 256) for that client. The agent runner fans out events explicitly. If a client's channel fills, drop the client with a logged warning rather than silently losing events. This gives per-client backpressure and eliminates the `RecvError::Lagged` problem.

### Implementation Phases

#### Phase 1: Daemon scaffold + async migration

Introduce tokio, axum, tokio-tungstenite. Stand up the daemon process with:

- `jj gateway start` — starts daemon, writes PID file (with flock), binds port, generates bearer token
- `jj gateway stop` — graceful shutdown via SIGTERM
- `jj gateway status` — check if running (read PID file, attempt TCP connect)
- WebSocket accepts connections with token validation
- Health check HTTP endpoint (`GET /health`)
- Origin header validation on WebSocket upgrade

Key files:
- `src/gateway/mod.rs` — daemon entry point, server setup
- `src/gateway/ws.rs` — WebSocket connection handling, frame parsing, auth
- `src/gateway/protocol.rs` — frame types, serialization
- `src/main.rs` — add `gateway` subcommand

**Axum setup pattern:**
```rust
use axum::{extract::ws::WebSocketUpgrade, routing::any, Router};

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    // Validate Origin header
    // Validate bearer token
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

let app = Router::new()
    .route("/ws", any(ws_handler))
    .route("/health", get(|| async { "ok" }))
    .with_state(Arc::new(app_state));

let listener = TcpListener::bind("127.0.0.1:9123").await?;
axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
```

**Graceful shutdown:**
```rust
async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    let mut sigterm = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::terminate()
    ).unwrap();
    tokio::select! {
        _ = ctrl_c => {},
        _ = sigterm.recv() => {},
    }
}
```

No agent execution yet. Just the skeleton.

#### Phase 2: Sessions + agent loop (merged)

Sessions without an agent loop are untestable. Merge the original Phases 2 and 3.

**Session management:**
- `sessions.json` index (session_key -> session_id mapping), protected by `RwLock`
- `session.open` — create-if-missing, auto-subscribes client, returns session_id + metadata
- `session.history` — load last N events from transcript tail (not full file)
- `session.list` — return all sessions with metadata

**Agent execution (spawn_blocking bridge first):**
- `session.send` — append user message, schedule agent run
- Wrap existing sync `run_agent_loop` in `tokio::task::spawn_blocking`
- Pipe output into per-client `mpsc` channels instead of printing to stdout
- Per-session `Semaphore(1)` ensures one agent run at a time
- Write `run.started` and `run.completed` markers to transcript

**Streaming (async OpenAI client):**
- Convert `OpenAIClient` to async with `reqwest` + `reqwest-eventsource` for SSE parsing
- Each SSE chunk becomes a `delta` event sent to all subscribed clients
- `final` event includes the complete assembled text

Key files:
- `src/gateway/session.rs` — SessionManager, per-session state + locks
- `src/gateway/agent_runner.rs` — async agent loop wrapper, event broadcasting
- `src/openai.rs` — async + streaming variant (keep sync version for `--direct`)

**Agent tools for gateway (agent-native):**
When running inside the daemon, inject additional tools into the agent's tool registry:
- `gateway_session_open` — create a new session (for sub-agent delegation)
- `gateway_session_send` — send a message to another session (fire-and-forget)
- `gateway_session_list` — list all sessions
- `gateway_session_history` — read another session's transcript

These call the same internal `SessionManager` methods the WebSocket handlers use. The agent becomes a first-class client of its own gateway.

**Tool schema requirement** (from institutional learning): Every tool parameter must have full `properties` schema or detailed `description`. No bare `{ "type": "object" }`. Test with a fresh conversation to verify the LLM can use each tool correctly from schema alone.

#### Phase 3: CLI adapter (`jj chat` via daemon)

- `jj chat` detects daemon (check PID file + port), auto-starts if needed
- Reads bearer token from `~/.jj/gateway/token`
- Connects via WebSocket, opens session (auto-subscribes)
- REPL loop: user input -> `session.send`, render `delta` events as streaming text
- Slash commands: `/help`, `/session <key>`, `/history [n]`, `/sessions` (list), `/model <name>`
- Ctrl+C mid-stream: client disconnects, agent run continues to completion (writes to JSONL)
- `jj chat --direct` preserved as sync fallback (current behavior)

**WebSocket client pattern:**
```rust
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};

let (ws, _) = connect_async("ws://127.0.0.1:9123/ws").await?;
let (mut write, mut read) = ws.split();

// Send in one task, receive in another
```

Key files:
- `src/chat.rs` — refactor: daemon mode vs direct mode
- `src/gateway/cli_client.rs` — WebSocket client logic for CLI

#### Phase 4: Web UI

- SPA served from daemon's HTTP server (embedded static assets via `rust-embed`)
- **Vanilla JS, no framework, no npm** — this is a developer tool, not a consumer product
- Connects to daemon via same WebSocket protocol as CLI

Views:
- **Session list**: shows all sessions with last message preview, timestamps, status (idle/running)
- **Session view**: message history + streaming responses + input box
- **Session switching**: click any session in the sidebar to join/observe it
- **Real-time updates**: session list updates when new sessions are created or sessions receive messages

The web UI is a read/write client — same capabilities as the CLI, just rendered in a browser.

Key files:
- `web/` — SPA source (HTML, JS, CSS) — single `index.html` with inline script if possible
- `src/gateway/http.rs` — static file serving via `axum-embed`

#### Phase 5: Scheduling (stretch)

- Internal cron timer in the daemon (simple `tokio::time::interval` timers)
- Config in env vars or `~/.jj/gateway/config.toml` under `[schedules]`
- Fires by injecting a system event into the target session via the same internal `session.send` path
- Heartbeat support: periodic agent turns that read `HEARTBEAT.md`

**Agent-controllable scheduling:**
- Expose `schedule_create`, `schedule_list`, `schedule_delete` as agent tools
- An agent can say "remind me to check this in 2 hours" by calling `schedule_create`
- A schedule is just "inject this message into this session at this time"

Key files:
- `src/gateway/scheduler.rs` — timer management
- Config additions

## WebSocket Protocol (Simplified for v0.1)

Frame envelope:
```json
{"type": "req", "id": "uuid", "method": "session.send", "params": {...}}
{"type": "res", "id": "uuid", "ok": true, "payload": {...}}
{"type": "res", "id": "uuid", "ok": false, "error": {"code": "session.busy", "message": "..."}}
{"type": "event", "event": "delta", "session_id": "...", "payload": {"text": "..."}}
```

Methods (v0.1):
| Method | Side-effecting | Description |
|---|---|---|
| `session.open` | Yes | Create-if-missing, auto-subscribe, returns session_id |
| `session.send` | Yes | Append message, trigger agent run |
| `session.history` | No | Fetch last N events |
| `session.list` | No | List all sessions with metadata |

Events:
| Event | Description |
|---|---|
| `delta` | Streaming token chunk |
| `final` | Complete assistant response (authoritative) |
| `tool_activity` | Tool call + result (combined, for UI rendering) |
| `error` | Error during run |

### Research Insights: Protocol Simplifications

**Dropped for v0.1 (add when actually needed):**
- `gateway.hello` handshake — just connect and authenticate. Version negotiation is YAGNI when server and client ship together.
- `gateway.shutdown` — redundant with `jj gateway stop` via signal.
- `session.subscribe` as separate method — merged into `session.open`.
- `idempotency_key` — localhost TCP is reliable; one client per session in practice. Add if duplication actually occurs.
- `seq` numbering — single TCP connection already orders messages. `final` event is the authoritative recovery point.
- `run.started` / `run.completed` as events — these are internal transcript markers, not client-facing events. Clients infer run state from delta/final/error.

**Sender attribution (for agent-native):**
Add optional `sender` to `session.send` params:
```json
{"type": "req", "method": "session.send", "params": {
  "session_key": "research-subtask",
  "content": "Investigate X",
  "sender": {"type": "agent", "session_id": "main"}
}}
```
This enables audit trails for cross-session agent delegation.

## Acceptance Criteria

### Phase 1 (Scaffold)
- [ ] `jj gateway start` launches daemon, binds port, writes PID file, generates token
- [ ] `jj gateway stop` sends shutdown signal
- [ ] `jj gateway status` reports running/stopped
- [ ] WebSocket accepts connections with token validation
- [ ] Origin header validated on WebSocket upgrade
- [ ] `GET /health` returns 200

### Phase 2 (Sessions + Agent Loop)
- [ ] `session.open` creates new session or returns existing, auto-subscribes
- [ ] `session.list` returns all sessions
- [ ] `session.history` returns last N transcript events
- [ ] `session.send` triggers async agent run with streaming
- [ ] Connected clients receive `delta` events in real time
- [ ] Per-session serialization prevents concurrent agent runs
- [ ] Agent run completes even if client disconnects
- [ ] `run.started`/`run.completed` markers written to transcript
- [ ] `sessions.json` persists across daemon restarts
- [ ] Gateway tools (session_open, session_send, etc.) available to agent

### Phase 3 (CLI)
- [ ] `jj chat` auto-starts daemon if not running
- [ ] Reads bearer token automatically
- [ ] Streaming token display in terminal
- [ ] `/session`, `/sessions`, `/history` slash commands work
- [ ] `jj chat --direct` still works without daemon
- [ ] Reconnect on daemon restart

### Phase 4 (Web UI)
- [ ] Session list view with real-time updates
- [ ] Join any session and see full history
- [ ] Send messages and see streaming responses
- [ ] Multiple sessions observable simultaneously
- [ ] Served from daemon's embedded HTTP server
- [ ] Vanilla JS, no framework dependencies

### Phase 5 (Scheduling)
- [ ] Configured schedules fire at correct times
- [ ] Agent can create/list/delete schedules via tools
- [ ] Job results visible to subscribed clients

## Performance Considerations

### Transcript Loading
- Load only last N events on `session.open` (not full file)
- For `session.history`, seek to end of JSONL file and read backwards
- Cap in-memory transcript to a sliding window (e.g., last 500 events)
- Older events read from disk on demand

### JSONL Write Batching
- Buffer events in memory, flush every 100ms or every N events (whichever first)
- Use `tokio::task::spawn_blocking` for file I/O to avoid blocking the async runtime
- `fsync` only at run boundaries (`run.completed`), not after every delta

### Session State
- Separate read lock (`RwLock`) from agent-run lock (`Semaphore(1)`)
- History reads and event broadcast are lock-free relative to each other
- `sessions.json` protected by its own `RwLock`, separate from per-session locks

## Dependencies & Risks

**New dependencies:**
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
axum = { version = "0.8", features = ["ws"] }
tokio-tungstenite = "0.26"
futures-util = "0.3"
reqwest = { version = "0.12", features = ["json", "stream"] }
reqwest-eventsource = "0.6"
rust-embed = { version = "8", features = ["compression"] }
axum-embed = "0.2"
tracing = "0.1"
tracing-subscriber = "0.3"
fs2 = "0.4"  # file locking for PID guard
```

**Consider feature-gating:**
```toml
[features]
default = ["gateway"]
gateway = ["tokio", "axum", "tokio-tungstenite", "rust-embed", "axum-embed", "reqwest-eventsource"]
```

**Risks:**
- Async migration touches `openai.rs` and `agent.rs` — core paths. Mitigated by `spawn_blocking` bridge first, keeping `--direct` fallback throughout.
- Web UI is new territory for this repo. Keep it minimal — vanilla JS, no build tooling.
- Per-session locking must be correct or agent runs will race. Use `Semaphore(1)` for agent exclusion, `RwLock` for reads.
- Dual sync/async maintenance burden — the `--direct` path stays working but both codepaths need testing.

## References

- JJ Gateway v0.1 spec: `jj_vault/sources/2026/01/31/jj-gateway-v-0.md`
- WebSocket protocol: `jj_vault/knowledge/system/jj-gateway-websocket-protocol-v0-1.md`
- Persistence model: `jj_vault/knowledge/system/jj-gateway-persistence-model.md`
- CLI UX spec: `jj_vault/knowledge/system/jj-gateway-cli-ux-jj-chat-v0-1.md`
- OpenClaw scheduling: `jj_vault/knowledge/system/openclaw-scheduling.md`
- OpenClaw gateway control plane: `jj_vault/knowledge/system/openclaw-gateway-control-plane.md`
- Crash-only design: `jj_vault/knowledge/prefs/crash-only-replayable-systems.md`
- Tool schema gotcha: `docs/solutions/integration-issues/untyped-tool-schemas-cause-empty-llm-output.md`
- Axum WebSocket docs: https://docs.rs/axum/latest/axum/extract/ws/index.html
- Tokio bridging sync/async: https://tokio.rs/tokio/topics/bridging
- reqwest-eventsource: https://docs.rs/reqwest-eventsource/
