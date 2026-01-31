---
id: mem_01KGAKV0FZ5W3EQZHWMDMBK7PS
title: Factory.ai Agent Readiness
type: project
status: active
tags:
- codebase-readiness
- ai-agents
- devex
confidence: 0.75
created_at: 2026-01-31T18:07:58.463134Z
updated_at: 2026-01-31T18:07:58.463134Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKS8MG1Q0BZ28ZC7MSHR2F
supersedes: []
---
## Summary
Factory.ai “Agent Readiness” (Jan 2026) is a framework and set of tools for assessing whether a codebase’s feedback loops and engineering practices support reliable AI agent contributions.

## Core claim
Uneven agent outcomes are usually caused by the **codebase** (slow/unclear feedback loops, missing docs, weak tests), not by the model.

## Technical pillars (8)
1. Pre-commit hooks
2. Environment documentation
3. Build process documentation
4. Testing infrastructure
5. CI/CD configuration
6. Security scanning
7. Code ownership (CODEOWNERS)
8. Branch protection

## Maturity levels (5)
- Level 1: Basic
- Level 2: Foundational
- Level 3: Agent-ready (target for most repos)
- Level 4: Comprehensive
- Level 5: Exemplary

## Interfaces
- CLI: `/readiness-report` (Factory Droid)
- Dashboard: https://app.factory.ai/analytics/readiness
- API: programmatic access for CI/CD integration

## Metrics
- Emphasizes actionable metrics like “% of active repos agent-ready” over averages.

## Source
- https://factory.ai/news/agent-readiness
