---
id: mem_01KGAHFNG2H41C12GRWJE9KAFD
title: claude.md from an earlier assistant — CTO Assistant Guide (source summary)
type: source_summary
status: active
tags:
- source
- assistant-guide
- loopwork
confidence: 0.86
created_at: 2026-01-31T17:26:49.602846Z
updated_at: 2026-01-31T17:26:49.602846Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAHEDMYRPF9ZX0JNVZ8T2SZ
supersedes: []
---
## Summary
This document defines a lightweight operating manual for a “CTO Assistant” that acts as a thought partner to **Jesse Andrews** at **LoopWork**. The assistant should use a **telegraphic style** (minimal filler/grammar, minimal tokens) and be explicit about blockers (state what’s missing). It is explicitly **not** a coding agent; separate agents do implementation, though the assistant can write code for research/investigation.

The workflow is structured around persistent context management: at **session start**, read `priorities.md` and relevant files under `context/`. At **session end**, update `priorities.md` if priorities changed, add new information to `context/*.md`, and record significant technical decisions in `decisions/` using an ADR format. The file itself is meant to hold only pointers, with details living in `context/`, `ideas/`, and `decisions/`.

Operational environment notes include repository locations: LoopWork work lives under `~/lw` (clone missing repos from `https://github.com/loopwork/<repo>.git`), and open-source work under `~/oss`. For web research, the assistant should search early, quote exact errors, and prefer sources from **2025–2026**.

Company/product context captured: LoopWork is pre-product with roughly **$2M runway**; Jesse is CTO (Berkeley) and **Carl** is CEO (SF). The team’s Q1 2025 direction was “Cursor/Claude Code for media,” starting with **images**, after pivoting from **Vibewire** (dev tools; oversaturated). A core technical effort is **Sparks** (`~/lw/sparks`): “agentic-first cloud compute” offering instant sandboxed containers and **btrfs snapshots**. The thesis is that AI apps feel “dead” when frozen; agentic behavior is central to the product.

The document also sketches an “AI Context Corpus” vision: capturing conversations and consumption as durable context (Granola transcripts, Readwise Reader highlights, a YouTube summarizer tool `~/lw/yt` storing data in `~/.yt/videos/`, chat exports, Claude logs, and a code-change changelog service). Key insight (2025-01-24): “The conversations themselves ARE the context.”

## Noted unknowns / gaps
- Carl’s background/focus
- ZOMG code location + learnings
- Target image workflows
- Granola API options
