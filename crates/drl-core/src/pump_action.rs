//! Typed pump-action chamber transitions for legacy shotgun behavior.

use drl_protocol::ActionCost;

/// Cost of cycling a pump-action weapon whose chamber is empty.
pub const PUMP_ACTION_COST: ActionCost = ActionCost::new(200);

/// Chamber state owned by a pump-action weapon instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PumpActionState {
  /// A round is ready to fire.
  Chambered,
  /// The chamber is empty after firing.
  Empty,
}

/// Reload behavior selected by a pump-action chamber transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReloadTransition {
  /// Cycle the action without consuming reserve ammunition.
  PumpOnly,
  /// Perform the ordinary ammunition reload path.
  Regular,
}

impl PumpActionState {
  /// Creates the chambered state used for a newly spawned Combat Shotgun.
  #[must_use]
  pub const fn new() -> Self {
    Self::Chambered
  }

  /// Returns whether firing is blocked while reserve clip ammunition remains.
  #[must_use]
  pub const fn blocks_fire(self, current_clip: u32) -> bool {
    matches!(self, Self::Empty) && current_clip > 0
  }

  /// Marks the chamber empty after a resolved shot.
  #[must_use]
  pub const fn after_fire(self) -> Self {
    Self::Empty
  }

  /// Chambers a round after an accepted walk when clip ammunition remains.
  #[must_use]
  pub const fn after_accepted_move(self, current_clip: u32) -> Self {
    if matches!(self, Self::Empty) && current_clip > 0 {
      Self::Chambered
    } else {
      self
    }
  }

  /// Selects pump-only reload when the empty chamber still has clip ammo.
  #[must_use]
  pub const fn reload_transition(self, current_clip: u32) -> ReloadTransition {
    if matches!(self, Self::Empty) && current_clip > 0 {
      ReloadTransition::PumpOnly
    } else {
      ReloadTransition::Regular
    }
  }

  /// Completes a pump-only action without changing clip ammunition.
  #[must_use]
  pub const fn after_pump(self) -> Self {
    Self::Chambered
  }

  /// Completes a regular reload that loaded at least one round.
  #[must_use]
  pub const fn after_regular_reload(self, loaded: u32) -> Self {
    if loaded > 0 { Self::Chambered } else { self }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn transitions_match_pump_action_contract() {
    let chambered = PumpActionState::new();
    assert!(!chambered.blocks_fire(4));
    assert_eq!(chambered.after_fire(), PumpActionState::Empty);

    let empty = PumpActionState::Empty;
    assert!(empty.blocks_fire(4));
    assert!(!empty.blocks_fire(0));
    assert_eq!(empty.reload_transition(4), ReloadTransition::PumpOnly);
    assert_eq!(empty.reload_transition(0), ReloadTransition::Regular);
    assert_eq!(empty.after_accepted_move(4), PumpActionState::Chambered);
    assert_eq!(empty.after_accepted_move(0), PumpActionState::Empty);
    assert_eq!(empty.after_pump(), PumpActionState::Chambered);
    assert_eq!(empty.after_regular_reload(1), PumpActionState::Chambered);
    assert_eq!(empty.after_regular_reload(0), PumpActionState::Empty);
    assert_eq!(PUMP_ACTION_COST, ActionCost::new(200));
  }
}
