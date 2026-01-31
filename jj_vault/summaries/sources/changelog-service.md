---
id: mem_01KGAMXM7V6SXDXQ8E7C3S0417
title: changelog-service (source summary)
type: source_summary
status: active
tags: []
confidence: 0.86
created_at: 2026-01-31T18:26:52.795260Z
updated_at: 2026-01-31T18:26:52.795260Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAMWX22FT1G07J7NENHHPP9
supersedes: []
---
## Summary
The document proposes a **Changelog Service**: a tool/service that targets a git repository (URL or local path) and generates **Markdown changelog rollups** at multiple granularities. The core pipeline is: ingest a repo + optional prompt templates per summary level; fetch commits (either all history or since the last run); have an AI summarize each commit using the commit message and diff (what changed and why); then roll those summaries up by time period or tags; finally emit Markdown artifacts per level.

A key design element is a **rollup hierarchy** tuned to different audiences and tones:
- **Commit**: technical developer-focused summaries (e.g., race condition fix details).
- **Day**: internal team recap (e.g., features shipped + bug counts).
- **Week**: stakeholder progress update framing milestones.
- **Release**: user-facing notes emphasizing benefits, not implementation.
- **Month**: strategic narrative (themes, readiness, direction).

The service should support **custom prompt templates per level** (e.g., `commit_prompt` vs `release_prompt`) to control tone and focus. Potential enhancements include path include/exclude filters (e.g., ignore docs changes for technical summaries), tag-based release boundaries (e.g., `v1.0.0 → v1.1.0`), and conventional commit parsing.

Primary use cases: improving internal awareness (“what happened this week?”), generating external release notes automatically, feeding recent changes into an AI context corpus, and replacing meetings with async standup summaries. It is positioned as similar in spirit to other small focused tools (like `yt`) and as another feed into a broader “reader sync” / context-ingestion pipeline.

Implementation notes suggest starting simple as a **CLI** that outputs Markdown from a repo path, with possible evolution into a scheduled watcher service (“Spark”) that monitors repos. Open questions include strategies for large diffs (truncation vs chunked summarization), where to store summaries (in-repo vs separate store), release detection without tags, and multi-repo/org-wide rollups.
