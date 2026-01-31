---
id: mem_01KGAJ2J15F5S2DW73BDP21SZB
title: architecture
type: source_summary
status: active
tags:
- architecture
- adr
- decision-making
- template
confidence: 0.9
created_at: 2026-01-31T17:37:08.645810Z
updated_at: 2026-01-31T17:37:08.645810Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAJ235S6J8WRPQ22AXRRMA1
supersedes: []
---
## Summary
This document defines a lightweight **Architecture Decision Session** workflow for making significant technical decisions, tailored to a very early-stage startup context.

It provides a reusable prompt instructing the assistant to first read CTO system context files in `context/`, then help evaluate an explicitly stated **Decision**, its **Context**, and a numbered list of **Options**. The assistant is asked to:

1. Analyze trade-offs for each option.
2. Prioritize what matters most given the company stage (**2-person, pre-product**).
3. Assess reversibility (how hard it is to change later).
4. Recommend an option with rationale.
5. Surface missing questions that should be asked.

After a decision is reached, the assistant should help document it as an **Architecture Decision Record (ADR)** saved under `decisions/NNNN-title.md` using the included template. The ADR template includes: title `ADR-NNNN`, status (Proposed/Accepted/Deprecated/Superseded), context, decision, consequences, alternatives considered, and a date line.
