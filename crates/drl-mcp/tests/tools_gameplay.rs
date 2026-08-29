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
  let actions_json = server.handle_request(actions_req);
  assert_eq!(actions_json, server.handle_request(actions_req));
  let actions_resp = JsonValue::parse(&actions_json).unwrap();
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
  assert_eq!(
    replay_data.get("format").and_then(JsonValue::as_str),
    Some("drl-rs-replay-v2")
  );
  assert_eq!(
    replay_data
      .get("schema_version")
      .and_then(JsonValue::as_u64),
    Some(2)
  );
  assert_eq!(
    cmds[0].get("action").and_then(JsonValue::as_str),
    Some("wait")
  );
}

#[test]
fn test_mcp_legal_action_pre_dispatch_gate_is_state_safe() {
  let mut server = ready_server();
  let ascii = "\n#####\n#@..#\n#####\n";
  let load_request = format!(
    r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"game_load_scenario","arguments":{{"ascii_map":{}}}}}}}"#,
    JsonValue::from(ascii).to_compact_string()
  );
  assert!(
    JsonValue::parse(&server.handle_request(&load_request))
      .unwrap()
      .get("result")
      .is_some()
  );

  let observation_request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"game_get_observation","arguments":{}}}"#;
  let metrics_request = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#;
  let replay_request = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}"#;
  let observation_before = server.handle_request(observation_request);
  let metrics_before = server.handle_request(metrics_request);
  let replay_before = server.handle_request(replay_request);

  for action in [
    r#"{"action":"move","direction":"North"}"#,
    r#"{"action":"descend"}"#,
    r#"{"action":"use","item_id":999}"#,
    r#"{"action":"drop","item_id":999}"#,
    r#"{"action":"unequip","slot":"Armor"}"#,
  ] {
    let request = format!(
      r#"{{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{{"name":"game_step_action","arguments":{action}}}}}"#
    );
    let response = JsonValue::parse(&server.handle_request(&request)).unwrap();
    assert_eq!(response.get("error"), None);
    assert_eq!(
      response
        .get("result")
        .and_then(|result| result.get("isError"))
        .and_then(JsonValue::as_bool),
      Some(true)
    );
    assert_eq!(
      response
        .get("result")
        .and_then(|result| result.get("data"))
        .and_then(|data| data.get("code"))
        .and_then(JsonValue::as_i64),
      Some(drl_mcp::protocol::error_codes::INVALID_ACTION as i64)
    );
    assert_eq!(
      server.handle_request(observation_request),
      observation_before
    );
    assert_eq!(server.handle_request(metrics_request), metrics_before);
    assert_eq!(server.handle_request(replay_request), replay_before);
  }

  let malformed = r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"teleport"}}}"#;
  let malformed_response = JsonValue::parse(&server.handle_request(malformed)).unwrap();
  assert_eq!(
    malformed_response
      .get("error")
      .and_then(|error| error.get("code"))
      .and_then(JsonValue::as_i64),
    Some(drl_mcp::protocol::error_codes::INVALID_PARAMS as i64)
  );
  assert_eq!(
    server.handle_request(observation_request),
    observation_before
  );

  let valid_wait = r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}"#;
  let wait_response = JsonValue::parse(&server.handle_request(valid_wait)).unwrap();
  assert!(wait_response.get("result").is_some());
  let replay_after = JsonValue::parse(&server.handle_request(replay_request)).unwrap();
  assert_eq!(
    replay_after
      .get("result")
      .and_then(|result| result.get("data"))
      .and_then(|data| data.get("commands"))
      .and_then(JsonValue::as_array)
      .map(Vec::len),
    Some(1)
  );
}

