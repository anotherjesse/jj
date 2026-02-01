---
title: "feat: Add generate_image tool"
type: feat
date: 2026-01-31
---

# feat: Add `generate_image` tool

## Overview

Add a `generate_image` tool to the agent that generates images via the `flux2` CLI, stores them in `jj_vault/media/` with user-specified paths (encouraging descriptive folder/filenames for uniqueness), and returns the relative path. If the path already exists, return an error. Ensure the full pipeline works: generation -> storage -> display via `draw` tool (CLI canvas) and web UI.

## Problem Statement / Motivation

The agent can display images (`draw` tool) but cannot create them. Adding generation closes the loop: the agent can produce visual content on demand and immediately show it to the user. Storing images under descriptive paths in the vault makes them durable and referenceable across sessions.

## Proposed Solution

### Tool Schema

```json
{
  "name": "generate_image",
  "description": "Generate an image using flux2 and store it in the vault. Path should use descriptive folders for uniqueness (e.g. 'diagrams/arch-v2.png', 'food/pepperoni-pizza.png'). Returns error if path already exists.",
  "parameters": {
    "type": "object",
    "properties": {
      "prompt": {
        "type": "string",
        "description": "Text description of the image to generate."
      },
      "path": {
        "type": "string",
        "description": "Relative path within media/ (e.g. 'food/pizza.png'). Must end with .png. Intermediate directories are created automatically."
      },
      "reason": {
        "type": "string"
      }
    },
    "required": ["prompt", "path", "reason"]
  }
}
```

### Tool Execution Flow

1. **Validate `path`**: must match `^[a-zA-Z0-9][a-zA-Z0-9/_-]*\.png$`, no `..`, no leading `/`
2. **Resolve full path**: `{vault_path}/media/{path}`
3. **Check existence**: if file exists, return `{"status":"error","error":"exists: media/{path}"}`
4. **Create directories**: `fs::create_dir_all()` for parent dirs
5. **Run flux2**: `flux2 "{prompt}" "{full_path}"` — capture stdout/stderr, timeout 5 min
6. **Verify output**: confirm file was created on disk
7. **Return**: `{"status":"ok","data":{"path":"media/{path}"}}`

### Error Cases

| Condition | Response |
|-----------|----------|
| File exists | `{"status":"error","error":"exists: media/food/pizza.png"}` |
| Invalid path | `{"status":"error","error":"invalid path: must be relative, alphanumeric/dashes/underscores, end with .png"}` |
| flux2 not found | `{"status":"error","error":"flux2 not found"}` |
| flux2 fails | `{"status":"error","error":"flux2 failed: {stderr}"}` — delete partial file |
| flux2 timeout | `{"status":"error","error":"flux2 timed out after 300s"}` — kill process, delete partial file |

### Display Pipeline

After `generate_image` returns successfully, the agent calls the existing `draw` tool with the returned path to display it.

**CLI (already works):** `draw` calls `rcast draw {path}` — no changes needed.

**Web UI (needs work):**
1. Add a static file handler in the gateway to serve `jj_vault/media/` at `/media/`
2. In the web UI, detect image paths in tool results and render `<img>` tags
3. When `tool_call_result` event arrives for `generate_image` or `draw`, render the image inline

## Technical Considerations

### Security — Path Traversal Prevention
- Validate path with strict regex before any filesystem operation
- After joining with vault path, canonicalize and verify the result starts with `{vault_path}/media/`
- Reject null bytes, backslashes, `..` sequences

### Partial File Cleanup
- If flux2 fails or times out, delete any file at the target path before returning error
- Use a temp file + rename pattern if atomicity is important (nice-to-have)

### Subprocess Timeout
- 5-minute timeout via `std::process::Command` + spawned thread with `wait_timeout`
- Or use `tokio::process::Command` with timeout if async context available

### Vault Init
- Add `media/` to vault init directories in `src/vault.rs` (currently `artifacts/` exists but not `media/`)

### Tool Schema Quality
- Per documented learning: every parameter has explicit description and type
- `path` description includes examples of good naming patterns

## Acceptance Criteria

- [x] `generate_image` tool registered in `tool_schemas()` with fully-typed schema
- [x] Path validation rejects traversal attempts (`..`, absolute paths, invalid chars)
- [x] Existence check returns JSON error `{"status":"error","error":"exists: media/..."}`
- [x] `flux2` CLI invoked with prompt and output path; stderr captured on failure
- [x] Partial files cleaned up on flux2 failure/timeout
- [x] Intermediate directories created automatically
- [x] Returned path is relative (`media/food/pizza.png`)
- [x] Agent can chain `generate_image` -> `draw` to show the result
- [x] Gateway serves `jj_vault/media/` as static files at `/media/`
- [x] Web UI renders images inline when tool results contain image paths
- [x] `media/` directory added to vault init

## Success Metrics

- Agent can generate and display an image end-to-end in both CLI and web modes
- Path traversal attempts are rejected (manual test)
- Duplicate path returns error without invoking flux2

## Dependencies & Risks

- **flux2 CLI** must be installed and on PATH — tool degrades gracefully if missing
- **rcast** must be available for CLI display (already a dependency for `draw`)
- Gateway must be running for web display (already required for web mode)
- Image generation may be slow (10-60s) — synchronous for now, could become async later

## Implementation Touchpoints

### `src/agent.rs`
- Add schema to `tool_schemas()` (~line 395)
- Add execution handler in `execute_tool()` match block (~line 630)
- Path validation, subprocess call, cleanup logic

### `src/vault.rs`
- Add `media/` to vault init directory list

### `src/gateway/session.rs` (or new handler)
- Add static file serving for `/media/` -> `jj_vault/media/`

### `web/index.html`
- Handle `tool_call_result` events for image tools
- Render `<img src="/media/{path}">` in chat messages

## References & Research

- Existing draw tool pattern: `src/agent.rs:594-629`
- Tool schema definitions: `src/agent.rs:207-397`
- Tool response format: `src/agent.rs:142-145` (`{status, data}` wrapper)
- Vault directory init: `src/vault.rs:6-26`
- Learning: untyped tool schemas cause LLM to skip fields — `docs/solutions/integration-issues/untyped-tool-schemas-cause-empty-llm-output.md`
- Web UI events: `web/index.html:139-175`
