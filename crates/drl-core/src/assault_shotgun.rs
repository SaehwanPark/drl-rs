//! Typed Assault Shotgun alternate/full-reload transition.

use drl_protocol::ActionCost;

/// Legacy alternate-reload score-count cap for the Assault Shotgun.
pub const ASSAULT_SHOTGUN_ALT_RELOAD_CAP: u32 = 2_500;

/// Result of planning an all-deficit Assault Shotgun reload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AssaultShotgunReloadPlan {
  /// Load the entire clip deficit and pay the capped reload cost.
  Load { amount: u32, cost: ActionCost },
  /// The clip has no deficit to fill.
  ClipFull,
  /// Reserve ammunition cannot satisfy the complete deficit.
  InsufficientAmmo,
}

/// Pure planner for the Assault Shotgun alternate/full-reload callback.
pub(crate) struct AssaultShotgunTransition;

impl AssaultShotgunTransition {
  /// Plans an atomic full reload from the available loose ammunition.
  #[must_use]
  pub(crate) const fn plan(
    current_clip: u32,
    clip_capacity: u32,
    available_ammo: u32,
    reload_cost: ActionCost,
  ) -> AssaultShotgunReloadPlan {
    let deficit = clip_capacity.saturating_sub(current_clip);
    if deficit == 0 {
      return AssaultShotgunReloadPlan::ClipFull;
    }
    if available_ammo < deficit {
      return AssaultShotgunReloadPlan::InsufficientAmmo;
    }

    let uncapped_cost = deficit.saturating_mul(reload_cost.as_u32());
    let capped_cost = if uncapped_cost > ASSAULT_SHOTGUN_ALT_RELOAD_CAP {
      ASSAULT_SHOTGUN_ALT_RELOAD_CAP
    } else {
      uncapped_cost
    };
    AssaultShotgunReloadPlan::Load {
      amount: deficit,
      cost: ActionCost::new(capped_cost),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn plans_full_deficit_with_capped_cost() {
    assert_eq!(
      AssaultShotgunTransition::plan(0, 6, 6, ActionCost::STANDARD),
      AssaultShotgunReloadPlan::Load {
        amount: 6,
        cost: ActionCost::new(2_500),
      }
    );
    assert_eq!(
      AssaultShotgunTransition::plan(3, 6, 3, ActionCost::STANDARD),
      AssaultShotgunReloadPlan::Load {
        amount: 3,
        cost: ActionCost::new(2_500),
      }
    );
    assert_eq!(
      AssaultShotgunTransition::plan(5, 6, 1, ActionCost::STANDARD),
      AssaultShotgunReloadPlan::Load {
        amount: 1,
        cost: ActionCost::STANDARD,
      }
    );
  }

  #[test]
  fn rejects_full_or_under_supplied_clips_before_mutation() {
    assert_eq!(
      AssaultShotgunTransition::plan(6, 6, 0, ActionCost::STANDARD),
      AssaultShotgunReloadPlan::ClipFull
    );
    assert_eq!(
      AssaultShotgunTransition::plan(2, 6, 3, ActionCost::STANDARD),
      AssaultShotgunReloadPlan::InsufficientAmmo
    );
  }
}
