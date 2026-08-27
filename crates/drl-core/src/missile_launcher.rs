//! Typed Missile Launcher alternate/full-reload transition.

use drl_protocol::ActionCost;

/// Legacy alternate-reload score-count cap for the Missile Launcher.
pub const MISSILE_LAUNCHER_ALT_RELOAD_CAP: u32 = 2_500;

/// Result of planning an all-deficit Missile Launcher reload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MissileLauncherReloadPlan {
  /// Load the entire clip deficit and pay the capped reload cost.
  Load { amount: u32, cost: ActionCost },
  /// The clip has no deficit to fill.
  ClipFull,
  /// Reserve ammunition cannot satisfy the complete deficit.
  InsufficientAmmo,
}

/// Pure planner for the Missile Launcher alternate/full-reload callback.
pub(crate) struct MissileLauncherTransition;

impl MissileLauncherTransition {
  /// Plans an atomic full reload from the available loose ammunition.
  #[must_use]
  pub(crate) const fn plan(
    current_clip: u32,
    clip_capacity: u32,
    available_ammo: u32,
    reload_cost: ActionCost,
  ) -> MissileLauncherReloadPlan {
    let deficit = clip_capacity.saturating_sub(current_clip);
    if deficit == 0 {
      return MissileLauncherReloadPlan::ClipFull;
    }
    if available_ammo < deficit {
      return MissileLauncherReloadPlan::InsufficientAmmo;
    }

    let uncapped_cost = deficit.saturating_mul(reload_cost.as_u32());
    let capped_cost = if uncapped_cost > MISSILE_LAUNCHER_ALT_RELOAD_CAP {
      MISSILE_LAUNCHER_ALT_RELOAD_CAP
    } else {
      uncapped_cost
    };
    MissileLauncherReloadPlan::Load {
      amount: deficit,
      cost: ActionCost::new(capped_cost),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn plans_full_deficit_with_legacy_cost_cap() {
    assert_eq!(
      MissileLauncherTransition::plan(0, 4, 4, ActionCost::STANDARD),
      MissileLauncherReloadPlan::Load {
        amount: 4,
        cost: ActionCost::new(2_500),
      }
    );
    assert_eq!(
      MissileLauncherTransition::plan(2, 4, 2, ActionCost::STANDARD),
      MissileLauncherReloadPlan::Load {
        amount: 2,
        cost: ActionCost::new(2_000),
      }
    );
    assert_eq!(
      MissileLauncherTransition::plan(3, 4, 1, ActionCost::STANDARD),
      MissileLauncherReloadPlan::Load {
        amount: 1,
        cost: ActionCost::STANDARD,
      }
    );
  }

  #[test]
  fn rejects_full_or_under_supplied_clips_before_mutation() {
    assert_eq!(
      MissileLauncherTransition::plan(4, 4, 0, ActionCost::STANDARD),
      MissileLauncherReloadPlan::ClipFull
    );
    assert_eq!(
      MissileLauncherTransition::plan(1, 4, 2, ActionCost::STANDARD),
      MissileLauncherReloadPlan::InsufficientAmmo
    );
  }
}
