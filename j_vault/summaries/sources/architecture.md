---
id: mem_01KGAM1WVESWXJK7C8CSJKQVPK
title: Architecture decision session prompt + ADR template
type: source_summary
status: active
tags:
- architecture
- adr
- process
- decision-making
confidence: 0.93
created_at: 2026-01-31T18:11:44.110358Z
updated_at: 2026-01-31T18:11:44.110358Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAM1GY4Z99KMV2ZZSJZK5HD
supersedes: []
---
## Summary
This source defines a lightweight **Architecture Decision Session** process for making significant technical decisions, especially suitable for an early-stage team.

The session prompt instructs the assistant to first read existing **CTO system context files** under `context/`, then work through a decision using a structured set of inputs:
- **Decision**: what is being decided
- **Context**: why it matters now
- **Options**: enumerated choices

The assistant is then asked to analyze:
1. Trade-offs of each option
2. What matters most given the team’s stage (**2-person, pre-product**)
3. Reversibility of the decision
4. Recommendation + rationale
5. Missing questions that should be asked

After reaching a conclusion, the assistant should document the outcome as an **ADR (Architecture Decision Record)** saved under `decisions/NNNN-title.md` using the included template.

## ADR Template (key fields)
An ADR document includes:
- **ADR-NNNN: Title**
- **Status**: Proposed | Accepted | Deprecated | Superseded
- **Context**: why the decision is needed
- **Decision**: what was decided
- **Consequences**: implications
- **Alternatives Considered**: what else was evaluated
- **Date**: YYYY-MM-DD
