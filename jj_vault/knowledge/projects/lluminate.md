---
id: mem_01KGAKV0FYSEVWPVB3QY73Y653
title: Lluminate
type: project
status: active
tags:
- creative-coding
- evolutionary-search
- llms
confidence: 0.77
created_at: 2026-01-31T18:07:58.462410Z
updated_at: 2026-01-31T18:07:58.462410Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKS8MG1Q0BZ28ZC7MSHR2F
supersedes: []
---
## Summary
Lluminate is a project by Joel Simon (Dec 2025) that uses evolutionary search plus explicit creative strategies to drive open-ended creative exploration with reasoning LLMs, counteracting homogeneous “average” outputs.

## Approach (as described)
1. Create a population summary for context.
2. Inject a random creative strategy (e.g., Oblique Strategies, SCAMPER, conceptual blending).
3. Evolve outputs via mutation and crossover.
4. Embed artifacts and score novelty via cosine distance to k nearest neighbors.
5. Select the most diverse individuals for the next generation.

## Key findings (as described)
- Mutating existing artifacts outperforms generating from scratch for novelty.
- Crossover increases novelty strongly.
- Novelty correlates with complexity (longer code).
- Increasing reasoning level did not significantly increase diversity.

## Links
- Write-up: https://www.joelsimon.net/lluminate
- Repo: https://github.com/joel-simon/lluminate
