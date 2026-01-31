---
id: mem_01KGAMXM7WE277BPFHY37VF61M
title: Changelog Service
type: project
status: active
tags:
- git
- changelog
- summarization
- cli
- release-notes
confidence: 0.84
created_at: 2026-01-31T18:26:52.796514Z
updated_at: 2026-01-31T18:26:52.796514Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAMWX22FT1G07J7NENHHPP9
supersedes: []
summary: Proposed service that scans a git repo to AI-summarize diffs+messages into Markdown changelogs by commit/day/week/release/month.
---
## Summary
A proposed tool/service that points at a git repository and produces **AI-generated changelog summaries** rolled up at different granularities (commit/day/week/release/month) and emitted as Markdown.

## How it works
- **Input:** git repo URL or local path; optional custom prompt templates per summary level.
- **Process:**
  - Fetch commits (all history or since last run).
  - Summarize each commit from diff + message (focus on what changed and why).
  - Roll up summaries by time period and/or tag/release boundaries.
- **Output:** Markdown changelog files per level.

## Rollup levels / audiences
- **Commit (developers):** technical detail.
- **Day (team):** internal recap.
- **Week (stakeholders):** milestone/progress framing.
- **Release (users):** benefit-oriented release notes.
- **Month (strategic):** themes and big-picture narrative.

## Customization ideas
- Per-level prompt templates (e.g., technical commit prompt vs user-facing release prompt).
- Include/exclude paths (e.g., ignore docs changes in technical output).
- Tag-based releases (e.g., `v1.0.0 → v1.1.0`).
- Conventional commit parsing.

## Implementation notes
Start as a simple **CLI** that takes a repo path and outputs Markdown; could later become a scheduled watcher service ("Spark") that monitors repos.

## Open questions
- Handling large diffs (truncate vs chunked summarization).
- Where summaries live (in-repo `CHANGELOG.md` vs external store).
- Detecting releases without tags.
- Multi-repo/org-wide rollups.