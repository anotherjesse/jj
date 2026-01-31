---
id: mem_01KGAKV0FSYYG6T2BEND68437P
title: AI & Agentic Patterns - Reference Collection
type: source_summary
status: active
tags:
- agents
- context-management
- verification
- feedback-loops
- tooling
- mcp
confidence: 0.78
created_at: 2026-01-31T18:07:58.457827Z
updated_at: 2026-01-31T18:07:58.457827Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKS8MG1Q0BZ28ZC7MSHR2F
supersedes: []
---
## Summary
This document is a curated reference collection of practitioner patterns for building and working with AI coding agents. A dominant theme is **closing feedback loops** so agents can self-correct without consuming expensive human review (“back pressure”). Banay argues that teams should stop spending human attention on trivial fixups (e.g., missing imports) by giving agents the ability to run builds/tests, read errors, and iterate until inconsistencies are eliminated. Strong type systems and high-quality compiler errors (e.g., Rust/Elm) function as “natural back pressure,” and UI automation (e.g., Playwright/DevTools screenshots) extends verification beyond code.

A complementary practitioner workflow is described by **Peter Steinberger** (“Shipping at Inference-Speed”): with advanced coding models (GPT‑5.2/Codex), the bottleneck shifts from writing code to inference latency and hard thinking. His workflow emphasizes parallelism (3–8 projects at once), heavy use of queued tasks, minimal manual code reading, and maintaining per-project docs folders to stabilize context. He recommends starting projects as a **CLI-first** surface so agents can invoke it, verify output, and automate skills.

**Factory.ai’s Agent Readiness** frames uneven agent performance as primarily a codebase problem: repos need fast local feedback (pre-commit hooks), clear environment/build docs, solid tests, CI/CD, security scanning, explicit ownership, and branch protections. It offers maturity levels and reporting via CLI/dashboard/API, with actionable targets like “% agent-ready repos.”

On creativity, **Lluminate** (Joel Simon) addresses LLM homogenization using evolutionary search plus explicit creative strategies (e.g., Oblique Strategies, SCAMPER), selecting for novelty via embedding distance; findings include mutation/crossover outperforming de novo generation.

For reading/context use, fast.ai authors propose **LLM-assisted close reading**: convert text to Markdown, create chapter summaries for context, avoid spoilers, and generate end-of-chapter overviews (optionally Anki).

Finally, Cloudflare’s **Code Mode** suggests an architecture where the model writes TypeScript that calls MCP tools (executed in a V8 isolate) rather than making direct tool calls, leveraging abundant TypeScript priors and enabling multi-step chaining without repeatedly re-tokenizing outputs.

## Notable cross-cutting themes
- Context management as a first-class artifact (docs, summaries-of-summaries, cross-project references).
- Automated feedback loops over human review time.
- CLI-first interfaces for verifiable agent workflows.
- Probabilistic iteration: many experiments over one “golden attempt.”
