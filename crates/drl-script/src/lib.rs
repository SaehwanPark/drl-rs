//! Scripting and content integration boundary for DRL-Rust.
//!
//! Provides a controlled boundary for Lua content and scripts
//! without exposing direct mutable simulation state.

/// Returns the scripting layer name.
#[must_use]
pub fn script_layer_name() -> &'static str {
  "drl-script"
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_script_layer_name() {
    assert_eq!(script_layer_name(), "drl-script");
  }
}
