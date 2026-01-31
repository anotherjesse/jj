---
id: mem_01KGAKV0G0XSZ07TYRZ2WAK9XW
title: LLM-assisted close reading (fast.ai workflow)
type: project
status: active
tags:
- context-management
- reading
- workflows
- anki
confidence: 0.73
created_at: 2026-01-31T18:07:58.464322Z
updated_at: 2026-01-31T18:07:58.464322Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKS8MG1Q0BZ28ZC7MSHR2F
supersedes: []
---
## Summary
A fast.ai workflow (Jan 2026) for using LLMs to support “close reading”: iteratively reading text while pausing to ask questions, make connections, and explore rabbit holes, while maintaining compact context across chapters.

## Process (as described)
1. Convert PDFs to Markdown.
2. Generate chapter summaries to use as context.
3. Instruct the LLM not to give spoilers.
4. Read through the full text while asking questions.
5. Generate a conversation overview at chapter end to seed the next chapter.
6. Optionally have the LLM ask comprehension questions.
7. Optionally generate Anki cards via fastanki.

## Tools mentioned
- SolveIt platform: https://solve.it.com/
- fastanki (Anki card generation)

## Source
- https://www.fast.ai/posts/2026-01-21-reading-LLMs/
