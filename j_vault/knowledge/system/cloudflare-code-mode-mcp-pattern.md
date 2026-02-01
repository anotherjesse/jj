---
id: mem_01KGAKV0FZXXJYE6ETGD6YCFQY
title: Cloudflare Code Mode MCP pattern
type: system
status: active
tags:
- mcp
- tooling-architecture
- sandboxing
- typescript
confidence: 0.74
created_at: 2026-01-31T18:07:58.463715Z
updated_at: 2026-01-31T18:07:58.463715Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKS8MG1Q0BZ28ZC7MSHR2F
supersedes: []
summary: 'Cloudflare Code Mode: LLM writes TypeScript that calls MCP tool APIs via one TS-execution tool, enabling chained steps and leveraging TS priors.'
---
## Summary
Cloudflare’s “Code Mode” proposes using LLMs to **write code that calls MCP tools** rather than having the LLM call MCP tools directly. MCP schemas are converted into a TypeScript API; the agent receives a single tool to execute TypeScript.

## Rationale
- LLMs have extensive real-world TypeScript priors but limited training on contrived tool-call formats.
- For multi-step tasks, code can chain tool invocations without repeatedly feeding intermediate outputs back through the model.

## Implementation notes (as described)
- MCP schema → TypeScript API with doc comments.
- Code executes in a V8 isolate sandbox (millisecond startup; lighter than containers).
- No internet access inside the isolate; only bindings to MCP servers.
- API keys are not exposed to generated code.
- Uses a Worker Loader API for on-demand isolate creation.

## Source
- https://blog.cloudflare.com/code-mode/