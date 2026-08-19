//! Model Context Protocol interface and tools for DRL-Rust.
//!
//! Exposes semantic observations and actions to AI test agents.

/// Returns the MCP server component name.
#[must_use]
pub fn mcp_name() -> &'static str {
  "drl-mcp"
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_mcp_name() {
    assert_eq!(mcp_name(), "drl-mcp");
  }
}
