---
id: mem_01KGAKXHCEAF85SRTJHP4S24QP
title: CTO System - LoopWork (README)
type: source_summary
status: active
tags:
- loopwork
- cto-system
- knowledge-base
- process
confidence: 0.86
created_at: 2026-01-31T18:09:21.294864Z
updated_at: 2026-01-31T18:09:21.294864Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKX37PMJ4YRJNTWCT6RX3V
supersedes: []
---
## Summary
This README describes a local "CTO System" knowledge base maintained by Claude Code to act as an effective CTO assistant for **Jesse Andrews** and **LoopWork**. The system is intended to be read at the start of Claude Code sessions to provide shared context about what LoopWork is building, current priorities, technical decisions/history, and recent activity.

The document sets an expectation for how Jesse interacts with the assistant: Jesse can simply ask what’s going on or what’s needed, and the assistant should consult the context files and ask follow-up questions when necessary.

### Repository layout
The knowledge base is organized under a `cto/` directory with:
- `README.md` (this file)
- `priorities.md` for current priorities (maintained/updated by the assistant)
- `context/` for durable background:
  - `company.md` (team, runway, strategic direction)
  - `sparks.md` (Sparks project details: agentic-first compute)
  - `tech-stack.md` (technical decisions)
  - `roadmap.md` (milestones and plan)
- `prompts/` for optional session starters (daily, weekly, architecture, code review, debug, strategy)
- `decisions/` for architecture decision records
- `logs/` for session logs when useful

### Update discipline
After sessions with significant changes, the assistant should update `priorities.md`, relevant `context/` files, and create entries in `decisions/` for major technical decisions. The system notes its creation date as **2025-01-23**.
