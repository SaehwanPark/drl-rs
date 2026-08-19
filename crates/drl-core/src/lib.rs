//! Deterministic headless simulation core for DRL-Rust.
//!
//! This crate contains pure game simulation logic and domain models.
//! It must remain independent of rendering, audio, OS APIs, filesystem IO,
//! and MCP transports.

/// Returns the core simulation engine name.
#[must_use]
pub fn engine_name() -> &'static str {
  "drl-core"
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_engine_name() {
    assert_eq!(engine_name(), "drl-core");
  }
}
