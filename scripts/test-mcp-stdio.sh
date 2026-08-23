#!/bin/sh

set -eu

cd "$(dirname "$0")/.."
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/drl-mcp-stdio.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

requests="$temp_dir/requests.jsonl"
cat >"$requests" <<'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"drl-contract","version":"1"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"ping"}
{"jsonrpc":"2.0","id":3,"method":"tools/list"}
{"jsonrpc":"2.0","id":4,"method":"resources/list"}
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":777,"width":20,"height":10,"max_turns":5}}}
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"game_get_observation","arguments":{}}}
{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"game_list_actions","arguments":{}}}
{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}
{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}
{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}
{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"game_get_dev_state","arguments":{}}}
{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"game_reset","arguments":{}}}
{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"game_load_scenario","arguments":{"ascii_map":"#####\n#@.>#\n#####","max_turns":4}}}
{"jsonrpc":"2.0","id":14,"method":"resources/read","params":{"uri":"drl://rules/actions"}}
{"jsonrpc":"2.0","id":15,"method":"resources/read","params":{"uri":"drl://session/metrics"}}
EOF

fallback_requests="$temp_dir/fallback-requests.jsonl"
cat >"$fallback_requests" <<'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2099-01-01","capabilities":{},"clientInfo":{"name":"future-client","version":"1"}}}
EOF

for run in 1 2; do
  cargo run -q -p drl-app -- --mcp <"$requests" >"$temp_dir/run-$run.jsonl"
done
cmp "$temp_dir/run-1.jsonl" "$temp_dir/run-2.jsonl"

python3 - "$temp_dir/run-1.jsonl" <<'PY'
import json
import sys

lines = [line for line in open(sys.argv[1], encoding="utf-8") if line.strip()]
if len(lines) != 15:
    raise SystemExit(f"expected 15 JSON-RPC responses, found {len(lines)}")
responses = [json.loads(line) for line in lines]
if [response.get("id") for response in responses] != list(range(1, 16)):
    raise SystemExit("stdio response IDs are not in request order")
if not responses[0].get("result", {}).get("serverInfo", {}).get("name") == "drl-mcp":
    raise SystemExit("initialize response lacks server identity")
if responses[0].get("result", {}).get("protocolVersion") != "2024-11-05":
    raise SystemExit("supported initialize did not echo protocol version")
if not responses[2].get("result", {}).get("tools"):
    raise SystemExit("tools/list returned no tools")
if not responses[3].get("result", {}).get("resources"):
    raise SystemExit("resources/list returned no resources")
if responses[4].get("result", {}).get("data", {}).get("status") != "GameStarted":
    raise SystemExit("game_start did not report GameStarted")
if not responses[6].get("result", {}).get("data", {}).get("legal_actions"):
    raise SystemExit("game_list_actions returned no legal actions")
if responses[8].get("result", {}).get("data", {}).get("turns_survived") is None:
    raise SystemExit("game_get_metrics returned no turns_survived field")
if not responses[9].get("result", {}).get("data", {}).get("commands"):
    raise SystemExit("game_save_replay returned no commands")
if responses[10].get("error", {}).get("code") != -32002:
    raise SystemExit("dev-state request did not preserve permission denial")
if responses[11].get("result", {}).get("data", {}).get("status") != "SessionReset":
    raise SystemExit("game_reset did not report SessionReset")
if responses[12].get("result", {}).get("data", {}).get("status") != "ScenarioLoaded":
    raise SystemExit("game_load_scenario did not report ScenarioLoaded")
if not responses[13].get("result", {}).get("contents"):
    raise SystemExit("rules resource read returned no contents")
if not responses[14].get("result", {}).get("contents"):
    raise SystemExit("metrics resource read returned no contents")
PY

for run in 1 2; do
  cargo run -q -p drl-app -- --mcp <"$fallback_requests" >"$temp_dir/fallback-run-$run.jsonl"
done
cmp "$temp_dir/fallback-run-1.jsonl" "$temp_dir/fallback-run-2.jsonl"

python3 - "$temp_dir/fallback-run-1.jsonl" <<'PY'
import json
import sys

responses = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8") if line.strip()]
if len(responses) != 1 or responses[0].get("result", {}).get("protocolVersion") != "2024-11-05":
    raise SystemExit("unsupported initialize did not return supported fallback")
PY

printf '%s\n' 'MCP stdio lifecycle contract: PASS (repeatable JSON-RPC, tools, resources, gameplay, fairness)'
