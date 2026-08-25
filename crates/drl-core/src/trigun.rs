//! Typed Trigun alternate-reload transition derived from the pinned legacy evidence.

use drl_protocol::HitPoints;

/// Current HP cost of the Trigun alternate reload.
pub const TRIGUN_HP_COST: u32 = 5;
/// Maximum-HP cost of the Trigun alternate reload.
pub const TRIGUN_MAX_HP_COST: u32 = 5;
/// Score-count cost of the Trigun alternate reload.
pub const TRIGUN_SCORE_COST: i32 = 1_000;
/// Minimum maximum HP allowed by the alternate reload.
pub const TRIGUN_MIN_MAX_HP: u32 = 10;
/// Minimum current HP left by the alternate reload itself.
pub const TRIGUN_MIN_HP: u32 = 1;
/// Legacy `being:nuke(1)` request represented by the typed nuke state.
pub const TRIGUN_NUKE_TIMER: u32 = 1;

/// Failure of the pure Trigun actor transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrigunError {
  /// The actor's maximum HP is already at the legacy floor.
  MaximumHealthTooLow,
  /// The transition cannot be applied to a dead actor.
  Dead,
}

/// Costs and resulting resources from a successful Trigun alternate reload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrigunCost {
  pub remaining_hp: HitPoints,
  pub score_count_remaining: i32,
}

/// Pure actor-owned Trigun alternate-reload transition.
pub struct TrigunTransition;

impl TrigunTransition {
  /// Applies the legacy-backed health and score costs without RNG or equipment mutation.
  pub fn apply(hp: &mut HitPoints, score_count: &mut i32) -> Result<TrigunCost, TrigunError> {
    if hp.current == 0 {
      return Err(TrigunError::Dead);
    }
    if hp.max <= TRIGUN_MIN_MAX_HP {
      return Err(TrigunError::MaximumHealthTooLow);
    }

    let remaining_max = hp
      .max
      .saturating_sub(TRIGUN_MAX_HP_COST)
      .max(TRIGUN_MIN_MAX_HP);
    let remaining_current = hp
      .current
      .saturating_sub(TRIGUN_HP_COST)
      .max(TRIGUN_MIN_HP)
      .min(remaining_max);
    *hp = HitPoints::new(remaining_current, remaining_max);
    *score_count = score_count.saturating_sub(TRIGUN_SCORE_COST);

    Ok(TrigunCost {
      remaining_hp: *hp,
      score_count_remaining: *score_count,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn applies_costs_and_clamps_health() {
    let mut hp = HitPoints::new(12, 20);
    let mut score = 2_000;

    let cost = TrigunTransition::apply(&mut hp, &mut score).unwrap();

    assert_eq!(cost.remaining_hp, HitPoints::new(7, 15));
    assert_eq!(cost.score_count_remaining, 1_000);
    assert_eq!(hp, HitPoints::new(7, 15));
    assert_eq!(score, 1_000);
  }

  #[test]
  fn clamps_to_legacy_health_floors_and_signed_score_cost() {
    let mut hp = HitPoints::new(3, 11);
    let mut score = 0;

    let cost = TrigunTransition::apply(&mut hp, &mut score).unwrap();

    assert_eq!(cost.remaining_hp, HitPoints::new(1, 10));
    assert_eq!(score, -1_000);
  }

  #[test]
  fn rejects_low_max_health_without_mutation() {
    let mut hp = HitPoints::new(10, 10);
    let mut score = 9;
    let before = (hp, score);

    assert_eq!(
      TrigunTransition::apply(&mut hp, &mut score),
      Err(TrigunError::MaximumHealthTooLow)
    );
    assert_eq!((hp, score), before);
  }
}
