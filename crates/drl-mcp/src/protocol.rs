//! MCP and JSON-RPC 2.0 protocol specifications for DRL-Rust.

use crate::json::JsonValue;
use std::collections::BTreeMap;

/// Supported MCP protocol version string.
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
/// Current DRL-MCP server version.
pub const DRL_MCP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Server identifier.
pub const SERVER_NAME: &str = "drl-mcp";

/// Standard JSON-RPC 2.0 error codes.
pub mod error_codes {
  /// Invalid JSON was received by the server.
  pub const PARSE_ERROR: i32 = -32700;
  /// The JSON sent is not a valid Request object.
  pub const INVALID_REQUEST: i32 = -32600;
  /// The method does not exist / is not available.
  pub const METHOD_NOT_FOUND: i32 = -32601;
  /// Invalid method parameter(s).
  pub const INVALID_PARAMS: i32 = -32602;
  /// Internal JSON-RPC error.
  pub const INTERNAL_ERROR: i32 = -32603;

  /// Game session not started.
  pub const SESSION_NOT_ACTIVE: i32 = -32000;
  /// Invalid player action or simulation failure.
  pub const INVALID_ACTION: i32 = -32001;
  /// Permission denied (e.g. dev mode required).
  pub const PERMISSION_DENIED: i32 = -32002;
}

/// JSON-RPC 2.0 Request envelope.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonRpcRequest {
  /// Request ID (null for notifications).
  pub id: Option<JsonValue>,
  /// Method name.
  pub method: String,
  /// Request parameters.
  pub params: Option<JsonValue>,
}

impl JsonRpcRequest {
  /// Parses a JSON-RPC 2.0 request from raw JSON string.
  pub fn parse(input: &str) -> Result<Self, JsonRpcError> {
    let root = JsonValue::parse(input)
      .map_err(|e| JsonRpcError::new(error_codes::PARSE_ERROR, format!("Parse error: {e}")))?;

    let obj = root.as_object().ok_or_else(|| {
      JsonRpcError::new(
        error_codes::INVALID_REQUEST,
        "Request body must be a JSON object",
      )
    })?;

    let jsonrpc = obj
      .get("jsonrpc")
      .and_then(|v| v.as_str())
      .unwrap_or_default();
    if jsonrpc != "2.0" {
      return Err(JsonRpcError::new(
        error_codes::INVALID_REQUEST,
        "Field 'jsonrpc' must be '2.0'",
      ));
    }

    let method = obj
      .get("method")
      .and_then(|v| v.as_str())
      .ok_or_else(|| {
        JsonRpcError::new(
          error_codes::INVALID_REQUEST,
          "Missing or invalid 'method' field",
        )
      })?
      .to_string();

    let id = obj.get("id").cloned();
    let params = obj.get("params").cloned();

    Ok(Self { id, method, params })
  }
}

/// JSON-RPC 2.0 Error object.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonRpcError {
  /// Integer error code.
  pub code: i32,
  /// Short human-readable message.
  pub message: String,
  /// Optional structured error data.
  pub data: Option<JsonValue>,
}

impl JsonRpcError {
  /// Creates a new error without additional data.
  #[must_use]
  pub fn new(code: i32, message: impl Into<String>) -> Self {
    Self {
      code,
      message: message.into(),
      data: None,
    }
  }

  /// Creates a new error with additional data.
  #[must_use]
  pub fn with_data(code: i32, message: impl Into<String>, data: JsonValue) -> Self {
    Self {
      code,
      message: message.into(),
      data: Some(data),
    }
  }

  /// Converts this error into a JSON object.
  #[must_use]
  pub fn to_json_value(&self) -> JsonValue {
    let mut map = BTreeMap::new();
    map.insert("code".to_string(), JsonValue::from(self.code));
    map.insert(
      "message".to_string(),
      JsonValue::String(self.message.clone()),
    );
    if let Some(ref data) = self.data {
      map.insert("data".to_string(), data.clone());
    }
    JsonValue::Object(map)
  }
}

