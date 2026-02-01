---
id: mem_01KGAM909GVTGW385BFYMHBF08
title: code-review (source summary)
type: source_summary
status: active
tags:
- code-review
- prompt
- process
confidence: 0.95
created_at: 2026-01-31T18:15:37.008807Z
updated_at: 2026-01-31T18:15:37.008807Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAM8JFYJG4W7GKVD5BGF0G4
supersedes: []
---
## Summary
This document defines a reusable **code review session prompt** intended for a pre-product startup with a **2-person team**, optimizing for shipping velocity over perfection.

The primary prompt instructs the assistant to first **read CTO system context files located in `context/`**, then review a specified **file/PR/directory** with attention to five dimensions:
1. **Correctness**: whether the code works and covers edge cases.
2. **Security**: vulnerabilities with emphasis on **OWASP Top 10**.
3. **Performance**: obvious bottlenecks.
4. **Maintainability**: ease of future change.
5. **Simplicity**: whether the solution is overengineered for the company’s current stage.

The guidance explicitly sets prioritization rules for feedback:
- **Prioritize shipping over perfection**.
- **Flag only issues that matter now** (avoid pedantic or premature optimization).
- **Suggest simplifications** where possible.
- **Be direct**: focus on what’s wrong rather than praising what’s right.

A secondary “Quick Review” prompt is provided for smaller changes: paste the code and ask for a fast review that **only flags real issues**, again optimizing for speed in a pre-product environment.
