---
id: mem_01KGAJ0JXNRJKGFV0CJCKTMDNT
title: Debugging Session prompts (debug)
type: source_summary
status: active
tags:
- debugging
- prompts
- rubber-duck
confidence: 0.9
created_at: 2026-01-31T17:36:04.021678Z
updated_at: 2026-01-31T17:36:04.021678Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAJ08ZKE85JDWBVDQPKZFV7
supersedes: []
---
## Summary
This document defines two reusable prompts intended to structure troubleshooting when stuck on a software bug.

1) **Debugging Session prompt**: Instructs the assistant to first read relevant repo code, then accept a structured problem statement with fields for *expected behavior*, *actual behavior*, *what’s been tried* (as a numbered list), and *error messages/logs* (paste block). The assistant is asked to: (a) form hypotheses about what’s wrong, (b) identify what to investigate next, and (c) suggest fixes. It explicitly requests step-by-step reasoning and encourages asking clarifying questions when needed.

2) **Rubber Duck prompt**: A lighter-weight prompt for talking through a problem. The user explains what they’re trying to do, and the assistant’s role is to ask guiding questions that help the user arrive at the answer themselves.

Overall, the source is a small “runbook” of prompt templates for debugging and self-explanation, emphasizing structured inputs, hypothesis-driven investigation, and iterative clarification.
