#!/usr/bin/env bash
set -euo pipefail

: "${OPENAI_API_KEY:?OPENAI_API_KEY is required}"

BASE_URL="${OPENAI_BASE_URL:-https://api.openai.com}"
MODEL="${OPENAI_MODEL:-gpt-5.2-2025-12-11}"

echo "==> OpenAI smoke test ($MODEL)"

RESPONSE="$(curl -sS "$BASE_URL/v1/chat/completions" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d @- <<EOF
{
  "model": "$MODEL",
  "messages": [
    {"role": "system", "content": "You are a terse assistant."},
    {"role": "user", "content": "Reply with a single word."}
  ]
}
EOF
)"

python3 - <<'PY'
import json, sys

try:
    data = json.load(sys.stdin)
except json.JSONDecodeError as exc:
    raise SystemExit(f"Invalid JSON response: {exc}") from exc

if "error" in data:
    raise SystemExit(f"API error: {data['error']}")

choices = data.get("choices")
if not choices:
    raise SystemExit("No choices returned")

message = choices[0].get("message", {})
content = message.get("content")
if not content:
    raise SystemExit("No message content returned")

print("==> ok:", content.strip())
PY
<<< "$RESPONSE"
