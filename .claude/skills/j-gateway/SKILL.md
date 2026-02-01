# J Gateway Interaction

J is a memory-first AI agent with a durable vault of knowledge, append-only conversation threads, and a governance layer for memory changes. It runs as a local daemon with a WebSocket gateway. This skill lets Claude Code communicate with J — to delegate tasks, retrieve knowledge from its long-term memory, or check on background work.

## Why Talk to J?

J maintains **persistent, auditable memory** across conversations — knowledge about people, projects, preferences, and past decisions. Unlike Claude Code's ephemeral context, J's vault persists between sessions and is searchable via embeddings. Use J when you need:

- **Long-term context**: "What did Jesse decide about the auth approach last week?"
- **Background research**: Delegate a deep-think task and check back later
- **Knowledge retrieval**: Query J's vault for information it has accumulated
- **Task delegation**: Hand off work that benefits from J's tools (knowledge search, image generation, deep thinking)

J and Claude Code are complementary: Claude Code is fast and has filesystem access; J has durable memory and domain-specific tools.

## When to Use

Use this skill when the user wants to:
- Ask J a question or send it a task ("ask J about...", "send to J", "talk to J")
- List or manage J sessions ("J sessions", "show my J conversations")
- Check what J said or read conversation history ("check J", "what did J say")
- Retrieve something from J's memory or knowledge vault

## Prerequisites

The J gateway daemon must be running. If commands fail with connection errors, suggest:
```bash
j gateway start
```

The binary is `j`. The daemon binds to `127.0.0.1:9123`.

## Commands

### List sessions
```bash
j gateway list
```
Returns a JSON array of sessions with `session_key`, `thread_id`, `created_at`, `title`, `first_user_line`.

### Open/create a session
```bash
j gateway open <session_key>
# Default session key is "main"
j gateway open
```
Returns session metadata as JSON. Creates the session (and backing thread JSONL) if it doesn't exist.

### Read session history
```bash
j gateway history <session_key> --limit 20
```
Returns `{"events": [...], "count": N}`. Default limit is 50, max 500. Events include `user_message`, `assistant_message`, `tool_call`, `tool_result`, `system_note`, `inner_monologue`, `title_generated`.

### Send a message (fire-and-forget)
```bash
j gateway send <session_key> "your message here"
```
Returns `{"status": "accepted"}` immediately. The agent processes in the background. Use for long-running tasks you don't need to wait on.

### Send a message and wait for response
```bash
j gateway send <session_key> "your question here" --wait
# With custom timeout (seconds):
j gateway send <session_key> "your question here" --wait 60
```
Streams JSON events to stdout (one per line): `tool_call_start`, `tool_call_result`, `final`, `error`, `deep_think_complete`, `title_generated`. Exits after `final` or `error` event. Default timeout: 120s.

### Check daemon status
```bash
j gateway status
```

## Usage Guidelines

- **Use `--wait` when querying J for information** (questions, lookups, context retrieval)
- **Use fire-and-forget when delegating tasks** (long research, background work) — check back with `history` later
- **Default session is "main"** — use it unless the user specifies otherwise
- **Parse JSON output** and present results as readable markdown to the user
- If you get a `session.busy` error, the agent is already processing — wait and retry, or check `history` later
- All output is JSON — parse it, don't show raw JSON to the user unless they ask
- When reading history, look for `assistant_message` events for J's responses and `tool_call`/`tool_result` for what tools it used

## Event Types in History & Streaming

| Event | Description |
|-------|-------------|
| `user_message` | User input |
| `assistant_message` | J's text response |
| `tool_call` | Tool invocation (includes `tool_name`, `tool_args`, `reason`) |
| `tool_result` | Tool output |
| `system_note` | System markers (`run.started`, `run.completed`) |
| `inner_monologue` | Deep think output (background reflection) |
| `title_generated` | Auto-generated conversation title |

## Error Handling

Errors are printed to stderr as JSON with `error` and `message` fields:
- `daemon_not_running` — suggest `j gateway start`
- `auth_failed` — token issue, suggest restarting daemon
- `session_busy` — agent already running on this session, wait and retry
- `timeout` — agent took too long, check history later
- `connection_lost` — daemon crashed or network issue
