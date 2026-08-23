//! Integration tests for JSON-RPC 2.0 protocol handshake, routing, and error handling.

use drl_mcp::McpServer;
use drl_mcp::json::JsonValue;
use drl_mcp::protocol::error_codes;

fn ready_server() -> McpServer {
  let mut server = McpServer::new();
  let _ = server.handle_request(
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"drl-test","version":"1"}}}"#,
  );
  let _ = server.handle_request(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
  server
}

#[test]
fn test_jsonrpc_initialize_handshake() {
  let mut server = McpServer::new();

  let init_req = r#"{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": { "experimental": { "feature": true } },
      "clientInfo": {
        "name": "test-agent",
        "version": "1.0.0",
        "metadata": { "tier": "test" }
      }
    }
  }"#;

  let resp_str = server.handle_request(init_req);
  let resp = JsonValue::parse(&resp_str).expect("Valid JSON response");

  assert_eq!(resp.get("jsonrpc").and_then(|v| v.as_str()), Some("2.0"));
  assert_eq!(resp.get("id").and_then(|v| v.as_u64()), Some(1));

  let result = resp.get("result").expect("Initialize result present");
  assert_eq!(
    result.get("protocolVersion").and_then(|v| v.as_str()),
    Some("2024-11-05")
  );
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
fn test_jsonrpc_initialize_falls_back_for_unsupported_version() {
  let mut server = McpServer::new();
  let request = r#"{"jsonrpc":"2.0","id":7,"method":"initialize","params":{"protocolVersion":"2099-01-01","capabilities":{},"clientInfo":{"name":"future-client","version":"1"}}}"#;
  let response = JsonValue::parse(&server.handle_request(request)).unwrap();

  assert_eq!(
    response
      .get("result")
      .and_then(|result| result.get("protocolVersion"))
      .and_then(|version| version.as_str()),
    Some("2024-11-05")
  );
}

#[test]
fn test_jsonrpc_initialize_requires_protocol_version_string() {
  let mut server = McpServer::new();
  for request in [
    r#"{"jsonrpc":"2.0","id":8,"method":"initialize"}"#,
    r#"{"jsonrpc":"2.0","id":9,"method":"initialize","params":{}}"#,
    r#"{"jsonrpc":"2.0","id":10,"method":"initialize","params":{"protocolVersion":null,"capabilities":{},"clientInfo":{"name":"test","version":"1"}}}"#,
    r#"{"jsonrpc":"2.0","id":11,"method":"initialize","params":{"protocolVersion":2024,"capabilities":{},"clientInfo":{"name":"test","version":"1"}}}"#,
  ] {
    let response = JsonValue::parse(&server.handle_request(request)).unwrap();
    assert_eq!(
      response
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(|code| code.as_i64()),
      Some(error_codes::INVALID_PARAMS as i64)
    );
  }
}

#[test]
fn test_jsonrpc_initialize_requires_client_envelope_fields() {
  let mut server = McpServer::new();
  for request in [
    r#"{"jsonrpc":"2.0","id":20,"method":"initialize","params":null}"#,
    r#"{"jsonrpc":"2.0","id":21,"method":"initialize","params":[]}"#,
    r#"{"jsonrpc":"2.0","id":22,"method":"initialize","params":{"protocolVersion":"2024-11-05","clientInfo":{"name":"test","version":"1"}}}"#,
    r#"{"jsonrpc":"2.0","id":23,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":[],"clientInfo":{"name":"test","version":"1"}}}"#,
    r#"{"jsonrpc":"2.0","id":24,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{}}}"#,
    r#"{"jsonrpc":"2.0","id":25,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":1,"version":"1"}}}"#,
    r#"{"jsonrpc":"2.0","id":26,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":false}}}"#,
  ] {
    let response = JsonValue::parse(&server.handle_request(request)).unwrap();
    assert_eq!(
      response
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(|code| code.as_i64()),
      Some(error_codes::INVALID_PARAMS as i64)
    );
    assert!(!server.session().is_active());
  }
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
  let mut server = ready_server();
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
  assert!(tool_names.contains(&"game_verify_replay"));
  assert!(tool_names.contains(&"game_get_dev_state"));
}

