//! Model Context Protocol interface and tools for DRL-Rust.
//!
//! Exposes semantic observations and actions to AI test agents.

pub mod json;
pub mod protocol;
pub mod resources;
pub mod server;
pub mod session;
pub mod tools;

pub use json::JsonValue;
pub use protocol::{
  DRL_MCP_VERSION, JsonRpcError, JsonRpcRequest, JsonRpcResponse, MCP_PROTOCOL_VERSION,
  ResourceDefinition, SERVER_NAME, ToolDefinition, error_codes,
};
pub use resources::{get_all_resource_definitions, read_resource};
pub use server::McpServer;
pub use session::{LegalAction, McpSession, SessionConfig, compute_legal_actions, json_to_command};
pub use tools::{execute_tool, get_all_tool_definitions};

/// Returns the MCP server component name.
#[must_use]
pub fn mcp_name() -> &'static str {
  SERVER_NAME
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_mcp_name() {
    assert_eq!(mcp_name(), "drl-mcp");
  }
}
