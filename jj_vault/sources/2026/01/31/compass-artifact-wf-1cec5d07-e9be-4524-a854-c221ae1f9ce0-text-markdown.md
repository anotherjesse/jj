---
id: src_01KGAGSGAKF04F3R4GT3ENT6QB
title: compass_artifact_wf-1cec5d07-e9be-4524-a854-c221ae1f9ce0_text_markdown
ingested_at: 2026-01-31T17:14:43.411055+00:00
original_path: /Users/jesse/Downloads/compass_artifact_wf-1cec5d07-e9be-4524-a854-c221ae1f9ce0_text_markdown.md
tags: []
processing_status: complete
content_hash: sha256:d8a2c6d5ca7a27c5063d03e78fb59d771f95d51bfcf074618b9002ffbb3f56bb
---
# How OpenClaw works: A technical architecture deep-dive

OpenClaw is a **gateway-centric AI agent framework** where a single long-running daemon acts as the central control plane for messaging, scheduling, and tool execution. For someone with Elixir/Phoenix experience, think of the Gateway as analogous to a Phoenix application with a GenServer-per-session model and a multiplexed WebSocket endpoint—but written in TypeScript and running on Node.js ≥22.

The architecture solves a core problem: how to build an "always-on" personal AI assistant that connects to multiple messaging platforms, executes tools safely, and coordinates across scheduled tasks and concurrent sessions. Understanding five key architectural layers reveals how OpenClaw achieves this.

## The Gateway is the single control plane for everything

All messages flow through one daemon. The Gateway owns every channel connection (WhatsApp via Baileys, Telegram via grammY, Discord via discord.js, Slack via Bolt), exposes a WebSocket control plane on port **18789**, and enforces exactly one Gateway per host. This is deliberate—WhatsApp Web sessions, for example, can only be held by a single process.

The Gateway handles session management, multi-agent routing, tool execution coordination, cron scheduling, and heartbeat runs. Clients connect via WebSocket with a typed JSON protocol:

```
WhatsApp / Telegram / Slack / Discord / iMessage / WebChat
                        │
                        ▼
           ┌───────────────────────────────┐
           │           Gateway             │
           │    ws://127.0.0.1:18789       │
           │      (control plane)          │
           └──────────────┬────────────────┘
                          │
              ├─ Agent (LLM reasoning)
              ├─ CLI (openclaw …)
              ├─ WebChat UI
              ├─ macOS/iOS/Android nodes
              └─ Cron scheduler (internal)
```

The WebSocket protocol uses three frame types: `req` (client request with method/params), `res` (server response), and `event` (server-push for streaming). The first frame must be a `connect` handshake with authentication—Gateway hard-closes otherwise. Side-effecting methods like `agent` and `send` require **idempotency keys** to enable safe retries.

## The agent loop serializes runs per session

When a message arrives, the Gateway routes it to the appropriate agent based on **binding rules** (most-specific-wins matching by channel, account, peer, guild). The core agent loop then executes:

1. **Intake**: Validate params, resolve session key, persist metadata, return `{runId, acceptedAt}` immediately
2. **Queueing**: Serialize via per-session queue (critical for preventing tool/session races)
3. **Context assembly**: Load workspace files (AGENTS.md, SOUL.md), inject skills, build system prompt
4. **Model inference**: Stream to LLM, handle tool execution requests
5. **Tool execution**: Run tools with sandboxing, sanitize results
6. **Streaming output**: Push `assistant` and `tool` deltas via WebSocket events
7. **Persistence**: Write final transcript to JSONL, update token counts

The session-based serialization is the key insight—**runs are queued per session key**, preventing concurrent modifications to the same conversation state. For Elixir developers, this is like having a GenServer per session with a single mailbox. Sessions are keyed hierarchically: `agent:<agentId>:<channel>:dm:<peerId>` for DMs, `agent:<agentId>:<channel>:group:<id>` for groups.

## Scheduled events can spawn isolated agent instances

OpenClaw has two scheduling mechanisms: **cron jobs** and **heartbeats**. Both run inside the Gateway process—there are no separate scheduler daemons.

**Cron jobs** support three schedule types: one-shot (`at`), fixed interval (`every`), and cron expressions (`cron`). The critical architectural decision is the **session mode**:

- **Main session jobs** (`--session main`): Inject a system event into the existing main session. The agent handles it during its regular heartbeat cycle with full conversational context. No new instance spins up.
- **Isolated session jobs** (`--session isolated`): **Yes, these create a dedicated agent turn** in a fresh session (`cron:<jobId>`). Each run starts with no prior conversation carry-over, the prompt is prefixed for traceability, and results post back to the main session.

```bash
# One-shot reminder (main session - no new instance)
openclaw cron add --name "Reminder" --at "2026-02-01T09:00:00Z" \
  --session main --system-event "Submit expense report" --wake now

# Isolated recurring job (new instance each run)
openclaw cron add --name "Morning briefing" --cron "0 7 * * *" \
  --tz "America/Los_Angeles" --session isolated \
  --message "Summarize inbox and calendar" --deliver
```

