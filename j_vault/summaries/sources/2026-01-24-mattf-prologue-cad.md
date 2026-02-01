---
id: mem_01KGAMKZK1CG73EEH3BE751PXM
title: '2026-01-24 — MattF: Prologue CAD, Sparks infra, skills vs MCP, and genetic novelty search'
type: source_summary
status: active
tags:
- source
- meeting-notes
- cad
- sparks
- skills
- genetic-algorithms
confidence: 0.78
created_at: 2026-01-31T18:21:36.737198Z
updated_at: 2026-01-31T18:21:36.737198Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAMJZD7RGD1SHMHS18R8A2G
supersedes: []
---
## Summary
Notes from a 2026-01-24 conversation with **MattF** (friend from Planet; J invested in MattF’s startup **Feather**) covering health, J’s modular/CLI-first infra philosophy, and MattF’s **Prologue CAD** generative design product.

### Health / personal
J reports being sick since **Dec 20** with a lingering cough; X-rays show nothing; seeing pulmonologists and getting a **CAT scan**. This apparently recurs annually.

### AI/dev philosophy & tool architecture
A recurring theme is building **accretive, composable systems** (“LEGO-like”), aligned with the **UNIX philosophy** (small tools, pipeable). J is moving away from a monolithic “**Vibewire**”-style approach toward independent modules:
- A Rust **kanban** manager (proto with 5 columns).
- **Sparks** (container/forking system).
- Multiple specialized **coding agents**.

The kanban UX: dragging a task to **“ready”** triggers a new Claude coding session in a **fresh fork**. The architecture is described as RPC-style state management.

### Sparks / infra details
Sparks enables near-instant environments with SSL + filesystem, fast **fork/snapshot** (claimed ~**0.25s**) via **btrfs** copy-on-write snapshots with diff tracking and streaming backups to **GCS**. An “**Eat My Data**” mode disables fsync in cloud contexts and uses **RAID10 local SSDs**, claiming up to **6,000×** disk speed improvement.

### Skills vs MCP tools
“Skills” are lightweight bundles (markdown instructions + scripts) that can be loaded probabilistically based on context (e.g., a frustration-triggered methodical debugging skill). Compared to MCP schemas, skills/CLI tooling are preferred for **context efficiency** and lighter-weight triggers.

### Prologue CAD: product concept and GTM
**Prologue CAD** is a 3-person team building an AI-powered CAD exploration tool optimized for **rapid forking and evaluation** (“**Tinder for CAD**”). Users swipe through many design variants on a canvas, with provenance tracking, and export **STEP files** to integrate with traditional CAD. The product emphasizes exploration over refinement, with a two-phase workflow (mass forking → basic editing) and built-in manufacturability checks (e.g., injection molding, CNC). GTM is positioned as **$40/mo** “where parts begin,” explicitly not trying to replace existing CAD.

### Novelty injection via genetic algorithms
A related effort (“**Luminate/Lluminate**” mentioned) uses genetic algorithms to drive novelty: fitness computed via **CLIP embedding** distances for diversity; mutation/crossover performed in **prompt space**; human feedback incorporated via **ELO-style** ranking. CAD can be rendered as **depth maps** and used with “Nano Banana Pro” image generation for realistic industrial mockups.

## Source
- Meeting transcript link: https://notes.granola.ai/t/2912f311-6d69-486b-bad7-de001cf2f35c-00demib2
- Source ID: `src_01KGAMJZD7RGD1SHMHS18R8A2G`
