//! Typed preflight for nuclear-weapon alternate overload actions.

use crate::grid::Tile;
use crate::nuke::NukeState;

/// Countdown used by a nuclear overload on a hazard tile.
pub const NUCLEAR_OVERLOAD_HAZARD_COUNTDOWN: u32 = 1;
/// Countdown used by a nuclear overload on an ordinary floor tile.
pub const NUCLEAR_OVERLOAD_FLOOR_COUNTDOWN: u32 = 100;
/// Score-count cost of a nuclear overload.
pub const NUCLEAR_OVERLOAD_SCORE_COST: i32 = 1_000;

/// Pure rejection reasons for nuclear alternate overload preflight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuclearOverloadError {
  /// Nuclear overload cannot be armed while standing on stairs.
  Stairs,
  /// The destructive action requires explicit confirmation.
  NotConfirmed,
  /// The weapon must have a full clip before overload arming.
  ClipNotFull,
  /// A nuke is already pending or has already resolved.
  NukeUnavailable,
}

/// Result of a successful nuclear overload preflight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NuclearOverloadPlan {
  countdown: u32,
}

impl NuclearOverloadPlan {
  /// Returns the countdown to arm in the existing typed nuke state.
  #[must_use]
  pub const fn countdown(self) -> u32 {
    self.countdown
  }
}

/// Validates a nuclear alternate overload without mutating game state.
pub fn plan(
  current_clip: u32,
  clip_capacity: u32,
  confirmed: bool,
  tile: Tile,
  nuke_state: NukeState,
) -> Result<NuclearOverloadPlan, NuclearOverloadError> {
  if tile == Tile::StairsDown {
    return Err(NuclearOverloadError::Stairs);
  }
  if !confirmed {
    return Err(NuclearOverloadError::NotConfirmed);
  }
  if current_clip < clip_capacity {
    return Err(NuclearOverloadError::ClipNotFull);
  }
  if nuke_state.countdown().is_some() || nuke_state.level_nuked() {
    return Err(NuclearOverloadError::NukeUnavailable);
  }

  let countdown = match tile {
    Tile::Acid | Tile::Lava => NUCLEAR_OVERLOAD_HAZARD_COUNTDOWN,
    _ => NUCLEAR_OVERLOAD_FLOOR_COUNTDOWN,
  };
  Ok(NuclearOverloadPlan { countdown })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn full_confirmed_floor_arms_long_countdown() {
    let plan = plan(24, 24, true, Tile::Floor, NukeState::new()).unwrap();
    assert_eq!(plan.countdown(), NUCLEAR_OVERLOAD_FLOOR_COUNTDOWN);
  }

  #[test]
  fn acid_and_lava_arm_immediate_countdown() {
    for tile in [Tile::Acid, Tile::Lava] {
      let plan = plan(24, 24, true, tile, NukeState::new()).unwrap();
      assert_eq!(plan.countdown(), NUCLEAR_OVERLOAD_HAZARD_COUNTDOWN);
    }
  }

  #[test]
  fn preflight_rejects_each_unsafe_input() {
    assert_eq!(
      plan(24, 24, true, Tile::StairsDown, NukeState::new()),
      Err(NuclearOverloadError::Stairs)
    );
    assert_eq!(
      plan(24, 24, false, Tile::Floor, NukeState::new()),
      Err(NuclearOverloadError::NotConfirmed)
    );
    assert_eq!(
      plan(23, 24, true, Tile::Floor, NukeState::new()),
      Err(NuclearOverloadError::ClipNotFull)
    );

    let mut nuke = NukeState::new();
    nuke.activate(2).unwrap();
    assert_eq!(
      plan(24, 24, true, Tile::Floor, nuke),
      Err(NuclearOverloadError::NukeUnavailable)
    );
  }
}
