Below is a **build spec + phased plan** you can hand to a coding agent. It’s designed for a **“conversation + tools”** agent (ChatGPT/Claude-style function calling) implemented in **Rust**, with **memory as a first-class, versioned artifact** that can evolve without an agent-SDK.

---

# J: Memory-First Agent System Spec (Rust + LLMs + Tools)

## 0) Product goals

### Primary goals (MVP → durable foundation)

1. **Durable, auditable memory** that survives across threads and can be corrected without corruption.
2. **Extensible tool system** (your existing tools + new ones) with governance, permissions, and logging.
3. **Event-driven + scheduled reflection** (daily/weekly review jobs) to consolidate and detect drift.
4. **Evolvable behavior**: the “rules of memory” live in editable docs/config, not hard-coded.

### Non-goals (initially)

* Full “agent swarm” orchestration, multi-agent marketplace, or complex UI graph.
* “Autonomous code-writing” inside the runtime (we can *support* it later via coding-agent handoffs).

---

# 1) Core architectural principles (“invariants”)

These should be explicitly written into an `agents.md` / `invariants.md` file and enforced in code.

## Invariants

1. **Raw threads are append-only.**
   No edits to raw conversation logs; only appends (and optional encryption at rest).

2. **Durable memory changes are reversible and attributable.**
   Every durable write must have:

   * a reason,
   * a source (thread/message refs),
   * an author (“j”, “librarian-job”, “user”, etc.),
   * a diff/patch trail.

3. **No silent overwrites of beliefs.**
   When something changes, we don’t “erase”; we *supersede* or *mark as contradicted* with pointers.

4. **Model suggests; system governs.**
   LLM can propose memory operations, but a policy gate decides: auto-apply vs queue for review.

5. **Retrieval uses tiers by default**
   Prefer: durable knowledge + summaries → only hit raw transcripts when needed.

6. **Tools are explicit and discoverable**
   Tool definitions are loaded from manifests; every tool call is logged with arguments and a required `reason`.

---

# 2) System components (Rust services/modules)

Think of J as a **single orchestrator loop** with a few backing subsystems.

## 2.1 Orchestrator (the runtime loop)

Responsible for:

* accepting a user message in a thread
* retrieving context (memory)
* calling LLM (function/tool calling)
* executing tools
* writing outputs back into the thread log
* running post-turn memory curation and governance

## 2.2 Storage layers (3 tiers)

1. **Thread Store (append-only)**

   * Stores every message, tool call, tool result, attachments metadata.
2. **Derived Store**

   * Thread summaries, daily digests, weekly rollups, “open questions”, etc.
3. **Knowledge Store (durable “wiki”)**

   * Curated memory items: preferences, projects, people, system rules, long-lived facts.

## 2.3 Index (vector + metadata)

* Embeddings for:

  * knowledge docs (chunked)
  * summaries (chunked)
  * optionally: raw transcripts (usually off by default due to size/cost)
* Metadata filters: tags, type, recency, confidence, status.

## 2.4 Governance layer

* Receives “proposed memory operations” (from LLM)
* Applies policy:

  * auto-apply low-risk
  * queue high-risk or contradictory
* Writes an audit entry + versions (Git commits or ledger)

## 2.5 Job runner (cron + event triggers)

* Daily 5am review
* Weekly consolidation
* Event triggers (e.g., “thread closed”, “knowledge updated”, “new screenshot added”)

## 2.6 Tool registry

* Loads tool manifests from disk
* Exposes tool schemas to LLM
* Executes tool calls with permission checks
* Logs every tool call/result

---

# 3) On-disk “vault” layout (Markdown-first, machine-parsable)

Everything lives in a single repo-like directory (call it `j_vault/`).

```
j_vault/
  agents.md                     # table-of-contents + operating rules
  invariants.md                 # the hard rules the system enforces
  config/
    j.runtime.yml              # which providers, defaults, toggles
    memory.policy.yml           # auto-apply thresholds, sensitive types, etc.
  prompts/
    j.system.md                # main assistant system prompt (templated)
    curator.system.md           # post-turn memory proposal prompt
    daily_review.system.md      # daily consolidation prompt
    weekly_review.system.md
  threads/
    2026/01/30/
      thr_01J...jsonl           # append-only log (jsonl)
  summaries/
    threads/
      thr_01J...md              # thread summary (front matter + content)
    daily/
      2026-01-30.md
    weekly/
      2026-W05.md
  knowledge/
    people/
      jesse.md
    projects/
      j-agent.md
    prefs/
      interaction.md
    system/
      tool_catalog.md
      memory_rules.md
  inbox/
    proposals/
      prop_01J...json           # memory ops awaiting review
    questions/
      2026-01-30.md             # queued clarifying questions
  artifacts/
    2026/01/30/
      img_01J...png
      img_01J...md              # metadata + source references
  audit/
    ledger.jsonl                # append-only change ledger (plus Git)
```

