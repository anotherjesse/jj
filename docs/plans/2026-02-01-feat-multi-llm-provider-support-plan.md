---
title: "feat: Multi-LLM Engine Support (OpenAI, Anthropic, Gemini protocols)"
type: feat
date: 2026-02-01
---

# Multi-LLM Engine Support

## Overview

Introduce an **engine** abstraction for chat completion that separates *wire protocol* (OpenAI API, Anthropic Messages API, Gemini API) from *model identity* (gpt-5-mini, claude-sonnet, gemini-flash, kimi, minimax, glm). A single engine can power many models from different vendors — e.g., Kimi, MiniMax, and GLM all speak the Anthropic Messages protocol with different base URLs and keys.

The embeddings layer (`src/embeddings.rs`) already demonstrates multi-provider dispatch — this plan extends that pattern to chat completions with the added concept of engines vs models.

## Problem Statement / Motivation

The system is currently hardcoded to OpenAI's chat completions API. `OpenAIClient` is used directly in 6+ locations with no trait abstraction. This creates vendor lock-in and prevents using models that speak different API protocols (Anthropic Messages, Gemini generateContent) or even models from other vendors that happen to speak the same protocol with slight tweaks.

## Current State Analysis

### What works well
- **Embeddings** (`src/embeddings.rs`): Clean `EmbeddingProvider` enum dispatch for OpenAI + Gemini — good pattern to follow
- **Tool calling**: 10+ tools with JSON schemas in `agent.rs:207-413`, execution in `agent.rs:415-786`
- **Config**: Model override via `--model` CLI flag, env vars, and `client.set_model()`

### Integration points that need abstraction

| Location | File | What it does |
|----------|------|-------------|
| Direct chat | `chat.rs:83` | Creates `OpenAIClient` for REPL |
| Agent loop | `agent.rs:52-180` | Takes `&OpenAIClient`, calls `client.chat()` |
| Deep think | `agent.rs:838-846` | Creates separate `OpenAIClient` for background reasoning |
| Gateway session | `gateway/session.rs:498-501, 561` | Creates clients for title gen + agent runs |
| Ingest | `ingest.rs:121-127` | Creates client for summarization |
| Backfill | `main.rs:461-466` | Creates client for backfill summaries |

### Key API differences across engines

| Feature | OpenAI engine | Anthropic engine | Gemini engine |
|---------|--------------|------------------|---------------|
| Endpoint | `POST /v1/chat/completions` | `POST /v1/messages` | `POST /v1beta/models/{m}:generateContent` |
| Auth | `Authorization: Bearer {key}` | `x-api-key: {key}` | `?key={key}` or OAuth |
| System prompt | `{"role":"system"}` message | Top-level `system` param | `systemInstruction` field |
| Tool schema | `tools[].function` | `tools[].input_schema` | `functionDeclarations` |
| Tool call response | `tool_calls[]` in message | `content[]` blocks with `type: "tool_use"` | `functionCall` in parts |
| Tool result | `{"role":"tool", "tool_call_id":...}` | `{"type":"tool_result", "tool_use_id":...}` | `functionResponse` in parts |
| Roles | system/user/assistant/tool | user/assistant | user/model |
| `max_tokens` | Optional | **Required** | Optional (`maxOutputTokens`) |
| Parallel tool calls | Yes | Yes | Yes |
| **Models using this engine** | GPT-5, GPT-5-mini, o-series | Claude, **Kimi, MiniMax, GLM** | Gemini Flash/Pro |

## Proposed Solution

### Core concept: Engine vs Model

An **engine** is a wire protocol implementation (how to format requests, parse responses, authenticate). A **model** is the specific model name sent in the request. Multiple vendors can share an engine with different `base_url` and `api_key` values.

```
Engine: anthropic
  ├── Model: claude-sonnet-4  (base_url: api.anthropic.com, key: ANTHROPIC_API_KEY)
  ├── Model: kimi-k2          (base_url: api.moonshot.cn,    key: KIMI_API_KEY)
  └── Model: glm-4-plus       (base_url: open.bigmodel.cn,   key: GLM_API_KEY)
```

This means `LLM_ENGINE` selects the protocol, while `LLM_MODEL`, `LLM_BASE_URL`, and `LLM_API_KEY` configure which specific service to hit. Provider-specific env vars (e.g., `ANTHROPIC_API_KEY`) serve as convenient defaults.

### Architecture

```
src/engine.rs        (new) — Engine trait + canonical types + factory
src/openai.rs        (modify) — implement Engine for OpenAI protocol
src/anthropic.rs     (new) — implement Engine for Anthropic Messages protocol
src/gemini_chat.rs   (new) — implement Engine for Gemini protocol
src/agent.rs         (modify) — use &dyn Engine instead of &OpenAIClient
src/chat.rs          (modify) — use factory to create engine
src/gateway/session.rs (modify) — use factory
```

### Engine trait and canonical types (`src/engine.rs`)

```rust
pub enum EngineKind { OpenAI, Anthropic, Gemini }

pub struct ChatMessage {
    pub role: Role,       // User | Assistant | System | Tool
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub tool_call_id: Option<String>,
}

pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

pub struct ChatResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
}

pub trait Engine: Send + Sync {
    fn chat(&self, messages: &[ChatMessage], tools: &[serde_json::Value]) -> Result<ChatResponse>;
    fn set_model(&mut self, model: String);
    fn model(&self) -> &str;
}

/// Build an engine from env vars.
/// Resolution order: LLM_ENGINE → LLM_API_KEY/LLM_BASE_URL/LLM_MODEL
/// Falls back to provider-specific vars (OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.)
pub fn create_engine() -> Result<Box<dyn Engine>> { ... }
```

