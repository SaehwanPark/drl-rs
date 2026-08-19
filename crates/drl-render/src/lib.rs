//! Presentation and GPU rendering layer for DRL-Rust.
//!
//! Renders simulation observations and events to display targets.

/// Returns the renderer component name.
#[must_use]
pub fn renderer_name() -> &'static str {
  "drl-render"
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_renderer_name() {
    assert_eq!(renderer_name(), "drl-render");
  }
}
