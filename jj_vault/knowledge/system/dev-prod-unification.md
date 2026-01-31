---
id: mem_01KGAKR399DVJJ83QAYMYDX8B5
title: Dev/Prod unification (single flow with flags)
type: system
status: active
tags:
- devops
- deployment
- reliability
- workflow
confidence: 0.8
created_at: 2026-01-31T18:06:23.017223Z
updated_at: 2026-01-31T18:06:23.017223Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKP9BVDCXHC4JF6841GQZS
supersedes: []
summary: Unify dev/prod by 2025-01-23 into one deploy/setup/verify flow using flags (not separate scripts) to reduce divergence and match prod.
---
## Fact / requirement
As of **2025-01-23**, **dev/prod unification** is a NEW, **blocking** priority: dev and prod currently use different scripts/flows/settings.

## Desired end state
- A **single path** for:
  - deploy
  - setup
  - verify
- Differences controlled via **flags** rather than divergent tooling.

## Rationale
- Reduce operational divergence and improve confidence that dev workflows match production behavior.