//! Integration tests for JSON-RPC 2.0 protocol handshake, routing, and error handling.

use drl_mcp::json::JsonValue;
use drl_mcp::protocol::error_codes;
use drl_mcp::{McpServer, get_all_resource_definitions, get_all_tool_definitions};

fn ready_server() -> McpServer {
  let mut server = McpServer::new();
  let _ = server.handle_request(
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"drl-test","version":"1"}}}"#,
  );
  let _ = server.handle_request(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
  server
}

fn collect_list_pages(server: &mut McpServer, method: &str, key: &str) -> Vec<JsonValue> {
  let mut cursor = None;
  let mut items = Vec::new();
  let mut request_id = 700;
  loop {
    let request = match cursor.as_deref() {
      Some(cursor) => format!(
        r#"{{"jsonrpc":"2.0","id":{},"method":"{}","params":{{"cursor":"{}"}}}}"#,
        request_id, method, cursor
      ),
      None => format!(
        r#"{{"jsonrpc":"2.0","id":{},"method":"{}"}}"#,
        request_id, method
      ),
    };
    let response = JsonValue::parse(&server.handle_request(&request)).unwrap();
    assert!(response.get("error").is_none(), "list page should succeed");
    let result = response.get("result").unwrap();
    items.extend(result.get(key).unwrap().as_array().unwrap().iter().cloned());
    cursor = result
      .get("nextCursor")
      .and_then(JsonValue::as_str)
      .map(str::to_string);
    request_id += 1;
    if cursor.is_none() {
      break;
    }
  }
  items
}

