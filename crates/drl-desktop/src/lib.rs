//! Thin native frontend boundary for DRL-Rust.
//!
//! The crate owns only native window/input and GPU lifecycle concerns. Gameplay
//! remains in `drl-core`; the shell sees fair observations and the shared
//! renderer-neutral scene/effect projections.

mod input;
mod renderer;
mod session;
mod window;

pub use input::command_for_key;
pub use renderer::{DesktopRenderer, RenderError, SurfaceStatus, framebuffer_extent};
pub use session::{DEMO_SCENARIO, DesktopSession, demo_scenario};
pub use window::{DesktopApp, DesktopError, run};

/// Validates the deterministic demo fixture without opening a native window.
///
/// This is useful for repository checks and keeps scenario construction
/// observable without making a display or GPU a build prerequisite.
pub fn validate_demo() -> Result<(), DesktopError> {
  let scenario = demo_scenario().map_err(DesktopError::Scenario)?;
  let session = DesktopSession::new(&scenario)
    .map_err(|error| DesktopError::Initialization(error.to_string()))?;
  if session.observation().map_width != scenario.width
    || session.observation().map_height != scenario.height
  {
    return Err(DesktopError::Initialization(
      "demo observation dimensions do not match scenario".to_string(),
    ));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn demo_validation_is_display_independent() {
    validate_demo().expect("demo scenario should instantiate without a display");
  }
}
