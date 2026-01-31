---
id: mem_01KGAJKA5PFT8VRP7SJY80Q6J6
title: 2026-01-24 — MattF on Prologue CAD, modular AI tooling, and genetic design exploration
type: source_summary
status: active
tags:
- source
- cad
- generative-design
- sparks
- skills
- mcp
- verification
- genetic-algorithms
confidence: 0.8
created_at: 2026-01-31T17:46:17.654773Z
updated_at: 2026-01-31T17:46:17.654773Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAJHJRN2PB47W6N7CVR3BDJ
supersedes: []
---
## Summary
MattF (friend from Planet; investor connection via his startup Feather) shared updates on health and a deep dive into his AI tooling philosophy and the **Prologue CAD** project. He’s been sick since Dec 20 with a lingering cough, is seeing pulmonologists, and is getting a CAT scan (X-rays inconclusive); he noted this tends to recur annually.

On AI development, he emphasized building **accretive, Lego-like systems** aligned with the **UNIX philosophy**: small tools that compose via piping rather than a monolith. He described moving away from a monolithic “vibewire” approach into modular pieces such as a Rust Kanban manager, **Sparks** (a container/forking/snapshot system), and separate coding agents. In his workflow, dragging a Kanban item to “ready” triggers a Claude coding session in a fresh fork. The infrastructure uses an RPC-style architecture for state management; Sparks provides instant container creation (SSL + filesystem), ~0.25s fork/snapshot via **btrfs** copy-on-write with diff tracking, and streaming backups to GCS. He also mentioned an “Eat My Data” mode that disables fsync in cloud contexts and uses RAID10 local SSDs for very large disk speedups.

He contrasted “**skills**” (markdown instructions + scripts, lightweight triggers, probabilistic loading) with heavier MCP schema-based tools, preferring CLI-based tools for context efficiency. Example: a frustration-triggered skill that forces methodical debugging.

**Prologue CAD** is a 3-person effort to make CAD **exploratory**: generate many variants and let users rapidly evaluate (“Tinder for CAD”), with a canvas for idea surfing and provenance, exporting STEP files for conventional CAD. The product is optimized for idea forking (not final refinement), with a two-phase flow (exploration → basic editing) and built-in manufacturability checks (injection molding, CNC). GTM positioning: ~$40/mo as “where parts begin,” a Trojan-horse complement to existing CAD; plan to add analysis/FEA later. Current progress includes a working pillow block generator producing ~10 variations using build123D, possibly exporting to OnShape.

For novelty, he referenced **Lluminate-style** genetic algorithms: mutation/crossover in prompt space; novelty/fitness via CLIP embedding distance; and human feedback via ELO-like rankings. He also described rendering CAD depth maps and using high-quality image generation (“Nano Banana Pro”) for realistic industrial mockups.

## Source links
- Meeting transcript: https://notes.granola.ai/t/2912f311-6d69-486b-bad7-de001cf2f35c-00demib2