#[test]
fn test_jsonrpc_tools_list_publishes_truthful_input_schemas() {
  let mut server = ready_server();
  let response =
    JsonValue::parse(&server.handle_request(r#"{"jsonrpc":"2.0","id":90,"method":"tools/list"}"#))
      .unwrap();
  let tools = response
    .get("result")
    .and_then(|result| result.get("tools"))
    .and_then(JsonValue::as_array)
    .expect("tools array");
  let schema_for = |name: &str| {
    tools
      .iter()
      .find(|tool| tool.get("name").and_then(JsonValue::as_str) == Some(name))
      .and_then(|tool| tool.get("inputSchema"))
      .expect("tool input schema")
  };

  let start = schema_for("game_start");
  let start_props = start.get("properties").expect("game_start properties");
  assert_eq!(
    start_props
      .get("seed")
      .and_then(|field| field.get("minimum"))
      .and_then(JsonValue::as_u64),
    Some(0)
  );
  assert_eq!(
    start_props
      .get("seed")
      .and_then(|field| field.get("maximum"))
      .and_then(JsonValue::as_u64),
    Some(9_007_199_254_740_992)
  );
  assert_eq!(
    start_props
      .get("width")
      .and_then(|field| field.get("maximum"))
      .and_then(JsonValue::as_u64),
    Some(u32::MAX as u64)
  );

  let load = schema_for("game_load_scenario");
  assert_eq!(
    load
      .get("required")
      .and_then(JsonValue::as_array)
      .map(|required| required
        .iter()
        .any(|value| value.as_str() == Some("ascii_map"))),
    Some(true)
  );

  let step = schema_for("game_step_action");
  let required = step
    .get("required")
    .and_then(JsonValue::as_array)
    .expect("game_step_action required fields");
  assert!(
    required
      .iter()
      .any(|value| value.as_str() == Some("action"))
  );
  assert!(step.get("additionalProperties").is_none());
  let step_props = step.get("properties").expect("game_step_action properties");
  let action_enum = step_props
    .get("action")
    .and_then(|field| field.get("enum"))
    .and_then(JsonValue::as_array)
    .expect("action enum");
  for alias in ["move", "attack_melee", "melee", "fire", "shoot", "wait"] {
    assert!(
      action_enum
        .iter()
        .any(|value| value.as_str() == Some(alias))
    );
  }
  let direction_enum = step_props
    .get("direction")
    .and_then(|field| field.get("enum"))
    .and_then(JsonValue::as_array)
    .expect("direction enum");
  for alias in ["north", "n", "up", "k", "n_key", "."] {
    assert!(
      direction_enum
        .iter()
        .any(|value| value.as_str() == Some(alias))
    );
  }
  let slot_enum = step_props
    .get("slot")
    .and_then(|field| field.get("enum"))
    .and_then(JsonValue::as_array)
    .expect("slot enum");
  assert!(
    slot_enum
      .iter()
      .any(|value| value.as_str() == Some("weapon"))
  );
  assert!(
    slot_enum
      .iter()
      .any(|value| value.as_str() == Some("Armor"))
  );
  for alias in ["command", "x", "y"] {
    assert!(step_props.get(alias).is_some(), "missing alias {alias}");
  }
  for coordinate in ["target_x", "target_y", "x", "y"] {
    let field = step_props.get(coordinate).expect("coordinate field");
    assert_eq!(
      field.get("minimum").and_then(JsonValue::as_i64),
      Some(i32::MIN as i64)
    );
    assert_eq!(
      field.get("maximum").and_then(JsonValue::as_i64),
      Some(i32::MAX as i64)
    );
  }
  let item_id = step_props.get("item_id").expect("item_id field");
  assert_eq!(item_id.get("minimum").and_then(JsonValue::as_u64), Some(0));
  assert_eq!(
    item_id.get("maximum").and_then(JsonValue::as_u64),
    Some(9_007_199_254_740_992)
  );
}

#[test]
fn test_jsonrpc_verify_replay_is_deterministic_and_state_safe() {
  let mut server = ready_server();

  let inactive = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"game_verify_replay","arguments":{}}}"#,
  ))
  .unwrap();
  assert_eq!(
    inactive
      .get("error")
      .and_then(|error| error.get("code"))
      .and_then(|code| code.as_i64()),
    Some(error_codes::SESSION_NOT_ACTIVE as i64)
  );

  let _ = server.handle_request(
    r#"{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":91,"width":20,"height":10}}}"#,
  );
  let _ = server.handle_request(
    r#"{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}"#,
  );

  let metrics_before = server.handle_request(
    r#"{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#,
  );
  let replay_before = server.handle_request(
    r#"{"jsonrpc":"2.0","id":14,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}"#,
  );
  let verify_request = r#"{"jsonrpc":"2.0","id":15,"method":"tools/call","params":{"name":"game_verify_replay","arguments":{}}}"#;
  let verify_one = server.handle_request(verify_request);
  let verify_two = server.handle_request(verify_request);
  assert_eq!(verify_one, verify_two);

  let verify = JsonValue::parse(&verify_one).unwrap();
  let data = verify.get("result").and_then(|result| result.get("data"));
  assert_eq!(
    data
      .and_then(|data| data.get("deterministic"))
      .and_then(|value| value.as_bool()),
    Some(true)
  );
  assert_eq!(
    data
      .and_then(|data| data.get("command_count"))
      .and_then(|value| value.as_u64()),
    Some(1)
  );
  assert_eq!(
    data
      .and_then(|data| data.get("version"))
      .and_then(|value| value.as_u64()),
    Some(1)
  );
  assert_eq!(
    metrics_before,
    server.handle_request(
      r#"{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#,
    )
  );
  assert_eq!(
    replay_before,
    server.handle_request(
      r#"{"jsonrpc":"2.0","id":14,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}"#,
    )
  );
}

