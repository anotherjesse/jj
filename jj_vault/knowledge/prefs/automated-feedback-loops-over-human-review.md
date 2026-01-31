---
id: mem_01KGAKVC1YXZ69B8WZFVD677NE
title: 'Preference: automated feedback loops over human review'
type: preference
status: active
tags:
- prefs
- verification
- agents
- devex
confidence: 0.72
created_at: 2026-01-31T18:08:10.302472Z
updated_at: 2026-01-31T18:08:10.302472Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKS8MG1Q0BZ28ZC7MSHR2F
supersedes: []
---
## Statement
Prefer engineering setups where agents can **run and verify** work end-to-end via fast, automated feedback loops (build/test/typecheck/UI checks), minimizing human review time.

## Rationale
The collection repeatedly emphasizes that agent effectiveness depends on tight feedback loops (“back pressure”), CLI-first surfaces, and automated verification.

## Practical implications
- Provide agents first-class access to build/test tools and error outputs.
- Add pre-commit hooks and local checks to shorten feedback latency.
- Use UI automation (screenshots/browser control) to verify UX changes.
- Start new functionality as a CLI when possible to enable closed-loop agent execution.
