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
{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"fire","target_x":2147483648,"target_y":0}}}
{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}
{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}
{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}
{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"game_verify_replay","arguments":{}}}
{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"game_get_dev_state","arguments":{}}}
{"jsonrpc":"2.0","id":14,"method":"tools/call","params":{"name":"game_reset","arguments":{}}}
{"jsonrpc":"2.0","id":15,"method":"tools/call","params":{"name":"game_load_scenario","arguments":{"ascii_map":"#####\n#@.>#\n#####","max_turns":4}}}
{"jsonrpc":"2.0","id":16,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"move","direction":"East"}}}
{"jsonrpc":"2.0","id":17,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"move","direction":"East"}}}
{"jsonrpc":"2.0","id":18,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"descend"}}}
{"jsonrpc":"2.0","id":19,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}
{"jsonrpc":"2.0","id":20,"method":"resources/read","params":{"uri":"drl://rules/actions"}}
{"jsonrpc":"2.0","id":21,"method":"resources/read","params":{"uri":"drl://session/metrics"}}
{"jsonrpc":"2.0","id":22,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":778,"width":20,"height":10}}}
{"jsonrpc":"2.0","id":23,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"use","item_id":999}}}
{"jsonrpc":"2.0","id":24,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}
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
if len(lines) != 24:
    raise SystemExit(f"expected 24 JSON-RPC responses, found {len(lines)}")
responses = [json.loads(line) for line in lines]
if [response.get("id") for response in responses] != list(range(1, 25)):
    raise SystemExit("stdio response IDs are not in request order")
if not responses[0].get("result", {}).get("serverInfo", {}).get("name") == "drl-mcp":
    raise SystemExit("initialize response lacks server identity")
if responses[0].get("result", {}).get("protocolVersion") != "2024-11-05":
    raise SystemExit("supported initialize did not echo protocol version")
tools = responses[2].get("result", {}).get("tools")
if not tools:
    raise SystemExit("tools/list returned no tools")
tool_map = {tool.get("name"): tool for tool in tools}
start_props = tool_map["game_start"]["inputSchema"]["properties"]
if start_props["seed"]["maximum"] != 9007199254740992:
    raise SystemExit("game_start seed schema lacks JSON-safe maximum")
if start_props["width"]["maximum"] != 4294967295:
    raise SystemExit("game_start width schema lacks u32 maximum")
step_schema = tool_map["game_step_action"]["inputSchema"]
discriminator = step_schema.get("anyOf", [])
if len(discriminator) != 2 or {field for branch in discriminator for field in branch.get("required", [])} != {"action", "command"}:
    raise SystemExit("game_step_action schema lacks action-or-command discriminator")
if "fire" not in step_schema["properties"]["action"].get("enum", []):
    raise SystemExit("game_step_action schema lacks fire action")
if "command" not in step_schema["properties"] or "x" not in step_schema["properties"]:
    raise SystemExit("game_step_action schema lacks accepted aliases")
if step_schema["properties"]["target_x"]["minimum"] != -2147483648:
    raise SystemExit("game_step_action coordinate schema lacks i32 minimum")
if step_schema["properties"]["item_id"]["maximum"] != 9007199254740992:
    raise SystemExit("game_step_action item_id schema lacks JSON-safe maximum")
conditions = step_schema.get("allOf", [])
if len(conditions) != 5:
    raise SystemExit("game_step_action schema lacks action-specific conditions")
if conditions[0].get("then", {}).get("required") != ["direction"]:
    raise SystemExit("move/melee condition lacks direction requirement")
if conditions[2].get("then", {}).get("required") != ["item_id"]:
    raise SystemExit("item condition lacks item_id requirement")
if conditions[3].get("then", {}).get("required") != ["slot"]:
    raise SystemExit("unequip condition lacks slot requirement")
if conditions[0].get("if", {}).get("anyOf", [])[1].get("not", {}).get("required") != ["action"]:
    raise SystemExit("command condition does not defer to action precedence")
ranged = conditions[1].get("then", {}).get("anyOf", [])
if [branch.get("required") for branch in ranged] != [["target_x", "target_y"], ["target_x", "y"], ["x", "target_y"], ["x", "y"]]:
    raise SystemExit("ranged condition lacks coordinate alternatives")
if step_schema.get("additionalProperties") is False:
    raise SystemExit("game_step_action schema unexpectedly rejects unknown properties")
if not responses[3].get("result", {}).get("resources"):
    raise SystemExit("resources/list returned no resources")
if responses[4].get("result", {}).get("data", {}).get("status") != "GameStarted":
    raise SystemExit("game_start did not report GameStarted")
if not responses[6].get("result", {}).get("data", {}).get("legal_actions"):
    raise SystemExit("game_list_actions returned no legal actions")
if responses[7].get("error", {}).get("code") != -32602:
    raise SystemExit("invalid numeric action did not return INVALID_PARAMS")
if responses[9].get("result", {}).get("data", {}).get("turns_survived") is None:
    raise SystemExit("game_get_metrics returned no turns_survived field")
if not responses[10].get("result", {}).get("data", {}).get("commands"):
    raise SystemExit("game_save_replay returned no commands")
if responses[11].get("result", {}).get("data", {}).get("deterministic") is not True:
    raise SystemExit("game_verify_replay did not report deterministic replay")
if responses[11].get("result", {}).get("data", {}).get("command_count") != 1:
    raise SystemExit("game_verify_replay reported the wrong command count")
if responses[12].get("error", {}).get("code") != -32002:
    raise SystemExit("dev-state request did not preserve permission denial")
if responses[13].get("result", {}).get("data", {}).get("status") != "SessionReset":
    raise SystemExit("game_reset did not report SessionReset")
if responses[14].get("result", {}).get("data", {}).get("status") != "ScenarioLoaded":
    raise SystemExit("game_load_scenario did not report ScenarioLoaded")
if responses[17].get("result", {}).get("data", {}).get("outcome") != "Victory":
    raise SystemExit("scenario descend did not report Victory")
if responses[18].get("error", {}).get("code") != -32001:
    raise SystemExit("post-victory action was not rejected")
if not responses[19].get("result", {}).get("contents"):
    raise SystemExit("rules resource read returned no contents")
if not responses[20].get("result", {}).get("contents"):
    raise SystemExit("metrics resource read returned no contents")
if responses[21].get("result", {}).get("data", {}).get("status") != "GameStarted":
    raise SystemExit("pre-dispatch state-safety fixture did not restart a game")
if responses[22].get("error", {}).get("code") != -32001:
    raise SystemExit("recognized but unadvertised action did not return INVALID_ACTION")
if responses[23].get("result", {}).get("data", {}).get("game_over") is not False:
    raise SystemExit("valid action after pre-dispatch rejection did not execute")
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
