#!/usr/bin/env bash
# One-time migration: fix thread headers with correct kind detection.
# Replaces existing headers and detects ingest threads by checking if
# the first event is a tool_call (ingest threads start with tool calls,
# not user messages) or if any early content mentions "ingestion".
# DO NOT COMMIT this script.

set -euo pipefail

VAULT="${1:-jj_vault}"
THREADS_DIR="$VAULT/threads"

if [ ! -d "$THREADS_DIR" ]; then
  echo "No threads dir at $THREADS_DIR"
  exit 1
fi

count=0
for f in $(find "$THREADS_DIR" -name 'thr_*.jsonl' -type f); do
  thread_id=$(basename "$f" .jsonl)
  created=$(stat -f '%Sm' -t '%Y-%m-%dT%H:%M:%SZ' "$f" 2>/dev/null || date -u '+%Y-%m-%dT%H:%M:%SZ')

  # Remove existing header if present
  first=$(head -1 "$f")
  has_header=false
  if echo "$first" | python3 -c "import sys,json; d=json.load(sys.stdin); exit(0 if d.get('jj_thread') else 1)" 2>/dev/null; then
    has_header=true
  fi

  # Detect kind: check if first few events are tool_calls or mention ingestion
  kind=$(python3 -c "
import json, sys
lines = open('$f').readlines()
start = 1 if $( $has_header && echo 'True' || echo 'False' ) else 0
for line in lines[start:start+5]:
    try:
        e = json.loads(line)
        t = e.get('type','')
        c = str(e.get('content',''))
        if 'process the following document for ingestion' in c.lower():
            print('ingest'); sys.exit(0)
        tn = str(e.get('tool_name',''))
        if t == 'tool_call' and tn in ('knowledge_search','knowledge_apply','knowledge_read'):
            print('ingest'); sys.exit(0)
    except: pass
print('chat')
")

  agent="jj"
  if [ "$kind" = "ingest" ]; then
    agent="ingest"
  fi

  header="{\"jj_thread\":true,\"thread_id\":\"$thread_id\",\"kind\":\"$kind\",\"agent\":\"$agent\",\"created_at\":\"$created\"}"

  tmp=$(mktemp)
  echo "$header" > "$tmp"
  if [ "$has_header" = true ]; then
    tail -n +2 "$f" >> "$tmp"
  else
    cat "$f" >> "$tmp"
  fi
  mv "$tmp" "$f"

  echo "  $thread_id  [$kind]"
  count=$((count + 1))
done

echo "Migrated $count threads."
