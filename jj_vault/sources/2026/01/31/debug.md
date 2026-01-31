---
id: src_01KGAJ08ZKE85JDWBVDQPKZFV7
title: debug
ingested_at: 2026-01-31T17:35:53.843639+00:00
original_path: /Users/jesse/cto/prompts/debug.md
tags: []
processing_status: complete
content_hash: sha256:5cc88c55250680729d7d977d4d5a9720465eb0ac2930ce346b696b3525993aa7
---
# Debugging Session

Run this when you're stuck on a bug.

---

## Prompt

```
Read the relevant code in this repo.

I'm debugging an issue:

**What should happen**: [Expected behavior]

**What actually happens**: [Actual behavior]

**What I've tried**:
1.
2.

**Error messages/logs**:
```
[paste here]
```

Help me:
1. Form hypotheses about what's wrong
2. Identify what to investigate
3. Suggest fixes

Think step by step. Ask clarifying questions if needed.
```

---

## Rubber Duck Prompt

For when you just need to talk through it:

```
I'm stuck. Let me explain what I'm trying to do and help me think through it:

[Explain the problem]

Ask me questions to help me find the answer myself.
```