#[test]
fn test_jsonrpc_verify_replay_reconstructs_procedural_layout() {
  let mut server = ready_server();
  let _ = server.handle_request(
    r#"{"jsonrpc":"2.0","id":20,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":16,"width":40,"height":20}}}"#,
  );

  for id in 21..=37 {
    let response = JsonValue::parse(&server.handle_request(&format!(
      r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"game_step_action","arguments":{{"action":"move","direction":"North"}}}}}}"#
    )))
    .unwrap();
    assert!(
      response.get("error").is_none(),
      "unexpected step error: {response:?}"
    );
  }

  let verify = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":38,"method":"tools/call","params":{"name":"game_verify_replay","arguments":{}}}"#,
  ))
  .unwrap();
  let data = verify.get("result").and_then(|result| result.get("data"));
  assert_eq!(
    data
      .and_then(|data| data.get("deterministic"))
      .and_then(|value| value.as_bool()),
    Some(true)
  );
  assert_eq!(
    data
      .and_then(|data| data.get("command_count"))
      .and_then(|value| value.as_u64()),
    Some(17)
  );
}

#[test]
fn test_jsonrpc_resources_list_and_read() {
  let mut server = ready_server();

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
  let mut server = ready_server();

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

#[test]
fn test_jsonrpc_rejects_non_scalar_request_ids() {
  let mut server = McpServer::new();
  for request in [
    r#"{"jsonrpc":"2.0","id":true,"method":"ping"}"#,
    r#"{"jsonrpc":"2.0","id":[],"method":"ping"}"#,
    r#"{"jsonrpc":"2.0","id":{},"method":"ping"}"#,
  ] {
    let response = JsonValue::parse(&server.handle_request(request)).unwrap();
    assert!(response.get("id").is_some_and(JsonValue::is_null));
    assert_eq!(
      response
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(|code| code.as_i64()),
      Some(error_codes::INVALID_REQUEST as i64)
    );
  }
}

#[test]
fn test_jsonrpc_rejects_non_object_method_params_without_execution() {
  let mut server = ready_server();
  for request in [
    r#"{"jsonrpc":"2.0","id":30,"method":"tools/call","params":[]}"#,
    r#"{"jsonrpc":"2.0","id":31,"method":"tools/call","params":{"name":"game_start","arguments":[]}}"#,
    r#"{"jsonrpc":"2.0","id":32,"method":"tools/call","params":{"name":"game_start","arguments":null}}"#,
    r#"{"jsonrpc":"2.0","id":33,"method":"resources/read","params":[]}"#,
  ] {
    let response = JsonValue::parse(&server.handle_request(request)).unwrap();
    assert_eq!(
      response
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(|code| code.as_i64()),
      Some(error_codes::INVALID_PARAMS as i64)
    );
  }
  assert!(!server.session().is_active());
}

#[test]
fn test_jsonrpc_rejects_invalid_numeric_tool_arguments_without_execution() {
  let mut server = ready_server();
  for request in [
    r#"{"jsonrpc":"2.0","id":40,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":"7"}}}"#,
    r#"{"jsonrpc":"2.0","id":41,"method":"tools/call","params":{"name":"game_start","arguments":{"width":4294967296}}}"#,
    r##"{"jsonrpc":"2.0","id":42,"method":"tools/call","params":{"name":"game_load_scenario","arguments":{"ascii_map":"#@>#","max_turns":true}}}"##,
    r#"{"jsonrpc":"2.0","id":43,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":9007199254740993}}}"#,
    r#"{"jsonrpc":"2.0","id":44,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":18446744073709551616}}}"#,
    r#"{"jsonrpc":"2.0","id":45,"method":"tools/call","params":{"name":"game_start","arguments":{"max_turns":18446744073709551616}}}"#,
    r#"{"jsonrpc":"2.0","id":46,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":1.0000000000000001}}}"#,
    r#"{"jsonrpc":"2.0","id":47,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":9007199254740991.5}}}"#,
    r#"{"jsonrpc":"2.0","id":48,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":1e-100000000000}}}"#,
    r#"{"jsonrpc":"2.0","id":49,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":1e9223372036854775807}}}"#,
  ] {
    let response = JsonValue::parse(&server.handle_request(request)).unwrap();
    assert_eq!(
      response
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(|code| code.as_i64()),
      Some(error_codes::INVALID_PARAMS as i64)
    );
  }
  assert!(!server.session().is_active());
}

#[test]
fn test_jsonrpc_accepts_maximum_exact_json_integer() {
  let mut server = ready_server();
  let response = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":50,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":9007199254740992}}}"#,
  ))
  .unwrap();
  assert!(response.get("error").is_none());
  assert_eq!(
    response
      .get("result")
      .and_then(|result| result.get("data"))
      .and_then(|data| data.get("seed"))
      .and_then(|seed| seed.as_u64()),
    Some(9_007_199_254_740_992)
  );
}

