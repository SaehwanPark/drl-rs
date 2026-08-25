//! Typed Subtle Knife alternate-invoke transition.

use drl_protocol::HitPoints;

/// HP paid by a successful Subtle Knife invoke.
pub const SUBTLE_KNIFE_HP_COST: u32 = 5;
/// Score-count paid by a successful Subtle Knife invoke.
pub const SUBTLE_KNIFE_SCORE_COST: u32 = 1_000;
/// Fixed internal damage applied to each selected target.
pub const SUBTLE_KNIFE_TARGET_DAMAGE: u32 = 15;

/// Explicit actor status state used by the Subtle Knife lockout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TiredStatus {
  /// The actor may invoke the knife.
  #[default]
  Ready,
  /// The actor has already invoked the knife.
  Tired,
}

/// Pure actor-cost transition for Subtle Knife invoke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubtleKnifeTransition;

/// Result of a successful actor-cost transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubtleKnifeCost {
  /// Remaining HP after the clamped cost.
  pub remaining_hp: u32,
  /// Remaining score count after the saturating cost.
  pub score_count_remaining: i32,
}

/// Typed rejection reason before any gameplay state is mutated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtleKnifeError {
  /// The actor already has the one-use-per-perk tired condition.
  Tired,
}

impl SubtleKnifeTransition {
  /// Applies the actor-side invoke cost and status atomically.
  pub fn apply(
    hp: &mut HitPoints,
    tired: &mut TiredStatus,
    score_count: &mut i32,
  ) -> Result<SubtleKnifeCost, SubtleKnifeError> {
    if *tired == TiredStatus::Tired {
      return Err(SubtleKnifeError::Tired);
    }

    hp.current = hp.current.saturating_sub(SUBTLE_KNIFE_HP_COST).max(1);
    *tired = TiredStatus::Tired;
    *score_count = score_count.saturating_sub(SUBTLE_KNIFE_SCORE_COST as i32);
    Ok(SubtleKnifeCost {
      remaining_hp: hp.current,
      score_count_remaining: *score_count,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn invoke_pays_cost_and_sets_tired() {
    let mut hp = HitPoints::new(20, 50);
    let mut tired = TiredStatus::Ready;
    let mut score_count = 2_000;

    assert_eq!(
      SubtleKnifeTransition::apply(&mut hp, &mut tired, &mut score_count),
      Ok(SubtleKnifeCost {
        remaining_hp: 15,
        score_count_remaining: 1_000,
      })
    );
    assert_eq!(tired, TiredStatus::Tired);
  }

  #[test]
  fn invoke_clamps_hp_to_one() {
    let mut hp = HitPoints::new(3, 50);
    let mut tired = TiredStatus::Ready;
    let mut score_count = 0;

    let result = SubtleKnifeTransition::apply(&mut hp, &mut tired, &mut score_count).unwrap();
    assert_eq!(result.remaining_hp, 1);
    assert_eq!(hp.current, 1);
  }

  #[test]
  fn tired_invocation_is_non_mutating() {
    let mut hp = HitPoints::new(20, 50);
    let mut tired = TiredStatus::Tired;
    let mut score_count = 2_000;
    let before = (hp, tired, score_count);

    assert_eq!(
      SubtleKnifeTransition::apply(&mut hp, &mut tired, &mut score_count),
      Err(SubtleKnifeError::Tired)
    );
    assert_eq!((hp, tired, score_count), before);
  }
}
