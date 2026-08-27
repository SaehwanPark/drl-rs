//! Typed Combat Shotgun alternate/full-reload transition.

use drl_protocol::ActionCost;

/// Legacy alternate-reload score-count cap for the Combat Shotgun.
pub const COMBAT_SHOTGUN_ALT_RELOAD_CAP: u32 = 2_500;

/// Result of planning an all-deficit Combat Shotgun reload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CombatShotgunReloadPlan {
  /// Load the entire clip deficit and pay the capped reload cost.
  Load { amount: u32, cost: ActionCost },
  /// The clip has no deficit to fill.
  ClipFull,
  /// Reserve ammunition cannot satisfy the complete deficit.
  InsufficientAmmo,
}

/// Pure planner for the Combat Shotgun alternate/full-reload callback.
pub(crate) struct CombatShotgunTransition;

impl CombatShotgunTransition {
  /// Plans an atomic full reload from the available loose ammunition.
  #[must_use]
  pub(crate) const fn plan(
    current_clip: u32,
    clip_capacity: u32,
    available_ammo: u32,
    reload_cost: ActionCost,
  ) -> CombatShotgunReloadPlan {
    let deficit = clip_capacity.saturating_sub(current_clip);
    if deficit == 0 {
      return CombatShotgunReloadPlan::ClipFull;
    }
    if available_ammo < deficit {
      return CombatShotgunReloadPlan::InsufficientAmmo;
    }

    let uncapped_cost = deficit.saturating_mul(reload_cost.as_u32());
    let capped_cost = if uncapped_cost > COMBAT_SHOTGUN_ALT_RELOAD_CAP {
      COMBAT_SHOTGUN_ALT_RELOAD_CAP
    } else {
      uncapped_cost
    };
    CombatShotgunReloadPlan::Load {
      amount: deficit,
      cost: ActionCost::new(capped_cost),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn plans_deficits_with_legacy_cost_cap() {
    assert_eq!(
      CombatShotgunTransition::plan(4, 5, 1, ActionCost::STANDARD),
      CombatShotgunReloadPlan::Load {
        amount: 1,
        cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      CombatShotgunTransition::plan(3, 5, 2, ActionCost::STANDARD),
      CombatShotgunReloadPlan::Load {
        amount: 2,
        cost: ActionCost::new(2_000),
      }
    );
    assert_eq!(
      CombatShotgunTransition::plan(2, 5, 3, ActionCost::STANDARD),
      CombatShotgunReloadPlan::Load {
        amount: 3,
        cost: ActionCost::new(2_500),
      }
    );
    assert_eq!(
      CombatShotgunTransition::plan(0, 5, 5, ActionCost::STANDARD),
      CombatShotgunReloadPlan::Load {
        amount: 5,
        cost: ActionCost::new(2_500),
      }
    );
  }

  #[test]
  fn rejects_full_or_under_supplied_clips_before_mutation() {
    assert_eq!(
      CombatShotgunTransition::plan(5, 5, 0, ActionCost::STANDARD),
      CombatShotgunReloadPlan::ClipFull
    );
    assert_eq!(
      CombatShotgunTransition::plan(2, 5, 2, ActionCost::STANDARD),
      CombatShotgunReloadPlan::InsufficientAmmo
    );
  }
}
