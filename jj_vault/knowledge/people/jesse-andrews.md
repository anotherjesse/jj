---
id: mem_01KGAKJQS54M6BHQQFF9JPMHRS
title: Jesse Andrews
type: person
status: active
tags:
- cto
- founder
- engineering
- ai
- sparks
- rust
- cloud
confidence: 0.8
created_at: 2026-01-31T18:03:27.397560Z
updated_at: 2026-01-31T20:48:29.544082Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKHQV1JD69416ZPE0RZ06Q
- thread_id: ''
  event_ids:
  - src_01KGAKKWB9TDHWX5V9423PZ1BG
- thread_id: ''
  event_ids:
  - src_01KGAMF16BHX3QRAJSV7DV8F6K
supersedes: []
summary: Jesse Andrews, CTO at LoopWork in Berkeley; ex-NASA OpenStack builder, Planet.com SVP Eng, GitHub @anotherjesse, deep AI (2018–21).
---
## Role
- CTO at LoopWork

## Location
- Berkeley, CA

## Profiles
- GitHub: @anotherjesse

## Background
- Built OpenStack at NASA (helped start the project and built the team).
- Planet.com: SVP Product/Engineering; scaled team to ~75; later Fellow.
- Deep AI experience (2018–2021): embeddings, autoencoders.
- Created userscripts.org; built multiple large open-source projects.

## Notable beliefs / insights
- Belief that LLMs can bridge communication gaps between disciplines (e.g., legal, product, engineering, marketing).
- Early LLM conviction influenced by Riley Goodside’s GPT-3 graphviz demo.
## Updates from Sparks source (2025-01-23)
- Sparks is described as Jesse’s **first Rust project** (quote: “if it compiles, it works”).
- Frames Sparks as “agentic-first cloud compute,” drawing an analogy to OpenStack’s enabling of new capabilities beyond cost savings (scale to thousands, back to zero; shift time/space).

## Notes
These items are beliefs/framings associated with Jesse in the Sparks design doc; they may also represent a broader team narrative, but are attributed to “Jesse” in the source text.


## Immediate priorities (2025-01-23 sync)
- Fix **tmux sizing issue** in SSH sessions (Carl’s #1 blocker; `Ctrl+L` then `reset` is a temporary fix).
- Implement simple **Postgres patterns**: persistent instances for prod; ephemeral/throwaway instances for tests/verification; backup + fork workflow for staging (reference: `test_postgres.sh`).
- Improve SSH ergonomics: enable multiple connections per node; restore **SCP/SFTP** (blocks Zed integration); fix Zed remote-edit links.
- Enable dev Spark containers with **KVM**: run Kate inside container with networking (only required from dev node).

## Update (2026-01-31)
- **tmux sizing in SSH/tmux: resolved** (completed a couple days prior to 2026-01-31). Previously a top operational blocker.
