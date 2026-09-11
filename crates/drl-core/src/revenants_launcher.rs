//! Typed Revenant's Launcher explosion geometry and damage policy.

use drl_protocol::Position;

use crate::{explosion::radius_blast_positions, grid::Map, rng::GameRng};

/// Number of damage dice rolled for each Revenant's Launcher blast cell.
pub const REVENANTS_LAUNCHER_EXPLOSION_DAMAGE_DICE: u32 = 7;
/// Sides on each Revenant's Launcher explosion damage die.
pub const REVENANTS_LAUNCHER_EXPLOSION_DAMAGE_DIE_SIDES: u32 = 6;
/// Legacy Revenant's Launcher explosion delay retained as event metadata.
pub const REVENANTS_LAUNCHER_EXPLOSION_DELAY: u32 = 40;
/// Bounded Revenant's Launcher explosion radius.
pub const REVENANTS_LAUNCHER_EXPLOSION_RADIUS: u32 = 3;
/// Legacy default explosion knockback strength.
pub const REVENANTS_LAUNCHER_EXPLOSION_KNOCKBACK: u32 = 8;
/// Legacy threshold above which a blast destroys a ground item.
pub const REVENANTS_LAUNCHER_GROUND_ITEM_DESTRUCTION_THRESHOLD: u32 = 10;

/// Returns the bounded radius-3 blast cells in deterministic order.
#[must_use]
pub fn radius_three_blast_positions(map: &Map, center: Position) -> Vec<Position> {
  radius_blast_positions(map, center, REVENANTS_LAUNCHER_EXPLOSION_RADIUS)
}

/// Rolls one explicit `7d6` Revenant's Launcher explosion result.
pub fn roll_explosion_damage(rng: &mut GameRng) -> u32 {
  (0..REVENANTS_LAUNCHER_EXPLOSION_DAMAGE_DICE)
    .map(|_| rng.gen_range(1..REVENANTS_LAUNCHER_EXPLOSION_DAMAGE_DIE_SIDES + 1))
    .sum()
}

/// Applies the pinned integer distance falloff to one blast-cell roll.
///
/// This is the same bounded rule used by the typed Rocket Launcher profile:
/// the center and first two Chebyshev-distance rings keep full damage, while
/// every later pair of rings increases the integer divisor by one.
pub use crate::rocket_launcher::apply_distance_falloff;

/// Returns whether a post-falloff blast result destroys a representable item.
#[must_use]
pub const fn should_destroy_ground_item(damage: u32) -> bool {
  damage > REVENANTS_LAUNCHER_GROUND_ITEM_DESTRUCTION_THRESHOLD
}

/// Converts a rolled explosion result to the pinned integer knockback distance.
#[must_use]
pub const fn knockback_distance(damage: u32) -> u32 {
  damage / REVENANTS_LAUNCHER_EXPLOSION_KNOCKBACK
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
  fn explosion_damage_is_deterministic_and_stays_within_seven_d_six_bounds() {
    let mut first = GameRng::from_seed(10_300);
    let mut second = GameRng::from_seed(10_300);
    assert_eq!(
      roll_explosion_damage(&mut first),
      roll_explosion_damage(&mut second)
    );
    let mut probe = GameRng::from_seed(10_301);
    assert!((7..=42).contains(&roll_explosion_damage(&mut probe)));
  }

  #[test]
  fn distance_falloff_preserves_center_and_drops_every_second_ring() {
    assert_eq!(apply_distance_falloff(42, 0), 42);
    assert_eq!(apply_distance_falloff(42, 1), 42);
    assert_eq!(apply_distance_falloff(42, 2), 42);
    assert_eq!(apply_distance_falloff(42, 3), 21);
    assert_eq!(apply_distance_falloff(42, 4), 21);
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
    assert_eq!(knockback_distance(42), 5);
  }
}
