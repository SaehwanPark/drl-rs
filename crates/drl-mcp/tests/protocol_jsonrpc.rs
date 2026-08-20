//! Integration tests for JSON-RPC 2.0 protocol handshake, routing, and error handling.

use drl_mcp::McpServer;
use drl_mcp::json::JsonValue;
use drl_mcp::protocol::error_codes;

#[test]
fn test_jsonrpc_initialize_handshake() {
  let mut server = McpServer::new();

  let init_req = r#"{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": { "name": "test-agent", "version": "1.0.0" }
    }
  }"#;

  let resp_str = server.handle_request(init_req);
  let resp = JsonValue::parse(&resp_str).expect("Valid JSON response");

  assert_eq!(resp.get("jsonrpc").and_then(|v| v.as_str()), Some("2.0"));
  assert_eq!(resp.get("id").and_then(|v| v.as_u64()), Some(1));

  let result = resp.get("result").expect("Initialize result present");
  let server_info = result.get("serverInfo").expect("serverInfo present");
  assert_eq!(
    server_info.get("name").and_then(|v| v.as_str()),
    Some("drl-mcp")
  );

  let caps = result.get("capabilities").expect("capabilities present");
  assert!(caps.get("tools").is_some());
  assert!(caps.get("resources").is_some());
}

#[test]
fn test_jsonrpc_ping() {
  let mut server = McpServer::new();
  let ping_req = r#"{"jsonrpc":"2.0","id":42,"method":"ping"}"#;
  let resp_str = server.handle_request(ping_req);
  let resp = JsonValue::parse(&resp_str).expect("Valid JSON response");

  assert_eq!(resp.get("id").and_then(|v| v.as_u64()), Some(42));
  assert!(resp.get("result").is_some());
  assert!(resp.get("error").is_none());
}

#[test]
fn test_jsonrpc_tools_list() {
  let mut server = McpServer::new();
  let list_req = r#"{"jsonrpc":"2.0","id":"tools-1","method":"tools/list"}"#;
  let resp_str = server.handle_request(list_req);
  let resp = JsonValue::parse(&resp_str).expect("Valid JSON response");

  assert_eq!(resp.get("id").and_then(|v| v.as_str()), Some("tools-1"));

  let result = resp.get("result").expect("Tools list result");
  let tools = result
    .get("tools")
    .and_then(|v| v.as_array())
    .expect("Tools array");

  let tool_names: Vec<&str> = tools
    .iter()
    .filter_map(|t| t.get("name").and_then(|v| v.as_str()))
    .collect();

  assert!(tool_names.contains(&"game_start"));
  assert!(tool_names.contains(&"game_load_scenario"));
  assert!(tool_names.contains(&"game_get_observation"));
  assert!(tool_names.contains(&"game_list_actions"));
  assert!(tool_names.contains(&"game_step_action"));
  assert!(tool_names.contains(&"game_reset"));
  assert!(tool_names.contains(&"game_get_metrics"));
  assert!(tool_names.contains(&"game_save_replay"));
  assert!(tool_names.contains(&"game_get_dev_state"));
}

#[test]
fn test_jsonrpc_resources_list_and_read() {
  let mut server = McpServer::new();

  let list_req = r#"{"jsonrpc":"2.0","id":2,"method":"resources/list"}"#;
  let resp_str = server.handle_request(list_req);
  let resp = JsonValue::parse(&resp_str).unwrap();
  let res_list = resp
    .get("result")
    .unwrap()
    .get("resources")
    .unwrap()
    .as_array()
    .unwrap();
  assert!(res_list.len() >= 4);

  let read_req =
    r#"{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"drl://rules/actions"}}"#;
  let read_resp_str = server.handle_request(read_req);
  let read_resp = JsonValue::parse(&read_resp_str).unwrap();
  let contents = read_resp
    .get("result")
    .unwrap()
    .get("contents")
    .unwrap()
    .as_array()
    .unwrap();
  assert_eq!(contents.len(), 1);
  let text = contents[0].get("text").unwrap().as_str().unwrap();
  assert!(text.contains("Semantic Action Catalog"));
}

#[test]
fn test_jsonrpc_error_handling() {
  let mut server = McpServer::new();

  // 1. Malformed JSON
  let malformed = "NOT A JSON OBJECT";
  let resp1_str = server.handle_request(malformed);
  let resp1 = JsonValue::parse(&resp1_str).unwrap();
  let err1 = resp1.get("error").unwrap();
  assert_eq!(
    err1.get("code").and_then(|v| v.as_i64()),
    Some(error_codes::PARSE_ERROR as i64)
  );

  // 2. Unknown method
  let unknown = r#"{"jsonrpc":"2.0","id":99,"method":"non_existent_method"}"#;
  let resp2_str = server.handle_request(unknown);
  let resp2 = JsonValue::parse(&resp2_str).unwrap();
  let err2 = resp2.get("error").unwrap();
  assert_eq!(
    err2.get("code").and_then(|v| v.as_i64()),
    Some(error_codes::METHOD_NOT_FOUND as i64)
  );

  // 3. Calling tool before game session started
  let obs_req =
    r#"{"jsonrpc":"2.0","id":100,"method":"tools/call","params":{"name":"game_get_observation"}}"#;
  let resp3_str = server.handle_request(obs_req);
  let resp3 = JsonValue::parse(&resp3_str).unwrap();
  let err3 = resp3.get("error").unwrap();
  assert_eq!(
    err3.get("code").and_then(|v| v.as_i64()),
    Some(error_codes::SESSION_NOT_ACTIVE as i64)
  );
}
