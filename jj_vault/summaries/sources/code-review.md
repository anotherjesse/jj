---
id: mem_01KGAJ92Z18H644BN576ET7HKX
title: code-review
type: source_summary
status: active
tags:
- code-review
- prompt
- engineering
confidence: 0.93
created_at: 2026-01-31T17:40:42.593701Z
updated_at: 2026-01-31T17:40:42.593701Z
sources:
- thread_id: src_01KGAJ8MPQPH2C3RZF77BY3A08
  event_ids: []
supersedes: []
---
## Summary
This document defines a reusable **code review session prompt** intended for a **pre-product, 2-person startup**. It instructs the assistant to first read CTO system context files located in `context/`, then perform a thorough review of a specified file/PR/directory.

The review is explicitly scoped to what matters *now* (shipping over perfection) and asks for direct, critical feedback: “Tell me what’s wrong, not what’s right.” It prioritizes actionable issues and simplifications over exhaustive best-practice polish.

The thorough review is organized around five pillars:
1. **Correctness**: verify behavior and call out edge cases.
2. **Security**: identify vulnerabilities, explicitly referencing **OWASP Top 10** as a guide.
3. **Performance**: flag obvious bottlenecks.
4. **Maintainability**: assess how easy the code will be to change.
5. **Simplicity**: detect overengineering relative to the current stage.

A second, shorter **Quick Review Prompt** is included for smaller changes. It reiterates the startup context (optimize for speed) and asks to “only flag real issues,” with the code pasted inline.

## Intended usage
- Use the full prompt for substantial PRs or architectural changes.
- Use the quick prompt for small diffs where fast feedback is more valuable than deep analysis.