fn assert_tool_error(response: &JsonValue, code: i32) {
  assert!(
    response.get("error").is_none(),
    "tool failure escaped as RPC error: {response:?}"
  );
  let result = response.get("result").expect("tool error result");
  assert_eq!(
    result.get("isError").and_then(JsonValue::as_bool),
    Some(true)
  );
  let content = result
    .get("content")
    .and_then(JsonValue::as_array)
    .expect("tool error content");
  assert_eq!(content.len(), 1);
  assert_eq!(
    content[0].get("type").and_then(JsonValue::as_str),
    Some("text")
  );
  assert!(content[0].get("text").and_then(JsonValue::as_str).is_some());
  let data = result.get("data").expect("tool error details");
  assert_eq!(
    data.get("code").and_then(JsonValue::as_i64),
    Some(code as i64)
  );
  assert!(data.get("message").and_then(JsonValue::as_str).is_some());
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
  let tools = collect_list_pages(&mut server, "tools/list", "tools");

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
  let tools = collect_list_pages(&mut server, "tools/list", "tools");
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
      .and_then(|field| field.get("minimum"))
      .and_then(JsonValue::as_u64),
    Some(3)
  );
  assert_eq!(
    start_props
      .get("width")
      .and_then(|field| field.get("maximum"))
      .and_then(JsonValue::as_u64),
    Some(512)
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
  let discriminator = step
    .get("anyOf")
    .and_then(JsonValue::as_array)
    .expect("action-or-command discriminator");
  assert_eq!(discriminator.len(), 2);
  for (branch, field) in discriminator.iter().zip(["action", "command"]) {
    assert_eq!(
      branch
        .get("required")
        .and_then(JsonValue::as_array)
        .map(|required| required.iter().any(|value| value.as_str() == Some(field))),
      Some(true)
    );
  }
  assert!(step.get("additionalProperties").is_none());
  let conditions = step
    .get("allOf")
    .and_then(JsonValue::as_array)
    .expect("action-specific conditions");
  assert_eq!(conditions.len(), 5);
  let then_required = |condition: &JsonValue| {
    condition
      .get("then")
      .and_then(|then_schema| then_schema.get("required"))
      .and_then(JsonValue::as_array)
      .map(|required| {
        required
          .iter()
          .filter_map(JsonValue::as_str)
          .map(str::to_string)
          .collect::<Vec<_>>()
      })
      .unwrap_or_default()
  };
  assert_eq!(then_required(&conditions[0]), vec!["direction".to_string()]);
  assert_eq!(then_required(&conditions[2]), vec!["item_id".to_string()]);
  assert_eq!(then_required(&conditions[3]), vec!["slot".to_string()]);
  assert!(then_required(&conditions[4]).is_empty());
  assert_eq!(
    conditions[0]
      .get("if")
      .and_then(|schema| schema.get("anyOf"))
      .and_then(JsonValue::as_array)
      .and_then(|branches| branches.get(1))
      .and_then(|branch| branch.get("not"))
      .and_then(|schema| schema.get("required"))
      .and_then(JsonValue::as_array)
      .and_then(|required| required.first())
      .and_then(JsonValue::as_str),
    Some("action"),
    "command conditions must defer to a present action"
  );
  let ranged_alternatives = conditions[1]
    .get("then")
    .and_then(|then_schema| then_schema.get("anyOf"))
    .and_then(JsonValue::as_array)
    .expect("ranged coordinate alternatives");
  assert_eq!(ranged_alternatives.len(), 4);
  assert_eq!(
    then_required(&conditions[1]),
    Vec::<String>::new(),
    "ranged branch uses anyOf rather than a flat required set"
  );
  assert_eq!(
    ranged_alternatives[0]
      .get("required")
      .and_then(JsonValue::as_array)
      .map(|required| required
        .iter()
        .filter_map(JsonValue::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>()),
    Some(vec!["target_x".to_string(), "target_y".to_string()])
  );
  assert_eq!(
    ranged_alternatives[1]
      .get("required")
      .and_then(JsonValue::as_array)
      .map(|required| required
        .iter()
        .filter_map(JsonValue::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>()),
    Some(vec!["target_x".to_string(), "y".to_string()])
  );
  assert_eq!(
    ranged_alternatives[2]
      .get("required")
      .and_then(JsonValue::as_array)
      .map(|required| required
        .iter()
        .filter_map(JsonValue::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>()),
    Some(vec!["x".to_string(), "target_y".to_string()])
  );
  assert_eq!(
    ranged_alternatives[3]
      .get("required")
      .and_then(JsonValue::as_array)
      .map(|required| required
        .iter()
        .filter_map(JsonValue::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>()),
    Some(vec!["x".to_string(), "y".to_string()])
  );
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

  let verify_replay = schema_for("game_verify_replay");
  assert_eq!(
    verify_replay
      .get("properties")
      .and_then(|properties| properties.get("replay"))
      .and_then(|replay| replay.get("type"))
      .and_then(JsonValue::as_str),
    Some("object")
  );
}

#[test]
fn test_jsonrpc_list_pagination_is_deterministic_and_state_safe() {
  let mut server = ready_server();
  let first_tools_request = r#"{"jsonrpc":"2.0","id":100,"method":"tools/list"}"#;
  let first_tools = server.handle_request(first_tools_request);
  assert_eq!(first_tools, server.handle_request(first_tools_request));
  let first_tools_value = JsonValue::parse(&first_tools).unwrap();
  assert_eq!(
    first_tools_value
      .get("result")
      .and_then(|result| result.get("tools"))
      .and_then(JsonValue::as_array)
      .map(Vec::len),
    Some(4)
  );
  assert_eq!(
    first_tools_value
      .get("result")
      .and_then(|result| result.get("nextCursor"))
      .and_then(JsonValue::as_str),
    Some("tools-v1-4")
  );

  let first_resources = JsonValue::parse(
    &server.handle_request(r#"{"jsonrpc":"2.0","id":101,"method":"resources/list"}"#),
  )
  .unwrap();
  assert_eq!(
    first_resources
      .get("result")
      .and_then(|result| result.get("resources"))
      .and_then(JsonValue::as_array)
      .map(Vec::len),
    Some(2)
  );
  assert_eq!(
    first_resources
      .get("result")
      .and_then(|result| result.get("nextCursor"))
      .and_then(JsonValue::as_str),
    Some("resources-v1-2")
  );

  let tools = collect_list_pages(&mut server, "tools/list", "tools");
  let expected_tools = get_all_tool_definitions()
    .into_iter()
    .map(|tool| tool.to_json_value())
    .collect::<Vec<_>>();
  assert_eq!(tools, expected_tools);
  let tool_names = tools
    .iter()
    .map(|tool| tool.get("name").and_then(JsonValue::as_str).unwrap())
    .collect::<Vec<_>>();
  let mut unique_names = tool_names.clone();
  unique_names.sort_unstable();
  unique_names.dedup();
  assert_eq!(unique_names.len(), tool_names.len());

  let resources = collect_list_pages(&mut server, "resources/list", "resources");
  let expected_resources = get_all_resource_definitions()
    .into_iter()
    .map(|resource| resource.to_json_value())
    .collect::<Vec<_>>();
  assert_eq!(resources, expected_resources);

  let started = server.handle_request(
    r#"{"jsonrpc":"2.0","id":99,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":7,"width":20,"height":10}}}"#,
  );
  assert!(JsonValue::parse(&started).unwrap().get("result").is_some());
  let before_metrics = server.handle_request(
    r#"{"jsonrpc":"2.0","id":102,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#,
  );
  let invalid_requests = [
    r#"{"jsonrpc":"2.0","id":103,"method":"tools/list","params":null}"#,
    r#"{"jsonrpc":"2.0","id":104,"method":"tools/list","params":[]}"#,
    r#"{"jsonrpc":"2.0","id":105,"method":"tools/list","params":{"cursor":4}}"#,
    r#"{"jsonrpc":"2.0","id":106,"method":"tools/list","params":{"cursor":"resources-v1-2"}}"#,
    r#"{"jsonrpc":"2.0","id":107,"method":"tools/list","params":{"cursor":"tools-v1-6"}}"#,
    r#"{"jsonrpc":"2.0","id":108,"method":"tools/list","params":{"cursor":"tools-v1-12"}}"#,
    r#"{"jsonrpc":"2.0","id":109,"method":"resources/list","params":{"cursor":"tools-v1-4"}}"#,
    r#"{"jsonrpc":"2.0","id":110,"method":"resources/list","params":{"cursor":"resources-v1-4"}}"#,
  ];
  for request in invalid_requests {
    let response = JsonValue::parse(&server.handle_request(request)).unwrap();
    assert_eq!(
      response
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(JsonValue::as_i64),
      Some(error_codes::INVALID_PARAMS as i64)
    );
  }
  assert_eq!(
    before_metrics,
    server.handle_request(
      r#"{"jsonrpc":"2.0","id":102,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#,
    )
  );
}

#[test]
fn test_stdio_batch_list_pagination_preserves_order_and_notifications() {
  let batch = concat!(
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"batch-test","version":"1"}}},"#,
    r#"{"jsonrpc":"2.0","method":"notifications/initialized"},"#,
    r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"},"#,
    r#"{"jsonrpc":"2.0","method":"tools/list","params":{"cursor":"tools-v1-4"}},"#,
    r#"{"jsonrpc":"2.0","id":3,"method":"resources/list"}"#,
  );
  let mut server = McpServer::new();
  let mut output = Vec::new();
  server
    .run_stdio(std::io::Cursor::new(format!("[{batch}]\n")), &mut output)
    .unwrap();
  let response = JsonValue::parse(String::from_utf8(output).unwrap().trim()).unwrap();
  let responses = response.as_array().expect("batch response array");
  assert_eq!(responses.len(), 3);
  assert_eq!(
    responses
      .iter()
      .map(|response| response.get("id").and_then(JsonValue::as_u64))
      .collect::<Vec<_>>(),
    vec![Some(1), Some(2), Some(3)]
  );
  assert_eq!(
    responses[1]
      .get("result")
      .and_then(|result| result.get("nextCursor"))
      .and_then(JsonValue::as_str),
    Some("tools-v1-4")
  );
  assert_eq!(
    responses[2]
      .get("result")
      .and_then(|result| result.get("nextCursor"))
      .and_then(JsonValue::as_str),
    Some("resources-v1-2")
  );
}

#[test]
fn test_jsonrpc_verify_replay_is_deterministic_and_state_safe() {
  let mut server = ready_server();

  let inactive = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"game_verify_replay","arguments":{}}}"#,
  ))
  .unwrap();
  assert_tool_error(&inactive, error_codes::SESSION_NOT_ACTIVE);
  let inactive_save = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":16,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}"#,
  ))
  .unwrap();
  assert_tool_error(&inactive_save, error_codes::SESSION_NOT_ACTIVE);

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
fn test_jsonrpc_supplied_replay_verification_is_read_only_and_inactive_safe() {
  let mut active = ready_server();
  let _ = active.handle_request(
    r#"{"jsonrpc":"2.0","id":40,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":123,"width":20,"height":10}}}"#,
  );
  let _ = active.handle_request(
    r#"{"jsonrpc":"2.0","id":41,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"wait"}}}"#,
  );
  let saved = JsonValue::parse(&active.handle_request(
    r#"{"jsonrpc":"2.0","id":42,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}"#,
  ))
  .unwrap();
  let replay = saved
    .get("result")
    .and_then(|result| result.get("data"))
    .expect("replay export data")
    .to_compact_string();
  let supplied_request = format!(
    r#"{{"jsonrpc":"2.0","id":43,"method":"tools/call","params":{{"name":"game_verify_replay","arguments":{{"replay":{replay}}}}}}}"#
  );
  let before_metrics = active.handle_request(
    r#"{"jsonrpc":"2.0","id":44,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#,
  );
  let first = active.handle_request(&supplied_request);
  let second = active.handle_request(&supplied_request);
  assert_eq!(first, second);
  let response = JsonValue::parse(&first).unwrap();
  let data = response.get("result").and_then(|result| result.get("data"));
  assert_eq!(
    data
      .and_then(|data| data.get("deterministic"))
      .and_then(JsonValue::as_bool),
    Some(true)
  );
  assert_eq!(
    data
      .and_then(|data| data.get("command_count"))
      .and_then(JsonValue::as_u64),
    Some(1)
  );
  assert_eq!(
    before_metrics,
    active.handle_request(
      r#"{"jsonrpc":"2.0","id":44,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#,
    )
  );

  let mut inactive = ready_server();
  let inactive_supplied = JsonValue::parse(&inactive.handle_request(&supplied_request)).unwrap();
  assert!(inactive_supplied.get("result").is_some());
  let inactive_current = JsonValue::parse(&inactive.handle_request(
    r#"{"jsonrpc":"2.0","id":45,"method":"tools/call","params":{"name":"game_verify_replay","arguments":{}}}"#,
  ))
  .unwrap();
  assert_tool_error(&inactive_current, error_codes::SESSION_NOT_ACTIVE);
}

