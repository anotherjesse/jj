---
id: src_01KGAM69X9EWDQ54A2SB15GTCY
title: weekly
ingested_at: 2026-01-31T18:14:08.553339+00:00
original_path: /Users/jesse/cto/prompts/weekly.md
tags: []
processing_status: complete
content_hash: sha256:5dadbac88b6ad13afaf13ab234861855a659c95b435f1840990b318c65705c37
---
# Weekly CTO Review

Run this prompt at the end of each week (Friday) or start of week (Monday).

---

## Prompt

```
Read my CTO system context files in context/, priorities.md, and any logs from this week in logs/.

Help me with my weekly CTO review:

## Retrospective
1. What did we ship/accomplish this week?
2. What didn't get done that should have?
3. What did I learn?
4. What should I stop, start, or continue doing?

## Planning
1. What are the top 3 priorities for next week?
2. Are we on track with our roadmap? Any adjustments needed?
3. Any technical debt accumulating that needs attention?
4. What decisions need to be made?

## Team & Communication
1. What should I sync with Carl about?
2. Any external stakeholders to update?

## Self-Care
1. Am I sustainable at this pace?
2. What's one thing I can do better for my own effectiveness?

Be direct and challenge my thinking. Help me stay focused on what matters most at pre-product stage.
```

---

## After the Session
- Update `context/roadmap.md`
- Update `priorities.md`
- Archive important decisions to `decisions/`
