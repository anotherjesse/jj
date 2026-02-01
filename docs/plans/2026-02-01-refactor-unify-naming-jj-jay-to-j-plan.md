---
title: "Unify naming: rename jj/jay to j everywhere"
type: refactor
date: 2026-02-01
---

# Unify naming: rename jj/jay to j everywhere

Rename all occurrences of "jj", "jay", and "JJ" to "j" / "J" across code, config, docs, and directory names.

## Acceptance Criteria

- [x] `cargo build` succeeds with binary named `j`
- [x] `cargo run -- vault init --path j_vault` works
- [x] `cargo run -- chat --vault j_vault` works
- [x] `scripts/verify.sh` passes
- [x] No remaining references to `jj` or `jay` in repo (except git history / this plan)

## 1. Cargo & Binary

- [x] `Cargo.toml`: package name `jay` → `j`

## 2. Source Code

- [x] `src/main.rs`: `#[command(name = "jay")]` → `"j"`, all `jj_vault` doc refs → `j_vault`
- [x] `src/vault.rs`: `jj.runtime.yml` → `j.runtime.yml`, `jj.system.md` → `j.system.md`, default vault `"jj_vault"` → `"j_vault"`
- [x] `src/chat.rs`: agent name `"jj"` → `"j"`, REPL prompt `"jj> "` → `"j> "`, system prompt path
- [x] `src/thread_store.rs`: struct field `jj_thread` → `j_thread`
- [x] `src/gateway/mod.rs`: `.jj` → `.j` home dir, `JJ_GATEWAY_PORT` → `J_GATEWAY_PORT`, log messages
- [x] `src/gateway/session.rs`: agent name `"jj"` → `"j"`
- [x] `src/agent.rs`: tool descriptions "JJ vault" → "j vault", temp file `jj_draw` → `j_draw`
- [x] `src/ingest.rs`: error message `jay vault init` → `j vault init`

## 3. Directory Renames

- [x] `jj_vault/` → `j_vault/`
- [x] `.claude/skills/jj-gateway/` → `.claude/skills/j-gateway/`
- [x] Code path `~/.jj/` → `~/.j/` (gateway config dir)

## 4. File Renames (inside vault)

- [x] `j_vault/config/jj.runtime.yml` → `j.runtime.yml`
- [x] `j_vault/prompts/jj.system.md` → `j.system.md`
- [x] `j_vault/knowledge/projects/jj-gateway.md` → `j-gateway.md`
- [x] `j_vault/knowledge/system/jj-gateway-*.md` → `j-gateway-*.md` (5 files)
- [x] `j_vault/knowledge/system/dual-model-subconscious-loop.md` — update tags
- [x] `j_vault/sources/2026/01/31/jj-gateway-v-0.md` → `j-gateway-v-0.md`
- [x] `j_vault/summaries/sources/jj-gateway-v-0.md` → `j-gateway-v-0.md`

## 5. Config & Scripts

- [x] `Justfile`: default path `jj_vault` → `j_vault`, env var `JJ_GATEWAY_PORT` → `J_GATEWAY_PORT`, `.jj/gateway` → `.j/gateway`
- [x] `scripts/verify.sh`: vault dir `jj_vault` → `j_vault`

## 6. Documentation

- [x] `CLAUDE.md`: all `jj_vault` → `j_vault`
- [x] `AGENTS.md`: title, vault paths
- [x] `plan.md`: title, all vault paths and references
- [x] `.claude/skills/j-gateway/SKILL.md`: all `jj`/`jay` refs → `j`
- [x] `j_vault/agents.md`, `j_vault/invariants.md`: headings
- [x] All knowledge docs inside vault: content references
- [x] `docs/plans/*.md`: vault path references (best-effort, these are historical)

## Execution Order

1. Rename directories first (`jj_vault` → `j_vault`, skill dir)
2. Rename files inside vault
3. Search-and-replace across all source/config/docs
4. Update `Cargo.toml`
5. `cargo build` to verify
6. Run `scripts/verify.sh`
7. Final `rg -i 'jj|jay' --type-not md` sweep to catch stragglers

## References

- All file paths documented in research above
