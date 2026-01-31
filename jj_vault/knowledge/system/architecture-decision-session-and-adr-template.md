---
id: mem_01KGAM2769N1M9W864633TEX4D
title: Architecture decision session workflow + ADR template
type: system
status: active
tags:
- architecture
- adr
- process
- decision-making
confidence: 0.9
created_at: 2026-01-31T18:11:54.697356Z
updated_at: 2026-01-31T18:11:54.697356Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAM1GY4Z99KMV2ZZSJZK5HD
supersedes: []
---
## Purpose
A standardized workflow for making **significant technical/architecture decisions** with the assistant.

## Procedure
1. **Read system context**: Before advising, the assistant should read the CTO system context files in `context/`.
2. Collect decision inputs:
   - **Decision**: what is being decided
   - **Context**: why it matters now
   - **Options**: list the candidate approaches
3. Analyze and answer:
   1. Trade-offs of each option
   2. What matters most given the stage (**2-person, pre-product**)
   3. Reversibility of the decision
   4. Recommendation and rationale
   5. Additional questions to ask
4. **Document outcome as an ADR**: Save a record to `decisions/NNNN-title.md` using the ADR template in the source.

## ADR template fields
- Title: `# ADR-NNNN: [Title]`
- Status: Proposed | Accepted | Deprecated | Superseded
- Context
- Decision
- Consequences
- Alternatives Considered
- Date line: `*Date: YYYY-MM-DD*`
