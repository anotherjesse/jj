---
id: mem_01KGAJB33G8EASEQST61E5PYB7
title: 'Strategy prompts: technical strategy session + competitive analysis'
type: source_summary
status: active
tags:
- strategy
- cto
- prompt
- competitive-analysis
- workflow
confidence: 0.86
created_at: 2026-01-31T17:41:48.272439Z
updated_at: 2026-01-31T17:41:48.272439Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAJAPWFA6EP35BG7WQN7W07
supersedes: []
---
## Summary
The document defines two reusable prompts for running higher-level CTO thinking sessions, intended for a **2-person team** at a **pre-product** stage (optionally with funding context). The first prompt, **Technical Strategy Session**, instructs the assistant to read existing CTO system context files in `context/` and then help think through a strategic technical question. It provides framing questions to drive discussion, including: the appropriate investment balance between **infrastructure vs. features**, **build vs. buy vs. open source** choices, timing for hiring the **first engineer**, identifying and sizing key **technical bets**, and deciding where to **take shortcuts** vs. invest in **quality**. It explicitly asks the assistant to **challenge assumptions**, surface blind spots, and compare the user’s approach with what a “world-class CTO” would do at this stage.

The second prompt, **Competitive Analysis Prompt**, focuses on **technical competitive positioning** against a named competitor or alternative. After reading the CTO system context, the assistant should infer (from public information) the competitor’s technical approach, identify likely technical advantages, propose areas where the user’s team might have technical opportunities, articulate what to learn from the competitor, and specify what to intentionally do differently.

Overall, the document is a lightweight workflow artifact: it standardizes inputs (topic/competitor + context), sets a critical/strategic assistant stance, and provides structured question lists to produce clearer technical direction and differentiation.
