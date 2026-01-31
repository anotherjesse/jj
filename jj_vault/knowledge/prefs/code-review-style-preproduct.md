---
id: mem_01KGAMA58NTAP1H82M68DP1YGF
title: 'Preference: code review style (pre-product, ship-first)'
type: preference
status: active
tags:
- code-review
- speed
- simplicity
- security
confidence: 0.86
created_at: 2026-01-31T18:16:14.869185Z
updated_at: 2026-01-31T18:16:14.869185Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAM8JFYJG4W7GKVD5BGF0G4
supersedes: []
---
## Statement
When requesting code reviews, JJ wants feedback optimized for a **pre-product, 2-person team**:
- Prioritize **shipping over perfection**.
- **Flag only issues that matter now** (avoid pedantry and premature optimization).
- Suggest **simplifications** where possible.
- Be **direct** and focus on what’s wrong rather than praising what’s right.

## Review focus areas
1. Correctness (edge cases)
2. Security (OWASP Top 10 lens)
3. Performance (obvious bottlenecks)
4. Maintainability (ease of future change)
5. Simplicity (avoid overengineering for current stage)

## Usage patterns
- “Thorough” review: review a specified file/PR/directory after reading `context/`.
- “Quick” review: for smaller changes, paste code and only flag real issues.

## Related
- This overlaps with the broader preference for direct, challenging feedback, but is specific to **code review** and **stage-appropriate pragmatism**.
