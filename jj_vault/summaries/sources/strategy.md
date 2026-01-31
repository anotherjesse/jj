---
id: mem_01KGAMD0DSK8HYYRF5YGCXKNBQ
title: Strategy session prompts (technical strategy + competitive analysis)
type: source_summary
status: active
tags:
- strategy
- cto-system
- prompts
confidence: 0.88
created_at: 2026-01-31T18:17:48.217775Z
updated_at: 2026-01-31T18:17:48.217775Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAMCKAT6BVGB7B883DM1DRK
supersedes: []
---
## Summary
This source defines two reusable **session-starter prompts** meant for bigger-picture technical thinking within JJ’s “CTO system.”

1) **Technical Strategy Session**: A guided prompt to help JJ think through an open-ended strategic technical question (to be filled in under **Topic**). The assistant is instructed to first read the **CTO system context files in `context/`** and then help evaluate common early-stage tradeoffs for a **2-person, pre-product team** (optionally noting funding status). The prompt explicitly asks the assistant to **challenge assumptions** and surface blind spots, including what a “world-class CTO” would do differently at this stage. Example framing questions include:
- Appropriate investment level in **infrastructure vs. features**
- **Build vs. buy vs. open source** decisions
- When to **hire the first engineer**
- What **technical bets** are being made
- Where to **take shortcuts** vs. invest in **quality**

2) **Competitive Analysis Prompt**: A structured prompt for thinking about **technical competitive positioning** relative to a named competitor/alternative. After reading the CTO system context, the assistant should assess (based on public info): the competitor’s technical approach, likely technical advantages, the team’s opportunities, lessons to learn, and what to intentionally do differently.

Overall, the document formalizes how the assistant should be used for strategy work: read existing context first, then provide direct, assumption-challenging guidance tailored to the team’s early stage.