### Why this layout works

* **Human-readable** (you can open in any editor)
* **Machine-governable** (front matter + schemas)
* **Evolvable** (new directories/types can be added without migrations)

---

# 4) Data formats / schemas

## 4.1 Thread log format (JSONL, append-only)

One JSON object per line; no edits, only append.

Event types:

* `user_message`
* `assistant_message`
* `tool_call`
* `tool_result`
* `system_note` (internal runtime notes)
* `attachment_added`

Each event includes:

* `thread_id`, `event_id`, `ts`
* `role` (user/assistant/tool/system)
* `content` (string or structured)
* `tool_name`, `tool_args`, `tool_result` where relevant
* `reason` required for tool calls (and memory writes)

## 4.2 Knowledge doc format (Markdown + YAML front matter)

Example: `knowledge/prefs/interaction.md`

```yaml
---
id: mem_01JABC...
title: "Interaction preferences"
type: preference
status: active            # active|superseded|contradicted|deprecated|draft
tags: [voice, ux, iteration]
confidence: 0.75          # 0..1
created_at: 2026-01-30T10:12:00Z
updated_at: 2026-01-30T10:40:00Z
sources:
  - thread_id: thr_01J...
    event_ids: [evt_..., evt_...]
    excerpt_hash: sha256:...
supersedes: []
---
## What to do
- Prefer minimal upfront design; let rules evolve.
- Keep raw logs; keep durable wiki curated.

## Notes
Anything that changes should be superseded, not erased.
```

### ID vs human-friendly naming

* Filename/slug is human (e.g., `interaction.md`)
* `id` is stable machine ID (ULID/UUID)
* Links use `id` internally; UI shows titles/slugs.

## 4.3 Memory proposal format (JSON)

LLM produces *proposals*; governance applies them.

Example `inbox/proposals/prop_01J...json`:

```json
{
  "proposal_id": "prop_01J...",
  "ts": "2026-01-30T10:45:00Z",
  "author": "curator",
  "reason": "User stated stable preference about evolving rules and minimizing upfront scaffolding.",
  "ops": [
    {
      "op": "upsert_knowledge",
      "doc_path": "knowledge/prefs/interaction.md",
      "doc_id": "mem_01JABC...",
      "patch": {
        "tags_add": ["iteration"],
        "body_append": "- Emphasize governance and reversible changes."
      },
      "sources": [
        {"thread_id":"thr_01J...", "event_ids":["evt_..."], "excerpt_hash":"sha256:..."}
      ],
      "risk": "low"
    },
    {
      "op": "queue_question",
      "question": "Do you want J to auto-apply preference updates, or require review for anything labeled 'preference'?",
      "risk": "medium"
    }
  ]
}
```

---

# 5) Memory lifecycle (the key workflows)

## 5.1 Pre-response retrieval (per user turn)

**Input**: user message + current thread context
**Output**: “context packet” for LLM

Algorithm (tiered):

1. Retrieve top-K relevant **knowledge docs**
2. Retrieve top-K relevant **thread summaries**
3. Optionally retrieve specific raw transcript snippets **only if needed**
4. Include any **open questions** and **pending proposals** (if relevant)

Context packet structure:

* `User profile` (minimal, from knowledge)
* `Relevant durable memory`
* `Relevant recent summaries`
* `Current thread summary` (if exists)
* `Open questions (queue)`
* `Constraints/invariants (short)`

## 5.2 Post-response curation (per turn)

After J answers (and tools run), run a second LLM pass (“Curator”) that:

* extracts candidate memories
* proposes updates
* proposes clarifying questions
* identifies contradictions/drift

This produces the JSON proposal file(s).

## 5.3 Governance + apply

Policy rules decide:

* **Auto-apply** (low-risk, strong evidence, non-sensitive)
* **Queue for review** (preference changes, identity/health/finance, contradictions, low confidence)
* **Reject** (hallucinated memory, no source, too vague)

Applying an op means:

* patch docs
* write audit ledger entry
* create a Git commit (or equivalent)
* re-embed changed doc chunks

## 5.4 Consolidation jobs (daily/weekly)

Daily (5am):

* summarize yesterday’s threads
* merge repeated facts into durable knowledge
* detect contradictions
* generate “questions to ask user” queue

Weekly:

* dedupe tags/docs
* lower confidence on stale memories
* propose archiving/renaming docs
* run regressions (see eval section)

---

# 6) Tooling and extensibility (without an agent SDK)

## 6.1 Tool manifest format (on disk)