/// JSON-RPC 2.0 Response envelope.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonRpcResponse {
  /// Request ID matching the request.
  pub id: JsonValue,
  /// Success result value.
  pub result: Option<JsonValue>,
  /// Error object if failed.
  pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
  /// Creates a successful response.
  #[must_use]
  pub fn success(id: JsonValue, result: JsonValue) -> Self {
    Self {
      id,
      result: Some(result),
      error: None,
    }
  }

  /// Creates an error response.
  #[must_use]
  pub fn error(id: JsonValue, error: JsonRpcError) -> Self {
    Self {
      id,
      result: None,
      error: Some(error),
    }
  }

  /// Converts this response to a JSON value.
  #[must_use]
  pub fn to_json_value(&self) -> JsonValue {
    let mut map = BTreeMap::new();
    map.insert("jsonrpc".to_string(), JsonValue::from("2.0"));
    map.insert("id".to_string(), self.id.clone());
    if let Some(ref res) = self.result {
      map.insert("result".to_string(), res.clone());
    }
    if let Some(ref err) = self.error {
      map.insert("error".to_string(), err.to_json_value());
    }
    JsonValue::Object(map)
  }

  /// Converts this response into a compact JSON string.
  #[must_use]
  pub fn to_json_string(&self) -> String {
    self.to_json_value().to_compact_string()
  }
}

/// Description of an MCP Tool.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolDefinition {
  /// Unique tool name.
  pub name: String,
  /// Description of the tool's behavior and utility.
  pub description: String,
  /// JSON schema describing the expected input arguments.
  pub input_schema: JsonValue,
}

impl ToolDefinition {
  /// Converts this definition into a JSON object matching MCP tool spec.
  #[must_use]
  pub fn to_json_value(&self) -> JsonValue {
    let mut map = BTreeMap::new();
    map.insert("name".to_string(), JsonValue::String(self.name.clone()));
    map.insert(
      "description".to_string(),
      JsonValue::String(self.description.clone()),
    );
    map.insert("inputSchema".to_string(), self.input_schema.clone());
    JsonValue::Object(map)
  }
}

/// Description of an MCP Resource.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceDefinition {
  /// Resource URI (e.g. `drl://rules/game`).
  pub uri: String,
  /// Short human-readable name.
  pub name: String,
  /// Description of the resource.
  pub description: String,
  /// MIME type (usually `application/json` or `text/markdown`).
  pub mime_type: String,
}

impl ResourceDefinition {
  /// Converts this definition into a JSON object matching MCP resource spec.
  #[must_use]
  pub fn to_json_value(&self) -> JsonValue {
    let mut map = BTreeMap::new();
    map.insert("uri".to_string(), JsonValue::String(self.uri.clone()));
    map.insert("name".to_string(), JsonValue::String(self.name.clone()));
    map.insert(
      "description".to_string(),
      JsonValue::String(self.description.clone()),
    );
    map.insert(
      "mimeType".to_string(),
      JsonValue::String(self.mime_type.clone()),
    );
    JsonValue::Object(map)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_valid_jsonrpc_request() {
    let raw = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"game_start","arguments":{"seed":42}}}"#;
    let req = JsonRpcRequest::parse(raw).unwrap();
    assert_eq!(req.id, Some(JsonValue::Number(1.0)));
    assert_eq!(req.method, "tools/call");
    assert!(req.params.is_some());
  }

  #[test]
  fn test_jsonrpc_response_serialization() {
    let mut res_obj = BTreeMap::new();
    res_obj.insert("status".to_string(), JsonValue::from("ok"));
    let res = JsonRpcResponse::success(JsonValue::Number(1.0), JsonValue::Object(res_obj));
    let json_str = res.to_json_string();
    assert!(json_str.contains("\"jsonrpc\":\"2.0\""));
    assert!(json_str.contains("\"id\":1"));
    assert!(json_str.contains("\"status\":\"ok\""));
  }
}
