//! Security and information boundary tests for MCP interface.

use drl_mcp::McpServer;
use drl_mcp::json::JsonValue;
use drl_mcp::protocol::error_codes;

#[test]
fn test_dev_mode_permission_boundary() {
  let mut server = McpServer::new();

  // Start game
  let start_req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":1}}}"#;
  let _ = server.handle_request(start_req);

  // Default: dev mode disabled -> game_get_dev_state fails
  let dev_req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"game_get_dev_state","arguments":{}}}"#;
  let dev_resp = JsonValue::parse(&server.handle_request(dev_req)).unwrap();
  let err = dev_resp.get("error").expect("Permission error expected");
  assert_eq!(
    err.get("code").and_then(|v| v.as_i64()),
    Some(error_codes::PERMISSION_DENIED as i64)
  );

  // Enable dev mode on session
  server.session_mut().set_dev_mode(true);

  let dev_resp2 = JsonValue::parse(&server.handle_request(dev_req)).unwrap();
  assert!(dev_resp2.get("result").is_some());
  let omni = dev_resp2.get("result").unwrap().get("data").unwrap();
  assert!(omni.get("tiles").is_some());
  assert!(omni.get("actors").is_some());
}

#[test]
fn test_observation_fairness_hides_distant_monsters() {
  let mut server = McpServer::new();

  // Wide map where monster is far away outside visibility
  let ascii = r#"
########################################
#@....................................h#
########################################
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
  let obs = load_resp
    .get("result")
    .unwrap()
    .get("data")
    .unwrap()
    .get("observation")
    .unwrap();

  let visible_actors = obs.get("visible_actors").unwrap().as_array().unwrap();

  // Distant Former Human 'h' at x=38 must NOT be visible to player at x=1
  assert_eq!(visible_actors.len(), 1);
  assert_eq!(
    visible_actors[0].get("is_player").and_then(|v| v.as_bool()),
    Some(true)
  );
}

#[test]
fn test_turn_limit_boundary_enforcement() {
  let mut server = McpServer::new();

  // Start game with max_turns = 3
  let start_req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":55,"max_turns":3}}}"#;
  let _ = server.handle_request(start_req);

  // Step 1
  let step1 = server.handle_request(r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}"#);
  let d1 = JsonValue::parse(&step1)
    .unwrap()
    .get("result")
    .unwrap()
    .get("data")
    .unwrap()
    .clone();
  assert_eq!(d1.get("game_over").and_then(|v| v.as_bool()), Some(false));

  // Step 2
  let step2 = server.handle_request(r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}"#);
  let d2 = JsonValue::parse(&step2)
    .unwrap()
    .get("result")
    .unwrap()
    .get("data")
    .unwrap()
    .clone();
  assert_eq!(d2.get("game_over").and_then(|v| v.as_bool()), Some(false));

  // Step 3 -> Reaches turn limit
  let step3 = server.handle_request(r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}"#);
  let d3 = JsonValue::parse(&step3)
    .unwrap()
    .get("result")
    .unwrap()
    .get("data")
    .unwrap()
    .clone();
  assert_eq!(d3.get("game_over").and_then(|v| v.as_bool()), Some(true));
  assert_eq!(
    d3.get("outcome").and_then(|v| v.as_str()),
    Some("TurnLimitReached")
  );
}
