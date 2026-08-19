//! Shared semantic protocol contracts for DRL-Rust.
//!
//! Defines command, observation, and event schemas shared across
//! frontends, test agents, MCP, and the simulation core.

/// Returns the protocol schema version for DRL-Rust.
#[must_use]
pub fn protocol_version() -> &'static str {
  "0.1.0"
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_protocol_version() {
    assert_eq!(protocol_version(), "0.1.0");
  }
}
