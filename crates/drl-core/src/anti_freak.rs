//! Typed Anti-Freak Jackal splash geometry, damage rolls, and knockback policy.

use drl_protocol::{Direction, Position};

use crate::{fov, grid::Map, rng::GameRng};

/// Number of damage dice used by the Anti-Freak Jackal splash.
pub const ANTI_FREAK_JACKAL_SPLASH_DICE: u32 = 5;
/// Sides on each Anti-Freak Jackal splash damage die.
pub const ANTI_FREAK_JACKAL_SPLASH_DIE_SIDES: u32 = 3;

/// Converts an impact-to-actor delta into the radial knockback direction.
///
/// The impact center has no outward direction and therefore is not displaced
/// by this bounded resolver.
#[must_use]
pub fn splash_knockback_direction(center: Position, target: Position) -> Option<Direction> {
  match Direction::from_delta(target.x - center.x, target.y - center.y) {
    Some(Direction::None) | None => None,
    direction => direction,
  }
}

/// Converts rolled splash damage and the typed knockback strength into tiles.
///
/// This preserves the legacy integer strength boundary (`damage / knockback`)
/// while keeping the displacement policy explicit and deterministic.
#[must_use]
pub fn splash_knockback_distance(damage: u32, knockback: u32) -> u32 {
  damage.checked_div(knockback).unwrap_or(0)
}

/// Returns the bounded radius-1 blast cells in deterministic order.
///
/// The impact center is considered first, followed by the eight neighboring
/// cells clockwise from north. Out-of-bounds cells and cells without a clear
/// ray from the impact center are omitted. Farther cells are never considered.
#[must_use]
pub fn radius_one_blast_positions(map: &Map, center: Position) -> Vec<Position> {
  if !map.is_in_bounds(center) {
    return Vec::new();
  }

  let mut positions = Vec::with_capacity(9);
  positions.push(center);
  for position in [
    center.offset(0, -1),
    center.offset(1, -1),
    center.offset(1, 0),
    center.offset(1, 1),
    center.offset(0, 1),
    center.offset(-1, 1),
    center.offset(-1, 0),
    center.offset(-1, -1),
  ] {
    if map.is_in_bounds(position) && fov::has_line_of_sight(map, center, position) {
      positions.push(position);
    }
  }
  positions
}

/// Rolls one explicit `5d3` splash damage result using the game RNG.
pub fn roll_splash_damage(rng: &mut GameRng) -> u32 {
  (0..ANTI_FREAK_JACKAL_SPLASH_DICE)
    .map(|_| rng.gen_range(1..ANTI_FREAK_JACKAL_SPLASH_DIE_SIDES + 1))
    .sum()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn radius_one_geometry_is_center_then_clockwise_neighbors() {
    let map = Map::simple_arena(5, 5);
    assert_eq!(
      radius_one_blast_positions(&map, Position::new(2, 2)),
      vec![
        Position::new(2, 2),
        Position::new(2, 1),
        Position::new(3, 1),
        Position::new(3, 2),
        Position::new(3, 3),
        Position::new(2, 3),
        Position::new(1, 3),
        Position::new(1, 2),
        Position::new(1, 1),
      ]
    );
  }

  #[test]
  fn radius_one_geometry_clamps_edges() {
    let map = Map::simple_arena(3, 3);
    assert_eq!(
      radius_one_blast_positions(&map, Position::new(0, 0)),
      vec![
        Position::new(0, 0),
        Position::new(1, 0),
        Position::new(1, 1),
        Position::new(0, 1),
      ]
    );
  }

  #[test]
  fn splash_damage_is_deterministic_and_stays_within_five_d_three_bounds() {
    let mut first = GameRng::from_seed(252);
    let mut second = GameRng::from_seed(252);
    let first_roll = roll_splash_damage(&mut first);
    let second_roll = roll_splash_damage(&mut second);
    assert_eq!(first_roll, second_roll);
    assert!((5..=15).contains(&first_roll));
  }

  #[test]
  fn splash_knockback_direction_is_radial_and_center_safe() {
    let center = Position::new(2, 2);
    assert_eq!(splash_knockback_direction(center, center), None);
    assert_eq!(
      splash_knockback_direction(center, Position::new(3, 1)),
      Some(Direction::NorthEast)
    );
    assert_eq!(
      splash_knockback_direction(center, Position::new(1, 3)),
      Some(Direction::SouthWest)
    );
  }

  #[test]
  fn splash_knockback_distance_uses_integer_damage_ratio() {
    assert_eq!(splash_knockback_distance(7, 8), 0);
    assert_eq!(splash_knockback_distance(8, 8), 1);
    assert_eq!(splash_knockback_distance(15, 8), 1);
    assert_eq!(splash_knockback_distance(15, 0), 0);
  }
}