#[test]
fn test_jsonrpc_rejects_unsafe_step_action_numbers_without_mutation() {
  let mut server = ready_server();
  let _ = server.handle_request(
    r#"{"jsonrpc":"2.0","id":60,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":7,"width":20,"height":10}}}"#,
  );
  let metrics_before = server.handle_request(
    r#"{"jsonrpc":"2.0","id":61,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#,
  );
  let replay_before = server.handle_request(
    r#"{"jsonrpc":"2.0","id":62,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}"#,
  );

  for (id, arguments) in [
    (
      63,
      r#"{"action":"fire","target_x":2147483648,"target_y":0}"#,
    ),
    (
      64,
      r#"{"action":"fire","target_x":-2147483649,"target_y":0}"#,
    ),
    (65, r#"{"action":"fire","target_x":1.5,"target_y":0}"#),
    (66, r#"{"action":"fire","target_x":"1","target_y":0}"#),
    (67, r#"{"action":"use","item_id":-1}"#),
    (68, r#"{"action":"use","item_id":9007199254740993}"#),
    (69, r#"{"action":"drop","item_id":true}"#),
  ] {
    let request = format!(
      r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"game_step_action","arguments":{arguments}}}}}"#
    );
    let response = JsonValue::parse(&server.handle_request(&request)).unwrap();
    assert_eq!(
      response
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(|code| code.as_i64()),
      Some(error_codes::INVALID_PARAMS as i64)
    );
  }

  assert_eq!(
    metrics_before,
    server.handle_request(
      r#"{"jsonrpc":"2.0","id":61,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#,
    )
  );
  assert_eq!(
    replay_before,
    server.handle_request(
      r#"{"jsonrpc":"2.0","id":62,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}"#,
    )
  );

  let valid = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":70,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}"#,
  ))
  .unwrap();
  assert!(valid.get("error").is_none());
}
