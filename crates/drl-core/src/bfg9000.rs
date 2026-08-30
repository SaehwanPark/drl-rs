//! Typed Standard BFG 9000 explosion geometry and damage policy.

use drl_protocol::Position;

use crate::{explosion::radius_blast_positions, grid::Map, rng::GameRng};

/// Number of damage dice rolled for each Standard BFG 9000 blast cell.
pub const BFG9000_EXPLOSION_DAMAGE_DICE: u32 = 10;
/// Number of sides on each Standard BFG 9000 explosion die.
pub const BFG9000_EXPLOSION_DAMAGE_DIE_SIDES: u32 = 6;
/// Legacy threshold above which a Standard BFG 9000 blast destroys a ground item.
pub const BFG9000_GROUND_ITEM_DESTRUCTION_THRESHOLD: u32 = 10;

/// Returns the bounded radius-8 blast cells in deterministic order.
#[must_use]
pub fn radius_eight_blast_positions(map: &Map, center: Position) -> Vec<Position> {
  radius_blast_positions(map, center, 8)
}

/// Rolls one explicit `10d6` Standard BFG 9000 explosion result.
pub fn roll_explosion_damage(rng: &mut GameRng) -> u32 {
  (0..BFG9000_EXPLOSION_DAMAGE_DICE)
    .map(|_| rng.gen_range(1..BFG9000_EXPLOSION_DAMAGE_DIE_SIDES + 1))
    .sum()
}

/// Returns whether a Standard BFG 9000 blast destroys an ordinary ground item.
#[must_use]
pub const fn should_destroy_bfg9000_ground_item(damage: u32) -> bool {
  damage > BFG9000_GROUND_ITEM_DESTRUCTION_THRESHOLD
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn radius_eight_geometry_is_center_then_clockwise_rings() {
    let map = Map::simple_arena(17, 17);
    let positions = radius_eight_blast_positions(&map, Position::new(8, 8));
    assert_eq!(positions.len(), 289);
    assert_eq!(positions[0], Position::new(8, 8));
    assert_eq!(positions[1], Position::new(8, 7));
    assert_eq!(positions[8], Position::new(7, 7));
    assert_eq!(positions[9], Position::new(8, 6));
    assert_eq!(positions[288], Position::new(7, 0));
  }

  #[test]
  fn explosion_damage_is_deterministic_and_stays_within_ten_d_six_bounds() {
    let mut first = GameRng::from_seed(10_000);
    let mut second = GameRng::from_seed(10_000);
    assert_eq!(
      roll_explosion_damage(&mut first),
      roll_explosion_damage(&mut second)
    );
    let mut probe = GameRng::from_seed(10_001);
    let damage = roll_explosion_damage(&mut probe);
    assert!((10..=60).contains(&damage));
  }

  #[test]
  fn ground_item_destruction_uses_strict_legacy_threshold() {
    assert!(!should_destroy_bfg9000_ground_item(10));
    assert!(should_destroy_bfg9000_ground_item(11));
  }
}
