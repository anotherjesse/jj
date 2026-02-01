---
id: mem_01KGAKMBWDMP063JZNZCGRJ3TH
title: OpenClaw tools and skills system (TypeBox + SKILL.md)
type: system
status: active
tags:
- openclaw
- tools
- typebox
- skills
confidence: 0.84
created_at: 2026-01-31T18:04:20.749906Z
updated_at: 2026-01-31T18:04:20.749906Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKJXVY906XCK7PZ4K4TCT8
supersedes: []
summary: 'Spec for OpenClaw tools/skills: define tools with TypeBox object schemas (no unions, use Optional), and skills in SKILL.md YAML frontmatter.'
---
## Tools

- Tools are defined with **TypeBox** schemas (`@sinclair/typebox`) to provide typed parameter definitions with runtime validation.
- Schema guardrails mentioned:
  - Avoid `Type.Union` (i.e., avoid `anyOf`/`oneOf`/`allOf`).
  - Prefer `Type.Optional()` instead of `| null`.
  - Keep the top-level schema as `type: "object"` with `properties`.

## Skills

- Skills are higher-level capabilities defined in `SKILL.md` files with YAML frontmatter (e.g., `name`, `description`, and metadata such as required binaries or environment variables).
- **Precedence hierarchy**: workspace skills override managed skills, which override bundled skills.
- **Load-time gating** filters skills based on required:
  - binaries
  - environment variables
  - config paths
  - OS platform

## Tool availability controls

- Tools can be allow/deny listed per agent.
- Profile presets are mentioned: `minimal`, `coding`, `messaging`, `full`.
- Tool groups provide shorthands, e.g.:
  - `group:fs` (file ops)
  - `group:runtime` (exec/process)
  - `group:web` (search/fetch)