**Heartbeats** are periodic agent turns in the main session (default: every 30 minutes) for proactive monitoring. The agent reads `HEARTBEAT.md` from the workspace, checks if anything needs attention, and either returns `HEARTBEAT_OK` (message dropped) or surfaces an alert.

Jobs persist in `~/.openclaw/cron/jobs.json` across Gateway restarts. Concurrency is controlled via `maxConcurrentRuns`.

## Coordination happens through Gateway RPC and file system state

Multi-agent coordination uses three mechanisms beyond file system writes:

**Gateway WebSocket control plane**: The primary coordination mechanism. All session state, routing decisions, and tool invocations flow through the Gateway. The WebSocket protocol provides real-time coordination without polling.

**Agent-to-agent communication tools** (disabled by default, opt-in via allowlisting):
- `sessions_send`: Runs reply-back ping-pong between agents (max turns configurable)
- `sessions_spawn`: Spawns sub-agent runs in isolated sessions

**Sub-agent spawning** creates sessions keyed as `agent:<agentId>:subagent:<uuid>`. Sub-agents default to the full tool set **minus session tools**—critically, they cannot recursively spawn sub-agents. Returns immediately with `{status: "accepted", runId, childSessionKey}`, then posts results to the requester after completion.

```json
{
  "tools": {
    "agentToAgent": {
      "enabled": true,
      "allow": ["home", "work"]
    }
  }
}
```

**File system state** provides durable memory:
- `~/.openclaw/workspace/memory/YYYY-MM-DD.md`: Daily logs (append-only)
- `~/.openclaw/workspace/MEMORY.md`: Curated long-term memory
- Sessions stored as JSONL transcripts

Each agent has an **isolated workspace**, session store, and auth profile directory. Auth profiles are never shared automatically—this prevents credential leakage between agents.

## Tools use TypeBox schemas with a layered skill system

Tools are defined using **TypeBox** (`@sinclair/typebox`) for typed schemas that validate at both definition and runtime:

```javascript
import { Type } from "@sinclair/typebox";

api.registerTool({
  name: "my_tool",
  description: "Do a thing",
  parameters: Type.Object({
    input: Type.String(),
    count: Type.Optional(Type.Number())
  }),
  async execute(_id, params) {
    return { content: [{ type: "text", text: params.input }] };
  },
});
```

Schema guardrails from the codebase: avoid `Type.Union` (no `anyOf/oneOf/allOf`), use `Type.Optional()` instead of `| null`, keep top-level as `type: "object"` with `properties`.

**Skills** are higher-level capabilities defined in `SKILL.md` files with YAML frontmatter:

```yaml
---
name: nano-banana-pro
description: Generate images via Gemini 3 Pro
metadata: {"openclaw":{"requires":{"bins":["uv"],"env":["GEMINI_API_KEY"]}}}
---
Instructions for using this skill...
```

Skills follow a **precedence hierarchy**: workspace skills override managed skills, which override bundled skills. Load-time gating filters skills by required binaries, env vars, config paths, and OS platform.

Tool availability is controlled per-agent with allow/deny lists and profile presets (`minimal`, `coding`, `messaging`, `full`). Tool groups provide shorthands: `group:fs` for file operations, `group:runtime` for exec/process, `group:web` for search/fetch.

## Core patterns for building something similar

**Gateway pattern**: A single long-running process as the control plane simplifies coordination dramatically. All state flows through one daemon, eliminating distributed coordination complexity. For Kubernetes, this means StatefulSet with persistent volume for `~/.openclaw/`.

**Provider abstraction**: Design model-agnostic interfaces from day one. OpenClaw's `models.providers` config supports any OpenAI/Anthropic-compatible API with custom base URLs.

**Channel abstraction**: Create consistent interfaces across messaging platforms. Each channel implementation handles platform-specific quirks (WhatsApp Web reconnection, Discord gateway, Slack Socket Mode) while exposing a uniform message interface.

**Session-based serialization**: Queue agent runs per session key to prevent race conditions. This is the GenServer-per-session pattern—essential for safe concurrent operation.

**Layered configuration**: Global → per-agent → per-provider → per-session. Skills use bundled → managed → workspace precedence.

**Dependency injection via `createDefaultDeps`**: The codebase uses a factory pattern for service initialization, enabling testability and consistent CLI option handling.

**Security through layering**: DM policy (pairing/allowlist/open) → group policy (mention gating) → tool policy (allow/deny) → sandbox (Docker isolation) → exec approvals. Fail-closed defaults with explicit escape hatches.

For an Elixir/Phoenix implementation, the natural mapping is: Gateway as Phoenix application, WebSocket protocol as Phoenix.Channel with custom framing, session queues as GenServers, agent runs as Task.Supervisor processes, channel connections as long-lived GenServer processes, and the workspace as append-only Event Sourcing with file-based persistence. The Lobster workflow shell (a separate repo) provides typed JSON pipelines for deterministic multi-step automation—similar in spirit to Broadway or Flow for composable data pipelines.

The key insight is that OpenClaw optimizes for **single-user, local-first deployment** where one Gateway handles everything. Scaling horizontally would require session affinity and shared state stores—a significant architectural change from the current design.