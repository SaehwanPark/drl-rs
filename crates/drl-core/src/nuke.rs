//! Deterministic scheduled level-nuke state used by Trigun's alternate reload.

/// Typed countdown and terminal state for a level nuke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NukeState {
  countdown: Option<u32>,
  level_nuked: bool,
}

impl NukeState {
  /// Creates an inactive nuke state.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      countdown: None,
      level_nuked: false,
    }
  }

  /// Returns the pending countdown, if any.
  #[must_use]
  pub const fn countdown(self) -> Option<u32> {
    self.countdown
  }

  /// Returns true after the level nuke has resolved.
  #[must_use]
  pub const fn level_nuked(self) -> bool {
    self.level_nuked
  }

  /// Schedules a nuke with an explicit positive countdown.
  pub fn activate(&mut self, countdown: u32) -> Result<(), NukeError> {
    if countdown == 0 || self.countdown.is_some() || self.level_nuked {
      return Err(NukeError::InvalidActivation);
    }
    self.countdown = Some(countdown);
    Ok(())
  }

  /// Advances one accepted turn boundary and reports whether the nuke resolved.
  pub fn tick(&mut self) -> bool {
    let Some(countdown) = self.countdown else {
      return false;
    };
    if countdown <= 1 {
      self.countdown = None;
      self.level_nuked = true;
      true
    } else {
      self.countdown = Some(countdown - 1);
      false
    }
  }
}

impl Default for NukeState {
  fn default() -> Self {
    Self::new()
  }
}

/// Invalid nuke scheduling attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NukeError {
  InvalidActivation,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn countdown_resolves_once() {
    let mut state = NukeState::new();
    state.activate(2).unwrap();
    assert_eq!(state.countdown(), Some(2));
    assert!(!state.tick());
    assert_eq!(state.countdown(), Some(1));
    assert!(state.tick());
    assert!(state.level_nuked());
    assert_eq!(state.countdown(), None);
    assert!(!state.tick());
  }
}
