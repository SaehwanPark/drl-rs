//! Typed target-dependent behavior for Charch's Null Pointer.

/// Boss targets lose 1000 score count, matching the pinned callback branch.
pub const NULL_POINTER_BOSS_SCORE_COST: i32 = 1000;
/// Ordinary targets lose 2000 score count, matching the pinned callback branch.
pub const NULL_POINTER_TARGET_SCORE_COST: i32 = 2000;
/// The callback's minimum score-count floor.
pub const NULL_POINTER_MIN_SCORE_COUNT: i32 = 1000;
/// Evidence-backed delayed explosion interval.
pub const NULL_POINTER_EXPLOSION_DELAY: u32 = 50;
/// Evidence-backed delayed explosion radius.
pub const NULL_POINTER_EXPLOSION_RADIUS: u32 = 1;
/// Evidence-backed fixed `10d1` damage result for the deferred explosion.
pub const NULL_POINTER_EXPLOSION_DAMAGE: u32 = 10;

/// Pure score-count transition selected by the target's explicit boss property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NullPointerHitTransition;

impl NullPointerHitTransition {
  /// Applies the target score branch and returns the resulting balance.
  pub fn apply(score_count: &mut i32, target_is_boss: bool) -> i32 {
    let cost = if target_is_boss {
      NULL_POINTER_BOSS_SCORE_COST
    } else {
      NULL_POINTER_TARGET_SCORE_COST
    };
    *score_count = score_count
      .saturating_sub(cost)
      .max(NULL_POINTER_MIN_SCORE_COUNT);
    *score_count
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn boss_and_ordinary_score_branches_preserve_floor() {
    let mut boss_score = 3500;
    assert_eq!(NullPointerHitTransition::apply(&mut boss_score, true), 2500);

    let mut target_score = 3500;
    assert_eq!(
      NullPointerHitTransition::apply(&mut target_score, false),
      1500
    );

    let mut low_score = 100;
    assert_eq!(NullPointerHitTransition::apply(&mut low_score, false), 1000);
  }
}