Each tool is described in a manifest that the runtime loads and exposes to the LLM.

`tools/twitter_search.tool.json` (example structure)

* tool name
* description
* JSON schema input
* output schema (best effort)
* side effects: `read_only | writes_external | writes_local | irreversible`
* permission class: `safe | confirm | admin`
* required fields: `reason`

This allows:

* dynamic enable/disable tools per environment
* tool discoverability (“tool catalog”)
* consistent logging + governance

## 6.2 Tool execution contract

Every tool call must include:

* `reason` (string)
* `expected_output` (optional)
* `sensitivity` (optional tag)

Every tool result returns:

* `status` (ok/error)
* `data` (structured)
* `redactions` (optional)
* `trace_id`

## 6.3 Tool catalog doc

Generate/maintain `knowledge/system/tool_catalog.md` automatically from manifests so the model can “read” capabilities.

---

# 7) Drift, conflict, and “truth maintenance”

This is the governance people underestimate, and you’re right to make it core.

## 7.1 Contradiction detection

When an update touches an existing doc:

* compare semantic similarity and diff magnitude
* if large change or opposite polarity:

  * set status of old doc: `superseded` or `contradicted`
  * create link: `supersedes: [old_id]`
  * queue a question if user confirmation is needed

## 7.2 Confidence + recency model

Each memory item has:

* confidence (0..1)
* last_confirmed_at (optional)
* decay policy (weekly job reduces confidence on stale volatile items)

## 7.3 Never delete by default

Deletion is an explicit operation:

* mark doc `deprecated`
* keep content, keep index (or remove from retrieval but keep in history)

---

# 8) “Evolution hooks” (so the system can improve itself safely)

## 8.1 Editable rules-of-the-road

The behavior is governed by files:

* `prompts/*.md`
* `config/memory.policy.yml`
* `invariants.md`

The daily job can propose changes to these—but they should **always go to review**, never auto-apply.

## 8.2 Evals/regression harness

Store a set of “golden threads” and expected outcomes:

* retrieval correctness (did it fetch the right memory?)
* memory correctness (did it store the right thing?)
* governance correctness (did it avoid unsafe auto-apply?)

This prevents “model got smarter, rules got worse” drift.

---

# 9) Provider integration (LLM abstraction)

You want to be able to swap providers.

Implement a Rust trait like:

* `generate(messages, tools, tool_choice, ...) -> assistant_message + tool_calls`
* `embed(texts) -> vectors`

Providers:

* OpenAI
* Anthropic

Design for:

* streaming tokens (needed for voice later)
* tool calling
* structured output mode (for proposals)

---

# 10) Phases / milestones

Each milestone includes deliverables + definition of done (DoD). This is what you hand to a coding agent.

---

## Milestone 1 — Vault + Thread Store (append-only truth)

**Goal:** Durable logging of everything, with a clean on-disk layout.

### Deliverables

* Create `j_vault/` layout scaffolding
* Thread CRUD:

  * create thread
  * append events (JSONL)
  * read thread (range/pagination)
* Event schema + validation
* Attachments metadata support (no fancy processing yet)
* `agents.md` + `invariants.md` created (initial content)

### DoD

* You can run a CLI or small HTTP API:

  * `POST /threads/{id}/events` appends
  * `GET /threads/{id}` reads
* Append-only enforced (attempted edit fails)
* All tool calls and assistant messages are logged as events

---

## Milestone 2 — Knowledge Store (Markdown memory items) + Versioning

**Goal:** A durable “wiki” memory with reversible changes.

### Deliverables

* Knowledge doc loader:

  * parse YAML front matter
  * validate required fields
  * read/write markdown docs
* Audit ledger:

  * append-only `audit/ledger.jsonl` entry per change
* Versioning mechanism:

  * recommended: Git commits per applied proposal
  * include commit message: proposal_id + reason

### DoD

* `upsert_knowledge(doc_path, patch)` works
* Every change results in:

  * updated doc
  * ledger entry
  * Git commit (or equivalent)
* Knowledge docs remain human-editable without breaking parser (strict but friendly)

---

## Milestone 3 — Retrieval: embeddings + metadata search (tiered)

**Goal:** J can fetch relevant memory before responding.

### Deliverables

* Postgres + vector index (pgvector)

  * table for chunks: `doc_id`, `chunk_id`, `text`, `embedding`, `metadata`
* Chunking strategy:

  * headings/sections as chunks
  * chunk size limits + overlap
* Search API:

  * `search_knowledge(query, filters) -> ranked chunks`
  * `search_summaries(query, filters) -> ranked chunks`
* “Context packet” builder with tiering

### DoD

* Given a query, retrieval returns:

  * top relevant knowledge chunks
  * plus relevant summaries