#[test]
fn test_jsonrpc_supplied_replay_rejects_malformed_input_without_mutation() {
  let mut server = ready_server();
  let _ = server.handle_request(
    r#"{"jsonrpc":"2.0","id":50,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":5,"width":20,"height":10}}}"#,
  );
  let before = server.handle_request(
    r#"{"jsonrpc":"2.0","id":51,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#,
  );
  for replay in [
    "null",
    "[]",
    "{\"format\":\"wrong\"}",
    "{\"format\":\"drl-rust-replay-v1\",\"schema_version\":1,\"version\":1,\"metadata\":{},\"player_config\":null,\"procedural_config\":null,\"seed\":1,\"width\":1,\"height\":1,\"player_start\":{\"x\":0,\"y\":0},\"initial_stairs\":null,\"initial_monsters\":[],\"initial_items\":[],\"custom_tiles\":[],\"commands\":[{\"action\":\"unknown\"}]}",
  ] {
    let request = format!(
      r#"{{"jsonrpc":"2.0","id":52,"method":"tools/call","params":{{"name":"game_verify_replay","arguments":{{"replay":{replay}}}}}}}"#
    );
    let response = JsonValue::parse(&server.handle_request(&request)).unwrap();
    assert_eq!(
      response
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(JsonValue::as_i64),
      Some(error_codes::INVALID_PARAMS as i64)
    );
  }
  let exported = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":53,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}"#,
  ))
  .unwrap();
  let mut invalid_simulation = exported
    .get("result")
    .and_then(|result| result.get("data"))
    .cloned()
    .expect("replay export data");
  invalid_simulation
    .as_object_mut()
    .unwrap()
    .insert("width".to_string(), JsonValue::from(0_u32));
  let request = format!(
    r#"{{"jsonrpc":"2.0","id":54,"method":"tools/call","params":{{"name":"game_verify_replay","arguments":{{"replay":{}}}}}}}"#,
    invalid_simulation.to_compact_string()
  );
  let response = JsonValue::parse(&server.handle_request(&request)).unwrap();
  assert_eq!(
    response
      .get("error")
      .and_then(|error| error.get("code"))
      .and_then(JsonValue::as_i64),
    Some(error_codes::INVALID_PARAMS as i64)
  );
  let mut invalid_execution = exported
    .get("result")
    .and_then(|result| result.get("data"))
    .cloned()
    .expect("replay export data");
  invalid_execution
    .as_object_mut()
    .and_then(|object| object.get_mut("commands"))
    .and_then(JsonValue::as_array_mut)
    .unwrap()
    .push(JsonValue::parse(r#"{"action":"descend"}"#).unwrap());
  let request = format!(
    r#"{{"jsonrpc":"2.0","id":57,"method":"tools/call","params":{{"name":"game_verify_replay","arguments":{{"replay":{}}}}}}}"#,
    invalid_execution.to_compact_string()
  );
  let response = JsonValue::parse(&server.handle_request(&request)).unwrap();
  assert_tool_error(&response, error_codes::INTERNAL_ERROR);
  let mut unsafe_config = exported
    .get("result")
    .and_then(|result| result.get("data"))
    .cloned()
    .expect("replay export data");
  unsafe_config
    .as_object_mut()
    .and_then(|object| object.get_mut("procedural_config"))
    .and_then(JsonValue::as_object_mut)
    .unwrap()
    .insert(
      "max_rooms".to_string(),
      JsonValue::RawNumber("4294967295".to_string()),
    );
  let request = format!(
    r#"{{"jsonrpc":"2.0","id":55,"method":"tools/call","params":{{"name":"game_verify_replay","arguments":{{"replay":{}}}}}}}"#,
    unsafe_config.to_compact_string()
  );
  let response = JsonValue::parse(&server.handle_request(&request)).unwrap();
  assert_eq!(
    response
      .get("error")
      .and_then(|error| error.get("code"))
      .and_then(JsonValue::as_i64),
    Some(error_codes::INVALID_PARAMS as i64)
  );
  assert_eq!(
    before,
    server.handle_request(
      r#"{"jsonrpc":"2.0","id":51,"method":"tools/call","params":{"name":"game_get_metrics","arguments":{}}}"#,
    )
  );
}

#[test]
fn test_jsonrpc_game_start_rejects_dimensions_outside_replay_bounds() {
  let mut server = ready_server();
  let response = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":56,"method":"tools/call","params":{"name":"game_start","arguments":{"width":513,"height":20}}}"#,
  ))
  .unwrap();
  assert_tool_error(&response, error_codes::INVALID_ACTION);
}

