//! MCP Server implementation and JSON-RPC dispatch loop.

use crate::json::JsonValue;
use crate::protocol::{
  DRL_MCP_VERSION, JsonRpcError, JsonRpcRequest, JsonRpcResponse, MCP_PROTOCOL_VERSION,
  SERVER_NAME, ToolDefinition, error_codes,
};
use crate::resources::{get_all_resource_definitions, read_resource};
use crate::session::McpSession;
use crate::tools::{execute_tool, get_all_tool_definitions};
use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

/// Server handling MCP JSON-RPC protocol requests.
#[derive(Debug)]
pub struct McpServer {
  session: McpSession,
}

impl Default for McpServer {
  fn default() -> Self {
    Self::new()
  }
}

impl McpServer {
  /// Creates a new MCP server with a fresh session.
  #[must_use]
  pub fn new() -> Self {
    Self {
      session: McpSession::new(),
    }
  }

  /// Mutable reference to the underlying game session.
  pub fn session_mut(&mut self) -> &mut McpSession {
    &mut self.session
  }

  /// Immutable reference to the underlying game session.
  #[must_use]
  pub const fn session(&self) -> &McpSession {
    &self.session
  }

  /// Dispatches a raw JSON-RPC request string and returns the response string.
  #[must_use]
  pub fn handle_request(&mut self, input: &str) -> String {
    let req = match JsonRpcRequest::parse(input) {
      Ok(r) => r,
      Err(err) => {
        let resp = JsonRpcResponse::error(JsonValue::Null, err);
        return resp.to_json_string();
      }
    };

    let id = req.id.clone().unwrap_or(JsonValue::Null);

    let res = match req.method.as_str() {
      "initialize" => self.handle_initialize(req.params.as_ref()),
      "notifications/initialized" | "initialized" => Ok(JsonValue::Object(BTreeMap::new())),
      "ping" => Ok(JsonValue::Object(BTreeMap::new())),
      "tools/list" => self.handle_tools_list(),
      "tools/call" => self.handle_tools_call(req.params.as_ref()),
      "resources/list" => self.handle_resources_list(),
      "resources/read" => self.handle_resources_read(req.params.as_ref()),
      other => Err(JsonRpcError::new(
        error_codes::METHOD_NOT_FOUND,
        format!("Method not found: '{other}'"),
      )),
    };

    let response = match res {
      Ok(val) => JsonRpcResponse::success(id, val),
      Err(err) => JsonRpcResponse::error(id, err),
    };

    response.to_json_string()
  }

  fn handle_initialize(&self, _params: Option<&JsonValue>) -> Result<JsonValue, JsonRpcError> {
    let mut map = BTreeMap::new();
    map.insert(
      "protocolVersion".to_string(),
      JsonValue::from(MCP_PROTOCOL_VERSION),
    );

    let mut caps = BTreeMap::new();
    caps.insert("tools".to_string(), JsonValue::Object(BTreeMap::new()));
    caps.insert("resources".to_string(), JsonValue::Object(BTreeMap::new()));
    map.insert("capabilities".to_string(), JsonValue::Object(caps));

    let mut info = BTreeMap::new();
    info.insert("name".to_string(), JsonValue::from(SERVER_NAME));
    info.insert("version".to_string(), JsonValue::from(DRL_MCP_VERSION));
    map.insert("serverInfo".to_string(), JsonValue::Object(info));

    Ok(JsonValue::Object(map))
  }

  fn handle_tools_list(&self) -> Result<JsonValue, JsonRpcError> {
    let tools = get_all_tool_definitions();
    let tools_json: Vec<JsonValue> = tools.iter().map(ToolDefinition::to_json_value).collect();

    let mut map = BTreeMap::new();
    map.insert("tools".to_string(), JsonValue::Array(tools_json));
    Ok(JsonValue::Object(map))
  }

  fn handle_tools_call(&mut self, params: Option<&JsonValue>) -> Result<JsonValue, JsonRpcError> {
    let p = params.ok_or_else(|| {
      JsonRpcError::new(
        error_codes::INVALID_PARAMS,
        "Missing parameters for 'tools/call'",
      )
    })?;

    let name = p.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
      JsonRpcError::new(
        error_codes::INVALID_PARAMS,
        "Missing 'name' field in 'tools/call' params",
      )
    })?;

    let default_args = JsonValue::Object(BTreeMap::new());
    let args = p.get("arguments").unwrap_or(&default_args);

    execute_tool(&mut self.session, name, args)
  }

  fn handle_resources_list(&self) -> Result<JsonValue, JsonRpcError> {
    let resources = get_all_resource_definitions();
    let res_json: Vec<JsonValue> = resources
      .iter()
      .map(crate::protocol::ResourceDefinition::to_json_value)
      .collect();

    let mut map = BTreeMap::new();
    map.insert("resources".to_string(), JsonValue::Array(res_json));
    Ok(JsonValue::Object(map))
  }

  fn handle_resources_read(&self, params: Option<&JsonValue>) -> Result<JsonValue, JsonRpcError> {
    let p = params.ok_or_else(|| {
      JsonRpcError::new(
        error_codes::INVALID_PARAMS,
        "Missing parameters for 'resources/read'",
      )
    })?;

    let uri = p.get("uri").and_then(|v| v.as_str()).ok_or_else(|| {
      JsonRpcError::new(
        error_codes::INVALID_PARAMS,
        "Missing 'uri' field in 'resources/read' params",
      )
    })?;

    read_resource(&self.session, uri)
  }

  /// Runs the MCP JSON-RPC server over stdio streams until EOF.
  pub fn run_stdio(&mut self, mut reader: impl BufRead, mut writer: impl Write) -> io::Result<()> {
    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
      let trimmed = line.trim();
      if !trimmed.is_empty() {
        let resp = self.handle_request(trimmed);
        writer.write_all(resp.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
      }
      line.clear();
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_server_initialize_and_ping() {
    let mut server = McpServer::new();
    let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let resp = server.handle_request(init_req);
    let val = JsonValue::parse(&resp).unwrap();
    assert_eq!(val.get("id").unwrap().as_u64().unwrap(), 1);
    assert_eq!(
      val
        .get("result")
        .unwrap()
        .get("serverInfo")
        .unwrap()
        .get("name")
        .unwrap()
        .as_str()
        .unwrap(),
      "drl-mcp"
    );

    let ping_req = r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#;
    let ping_resp = server.handle_request(ping_req);
    let ping_val = JsonValue::parse(&ping_resp).unwrap();
    assert!(ping_val.get("result").is_some());
  }

  #[test]
  fn test_server_tools_list_and_call() {
    let mut server = McpServer::new();
    let list_req = r#"{"jsonrpc":"2.0","id":10,"method":"tools/list"}"#;
    let list_resp = server.handle_request(list_req);
    let list_val = JsonValue::parse(&list_resp).unwrap();
    let tools = list_val
      .get("result")
      .unwrap()
      .get("tools")
      .unwrap()
      .as_array()
      .unwrap();
    assert!(!tools.is_empty());

    let start_req = r#"{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":99}}}"#;
    let start_resp = server.handle_request(start_req);
    let start_val = JsonValue::parse(&start_resp).unwrap();
    assert!(start_val.get("result").is_some());
  }
}
