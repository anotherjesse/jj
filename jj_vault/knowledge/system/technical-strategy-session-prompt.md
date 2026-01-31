---
id: mem_01KGAMDAFA49CZRKB1YNAF8Q4K
title: Technical strategy session + competitive analysis prompts
type: system
status: active
tags:
- cto-system
- prompt
- strategy
- competitive-analysis
confidence: 0.84
created_at: 2026-01-31T18:17:58.506318Z
updated_at: 2026-01-31T18:17:58.506318Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAMCKAT6BVGB7B883DM1DRK
supersedes: []
---
## Technical strategy session prompt
A reusable prompt for conducting **bigger-picture technical strategy** discussions.

### Instruction
- **Read CTO system context files in `context/`** before advising.

### Inputs
- **Topic**: the strategic technical question being wrestled with.
- **Context**: assumes a **2-person, pre-product team**; optionally add funding status.

### Framing questions (examples)
- Right level of investment in **infrastructure vs. features**
- **Build vs. buy vs. open source**
- When to **hire the first engineer**
- What **technical bets** are being made
- Where to **take shortcuts** vs. invest in **quality**

### Desired assistant behavior
- **Challenge assumptions**.
- Surface blind spots (“What am I missing?”).
- Suggest what a **world-class CTO** might do differently at this stage.

## Competitive analysis prompt (technical positioning)
A structured prompt to analyze a named **Competitor/Alternative**.

### Instruction
- **Read CTO system context** before analysis.

### Questions to answer
1. Competitor’s **technical approach** (based on public info)
2. Where they may have **technical advantages**
3. Where we may have **technical opportunities**
4. What we should **learn** from them
5. What we should **intentionally do differently**
