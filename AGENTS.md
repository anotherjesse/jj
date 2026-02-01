# J Project Notes

This repo builds a memory-first agent system in Rust. The authoritative build spec and milestones live in `plan.md`.

## How we develop
- Small, meaningful commits tied to milestones in `plan.md`.
- Keep thread logs append-only; durable memory updates must be attributable and reversible.
- Prefer simple CLIs + on-disk artifacts over complex services until milestones require it.
- Avoid committing secrets; `.env` stays local.

## Running locally
Prereqs: Rust toolchain, valid `.env` with an API key for your chosen engine.

Common commands:
- `cargo run -- vault init --path j_vault`
- `cargo run -- thread create --vault j_vault`
- `cargo run -- chat --vault j_vault`
- `scripts/verify.sh`
- `scripts/smoke_openai.sh`

### Engine configuration

The agent supports three engine protocols: `openai`, `anthropic`, `gemini`.
Set `LLM_ENGINE` to choose (default: `openai`).

Generic env vars (override provider-specific ones):
- `LLM_ENGINE` — `openai` | `anthropic` | `gemini`
- `LLM_API_KEY`, `LLM_BASE_URL`, `LLM_MODEL`

Provider-specific env vars (used as fallback defaults per engine):
- OpenAI: `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `OPENAI_MODEL` (default: `gpt-5-mini-2025-08-07`)
- Anthropic: `ANTHROPIC_API_KEY`, `ANTHROPIC_BASE_URL`, `ANTHROPIC_MODEL` (default: `claude-sonnet-4-20250514`), `ANTHROPIC_MAX_TOKENS` (default: 8192)
- Gemini: `GEMINI_API_KEY`, `GEMINI_BASE_URL`, `GEMINI_MODEL` (default: `gemini-2.0-flash`)

To use a non-default vendor on a shared engine (e.g., Kimi via Anthropic protocol):
```
LLM_ENGINE=anthropic LLM_API_KEY=sk-kimi-xxx LLM_BASE_URL=https://api.moonshot.cn LLM_MODEL=kimi-k2
```

Deep think uses the same engine by default. Set `DEEP_THINK_ENGINE` to override.
`OPENAI_DEEP_THINK_MODEL` still works for backward compat.

Using `just` (if installed):
- `just verify`
- `just smoketest`
- `just vault-init`
- `just chat`
- `just embed-index`

## Repo layout (current)
- `plan.md`: build spec + phased plan
- `src/`: CLI + stores + REPL
- `scripts/verify.sh`: end-to-end sanity check

## Notes for contributors
- If you change memory formats or invariants, update `plan.md` and any prompts in the vault.
- Keep new dependencies minimal and justify them in commit messages.
- When adding tools, include a schema and ensure every call logs a reason.
