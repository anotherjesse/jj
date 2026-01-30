#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_ARGS=(--manifest-path "$ROOT_DIR/Cargo.toml")

echo "==> cargo check"
cargo check "${BIN_ARGS[@]}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

VAULT_DIR="$TMP_DIR/jj_vault"

echo "==> init vault"
cargo run "${BIN_ARGS[@]}" --quiet -- vault init --path "$VAULT_DIR"

echo "==> create thread"
THREAD_PATH="$(cargo run "${BIN_ARGS[@]}" --quiet -- thread create --vault "$VAULT_DIR")"
test -f "$THREAD_PATH"

echo "==> append event"
cargo run "${BIN_ARGS[@]}" --quiet -- thread append \
  --thread "$THREAD_PATH" \
  --event-type user_message \
  --role user \
  --content "hello"

echo "==> read thread"
LINE="$(cargo run "${BIN_ARGS[@]}" --quiet -- thread read --thread "$THREAD_PATH" --limit 1)"
echo "$LINE" | grep -q "\"type\":\"user_message\""

echo "==> apply knowledge patch"
PATCH_PATH="$TMP_DIR/patch.json"
cat > "$PATCH_PATH" <<'JSON'
{
  "doc_path": "knowledge/prefs/interaction.md",
  "title": "Interaction preferences",
  "type": "preference",
  "confidence": 0.7,
  "body_append": "- Prefer minimal upfront design; let rules evolve."
}
JSON

cargo run "${BIN_ARGS[@]}" --quiet -- knowledge apply \
  --vault "$VAULT_DIR" \
  --patch "$PATCH_PATH" \
  --author "verifier" \
  --reason "verify knowledge apply"

test -f "$VAULT_DIR/knowledge/prefs/interaction.md"
test -f "$VAULT_DIR/audit/ledger.jsonl"

echo "==> verify ok"