### Configuration

**Generic (engine-agnostic) env vars** — sufficient for any single-engine setup:

```
LLM_ENGINE    = openai | anthropic | gemini  (default: openai)
LLM_API_KEY   = (overrides provider-specific key)
LLM_BASE_URL  = (overrides provider-specific base URL)
LLM_MODEL     = (overrides provider-specific model default)
```

**Provider-specific env vars** — used as fallback defaults per engine:

```
# OpenAI engine (existing, unchanged)
OPENAI_API_KEY, OPENAI_BASE_URL, OPENAI_MODEL (default: gpt-5-mini-2025-08-07)

# Anthropic engine
ANTHROPIC_API_KEY, ANTHROPIC_BASE_URL (default: https://api.anthropic.com)
ANTHROPIC_MODEL (default: claude-sonnet-4-20250514)
ANTHROPIC_MAX_TOKENS (default: 8192)

# Gemini engine
GEMINI_API_KEY, GEMINI_BASE_URL (default: https://generativelanguage.googleapis.com)
GEMINI_MODEL (default: gemini-2.0-flash)
```

**Using a non-default vendor on an engine** (e.g., Kimi via Anthropic protocol):

```bash
LLM_ENGINE=anthropic
LLM_API_KEY=sk-kimi-xxx
LLM_BASE_URL=https://api.moonshot.cn
LLM_MODEL=kimi-k2
```

### Design decisions

1. **Message storage stays OpenAI-format in thread JSONL.** Translation happens at the engine boundary (on send/receive). Existing vaults need zero migration.

2. **Threads are not locked to an engine.** Users can switch `LLM_ENGINE` and continue any thread. The canonical storage format makes this seamless.

3. **Tool schemas stored in OpenAI format** (current `tool_schemas()` output). Each engine translates at call time. Avoids maintaining parallel schema definitions.

4. **Deep think follows the main engine** unless `DEEP_THINK_ENGINE` + matching config is set. This allows mixing (e.g., Claude for chat, OpenAI for deep think).

5. **Default to OpenAI when `LLM_ENGINE` is unset** — backward compatible, zero config change for existing users.

6. **Validate eagerly at engine creation** — fail fast with a clear message listing valid engines and required env vars.

7. **`LLM_*` generic vars override provider-specific vars** — lets you point any engine at any compatible endpoint without adding new provider-specific env var sets for every vendor.

## Technical Considerations

### Message translation

Each engine impl converts `ChatMessage` to its wire format:
- **OpenAI engine**: Direct mapping (canonical format)
- **Anthropic engine**: Extract system messages to top-level `system` param, map `Tool` role to `tool_result` content blocks, wrap tool calls as `tool_use` blocks
- **Gemini engine**: Map `Assistant` to `model` role, flatten tool calls into `functionCall` parts, system prompt to `systemInstruction`

### Tool schema translation

OpenAI format → engine-specific format at call time:
- **OpenAI engine**: Pass through
- **Anthropic engine**: Rename `function.parameters` to `input_schema`, restructure wrapper
- **Gemini engine**: Rename to `functionDeclarations`, adjust parameter schema

### Institutional learning: tool schema quality

Per `docs/solutions/integration-issues/untyped-tool-schemas-cause-empty-llm-output.md`: Every `object` parameter must have `properties` or a detailed `description`. This applies equally to all providers — the schema quality matters even more when translating across formats.

## Acceptance Criteria

- [ ] `LLM_ENGINE=openai` (or unset) works identically to current behavior (no regression)
- [ ] `LLM_ENGINE=anthropic` + `ANTHROPIC_API_KEY` enables Claude as the chat engine
- [ ] `LLM_ENGINE=gemini` + `GEMINI_API_KEY` enables Gemini as the chat engine
- [ ] Non-default vendor on shared engine works (e.g., `LLM_ENGINE=anthropic` + Kimi base URL)
- [ ] All 10+ tools work correctly with each engine (tool call + result round-trip)
- [ ] Existing vaults work without migration when switching engines
- [ ] `--model` CLI flag overrides engine-specific model default
- [ ] `LLM_*` generic vars override provider-specific vars
- [ ] Missing API key for selected engine fails fast with clear error message
- [ ] Deep think works with each engine
- [ ] `scripts/verify.sh` passes for whichever engine is configured

## Success Metrics

- All existing smoke tests pass with each engine
- Tool execution round-trip succeeds for representative tools (knowledge_update, thread_create, deep_think) across all three engines
- No changes to thread JSONL format or vault structure

## Dependencies & Risks

| Risk | Mitigation |
|------|-----------|
| Tool calling format bugs across engines | Canonical storage + per-engine tests with fixture responses |
| Anthropic engine requires `max_tokens` | Default 8192, configurable via env var |
| Gemini role naming (`model` vs `assistant`) | Translation layer in `gemini_chat.rs` |
| Engine API changes | Isolate engine logic in single files, easy to update |
| Vendor quirks on shared engine (e.g., Kimi on Anthropic) | Engine impls accept config overrides; test with multiple vendors |
| Streaming not in scope | Current blocking client pattern preserved; streaming is a separate enhancement |

## References & Research

### Internal References
- Embedding multi-provider pattern: `src/embeddings.rs:6-82`
- Current OpenAI client: `src/openai.rs:19-113`
- Agent loop (main consumer): `src/agent.rs:52-180`
- Tool schemas: `src/agent.rs:207-413`
- Tool schema learning: `docs/solutions/integration-issues/untyped-tool-schemas-cause-empty-llm-output.md`
- Runtime config (unused): `jj_vault/config/jj.runtime.yml`
