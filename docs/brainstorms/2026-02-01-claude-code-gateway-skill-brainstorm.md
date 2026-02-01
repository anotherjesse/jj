# Brainstorm: Claude Code ↔ J Gateway Skill

**Date:** 2026-02-01
**Status:** Ready for planning

## What We're Building

A Claude Code skill that lets Claude interact with a running J agent through the gateway daemon. This enables two-way communication: querying J for context/memory while working on the codebase, and orchestrating J sessions (creating, sending tasks, reading history).

The skill relies on **CLI subcommands** (`j gateway list`, `j gateway open`, `j gateway history`, `j gateway send`) that speak to the WebSocket daemon and return structured output. The skill itself invokes these via Bash.

## Why This Approach

- **CLI subcommands** are the natural fit — they already have access to the token file, vault path, and daemon discovery logic via the existing `cli_client.rs`.
- Avoids needing external dependencies (python, websocat) in the skill.
- Subcommands are useful beyond the skill — any script or tool can call them.
- The skill layer is thin: it just knows *when* and *how* to call the subcommands.

## Key Decisions

- **CLI subcommands over script wrappers** — reuse existing Rust WebSocket client code.
- **All four operations** — list, open, history, send — for full read/write access.
- **Structured JSON output** from subcommands so Claude can parse results reliably.
- **Skill triggers** on phrases like "ask J", "check J", "send to J", "J sessions".

## Scope

### CLI Subcommands (Rust)
- `j gateway list` — list sessions (JSON array)
- `j gateway open <session>` — create/open a session, return metadata
- `j gateway history <session> [--limit N]` — fetch recent events as JSON
- `j gateway send <session> <message>` — send message, return immediately
- `j gateway send <session> <message> --wait` — send message, block until agent run completes, print final response

### Skill (SKILL.md)
- Describes when to use each subcommand
- Provides examples of common workflows
- Instructs Claude to parse JSON output and summarize for the user

## Open Questions

- ~~Should `gateway send` block or not?~~ **Decided:** Both — `send` is fire-and-forget, `send --wait` blocks until completion.
- Should the skill auto-detect which session to use (e.g. "main") or always ask?
- Output format: pure JSON, or human-readable with `--json` flag?
