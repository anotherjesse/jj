---
id: src_01KGAMWX22FT1G07J7NENHHPP9
title: changelog-service
ingested_at: 2026-01-31T18:26:29.058664+00:00
original_path: /Users/jesse/cto/ideas/changelog-service.md
tags: []
processing_status: complete
content_hash: sha256:061bc4f36302f2263dffc40cc9f33b0f46f64fb4aaf0dce007a6e1e8e8322ec8
---
# Changelog Service

## Idea

A tool/service that points at a git repo and produces rollup summaries at different granularities.

## How It Works

1. **Input:** Git repo URL or local path + optional custom prompts per level
2. **Process:**
   - Fetch commits (all or since last run)
   - AI summarizes each commit (diff + message → summary)
   - Roll up summaries by time period or tag
3. **Output:** Markdown changelogs at each level

## Rollup Hierarchy

| Level | Audience | Tone | Example |
|-------|----------|------|---------|
| Commit | Developers | Technical | "Fixed race condition in connection pool by adding mutex" |
| Day | Team | Internal | "Shipped SSH multi-connection support, fixed 3 bugs" |
| Week | Stakeholders | Progress update | "Major milestone: Postgres patterns complete" |
| Release | Users | User-facing | "New: You can now connect multiple SSH sessions" |
| Month | Strategic | Big picture | "January: Infrastructure hardening, preparing for beta" |

## Customization

Each level gets its own prompt template:
```
commit_prompt: "Summarize this code change technically. Focus on what and why."
release_prompt: "Write user-facing release notes. Focus on benefits, not implementation."
```

Could also support:
- Include/exclude paths (ignore docs changes in technical summary)
- Tag-based releases (v1.0.0 → v1.1.0)
- Conventional commit parsing

## Use Cases

1. **Internal awareness** - "What happened in Sparks this week?"
2. **External changelogs** - Auto-generate release notes
3. **AI context** - Feed recent changes into conversation corpus
4. **Async standups** - Daily/weekly summaries instead of meetings

## Similar To

- `yt` tool - small, focused, does one thing well
- Reader sync - another feed into the AI context corpus

## Implementation Notes

- Could be Rust CLI like `yt`
- Or a Spark that watches repos and generates on schedule
- Start simple: CLI that takes repo path, outputs markdown

## Open Questions

- How to handle large diffs? Truncate or summarize in chunks?
- Store summaries in the repo (CHANGELOG.md) or separate?
- How to detect "releases" if not using tags?
- Multi-repo rollups for org-wide summaries?

---
*Added: 2025-01-24*
