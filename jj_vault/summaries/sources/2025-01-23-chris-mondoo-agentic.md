---
id: mem_01KGAMRK35ZQD6F4KE08FNSKN1
title: Chris (Mondoo) — Agentic development discussion (2025-01-23)
type: source_summary
status: active
tags:
- agentic-coding
- verification
- multi-agent
- sandboxing
- claude
- codex
- gemini
- sparks
- skills
confidence: 0.8
created_at: 2026-01-31T18:24:07.781679Z
updated_at: 2026-01-31T18:24:07.781679Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAMQP2QCND16S0Q271PWVCJ
supersedes: []
---
## Summary
Meeting between Jesse Andrews and Chris (Mondoo CTO/VPE) on agentic development workflows and tooling.

Mondoo has rapidly adopted Claude Code since Claude 4.5 (Dec 2024), including a full website migration from Webflow to Next.js in ~2 weeks, Slack-driven marketing updates via Claude, elimination of a CMS in favor of a git repo + Claude abstractions, and rapid remediation of ~5,000 SEO errors. Customer success and UI teams are onboarding to Claude Code to reduce dependency on engineering; backend/data migrations are approached more cautiously due to risk. MQL policy development is nearing fully autonomous generation.

Jesse describes a multi-agent approach using Claude Code and Codex CLI: Codex is preferred for longer-running architectural/back-end tasks (can run 20–30 minutes unattended; sometimes hours via compaction), while Claude is stronger for UI, DevOps, and shell scripting. Model selection is task-dependent (e.g., “High” for implementation vs “Extra High” for spec analysis/clarifying questions); Gemini is used for large-context analysis and PR documentation. Core cost thesis: human review/understanding time is now the scarce resource, not model inference.

Key operational pattern: treat agent runs as probabilistic experiments rather than “golden PR” attempts. Run multiple implementations in parallel; if a run wedges, discard the sandbox and restart.

Reliability hinges on sandboxing + verification loops. Agents run in containers/VMs with limited blast radius; “YOLO” modes require explicit flags (e.g., `dangerously_allow_all` plus `IS_SANDBOX=1`). The “RALPH loop” concept is a simple while-loop that repeatedly instantiates a fresh agent session to pick the next incomplete task, make progress, commit when appropriate, then run lint/unit/integration tests and feed failures back. A longer (~25 minute) end-to-end verification loop deploys to full GCP/K8s environments.

For code review/understanding, Jesse uses Gemini to generate systematic PR narratives (“write for people who built this: what changed and why”), and requests justification when verification steps change. Tools mentioned include StageHand (BrowserBase) for LLM-driven visual/UI testing and “augmentation” tools for component annotation/feedback.

The discussion also covers philosophical shifts: faster iteration moves engineering value toward problem definition and system evolution, but creates enterprise risks (proliferation of many small, poorly understood systems) and raises concerns about junior engineering skill pipelines. A prediction is made that RL-fine-tuning advances will enable an explosion of LLM-friendly programming languages.
