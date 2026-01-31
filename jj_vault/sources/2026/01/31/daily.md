---
id: src_01KGAM3XVP0HWCMX3EZEKBCEYN
title: daily
ingested_at: 2026-01-31T18:12:50.678773+00:00
original_path: /Users/jesse/cto/prompts/daily.md
tags: []
processing_status: complete
content_hash: sha256:65b3294ec40b955bb30b4be4fa5433bf646394ba9943bae546070ace9520297b
---
# Daily CTO Check-in

Run this prompt each morning to start your day focused.

---

## Prompt

```
Read my CTO system context files in context/ and priorities.md.

Help me with my daily CTO check-in:

1. **Yesterday Review**: What did I accomplish? (I'll share)
2. **Today's Focus**: Based on my priorities and roadmap, what should I focus on today?
3. **Blockers**: Help me identify and strategize around any blockers
4. **Quick Wins**: Are there any quick wins I should tackle?
5. **Carl Sync**: Anything I should sync with Carl about today?

Keep responses concise and actionable. I'm a 2-person pre-product startup - speed matters.
```

---

## How to Run

```bash
cd ~/cto && claude
# Then paste the prompt above, or reference this file
```

## After the Session
- Update `logs/YYYY-MM-DD.md` with key decisions
- Update `priorities.md` if priorities shift
