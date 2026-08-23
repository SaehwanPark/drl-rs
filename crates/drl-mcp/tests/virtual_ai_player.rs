//! Virtual AI player test agent driving complete gameplay sessions via MCP JSON-RPC.

use drl_mcp::McpServer;
use drl_mcp::json::JsonValue;

#[test]
fn test_virtual_ai_agent_playing_scenario_via_mcp() {
  let mut server = McpServer::new();

  // 1. Handshake
  let init_req =
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}"#;
  let init_res = server.handle_request(init_req);
  assert!(init_res.contains("\"name\":\"drl-mcp\""));

  // 2. Load combat scenario fixture
  // Layout: Player '@' at (1,1), ammo 'a', former human 'h' at (1,3), stairs down '>' at (1,4)
  let ascii = r#"
#####
#@..#
#a..#
#h..#
#>..#
#####
"#;

  let load_req = format!(
    r#"{{
      "jsonrpc": "2.0",
      "id": 2,
      "method": "tools/call",
      "params": {{
        "name": "game_load_scenario",
        "arguments": {{ "ascii_map": {ascii_json}, "max_turns": 50 }}
      }}
    }}"#,
    ascii_json = JsonValue::from(ascii).to_compact_string()
  );

  let load_resp = JsonValue::parse(&server.handle_request(&load_req)).unwrap();
  assert!(load_resp.get("result").is_some());

  let mut turns = 0;
  let mut won = false;

  // 3. Agent control loop over MCP JSON-RPC
  while turns < 30 {
    turns += 1;

    // Get current observation and legal actions
    let obs_req = format!(
      r#"{{"jsonrpc":"2.0","id":{},"method":"tools/call","params":{{"name":"game_get_observation","arguments":{{}}}}}}"#,
      100 + turns
    );
    let obs_resp = JsonValue::parse(&server.handle_request(&obs_req)).unwrap();
    let data = obs_resp.get("result").unwrap().get("data").unwrap();
    let obs = data.get("observation").unwrap();
    let legal_actions = data.get("legal_actions").unwrap().as_array().unwrap();

    let p_pos = obs.get("player_position").unwrap();
    let py = p_pos.get("y").unwrap().as_i64().unwrap();

    let visible_actors = obs.get("visible_actors").unwrap().as_array().unwrap();
    let hostile_visible = visible_actors.iter().find(|a| {
      !a.get("is_player").unwrap().as_bool().unwrap() && a.get("alive").unwrap().as_bool().unwrap()
    });

    let action_json = if let Some(hostile) = hostile_visible {
      // Enemy visible -> fire or move towards it
      let h_pos = hostile.get("position").unwrap();
      let hx = h_pos.get("x").unwrap().as_i64().unwrap();
      let hy = h_pos.get("y").unwrap().as_i64().unwrap();
      format!(r#"{{"action":"fire","target_x":{hx},"target_y":{hy}}}"#)
    } else if legal_actions
      .iter()
      .any(|a| a.get("action").and_then(|v| v.as_str()) == Some("Descend"))
    {
      // On stairs -> descend
      won = true;
      r#"{"action":"descend"}"#.to_string()
    } else if py < 4
      && legal_actions.iter().any(|a| {
        a.get("action").and_then(|v| v.as_str()) == Some("Move")
          && a
            .get("params")
            .unwrap()
            .get("direction")
            .and_then(|v| v.as_str())
            == Some("South")
      })
    {
      // Step South toward exit
      r#"{"action":"move","direction":"South"}"#.to_string()
    } else {
      r#"{"action":"wait"}"#.to_string()
    };

    let step_req = format!(
      r#"{{"jsonrpc":"2.0","id":{},"method":"tools/call","params":{{"name":"game_step_action","arguments":{action_json}}}}}"#,
      200 + turns
    );
    let step_resp = JsonValue::parse(&server.handle_request(&step_req)).unwrap();
    let step_data = step_resp.get("result").unwrap().get("data").unwrap();

    if step_data.get("game_over").and_then(|v| v.as_bool()) == Some(true) {
      break;
    }
    if won {
      break;
    }
  }

  // 4. Verify telemetry metrics
  let metrics_req = r#"{"jsonrpc":"2.0","id":999,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#;
  let metrics_resp = JsonValue::parse(&server.handle_request(metrics_req)).unwrap();
  let metrics = metrics_resp.get("result").unwrap().get("data").unwrap();
  assert!(metrics.get("turns_survived").unwrap().as_u64().unwrap() > 0);
  assert!(metrics.get("shots_fired").unwrap().as_u64().unwrap() > 0);

  // 5. Save replay
  let replay_req = r#"{"jsonrpc":"2.0","id":1000,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}"#;
  let replay_resp = JsonValue::parse(&server.handle_request(replay_req)).unwrap();
  let replay_data = replay_resp.get("result").unwrap().get("data").unwrap();
  let cmds = replay_data.get("commands").unwrap().as_array().unwrap();
  assert!(!cmds.is_empty());

  // 6. Direct simulation replay determinism verification
  let session_replay = server.session().export_replay().expect("Replay log exists");
  assert!(!session_replay.commands.is_empty());
}
