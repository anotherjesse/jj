---
id: mem_01KGAKZNJXHWVSVW0CW2WVWSQH
title: Debugging Session prompt template (debug)
type: source_summary
status: active
tags:
- debugging
- prompt
- rubber-duck
confidence: 0.93
created_at: 2026-01-31T18:10:31.133865Z
updated_at: 2026-01-31T18:10:31.133865Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKZ7WZ4HP7N9VGQ6A2S732
supersedes: []
---
## Source
- Title: debug
- Source ID: src_01KGAKZ7WZ4HP7N9VGQ6A2S732
- Source: unknown

## Summary
This document defines two reusable prompts for debugging.

1) **Debugging Session prompt**: A structured template to run when stuck on a bug. It instructs the assistant to first read the relevant repository code, then diagnose an issue based on four provided sections: expected behavior, actual behavior, what has been tried (enumerated), and any error messages/logs (pasted). The assistant should then: (a) form hypotheses about what’s wrong, (b) identify what to investigate next, and (c) suggest fixes. It explicitly requests step-by-step reasoning and clarifying questions if needed.

2) **Rubber Duck prompt**: A lighter-weight template for talking through a problem. The user explains what they are trying to do, and the assistant’s role is to ask questions that help the user find the answer themselves.

## Notable details / constraints
- The debugging prompt expects repo context (“Read the relevant code in this repo”).
- Both prompts encourage clarifying questions; the debugging prompt also requests explicit hypothesis generation and investigation plan.
