# JJ Project Notes

This repo builds a memory-first agent system in Rust. The authoritative build spec and milestones live in `plan.md`.

## How we develop
- Small, meaningful commits tied to milestones in `plan.md`.
- Keep thread logs append-only; durable memory updates must be attributable and reversible.
- Prefer simple CLIs + on-disk artifacts over complex services until milestones require it.
- Avoid committing secrets; `.env` stays local.

## Running locally
Prereqs: Rust toolchain, valid `.env` with `OPENAI_API_KEY`.

Common commands:
- `cargo run -- vault init --path jj_vault`
- `cargo run -- thread create --vault jj_vault`
- `cargo run -- repl --vault jj_vault`
- `scripts/verify.sh`
- `scripts/smoke_openai.sh`

Defaults:
- OpenAI model defaults to `gpt-5.2-2025-12-11` unless `OPENAI_MODEL` is set.

Using `just` (if installed):
- `just verify`
- `just smoketest`
- `just vault-init`
- `just repl`
- `just embed-index`

## Repo layout (current)
- `plan.md`: build spec + phased plan
- `src/`: CLI + stores + REPL
- `scripts/verify.sh`: end-to-end sanity check

## Notes for contributors
- If you change memory formats or invariants, update `plan.md` and any prompts in the vault.
- Keep new dependencies minimal and justify them in commit messages.
- When adding tools, include a schema and ensure every call logs a reason.
