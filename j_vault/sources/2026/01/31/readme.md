---
id: src_01KGAKX37PMJ4YRJNTWCT6RX3V
title: README
ingested_at: 2026-01-31T18:09:06.806930+00:00
original_path: /Users/jesse/cto/README.md
tags: []
processing_status: complete
content_hash: sha256:6cedfa8d7c88294947750c3c181d764f0c5762e428c84e8c461d7046437f6983
---
# CTO System - LoopWork

This is my (Claude Code's) knowledge base about Jesse Andrews and LoopWork. I maintain these files to be an effective CTO assistant.

## How This Works

When Jesse runs Claude Code, I read these files to understand:
- What LoopWork is building
- Current priorities and context
- Technical decisions and history
- What happened recently

**Jesse**: Just tell me what's going on or what you need. I'll read the context and ask questions if needed.

## Folder Structure

```
cto/
├── README.md           # This file
├── priorities.md       # Current priorities (I update this)
├── context/
│   ├── company.md      # LoopWork, team, stage
│   ├── sparks.md       # Sparks project details
│   ├── tech-stack.md   # Technical decisions
│   └── roadmap.md      # Milestones and plan
├── prompts/            # Session starters (optional)
├── decisions/          # Architecture Decision Records
└── logs/               # Session logs (when useful)
```

## Context Files

| File | Purpose |
|------|---------|
| `context/company.md` | Team, runway, strategic direction |
| `context/sparks.md` | The Sparks project - agentic-first compute |
| `context/tech-stack.md` | Technical choices |
| `context/roadmap.md` | Where we're headed |
| `priorities.md` | What matters right now |

## Updating Context

After sessions where important things happen, I should update:
- `priorities.md` if priorities shift
- Relevant context files with new information
- `decisions/` for significant technical decisions

## Prompts (Optional)

The `prompts/` folder has session starters for common scenarios:
- `daily.md` - Morning check-in
- `weekly.md` - Weekly review
- `architecture.md` - Technical decisions
- `code-review.md` - Code review
- `debug.md` - Debugging help
- `strategy.md` - Big picture thinking

---
*System created: 2025-01-23*