#[test]
fn test_mcp_catalog_advertises_and_executes_explicit_melee() {
  let mut server = ready_server();
  let ascii = "\n#####\n#@h.#\n#####\n";
  let load_request = format!(
    r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"game_load_scenario","arguments":{{"ascii_map":{}}}}}}}"#,
    JsonValue::from(ascii).to_compact_string()
  );
  assert!(
    JsonValue::parse(&server.handle_request(&load_request))
      .unwrap()
      .get("result")
      .is_some()
  );

  let list = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"game_list_actions","arguments":{}}}"#,
  ))
  .unwrap();
  let actions = list
    .get("result")
    .and_then(|result| result.get("data"))
    .and_then(|data| data.get("legal_actions"))
    .and_then(JsonValue::as_array)
    .unwrap();
  assert!(actions.iter().any(|action| {
    action.get("action").and_then(JsonValue::as_str) == Some("AttackMelee")
      && action
        .get("params")
        .and_then(|params| params.get("direction"))
        .and_then(JsonValue::as_str)
        == Some("East")
  }));

  let attack = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"attack_melee","direction":"East"}}}"#,
  ))
  .unwrap();
  assert!(attack.get("result").is_some());

  let pistol_ascii = "\n######\n#@p..#\n######\n";
  let load_pistol = format!(
    r#"{{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{{"name":"game_load_scenario","arguments":{{"ascii_map":{}}}}}}}"#,
    JsonValue::from(pistol_ascii).to_compact_string()
  );
  assert!(
    JsonValue::parse(&server.handle_request(&load_pistol))
      .unwrap()
      .get("result")
      .is_some()
  );
  assert!(server
    .handle_request(
      r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"move","direction":"East"}}}"#,
    )
    .contains("\"result\""));
  assert!(server
    .handle_request(
      r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"pickup"}}}"#,
    )
    .contains("\"result\""));
  let observation = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"game_get_observation","arguments":{}}}"#,
  ))
  .unwrap();
  let pistol_id = observation
    .get("result")
    .and_then(|result| result.get("data"))
    .and_then(|data| data.get("observation"))
    .and_then(|data| data.get("inventory"))
    .and_then(JsonValue::as_array)
    .and_then(|items| {
      items
        .iter()
        .find(|item| item.get("category").and_then(JsonValue::as_str) == Some("Weapon"))
    })
    .and_then(|item| item.get("id"))
    .and_then(JsonValue::as_u64)
    .expect("scenario pistol in inventory");
  let equip = format!(
    r#"{{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{{"name":"game_step_action","arguments":{{"action":"equip","item_id":{pistol_id}}}}}}}"#
  );
  assert!(server.handle_request(&equip).contains("\"result\""));
  let actions = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"game_list_actions","arguments":{}}}"#,
  ))
  .unwrap();
  assert!(
    actions
      .get("result")
      .and_then(|result| result.get("data"))
      .and_then(|data| data.get("legal_actions"))
      .and_then(JsonValue::as_array)
      .is_some_and(|items| {
        items.iter().any(|item| {
          item.get("action").and_then(JsonValue::as_str) == Some("Unequip")
            && item
              .get("params")
              .and_then(|params| params.get("slot"))
              .and_then(JsonValue::as_str)
              == Some("Weapon")
        })
      })
  );
  let unequip = r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"unequip","slot":"Weapon"}}}"#;
  assert!(
    JsonValue::parse(&server.handle_request(unequip))
      .unwrap()
      .get("result")
      .is_some()
  );
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

#[test]
fn test_mcp_stairs_victory_and_terminal_gate() {
  let mut server = ready_server();
  let ascii = "\n#####\n#@>.#\n#####\n";
  let load_request = format!(
    r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"game_load_scenario","arguments":{{"ascii_map":{}}}}}}}"#,
    JsonValue::from(ascii).to_compact_string()
  );
  let load = JsonValue::parse(&server.handle_request(&load_request)).unwrap();
  assert!(load.get("result").is_some());

  let move_east = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"move","direction":"East"}}}"#;
  let moved = JsonValue::parse(&server.handle_request(move_east)).unwrap();
  assert!(moved.get("result").is_some());

  let descend = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"descend"}}}"#;
  let descended = JsonValue::parse(&server.handle_request(descend)).unwrap();
  let data = descended.get("result").unwrap().get("data").unwrap();
  assert_eq!(
    data.get("game_over").and_then(JsonValue::as_bool),
    Some(true)
  );
  assert_eq!(
    data.get("outcome").and_then(JsonValue::as_str),
    Some("Victory")
  );
  assert_eq!(
    data
      .get("legal_actions")
      .and_then(JsonValue::as_array)
      .map(Vec::len),
    Some(0)
  );

  let metrics_request = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#;
  let replay_request = r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}"#;
  let metrics_before = server.handle_request(metrics_request);
  let replay_before = server.handle_request(replay_request);
  let verify = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"game_verify_replay","arguments":{}}}"#,
  ))
  .unwrap();
  assert_eq!(
    verify
      .get("result")
      .and_then(|result| result.get("data"))
      .and_then(|data| data.get("deterministic"))
      .and_then(JsonValue::as_bool),
    Some(true)
  );

  let rejected = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}"#,
  ))
  .unwrap();
  assert_eq!(rejected.get("error"), None);
  assert_eq!(
    rejected
      .get("result")
      .and_then(|result| result.get("isError"))
      .and_then(JsonValue::as_bool),
    Some(true)
  );
  assert_eq!(
    rejected
      .get("result")
      .and_then(|result| result.get("data"))
      .and_then(|data| data.get("code"))
      .and_then(JsonValue::as_i64),
    Some(drl_mcp::protocol::error_codes::INVALID_ACTION as i64)
  );
  assert_eq!(metrics_before, server.handle_request(metrics_request));
  assert_eq!(replay_before, server.handle_request(replay_request));
  let list_after = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"game_list_actions","arguments":{}}}"#,
  ))
  .unwrap();
  assert_eq!(
    list_after
      .get("result")
      .and_then(|result| result.get("data"))
      .and_then(|data| data.get("legal_actions"))
      .and_then(JsonValue::as_array)
      .map(Vec::len),
    Some(0)
  );
}

