---
id: mem_01KGAJ4YZ4FWK7ZMGHAK1N2G06
title: Daily CTO Check-in prompt (source)
type: source_summary
status: active
tags:
- workflow
- daily
- cto
- prompt
confidence: 0.9
created_at: 2026-01-31T17:38:27.428875Z
updated_at: 2026-01-31T17:38:27.428875Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAJ4A3YVRTWWG5CA1139WBN
supersedes: []
---
## Summary
This source defines a daily **CTO check-in** workflow prompt intended to be run each morning to keep execution focused. The assistant is instructed to **read CTO system context files in `context/` and `priorities.md`** first, then guide a structured check-in:

1. **Yesterday Review**: user shares accomplishments; assistant helps reflect.
2. **Today’s Focus**: recommend what to prioritize based on priorities/roadmap.
3. **Blockers**: identify blockers and propose strategies to resolve them.
4. **Quick Wins**: suggest small high-leverage tasks.
5. **Carl Sync**: call out anything to sync with **Carl** that day.

Operational guidance: keep responses **concise and actionable**; assume a **2-person pre-product startup** where speed matters.

## How to run
- Run from `~/cto` using `claude`, then paste/reference this prompt.

## After the session
- Update `logs/YYYY-MM-DD.md` with key decisions.
- Update `priorities.md` if priorities shift.