#[test]
fn test_jsonrpc_supplied_custom_replay_verifies_without_session() {
  let mut source = ready_server();
  let load_request = format!(
    r#"{{"jsonrpc":"2.0","id":60,"method":"tools/call","params":{{"name":"game_load_scenario","arguments":{{"ascii_map":"{}","max_turns":4}}}}}}"#,
    "#####\n#@.>#\n#####"
  );
  let _ = source.handle_request(&load_request);
  let saved = JsonValue::parse(&source.handle_request(
    r#"{"jsonrpc":"2.0","id":61,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}"#,
  ))
  .unwrap();
  let replay = saved
    .get("result")
    .and_then(|result| result.get("data"))
    .expect("custom replay export data")
    .to_compact_string();

  let mut target = ready_server();
  let request = format!(
    r#"{{"jsonrpc":"2.0","id":62,"method":"tools/call","params":{{"name":"game_verify_replay","arguments":{{"replay":{replay}}}}}}}"#
  );
  let response = JsonValue::parse(&target.handle_request(&request)).unwrap();
  let data = response.get("result").and_then(|result| result.get("data"));
  assert_eq!(
    data
      .and_then(|data| data.get("deterministic"))
      .and_then(JsonValue::as_bool),
    Some(true)
  );
  assert_eq!(
    data
      .and_then(|data| data.get("command_count"))
      .and_then(JsonValue::as_u64),
    Some(0)
  );
  assert!(!target.session().is_active());

  let mut invalid_custom = saved
    .get("result")
    .and_then(|result| result.get("data"))
    .cloned()
    .expect("custom replay export data");
  invalid_custom
    .as_object_mut()
    .and_then(|object| object.get_mut("custom_tiles"))
    .and_then(JsonValue::as_array_mut)
    .unwrap()
    .push(JsonValue::parse(r#"{"position":{"x":99,"y":99},"kind":"floor"}"#).unwrap());
  let invalid_request = format!(
    r#"{{"jsonrpc":"2.0","id":63,"method":"tools/call","params":{{"name":"game_verify_replay","arguments":{{"replay":{}}}}}}}"#,
    invalid_custom.to_compact_string()
  );
  let invalid_response = JsonValue::parse(&target.handle_request(&invalid_request)).unwrap();
  assert_eq!(
    invalid_response
      .get("error")
      .and_then(|error| error.get("code"))
      .and_then(JsonValue::as_i64),
    Some(error_codes::INVALID_PARAMS as i64)
  );
}

#[test]
fn test_jsonrpc_resources_list_and_read() {
  let mut server = ready_server();

  let res_list = collect_list_pages(&mut server, "resources/list", "resources");
  assert_eq!(res_list.len(), get_all_resource_definitions().len());

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
  assert!(text.contains("`attack_melee`"));
  assert!(text.contains("`unequip`"));
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

  // Unknown tools remain JSON-RPC dispatch errors, unlike recognized runtime failures.
  let unknown_tool = r#"{"jsonrpc":"2.0","id":98,"method":"tools/call","params":{"name":"not_a_tool","arguments":{}}}"#;
  let unknown_tool_response = JsonValue::parse(&server.handle_request(unknown_tool)).unwrap();
  assert_eq!(
    unknown_tool_response
      .get("error")
      .and_then(|error| error.get("code"))
      .and_then(JsonValue::as_i64),
    Some(error_codes::METHOD_NOT_FOUND as i64)
  );

  // 3. Calling tool before game session started
  let obs_req =
    r#"{"jsonrpc":"2.0","id":100,"method":"tools/call","params":{"name":"game_get_observation"}}"#;
  let resp3_str = server.handle_request(obs_req);
  let resp3 = JsonValue::parse(&resp3_str).unwrap();
  assert_tool_error(&resp3, error_codes::SESSION_NOT_ACTIVE);
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
