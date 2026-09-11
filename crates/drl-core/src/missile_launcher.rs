//! Typed Missile Launcher reload and radius-three explosion policies.

use drl_protocol::{ActionCost, Position};

use crate::{explosion::radius_blast_positions, grid::Map, rng::GameRng};

/// Legacy alternate-reload score-count cap for the Missile Launcher.
pub const MISSILE_LAUNCHER_ALT_RELOAD_CAP: u32 = 2_500;
/// Number of damage dice rolled for each Missile Launcher blast cell.
pub const MISSILE_LAUNCHER_EXPLOSION_DAMAGE_DICE: u32 = 6;
/// Sides on each Missile Launcher explosion damage die.
pub const MISSILE_LAUNCHER_EXPLOSION_DAMAGE_DIE_SIDES: u32 = 6;
/// Legacy Missile Launcher explosion delay retained as event metadata.
pub const MISSILE_LAUNCHER_EXPLOSION_DELAY: u32 = 40;
/// Bounded Missile Launcher explosion radius.
pub const MISSILE_LAUNCHER_EXPLOSION_RADIUS: u32 = 3;
/// Legacy default explosion knockback strength.
pub const MISSILE_LAUNCHER_EXPLOSION_KNOCKBACK: u32 = 8;
/// Legacy threshold above which a blast destroys a ground item.
pub const MISSILE_LAUNCHER_GROUND_ITEM_DESTRUCTION_THRESHOLD: u32 = 10;

/// Returns the bounded radius-three blast cells in deterministic order.
#[must_use]
pub fn radius_three_blast_positions(map: &Map, center: Position) -> Vec<Position> {
  radius_blast_positions(map, center, MISSILE_LAUNCHER_EXPLOSION_RADIUS)
}

/// Rolls one explicit `6d6` Missile Launcher explosion result.
pub fn roll_explosion_damage(rng: &mut GameRng) -> u32 {
  (0..MISSILE_LAUNCHER_EXPLOSION_DAMAGE_DICE)
    .map(|_| rng.gen_range(1..MISSILE_LAUNCHER_EXPLOSION_DAMAGE_DIE_SIDES + 1))
    .sum()
}

/// Applies the bounded integer distance falloff used by typed explosive
/// profiles.
pub use crate::rocket_launcher::apply_distance_falloff;

/// Returns whether a post-falloff blast result destroys a representable item.
#[must_use]
pub const fn should_destroy_ground_item(damage: u32) -> bool {
  damage > MISSILE_LAUNCHER_GROUND_ITEM_DESTRUCTION_THRESHOLD
}

/// Converts a rolled explosion result to the pinned integer knockback distance.
#[must_use]
pub const fn knockback_distance(damage: u32) -> u32 {
  damage / MISSILE_LAUNCHER_EXPLOSION_KNOCKBACK
}

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
  fn radius_three_geometry_is_center_then_clockwise_rings() {
    let map = Map::simple_arena(9, 9);
    let positions = radius_three_blast_positions(&map, Position::new(4, 4));
    assert_eq!(positions.len(), 49);
    assert_eq!(positions[0], Position::new(4, 4));
    assert_eq!(positions[1], Position::new(4, 3));
    assert_eq!(positions[8], Position::new(3, 3));
    assert_eq!(positions[9], Position::new(4, 2));
    assert_eq!(positions[24], Position::new(3, 2));
    assert_eq!(positions[25], Position::new(4, 1));
    assert_eq!(positions[48], Position::new(3, 1));
  }

  #[test]
  fn radius_three_geometry_clamps_edges_and_blocks_rays() {
    let mut map = Map::simple_arena(7, 7);
    map.set_tile(Position::new(3, 2), crate::grid::Tile::Wall);
    let positions = radius_three_blast_positions(&map, Position::new(3, 3));
    assert!(!positions.contains(&Position::new(3, 0)));
    assert!(positions.iter().all(|position| map.is_in_bounds(*position)));
  }

  #[test]
  fn explosion_damage_is_deterministic_and_stays_within_six_d_six_bounds() {
    let mut first = GameRng::from_seed(10_400);
    let mut second = GameRng::from_seed(10_400);
    assert_eq!(
      roll_explosion_damage(&mut first),
      roll_explosion_damage(&mut second)
    );
    let mut probe = GameRng::from_seed(10_401);
    assert!((6..=36).contains(&roll_explosion_damage(&mut probe)));
  }

  #[test]
  fn distance_falloff_preserves_center_and_drops_every_second_ring() {
    assert_eq!(apply_distance_falloff(36, 0), 36);
    assert_eq!(apply_distance_falloff(36, 1), 36);
    assert_eq!(apply_distance_falloff(36, 2), 36);
    assert_eq!(apply_distance_falloff(36, 3), 18);
    assert_eq!(apply_distance_falloff(36, 4), 18);
  }

  #[test]
  fn ground_item_destruction_uses_strict_legacy_threshold() {
    assert!(!should_destroy_ground_item(10));
    assert!(should_destroy_ground_item(11));
  }

  #[test]
  fn knockback_distance_uses_integer_damage_ratio() {
    assert_eq!(knockback_distance(7), 0);
    assert_eq!(knockback_distance(8), 1);
    assert_eq!(knockback_distance(36), 4);
  }

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
