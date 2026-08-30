//! Typed Nuclear BFG 9000 explosion geometry and damage policy.

use drl_protocol::Position;

use crate::{
  bfg9000::radius_eight_blast_positions as shared_radius_eight_blast_positions, grid::Map,
  rng::GameRng,
};

/// Number of damage dice rolled for each Nuclear BFG 9000 blast cell.
pub const NUCLEAR_BFG9000_EXPLOSION_DAMAGE_DICE: u32 = 8;
/// Number of sides on each Nuclear BFG 9000 explosion die.
pub const NUCLEAR_BFG9000_EXPLOSION_DAMAGE_DIE_SIDES: u32 = 6;

/// Returns the bounded radius-8 blast cells in deterministic order.
#[must_use]
pub fn radius_eight_blast_positions(map: &Map, center: Position) -> Vec<Position> {
  shared_radius_eight_blast_positions(map, center)
}

/// Rolls one explicit `8d6` Nuclear BFG 9000 explosion result.
pub fn roll_explosion_damage(rng: &mut GameRng) -> u32 {
  (0..NUCLEAR_BFG9000_EXPLOSION_DAMAGE_DICE)
    .map(|_| rng.gen_range(1..NUCLEAR_BFG9000_EXPLOSION_DAMAGE_DIE_SIDES + 1))
    .sum()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn radius_eight_geometry_reuses_the_standard_bfg_contract() {
    let map = Map::simple_arena(17, 17);
    assert_eq!(
      radius_eight_blast_positions(&map, Position::new(8, 8)),
      crate::bfg9000::radius_eight_blast_positions(&map, Position::new(8, 8))
    );
  }

  #[test]
  fn explosion_damage_is_deterministic_and_stays_within_eight_d_six_bounds() {
    let mut first = GameRng::from_seed(10_100);
    let mut second = GameRng::from_seed(10_100);
    assert_eq!(
      roll_explosion_damage(&mut first),
      roll_explosion_damage(&mut second)
    );
    let mut probe = GameRng::from_seed(10_101);
    let damage = roll_explosion_damage(&mut probe);
    assert!((8..=48).contains(&damage));
  }
}
