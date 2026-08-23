#!/bin/sh

set -eu

cd "$(dirname "$0")/.."
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/drl-mcp-notifications.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

cat >"$temp_dir/requests.jsonl" <<'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"drl-notifications","version":"1"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"game_start","arguments":{"seed":7}}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"game_get_metrics"}}
not-json
{"jsonrpc":"2.0","id":null,"method":"ping"}
{"jsonrpc":"2.0","id":{},"method":"ping"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"game_start","arguments":[]}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":"7"}}}
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":18446744073709551616}}}
EOF

cargo run -q -p drl-app -- --mcp <"$temp_dir/requests.jsonl" >"$temp_dir/responses.jsonl"

python3 - "$temp_dir/responses.jsonl" <<'PY'
import json
import sys

lines = [line for line in open(sys.argv[1], encoding="utf-8") if line.strip()]
if len(lines) != 8:
    raise SystemExit(f"expected 8 responses for 10 requests, found {len(lines)}")
responses = [json.loads(line) for line in lines]
if responses[0].get("id") != 1 or "result" not in responses[0]:
    raise SystemExit("initialize request did not return a response")
if responses[1].get("id") != 2 or "result" not in responses[1]:
    raise SystemExit("identified metrics request did not observe notification-started session")
if responses[1]["result"]["data"].get("turns_survived") is None:
    raise SystemExit("metrics response lacks turns_survived")
if responses[2].get("error", {}).get("code") != -32700:
    raise SystemExit("malformed input did not return a parse error")
if responses[3].get("id") is not None or "result" not in responses[3]:
    raise SystemExit("explicit id:null request was incorrectly suppressed")
if responses[4].get("id") is not None or responses[4].get("error", {}).get("code") != -32600:
    raise SystemExit("non-scalar request id did not return invalid-request error")
if responses[5].get("id") != 3 or responses[5].get("error", {}).get("code") != -32602:
    raise SystemExit("non-object tool arguments did not return invalid-params error")
if responses[6].get("id") != 4 or responses[6].get("error", {}).get("code") != -32602:
    raise SystemExit("wrong-typed game_start argument did not return invalid-params error")
if responses[7].get("id") != 5 or responses[7].get("error", {}).get("code") != -32602:
    raise SystemExit("out-of-range game_start argument did not return invalid-params error")
PY

printf '%s\n' 'MCP notification transport contract: PASS (side effects, suppression, parse errors, null IDs)'