#[test]
fn test_mcp_death_terminal_gate_preserves_replay() {
  let mut server = ready_server();
  let ascii = "\n#####\n#@h.#\n#####\n";
  let load_request = format!(
    r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"game_load_scenario","arguments":{{"ascii_map":{}}}}}}}"#,
    JsonValue::from(ascii).to_compact_string()
  );
  let load = JsonValue::parse(&server.handle_request(&load_request)).unwrap();
  assert!(load.get("result").is_some());

  let mut death_seen = false;
  for id in 2..=100 {
    let request = r#"{"jsonrpc":"2.0","id":0,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}"#
      .replace("\"id\":0", &format!("\"id\":{id}"));
    let response = JsonValue::parse(&server.handle_request(&request)).unwrap();
    let Some(data) = response.get("result").and_then(|result| result.get("data")) else {
      panic!("unexpected pre-terminal step error: {response:?}");
    };
    if data.get("game_over").and_then(JsonValue::as_bool) == Some(true) {
      let outcome = data
        .get("outcome")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
      assert!(
        outcome.starts_with("Death"),
        "unexpected outcome: {outcome}"
      );
      death_seen = true;
      break;
    }
  }
  assert!(death_seen, "adjacent hostile did not kill the player");

  let terminal_observation = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":100,"method":"tools/call","params":{"name":"game_get_observation","arguments":{}}}"#,
  ))
  .unwrap();
  assert_eq!(
    terminal_observation
      .get("result")
      .and_then(|result| result.get("data"))
      .and_then(|data| data.get("legal_actions"))
      .and_then(JsonValue::as_array)
      .map(Vec::len),
    Some(0)
  );

  let metrics_request = r#"{"jsonrpc":"2.0","id":101,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#;
  let replay_request = r#"{"jsonrpc":"2.0","id":102,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}"#;
  let metrics_before = server.handle_request(metrics_request);
  let replay_before = server.handle_request(replay_request);
  let rejected = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":103,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}"#,
  ))
  .unwrap();
  assert_eq!(rejected.get("error"), None);
  assert_eq!(
    rejected
      .get("result")
      .and_then(|result| result.get("isError"))
      .and_then(JsonValue::as_bool),
    Some(true)
  );
  assert_eq!(
    rejected
      .get("result")
      .and_then(|result| result.get("data"))
      .and_then(|data| data.get("code"))
      .and_then(JsonValue::as_i64),
    Some(drl_mcp::protocol::error_codes::INVALID_ACTION as i64)
  );
  assert_eq!(metrics_before, server.handle_request(metrics_request));
  assert_eq!(replay_before, server.handle_request(replay_request));
}

#[test]
fn test_mcp_turn_limit_terminal_catalog_is_empty() {
  let mut server = ready_server();
  let ascii = "\n#####\n#@..#\n#####\n";
  let load_request = format!(
    r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"game_load_scenario","arguments":{{"ascii_map":{},"max_turns":1}}}}}}"#,
    JsonValue::from(ascii).to_compact_string()
  );
  assert!(
    JsonValue::parse(&server.handle_request(&load_request))
      .unwrap()
      .get("result")
      .is_some()
  );

  let wait = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}"#,
  ))
  .unwrap();
  let data = wait.get("result").unwrap().get("data").unwrap();
  assert_eq!(
    data.get("outcome").and_then(JsonValue::as_str),
    Some("TurnLimitReached")
  );
  assert_eq!(
    data
      .get("legal_actions")
      .and_then(JsonValue::as_array)
      .map(Vec::len),
    Some(0)
  );

  let list = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"game_list_actions","arguments":{}}}"#,
  ))
  .unwrap();
  assert_eq!(
    list
      .get("result")
      .and_then(|result| result.get("data"))
      .and_then(|data| data.get("legal_actions"))
      .and_then(JsonValue::as_array)
      .map(Vec::len),
    Some(0)
  );
}
