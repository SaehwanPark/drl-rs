#!/bin/sh

set -eu

cd "$(dirname "$0")/.."
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/drl-mcp-batches.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

cat >"$temp_dir/requests.jsonl" <<'EOF'
[{"jsonrpc":"2.0","method":"tools/call","params":{"name":"game_start","arguments":{"seed":9}}},{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"game_get_metrics"}},{"jsonrpc":"2.0","id":null,"method":"ping"}]
[]
EOF

for run in 1 2; do
  cargo run -q -p drl-app -- --mcp <"$temp_dir/requests.jsonl" >"$temp_dir/run-$run.jsonl"
done
cmp "$temp_dir/run-1.jsonl" "$temp_dir/run-2.jsonl"

python3 - "$temp_dir/run-1.jsonl" <<'PY'
import json
import sys

lines = [line for line in open(sys.argv[1], encoding="utf-8") if line.strip()]
if len(lines) != 2:
    raise SystemExit(f"expected 2 batch responses, found {len(lines)}")
first = json.loads(lines[0])
if not isinstance(first, list) or len(first) != 2:
    raise SystemExit("batch response did not preserve two identified/null-ID responses")
if first[0].get("id") != 1 or "result" not in first[0]:
    raise SystemExit("identified batch metrics request did not observe notification-started session")
if first[1].get("id") is not None or "result" not in first[1]:
    raise SystemExit("explicit null-ID batch request was not returned")
second = json.loads(lines[1])
if second.get("error", {}).get("code") != -32600:
    raise SystemExit("empty batch did not return invalid-request error")
PY

printf '%s\n' 'MCP batch transport contract: PASS (ordered responses, notification omission, null IDs, empty-batch rejection)'
