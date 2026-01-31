---
id: src_01KGAJ235S6J8WRPQ22AXRRMA1
title: architecture
ingested_at: 2026-01-31T17:36:53.433244+00:00
original_path: /Users/jesse/cto/prompts/architecture.md
tags: []
processing_status: complete
content_hash: sha256:0acc192da40ccd12563397669748f96d636389400f56729c28bb5e5461ab7c37
---
# Architecture Decision Session

Run this when you need to make a significant technical decision.

---

## Prompt

```
Read my CTO system context files in context/.

I need help making an architecture decision:

**Decision**: [Describe what you're deciding]

**Context**: [Why this decision matters now]

**Options I'm considering**:
1.
2.
3.

Help me think through this:
1. What are the trade-offs of each option?
2. Given our stage (2-person, pre-product), what matters most?
3. What's the reversibility of this decision?
4. What would you recommend and why?
5. What questions should I be asking that I'm not?

After we decide, help me document this in ADR format for decisions/.
```

---

## ADR Template

After the decision, save to `decisions/NNNN-title.md`:

```markdown
# ADR-NNNN: [Title]

## Status
[Proposed | Accepted | Deprecated | Superseded]

## Context
[Why is this decision needed?]

## Decision
[What did we decide?]

## Consequences
[What are the implications?]

## Alternatives Considered
[What else did we consider?]

---
*Date: YYYY-MM-DD*
```
