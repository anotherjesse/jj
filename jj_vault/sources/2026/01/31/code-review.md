---
id: src_01KGAJ8MPQPH2C3RZF77BY3A08
title: code-review
ingested_at: 2026-01-31T17:40:27.991995+00:00
original_path: /Users/jesse/cto/prompts/code-review.md
tags: []
processing_status: complete
content_hash: sha256:246c600cd5fe8dc038b1a84b46eb03a221bb115975e80cb523408360aace243b
---
# Code Review Session

Run this when you want a thorough code review.

---

## Prompt

```
Read my CTO system context files in context/.

I need a code review. Please review [file/PR/directory]:

Focus on:
1. **Correctness**: Does it work? Edge cases?
2. **Security**: Any vulnerabilities? (OWASP top 10)
3. **Performance**: Any obvious bottlenecks?
4. **Maintainability**: Will this be easy to change later?
5. **Simplicity**: Is this overengineered for our stage?

Given we're pre-product with a 2-person team:
- Prioritize shipping over perfection
- Flag only issues that matter NOW
- Suggest simplifications where possible

Be direct. Tell me what's wrong, not what's right.
```

---

## Quick Review Prompt (for smaller changes)

```
Quick review this code. Pre-product startup, optimize for speed. Only flag real issues:

[paste code]
```
