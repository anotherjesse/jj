# JJ Gateway Interaction

JJ is a memory-first AI agent with a durable vault of knowledge, append-only conversation threads, and a governance layer for memory changes. It runs as a local daemon with a WebSocket gateway. This skill lets Claude Code communicate with JJ — to delegate tasks, retrieve knowledge from its long-term memory, or check on background work.

## Why Talk to JJ?

JJ maintains **persistent, auditable memory** across conversations — knowledge about people, projects, preferences, and past decisions. Unlike Claude Code's ephemeral context, JJ's vault persists between sessions and is searchable via embeddings. Use JJ when you need:

- **Long-term context**: "What did Jesse decide about the auth approach last week?"
- **Background research**: Delegate a deep-think task and check back later
- **Knowledge retrieval**: Query JJ's vault for information it has accumulated
- **Task delegation**: Hand off work that benefits from JJ's tools (knowledge search, image generation, deep thinking)

JJ and Claude Code are complementary: Claude Code is fast and has filesystem access; JJ has durable memory and domain-specific tools.

## When to Use

Use this skill when the user wants to:
- Ask JJ a question or send it a task ("ask JJ about...", "send to JJ", "talk to JJ")
- List or manage JJ sessions ("JJ sessions", "show my JJ conversations")
- Check what JJ said or read conversation history ("check JJ", "what did JJ say")
- Retrieve something from JJ's memory or knowledge vault

## Prerequisites

The JJ gateway daemon must be running. If commands fail with connection errors, suggest:
```bash
jj gateway start
```

The binary is `jay` (aliased as `jj` via cargo). The daemon binds to `127.0.0.1:9123`.

## Commands

### List sessions
```bash
jj gateway list
```
Returns a JSON array of sessions with `session_key`, `thread_id`, `created_at`, `title`, `first_user_line`.

### Open/create a session
```bash
jj gateway open <session_key>
# Default session key is "main"
jj gateway open
```
Returns session metadata as JSON. Creates the session (and backing thread JSONL) if it doesn't exist.

### Read session history
```bash
jj gateway history <session_key> --limit 20
```
Returns `{"events": [...], "count": N}`. Default limit is 50, max 500. Events include `user_message`, `assistant_message`, `tool_call`, `tool_result`, `system_note`, `inner_monologue`, `title_generated`.

### Send a message (fire-and-forget)
```bash
jj gateway send <session_key> "your message here"
```
Returns `{"status": "accepted"}` immediately. The agent processes in the background. Use for long-running tasks you don't need to wait on.

### Send a message and wait for response
```bash
jj gateway send <session_key> "your question here" --wait
# With custom timeout (seconds):
jj gateway send <session_key> "your question here" --wait 60
```
Streams JSON events to stdout (one per line): `tool_call_start`, `tool_call_result`, `final`, `error`, `deep_think_complete`, `title_generated`. Exits after `final` or `error` event. Default timeout: 120s.

### Check daemon status
```bash
jj gateway status
```

## Usage Guidelines

- **Use `--wait` when querying JJ for information** (questions, lookups, context retrieval)
- **Use fire-and-forget when delegating tasks** (long research, background work) — check back with `history` later
- **Default session is "main"** — use it unless the user specifies otherwise
- **Parse JSON output** and present results as readable markdown to the user
- If you get a `session.busy` error, the agent is already processing — wait and retry, or check `history` later
- All output is JSON — parse it, don't show raw JSON to the user unless they ask
- When reading history, look for `assistant_message` events for JJ's responses and `tool_call`/`tool_result` for what tools it used

## Event Types in History & Streaming

| Event | Description |
|-------|-------------|
| `user_message` | User input |
| `assistant_message` | JJ's text response |
| `tool_call` | Tool invocation (includes `tool_name`, `tool_args`, `reason`) |
| `tool_result` | Tool output |
| `system_note` | System markers (`run.started`, `run.completed`) |
| `inner_monologue` | Deep think output (background reflection) |
| `title_generated` | Auto-generated conversation title |

## Error Handling

Errors are printed to stderr as JSON with `error` and `message` fields:
- `daemon_not_running` — suggest `jj gateway start`
- `auth_failed` — token issue, suggest restarting daemon
- `session_busy` — agent already running on this session, wait and retry
- `timeout` — agent took too long, check history later
- `connection_lost` — daemon crashed or network issue
