//! Deterministic square-radius blast geometry shared by typed explosions.

use drl_protocol::Position;

use crate::{fov, grid::Map};

/// Returns in-bounds, line-of-sight-cleared cells in center-then-ring order.
///
/// Each ring starts north of the center and proceeds clockwise. The endpoint
/// may be opaque, but an opaque intermediate cell blocks the ray. A zero or
/// out-of-range radius still returns the in-bounds center cell only.
#[must_use]
pub fn radius_blast_positions(map: &Map, center: Position, radius: u32) -> Vec<Position> {
  if !map.is_in_bounds(center) {
    return Vec::new();
  }

  let Some(max_radius) = i32::try_from(radius).ok() else {
    return vec![center];
  };

  let mut positions = vec![center];
  for radius in 1..=max_radius {
    let mut ring = Vec::with_capacity((radius as usize).saturating_mul(8));
    for x in 0..=radius {
      ring.push(center.offset(x, -radius));
    }
    for y in (-radius + 1)..=radius {
      ring.push(center.offset(radius, y));
    }
    for x in (-radius..=radius - 1).rev() {
      ring.push(center.offset(x, radius));
    }
    for y in (-radius..=radius - 1).rev() {
      ring.push(center.offset(-radius, y));
    }
    for x in (-radius + 1)..0 {
      ring.push(center.offset(x, -radius));
    }
    for position in ring {
      if !map.is_in_bounds(position) || !fov::has_line_of_sight(map, center, position) {
        continue;
      }
      positions.push(position);
    }
  }
  positions
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn radius_zero_returns_only_an_in_bounds_center() {
    let map = Map::simple_arena(5, 5);
    assert_eq!(
      radius_blast_positions(&map, Position::new(2, 2), 0),
      vec![Position::new(2, 2)]
    );
    assert!(radius_blast_positions(&map, Position::new(-1, 2), 0).is_empty());
  }

  #[test]
  fn radius_two_is_center_then_clockwise_rings() {
    let map = Map::simple_arena(9, 9);
    let positions = radius_blast_positions(&map, Position::new(4, 4), 2);
    assert_eq!(positions.len(), 25);
    assert_eq!(positions[0], Position::new(4, 4));
    assert_eq!(positions[1], Position::new(4, 3));
    assert_eq!(positions[8], Position::new(3, 3));
    assert_eq!(positions[9], Position::new(4, 2));
    assert_eq!(positions[24], Position::new(3, 2));
  }
}
