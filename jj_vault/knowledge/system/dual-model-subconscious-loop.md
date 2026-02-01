---
id: mem_01KGBTWBCG1WHD87N8DZR9H4T9
title: Dual-model 'subconscious' loop (fast chat + slow background analysis)
type: system
status: active
tags:
- interaction-pattern
- multi-model
- background-jobs
- jj-gateway
- ux
- canvas
- tldraw
- image-gen
- voice
confidence: 0.78
created_at: 2026-02-01T05:30:16.848016Z
updated_at: 2026-02-01T05:49:04.347825Z
sources:
- thread_id: ''
  event_ids: []
- thread_id: thread://current
  event_ids: []
supersedes: []
summary: 'Interaction pattern: fast model handles live chat; slow model runs in background to analyze/research and returns distilled thoughts for the fast model to use.'
---
## Idea
Run a **fast model** for realtime back-and-forth, while a **slow/powerful model** runs asynchronously as a “subconscious.”

### Proposed behavior
- User speaks/types a stream-of-consciousness.
- System continually (or on triggers) spawns a slow job that:
  1) returns a **2-sentence distilled take** ("initial thoughts")
  2) returns a longer **private scratchpad / inner-monologue-style reasoning** meant to enrich context.
- Fast model continues responding in realtime, but can incorporate outputs from the slow job as they arrive.

### Notes / risks
- Need a clear boundary for what gets surfaced to user vs kept internal.
- Background jobs should be interruptible/cancellable and attached to a session timeline.
- Useful for research, synthesis, and deeper thinking without blocking the main conversation.

## 2026-01-31 update: interaction/UX details
- Goal is **voice-first** with a **fast “mouth” model**; user generally **does not want to see** the slow model’s background thoughts.
- Slow model (“deep thoughts/subconscious”) can be **auto-triggered periodically** if enough conversation occurs without explicit call.
- Output needn’t be a complex schema initially; can be **raw internal thoughts** intended for the mouth model to incorporate, not spoken aloud.
- **Queue injection**: deep-thought results should be applied on the **next** fast-model turn (not mid-stream).
- **Staleness** noted: multiple overlapping deep-thought jobs could return out of order; user suggests starting simple, later add an **integration phase** to decide what to keep.
- User open to slow process being **agentic**: can see **full conversation context** and may **call tools** (search/knowledge/etc.).
- Possible architecture: a **router** chooses which tools/skills to load for deep-thought runs.
- Long-term: structured distillations might be written into the **knowledge base**, while raw deep-thought text is stored in-thread but primarily for the assistant.


## Addendum: visual surface / shared canvas (2026-01-31)
Jesse suggested pairing the fast/slow dual-model interaction with a **visual surface**: a screen the assistant can **draw on** (e.g., tldraw-style canvas) and the ability to **generate images** so the assistant can *show*, not just *tell*. This would let voice-first interaction still produce rich visual output without requiring constant reading.


## 2026-01-31 follow-up: scope trim to text-first
- Jesse prefers to treat the drawing/canvas idea as **orthogonal** and start **text-only**.
- Plan: get the fast/slow loop working first, then **add tools incrementally** (e.g., knowledge search, repo/code tools, web research) rather than expanding UX surface area early.
