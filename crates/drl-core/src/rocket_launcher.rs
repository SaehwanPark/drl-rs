//! Typed Rocket Launcher explosion geometry and damage policy.

use drl_protocol::Position;

use crate::{explosion::radius_blast_positions, grid::Map, rng::GameRng};

/// Number of damage dice rolled for each Rocket Launcher blast cell.
pub const ROCKET_LAUNCHER_EXPLOSION_DAMAGE_DICE: u32 = 6;
/// Sides on each Rocket Launcher explosion damage die.
pub const ROCKET_LAUNCHER_EXPLOSION_DAMAGE_DIE_SIDES: u32 = 6;
/// Legacy Rocket Launcher explosion delay retained as presentation metadata.
pub const ROCKET_LAUNCHER_EXPLOSION_DELAY: u32 = 40;
/// Bounded Rocket Launcher explosion radius.
pub const ROCKET_LAUNCHER_EXPLOSION_RADIUS: u32 = 4;
/// Legacy default explosion knockback strength.
pub const ROCKET_LAUNCHER_EXPLOSION_KNOCKBACK: u32 = 8;
/// Legacy threshold above which a Rocket Launcher blast destroys a ground item.
pub const ROCKET_LAUNCHER_GROUND_ITEM_DESTRUCTION_THRESHOLD: u32 = 10;

/// Returns the bounded radius-4 blast cells in deterministic order.
#[must_use]
pub fn radius_four_blast_positions(map: &Map, center: Position) -> Vec<Position> {
  radius_blast_positions(map, center, ROCKET_LAUNCHER_EXPLOSION_RADIUS)
}

/// Rolls one explicit `6d6` Rocket Launcher explosion result.
pub fn roll_explosion_damage(rng: &mut GameRng) -> u32 {
  (0..ROCKET_LAUNCHER_EXPLOSION_DAMAGE_DICE)
    .map(|_| rng.gen_range(1..ROCKET_LAUNCHER_EXPLOSION_DAMAGE_DIE_SIDES + 1))
    .sum()
}

/// Applies the legacy integer distance falloff to one blast-cell roll.
///
/// The center and first two Chebyshev-distance rings keep full damage; each
/// later pair of rings increases the integer divisor by one.
#[must_use]
pub const fn apply_distance_falloff(damage: u32, distance: u32) -> u32 {
  let divisor = match distance.saturating_add(1) / 2 {
    0 => 1,
    value => value,
  };
  damage / divisor
}

/// Returns whether a post-falloff blast result destroys a representable item.
#[must_use]
pub const fn should_destroy_ground_item(damage: u32) -> bool {
  damage > ROCKET_LAUNCHER_GROUND_ITEM_DESTRUCTION_THRESHOLD
}

/// Converts a rolled explosion result to the pinned integer knockback distance.
#[must_use]
pub const fn knockback_distance(damage: u32) -> u32 {
  damage / ROCKET_LAUNCHER_EXPLOSION_KNOCKBACK
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn radius_four_geometry_is_center_then_clockwise_rings() {
    let map = Map::simple_arena(9, 9);
    let positions = radius_four_blast_positions(&map, Position::new(4, 4));
    assert_eq!(positions.len(), 81);
    assert_eq!(positions[0], Position::new(4, 4));
    assert_eq!(positions[1], Position::new(4, 3));
    assert_eq!(positions[8], Position::new(3, 3));
    assert_eq!(positions[9], Position::new(4, 2));
    assert_eq!(positions[80], Position::new(3, 0));
  }

  #[test]
  fn radius_four_geometry_clamps_edges_and_blocks_rays() {
    let mut map = Map::simple_arena(7, 7);
    map.set_tile(Position::new(3, 2), crate::grid::Tile::Wall);
    let positions = radius_four_blast_positions(&map, Position::new(3, 3));
    assert!(!positions.contains(&Position::new(3, 0)));
    assert!(positions.iter().all(|position| map.is_in_bounds(*position)));
  }

  #[test]
  fn explosion_damage_is_deterministic_and_stays_within_six_d_six_bounds() {
    let mut first = GameRng::from_seed(10_200);
    let mut second = GameRng::from_seed(10_200);
    assert_eq!(
      roll_explosion_damage(&mut first),
      roll_explosion_damage(&mut second)
    );
    let mut probe = GameRng::from_seed(10_201);
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
}
