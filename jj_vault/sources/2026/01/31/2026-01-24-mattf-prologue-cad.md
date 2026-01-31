---
id: src_01KGAMJZD7RGD1SHMHS18R8A2G
title: 2026-01-24-mattf-prologue-cad
ingested_at: 2026-01-31T18:21:03.783922+00:00
original_path: /Users/jesse/cto/meetings/2026-01-24-mattf-prologue-cad.md
tags: []
processing_status: complete
content_hash: sha256:cdc45bf760f46604ddb892941c4c37c1f28d2e21278cb646b488bf70bab59a09
---
# AI tools and generative design exploration with prologue CAD project

Sat, 24 Jan 26

**With:** MattF (https://mattferraro.dev/) - friend from Planet, invested in his startup Feather

---

### Health Update & CAD Progress

- Sick since Dec 20 with lingering cough
- Seeing pulmonologists, getting CAT scan (X-rays show nothing)
- Similar pattern happens annually

### AI Development Philosophy

- Focus on building accretive systems with Lego-like composability
- UNIX philosophy: small tools doing one thing well
- Moving from monolithic vibewire to independent modular pieces
  - Kanban management (rust app)
  - Sparks (container/forking system)
  - Separate coding agents
- Enable piping/composition between tools

### Infrastructure Deep Dive

- Built Proto Kanban experience with 5 columns
- Drag to "ready" kicks off Claude code session with new fork
- RPC-style architecture for state management
- Sparks system features:
  - Instant container creation with SSL/filesystem
  - 0.25 second fork/snapshot capability using butterfs
  - Copy-on-write snapshots, diff tracking
  - Streaming to GCS for backup
- "Eat My Data" mode provides 6,000x disk speed improvement
  - Disables fsync for cloud environments
  - Uses RAID 10 with local SSDs

### Skills vs MCP Tools

- Skills package markdown instructions + scripts
- Lightweight trigger summaries vs heavy MCP schemas
- CLI tools preferred over MCP for context efficiency
- Skills allow probabilistic loading based on context
- Example: frustration-triggered skill for methodical debugging

### Prologue CAD Project Overview

- Three-person team building AI-powered CAD exploration tool
- Core concept: generate many variations, let users quickly evaluate
- "Tinder for CAD" - swipe through designs rapidly
- Canvas interface for surfing ideas with provenance tracking
- Outputs step files for integration with traditional CAD

### CAD Forking Innovation

- Traditional CAD programs penalize idea forking
- Prologue optimized for exploration, not final refinement
- Two-phase approach:
  1. Idea exploration with massive forking capability
  2. Basic CAD editing for final tweaks
- Built-in manufacturability checking (injection molding, CNC)

### Go-to-Market Strategy

- $40/month service positioned as "where parts begin"
- Trojan horse approach - avoid replacing existing CAD workflows
- Gradual feature addition (FEA, analysis) over time
- Current progress: working pillow block generator with 10 variations
- Using build123D scripts, considering export to OnShape

### Technical Implementation & Genetic Algorithms

- Luminate project for injecting novelty via genetic algorithms
- Fitness function uses CLIP embeddings for visual diversity
- Mutation/crossover in prompt space rather than code
- Human feedback integration through ELO-style ranking
- Applications: T-shirt design, 3D printing with SDFs
- Nano Banana Pro for high-quality image generation

---

## Key Insights for LoopWork

**Verification-driven development pattern:**
- Owner verification/QA, not implementation
- Coding agents iterate until all verification passes
- `just check`, `just deploy`, `just verify` pattern
- Allows 30-60 minute autonomous agent runs

**Genetic algorithms + LLMs:**
- LLMs bad at creativity/novelty alone
- GA injects variation, LLM compiles prompts to artifacts
- Fitness = distance in CLIP embedding space (novelty)
- Human feedback can influence fitness scores

**Depth maps + Nano Banana:**
- Render CAD as depth maps
- Use as reference images for realistic mockups
- "Put this part in industrial setting" type prompts

---

Chat with meeting transcript: [https://notes.granola.ai/t/2912f311-6d69-486b-bad7-de001cf2f35c-00demib2](https://notes.granola.ai/t/2912f311-6d69-486b-bad7-de001cf2f35c-00demib2)
