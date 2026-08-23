//! Integration tests for MCP semantic tool execution and gameplay lifecycle.

use drl_mcp::McpServer;
use drl_mcp::json::JsonValue;

fn ready_server() -> McpServer {
  let mut server = McpServer::new();
  let _ = server.handle_request(
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"drl-tools","version":"1"}}}"#,
  );
  let _ = server.handle_request(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
  server
}

#[test]
fn test_mcp_procedural_gameplay_tools() {
  let mut server = ready_server();

  // 1. Start procedural game with seed
  let start_req = r#"{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "game_start",
      "arguments": { "seed": 77, "width": 30, "height": 15, "max_turns": 200 }
    }
  }"#;
  let start_resp = JsonValue::parse(&server.handle_request(start_req)).unwrap();
  let start_res = start_resp.get("result").expect("Start result");
  assert_eq!(
    start_res
      .get("data")
      .and_then(|d| d.get("status"))
      .and_then(|s| s.as_str()),
    Some("GameStarted")
  );

  // 2. Get observation
  let obs_req = r#"{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": { "name": "game_get_observation", "arguments": {} }
  }"#;
  let obs_resp = JsonValue::parse(&server.handle_request(obs_req)).unwrap();
  let obs_data = obs_resp
    .get("result")
    .unwrap()
    .get("data")
    .unwrap()
    .get("observation")
    .unwrap();

  let p_pos = obs_data.get("player_position").unwrap();
  let px = p_pos.get("x").unwrap().as_i64().unwrap();
  let py = p_pos.get("y").unwrap().as_i64().unwrap();
  assert!(px >= 0 && py >= 0);

  // 3. List actions
  let actions_req = r#"{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": { "name": "game_list_actions", "arguments": {} }
  }"#;
  let actions_resp = JsonValue::parse(&server.handle_request(actions_req)).unwrap();
  let legal_actions = actions_resp
    .get("result")
    .unwrap()
    .get("data")
    .unwrap()
    .get("legal_actions")
    .unwrap()
    .as_array()
    .unwrap();
  assert!(!legal_actions.is_empty());

  // 4. Step a wait action
  let step_req = r#"{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "game_step_action",
      "arguments": { "action": "wait" }
    }
  }"#;
  let step_resp = JsonValue::parse(&server.handle_request(step_req)).unwrap();
  let step_data = step_resp.get("result").unwrap().get("data").unwrap();
  let events = step_data.get("events").unwrap().as_array().unwrap();
  assert!(!events.is_empty());

  // Step another wait action
  let step_req2 = r#"{
    "jsonrpc": "2.0",
    "id": 5,
    "method": "tools/call",
    "params": {
      "name": "game_step_action",
      "arguments": { "action": "wait" }
    }
  }"#;
  let _ = server.handle_request(step_req2);

  // 5. Query metrics
  let metrics_req = r#"{
    "jsonrpc": "2.0",
    "id": 6,
    "method": "tools/call",
    "params": { "name": "game_get_metrics", "arguments": {} }
  }"#;
  let metrics_resp = JsonValue::parse(&server.handle_request(metrics_req)).unwrap();
  let metrics_data = metrics_resp.get("result").unwrap().get("data").unwrap();
  assert_eq!(
    metrics_data.get("turns_survived").and_then(|v| v.as_u64()),
    Some(1)
  );

  // 6. Save replay
  let replay_req = r#"{
    "jsonrpc": "2.0",
    "id": 7,
    "method": "tools/call",
    "params": { "name": "game_save_replay", "arguments": {} }
  }"#;
  let replay_resp = JsonValue::parse(&server.handle_request(replay_req)).unwrap();
  let replay_data = replay_resp.get("result").unwrap().get("data").unwrap();
  let cmds = replay_data.get("commands").unwrap().as_array().unwrap();
  assert_eq!(cmds.len(), 2);
}

#[test]
fn test_mcp_scenario_combat_and_item_use() {
  let mut server = ready_server();

  // ASCII map: Player @, Former Human h, 9mm Ammo a, MedPack m, Exit >
  let ascii = r#"
#####
#@.h#
#ma>#
#####
"#;

  let load_req = format!(
    r#"{{
      "jsonrpc": "2.0",
      "id": 1,
      "method": "tools/call",
      "params": {{
        "name": "game_load_scenario",
        "arguments": {{ "ascii_map": {ascii_json} }}
      }}
    }}"#,
    ascii_json = JsonValue::from(ascii).to_compact_string()
  );

  let load_resp = JsonValue::parse(&server.handle_request(&load_req)).unwrap();
  assert!(load_resp.get("result").is_some());

  // Shoot at Former Human at (3, 1)
  let fire_req = r#"{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "game_step_action",
      "arguments": { "action": "fire", "target_x": 3, "target_y": 1 }
    }
  }"#;
  let fire_resp = JsonValue::parse(&server.handle_request(fire_req)).unwrap();
  let fire_data = fire_resp.get("result").unwrap().get("data").unwrap();
  let events = fire_data.get("events").unwrap().as_array().unwrap();
  assert!(
    events
      .iter()
      .any(|e| e.get("type").and_then(|v| v.as_str()) == Some("AttackResolved"))
  );

  // Move South to (1, 2) onto MedPack
  let move_s = r#"{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "game_step_action",
      "arguments": { "action": "move", "direction": "South" }
    }
  }"#;
  let move_s_resp = JsonValue::parse(&server.handle_request(move_s)).unwrap();
  assert!(move_s_resp.get("result").is_some());

  // Pick up MedPack
  let pickup_req = r#"{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "game_step_action",
      "arguments": { "action": "pickup" }
    }
  }"#;
  let pickup_resp = JsonValue::parse(&server.handle_request(pickup_req)).unwrap();
  let pickup_data = pickup_resp.get("result").unwrap().get("data").unwrap();
  let inv = pickup_data
    .get("observation")
    .unwrap()
    .get("inventory")
    .unwrap()
    .as_array()
    .unwrap();
  assert!(
    inv
      .iter()
      .any(|item| item.get("category").and_then(|v| v.as_str()) == Some("MedPack"))
  );

  // Use the MedPack from inventory
  let medpack_id = inv
    .iter()
    .find(|item| item.get("category").and_then(|v| v.as_str()) == Some("MedPack"))
    .unwrap()
    .get("id")
    .unwrap()
    .as_u64()
    .unwrap();

  let use_req = format!(
    r#"{{
      "jsonrpc": "2.0",
      "id": 5,
      "method": "tools/call",
      "params": {{
        "name": "game_step_action",
        "arguments": {{ "action": "use", "item_id": {medpack_id} }}
      }}
    }}"#
  );
  let use_resp = JsonValue::parse(&server.handle_request(&use_req)).unwrap();
  let use_data = use_resp.get("result").unwrap().get("data").unwrap();
  let use_events = use_data.get("events").unwrap().as_array().unwrap();
  assert!(
    use_events
      .iter()
      .any(|e| e.get("type").and_then(|v| v.as_str()) == Some("ItemUsed"))
  );
}
