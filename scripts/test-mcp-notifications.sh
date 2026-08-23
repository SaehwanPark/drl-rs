#!/bin/sh

set -eu

cd "$(dirname "$0")/.."
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/drl-mcp-notifications.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

cat >"$temp_dir/requests.jsonl" <<'EOF'
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"game_start","arguments":{"seed":7}}}
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"game_get_metrics"}}
not-json
{"jsonrpc":"2.0","id":null,"method":"ping"}
EOF

cargo run -q -p drl-app -- --mcp <"$temp_dir/requests.jsonl" >"$temp_dir/responses.jsonl"

python3 - "$temp_dir/responses.jsonl" <<'PY'
import json
import sys

lines = [line for line in open(sys.argv[1], encoding="utf-8") if line.strip()]
if len(lines) != 3:
    raise SystemExit(f"expected 3 responses for 5 requests, found {len(lines)}")
responses = [json.loads(line) for line in lines]
if responses[0].get("id") != 1 or "result" not in responses[0]:
    raise SystemExit("identified metrics request did not observe notification-started session")
if responses[0]["result"]["data"].get("turns_survived") is None:
    raise SystemExit("metrics response lacks turns_survived")
if responses[1].get("error", {}).get("code") != -32700:
    raise SystemExit("malformed input did not return a parse error")
if responses[2].get("id") is not None or "result" not in responses[2]:
    raise SystemExit("explicit id:null request was incorrectly suppressed")
PY

printf '%s\n' 'MCP notification transport contract: PASS (side effects, suppression, parse errors, null IDs)'
