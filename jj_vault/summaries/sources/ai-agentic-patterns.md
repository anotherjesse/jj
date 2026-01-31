---
id: mem_01KGAHYTZ14YD53NAX02QEPBPK
title: AI & Agentic Patterns — Reference Collection (source summary)
type: source_summary
status: active
tags:
- agents
- context-management
- feedback-loops
- verification
- tooling
- reading
- creativity
confidence: 0.86
created_at: 2026-01-31T17:35:06.721729Z
updated_at: 2026-01-31T17:35:06.721729Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAHWXK99F9EMPAY5CMPCB43
supersedes: []
---
## Summary
This document is a curated reference collection of patterns for making AI agents effective in software and creative workflows, emphasizing **tight feedback loops**, **context management**, and **verification via automation**.

A central theme is **“back pressure”**: agents perform better when the environment continuously surfaces correctness signals (build/type errors, tests, lint, UI assertions) so the agent can self-correct without consuming human review time. Banay’s *Don’t Waste Your Back Pressure* argues that you should stop spending human attention on trivial mistakes and instead give agents the ability to run builds/tests, interpret errors, and iterate until inconsistencies are eliminated; typed languages and high-quality error messages (e.g., Rust/Elm) act as natural back pressure. The collection also points to iterative “loop” styles (e.g., RALPH loops) and UI verification via browser/screenshot automation.

Practitioner workflow guidance (Peter Steinberger’s *Shipping at Inference-Speed*) claims that with modern coding models (GPT‑5.2/Codex), the bottleneck is inference time and “hard thinking,” not coding. Patterns include running multiple projects in parallel, heavily using queues, asking the model to revise rather than reverting, planning collaboratively then issuing a “build” command, and relying less on reading code line-by-line. Context practices include maintaining per-project docs folders and reusing patterns across repos by referencing other directories.

Factory.ai’s *Agent Readiness* frames inconsistent agent results as primarily a **codebase readiness** issue. It provides eight pillars (pre-commit hooks, environment/build docs, tests, CI/CD, security scanning, CODEOWNERS, branch protection) and maturity levels, recommending actionable metrics like the percentage of repos that are “agent-ready.”

The collection broadens beyond coding: Joel Simon’s *Lluminate* uses evolutionary search and explicit creativity strategies to counter LLM homogeneity and promote novelty, and fast.ai’s close-reading workflow describes using LLMs to read deeply by summarizing chapters, avoiding spoilers, and generating end-of-chapter context for continuation.

Finally, Cloudflare’s *Code Mode* proposes an architecture where the LLM writes TypeScript that calls MCP tools (rather than calling tools directly), executed in a V8 isolate sandbox with no internet and protected secrets.

## Cross-cutting takeaways (as stated)
- Context management is foundational (summaries-of-summaries, docs folders, JIT loading).
- Automated feedback beats human review for correctness.
- Run many experiments instead of seeking one perfect attempt.
- Start with CLI interfaces so agents can close the loop.
- As models improve, reduce scaffolding and rebuild SDLC assumptions around new constraints.
