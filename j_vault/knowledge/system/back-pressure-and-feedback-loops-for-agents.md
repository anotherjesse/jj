---
id: mem_01KGAKVC1Z34P5MC35F61G9V7Y
title: Back pressure and feedback loops for agents
type: system
status: active
tags:
- agents
- feedback-loops
- verification
confidence: 0.7
created_at: 2026-01-31T18:08:10.303318Z
updated_at: 2026-01-31T18:08:10.303318Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKS8MG1Q0BZ28ZC7MSHR2F
supersedes: []
summary: Back pressure boosts AI agent reliability via immediate automated correctness signals (types, tests, build/UI output) enabling early self-correction.
---
## Summary
“Back pressure” is a pattern for increasing AI agent reliability and horizon length by giving the agent **immediate, automated signals** about correctness and quality (compiler/type errors, tests, build output, UI screenshots). These signals let the agent detect mistakes early and self-correct.

## Mechanisms mentioned
- Typed languages / expressive type systems as natural back pressure.
- High-quality error messages (examples cited: Rust, Elm, Python).
- Tooling that allows agents to run builds/tests and iterate.
- Browser/screenshot automation (Playwright, Chrome DevTools) for UI verification.

## Practice
- “Loop agents until they have stamped out all inconsistencies.”

## Related references (from source)
- RALPH loops: https://ghuntley.com/ralph/
- Factory.ai Agent Readiness