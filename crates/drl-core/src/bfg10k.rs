//! Typed BFG 10K explosion geometry and damage policy.

use drl_protocol::Position;

use crate::{fov, grid::Map, rng::GameRng};

/// Number of damage dice rolled for each BFG 10K blast cell.
pub const BFG10K_EXPLOSION_DAMAGE_DICE: u32 = 6;
/// Number of sides on each BFG 10K explosion die.
pub const BFG10K_EXPLOSION_DAMAGE_DIE_SIDES: u32 = 4;
/// Legacy threshold above which a BFG 10K blast destroys loose ammunition.
pub const BFG10K_GROUND_ITEM_DESTRUCTION_THRESHOLD: u32 = 10;

/// Returns the bounded radius-2 blast cells in deterministic order.
///
/// Cells are emitted as the center, then the radius-1 ring, then the
/// radius-2 ring, each starting north and proceeding clockwise. Opaque
/// intermediate cells block a blast ray; the endpoint itself may be opaque.
#[must_use]
pub fn radius_two_blast_positions(map: &Map, center: Position) -> Vec<Position> {
  if !map.is_in_bounds(center) {
    return Vec::new();
  }

  let mut positions = vec![center];
  for radius in 1_u32..=2 {
    let radius = radius as i32;
    let mut ring = Vec::with_capacity((radius * 8) as usize);
    for x in 0..=radius {
      ring.push(center.offset(x, -radius));
    }
    for y in (-radius + 1)..=radius {
      ring.push(center.offset(radius, y));
    }
    for x in (-(radius)..=radius - 1).rev() {
      ring.push(center.offset(x, radius));
    }
    for y in (-(radius)..=radius - 1).rev() {
      ring.push(center.offset(-radius, y));
    }
    for x in (-radius + 1)..0 {
      ring.push(center.offset(x, -radius));
    }
    for position in ring {
      if position.distance_chebyshev(center) != radius as u32
        || !map.is_in_bounds(position)
        || !fov::has_line_of_sight(map, center, position)
      {
        continue;
      }
      positions.push(position);
    }
  }
  positions
}

/// Rolls one explicit `6d4` BFG 10K explosion result.
pub fn roll_explosion_damage(rng: &mut GameRng) -> u32 {
  (0..BFG10K_EXPLOSION_DAMAGE_DICE)
    .map(|_| rng.gen_range(1..BFG10K_EXPLOSION_DAMAGE_DIE_SIDES + 1))
    .sum()
}

/// Returns whether a BFG 10K blast destroys an ordinary loose-ammo stack.
#[must_use]
pub const fn should_destroy_bfg10k_ground_item(damage: u32) -> bool {
  damage > BFG10K_GROUND_ITEM_DESTRUCTION_THRESHOLD
}

/// Converts a rolled explosion result to the pinned integer knockback distance.
#[must_use]
pub fn knockback_distance(damage: u32, knockback: u32) -> u32 {
  damage.checked_div(knockback).unwrap_or(0)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn radius_two_geometry_is_center_then_clockwise_rings() {
    let map = Map::simple_arena(9, 9);
    let positions = radius_two_blast_positions(&map, Position::new(4, 4));
    assert_eq!(positions.len(), 25);
    assert_eq!(positions[0], Position::new(4, 4));
    assert_eq!(positions[1], Position::new(4, 3));
    assert_eq!(positions[8], Position::new(3, 3));
    assert_eq!(positions[9], Position::new(4, 2));
    assert_eq!(positions[24], Position::new(3, 2));
  }

  #[test]
  fn radius_two_geometry_omits_blocked_rays_and_clamps_edges() {
    let mut map = Map::simple_arena(5, 5);
    map.set_tile(Position::new(2, 1), crate::grid::Tile::Wall);
    let positions = radius_two_blast_positions(&map, Position::new(2, 2));
    assert!(!positions.contains(&Position::new(2, 0)));
    assert!(positions.contains(&Position::new(0, 2)));
    assert!(positions.iter().all(|position| map.is_in_bounds(*position)));
  }

  #[test]
  fn explosion_damage_is_deterministic_and_stays_within_six_d_four_bounds() {
    let mut first = GameRng::from_seed(10_000);
    let mut second = GameRng::from_seed(10_000);
    assert_eq!(
      roll_explosion_damage(&mut first),
      roll_explosion_damage(&mut second)
    );
    let mut probe = GameRng::from_seed(10_001);
    let damage = roll_explosion_damage(&mut probe);
    assert!((6..=24).contains(&damage));
  }

  #[test]
  fn knockback_distance_uses_integer_ratio() {
    assert_eq!(knockback_distance(15, 16), 0);
    assert_eq!(knockback_distance(16, 16), 1);
    assert_eq!(knockback_distance(24, 16), 1);
    assert_eq!(knockback_distance(24, 0), 0);
  }

  #[test]
  fn ground_item_destruction_uses_strict_legacy_threshold() {
    assert!(!should_destroy_bfg10k_ground_item(10));
    assert!(should_destroy_bfg10k_ground_item(11));
  }
}
