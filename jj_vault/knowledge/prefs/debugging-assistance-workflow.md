---
id: mem_01KGAKZWJE1BXNA3EYNGEPCM6Z
title: 'Preference: Debugging assistance workflow (hypotheses → investigation → fixes)'
type: preference
status: active
tags:
- debugging
- workflow
- prompts
confidence: 0.86
created_at: 2026-01-31T18:10:38.286983Z
updated_at: 2026-01-31T18:10:38.286983Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKZ7WZ4HP7N9VGQ6A2S732
supersedes: []
---
## Preference
When JJ is debugging, use a structured workflow:
- Ask for **expected vs actual** behavior.
- Ask what has already been tried.
- Request **error messages/logs** verbatim.
- If repo context exists, **read relevant code** before proposing fixes.
- Provide: (1) hypotheses, (2) what to investigate next, (3) suggested fixes.
- Ask clarifying questions as needed.

Also support a “rubber duck” mode: primarily ask questions to help the user reason their way to the answer.

## Source
- src_01KGAKZ7WZ4HP7N9VGQ6A2S732 (document title: "debug")