* Deterministic fallback when embeddings unavailable (keyword search)

---

## Milestone 4 — Memory curation + governance pipeline (the heart)

**Goal:** System can propose memories, apply safely, and queue questions.

### Deliverables

* Curator prompt + runner:

  * consumes: recent thread excerpt + J response
  * outputs: proposal JSON (strict schema)
* Governance policy engine:

  * risk scoring by op/type/confidence/sensitivity
  * auto-apply thresholds configurable in `config/memory.policy.yml`
* Proposal queue:

  * writes to `inbox/proposals/`
  * apply command moves proposals to “applied” state (or marks rejected)
* Open questions queue:

  * `inbox/questions/YYYY-MM-DD.md` append

### DoD

* After any conversation turn:

  * at least one proposal is generated (or an explicit “none”)
  * low-risk proposals auto-apply
  * higher-risk proposals are queued
* Every durable memory doc has sources pointing to thread events

---

## Milestone 5 — Scheduled jobs + consolidation (daily/weekly reflection)

**Goal:** J gains the “memory muscle” that improves over time.

### Deliverables

* Job registry on disk (Markdown or YAML)

  * schedule + prompt + inputs + tool permissions
* Job runner:

  * cron schedule (5am daily, weekly)
  * event-triggered jobs (thread closed)
* Daily job outputs:

  * `summaries/daily/YYYY-MM-DD.md`
  * updates/merges knowledge docs via proposals
  * populates open questions queue

### DoD

* Running daily job produces a useful digest and at least one memory hygiene improvement
* Weekly job reduces duplicates / flags contradictions
* Jobs are observable (logs, success/failure, traces)

---

## Milestone 6 — Tool registry v1 (make tools composable)

**Goal:** Integrate your existing tools cleanly without entangling core logic.

### Deliverables

* Tool manifest loader + validator
* Tool execution router with:

  * permission gating
  * required reason
  * structured result wrapping
* Auto-generate `knowledge/system/tool_catalog.md`

### DoD

* Add a new tool by dropping a manifest + Rust implementation and restarting
* LLM sees updated tool catalog and can call tool by schema
* Tool calls always appear in thread logs + audit

---

## Milestone 7 — Voice and multimodal artifacts (soon after memory)

**Goal:** Voice becomes “just another IO layer” feeding the same thread/memory system.

### Deliverables

* Audio ingest → STT → user_message event
* Streaming assistant output → TTS
* Store:

  * audio files as artifacts
  * transcripts in thread log
* Optional: screenshot ingestion pipeline (artifact + metadata + embedding)

### DoD

* Voice conversation creates a normal thread with the same memory behaviors
* Daily review includes voice threads and updates memory the same way

---

# 11) Implementation notes (what the coding agent should decide up front)

## Decisions to lock early

1. **Thread event schema** (JSONL fields, IDs, timestamps)
2. **Knowledge doc schema** (front matter required fields)
3. **Proposal schema** (strict JSON)
4. **Governance policy defaults** (auto-apply thresholds, sensitive types)
5. **Chunking strategy** (affects retrieval quality)

## Decisions to keep flexible

* taxonomy/folder structure beyond the basics
* exact memory heuristics (LLM-driven, configurable prompts)
* which tools exist (manifest-driven)
* which provider used (adapter)

---

# 12) Suggested initial memory policy (practical defaults)

Auto-apply (low risk):

* thread summaries
* daily digests
* tool catalog refresh
* non-sensitive project notes with strong source references

Queue for review (medium/high risk):

* “preferences” that affect behavior long-term
* identity/biographical facts
* anything health/finance/legal-related
* contradictions / “you changed your mind” updates
* changes to prompts/config/invariants

Always reject:

* proposals with no sources
* vague claims (“user likes productivity”) without evidence
* memory that looks like model inference masquerading as fact

---

# 13) What you hand the coding agent as the “build brief”

Copy/paste this checklist:

1. Implement `j_vault` layout + append-only thread JSONL store
2. Implement knowledge markdown store with YAML front matter
3. Add audit ledger + Git commit per applied durable write
4. Implement pgvector index for knowledge + summaries
5. Implement retrieval tiering and context packet builder
6. Implement curator LLM pass → proposal JSON
7. Implement governance engine: auto-apply vs queue
8. Implement daily/weekly job runner from on-disk job specs
9. Implement tool registry via manifests + strict logging

---

If you want, next message I can produce:

* the **exact JSON Schemas** (proposal ops, thread events, tool manifests)
* the **first draft prompts** for `j.system.md`, `curator.system.md`, and `daily_review.system.md` (templated, minimal, designed to evolve)
  …so your coding agent can start implementing against strict contracts immediately.
