//! Field-of-view (FOV) calculation and line-of-sight (LOS) ray tracing.
//!
//! Provides deterministic visibility calculations and obstacle occlusion checks
//! for player perception, monster line of sight, and ranged combat targeting.

use drl_protocol::Position;
use std::collections::BTreeSet;

use crate::grid::Map;

/// Default vision radius for player characters in simulation cells.
pub const DEFAULT_VISION_RADIUS: u32 = 8;

/// Generates discrete grid points along a straight line between two positions
/// using Bresenham's line algorithm.
#[must_use]
pub fn line_points(from: Position, to: Position) -> Vec<Position> {
  let mut points = Vec::new();
  let dx = (to.x - from.x).abs();
  let dy = -(to.y - from.y).abs();
  let sx = if from.x < to.x { 1 } else { -1 };
  let sy = if from.y < to.y { 1 } else { -1 };
  let mut err = dx + dy;
  let mut curr = from;

  loop {
    points.push(curr);
    if curr == to {
      break;
    }
    let e2 = 2 * err;
    if e2 >= dy {
      err += dy;
      curr.x += sx;
    }
    if e2 <= dx {
      err += dx;
      curr.y += sy;
    }
  }

  points
}

/// Checks whether an unblocked line of sight exists between `from` and `to`.
///
/// Returns `true` if all intermediate cells along the ray are transparent.
/// The origin and target cells are not checked for intermediate transparency,
/// meaning you can see/shoot a target that is on an opaque tile (e.g., examining a wall).
#[must_use]
pub fn has_line_of_sight(map: &Map, from: Position, to: Position) -> bool {
  if from == to {
    return true;
  }

  let points = line_points(from, to);
  // Intermediate points excluding `from` and `to`
  for &pos in points.iter().skip(1).take(points.len().saturating_sub(2)) {
    if !map.is_in_bounds(pos) || !map.is_transparent(pos) {
      return false;
    }
  }

  true
}

/// Computes the set of all positions visible from an `origin` position within `max_radius`.
///
/// Uses deterministic perimeter raycasting with obstacle occlusion. Transparent cells
/// transmit vision, while opaque cells (walls, closed doors) are illuminated but block
/// sight beyond them.
#[must_use]
pub fn compute_fov(map: &Map, origin: Position, max_radius: u32) -> BTreeSet<Position> {
  let mut visible = BTreeSet::new();

  if !map.is_in_bounds(origin) {
    return visible;
  }

  visible.insert(origin);

  let r = max_radius as i32;
  let min_x = (origin.x - r).max(0);
  let max_x = (origin.x + r).min(map.width() as i32 - 1);
  let min_y = (origin.y - r).max(0);
  let max_y = (origin.y + r).min(map.height() as i32 - 1);

  // Cast rays to all perimeter cells of the bounding box
  let mut perimeter_targets = Vec::new();
  for x in min_x..=max_x {
    perimeter_targets.push(Position::new(x, min_y));
    perimeter_targets.push(Position::new(x, max_y));
  }
  for y in (min_y + 1)..max_y {
    perimeter_targets.push(Position::new(min_x, y));
    perimeter_targets.push(Position::new(max_x, y));
  }

  for target in perimeter_targets {
    let points = line_points(origin, target);
    for &pos in &points {
      if !map.is_in_bounds(pos) {
        break;
      }
      if origin.distance_chebyshev(pos) > max_radius {
        break;
      }

      visible.insert(pos);

      // If this tile blocks light, vision stops beyond this point along the ray
      if !map.is_transparent(pos) {
        break;
      }
    }
  }

  visible
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::grid::Tile;

  #[test]
  fn test_line_points_straight_and_diagonal() {
    // Horizontal
    let pts_h = line_points(Position::new(0, 0), Position::new(3, 0));
    assert_eq!(
      pts_h,
      vec![
        Position::new(0, 0),
        Position::new(1, 0),
        Position::new(2, 0),
        Position::new(3, 0)
      ]
    );

    // Diagonal
    let pts_d = line_points(Position::new(1, 1), Position::new(3, 3));
    assert_eq!(
      pts_d,
      vec![
        Position::new(1, 1),
        Position::new(2, 2),
        Position::new(3, 3)
      ]
    );
  }

  #[test]
  fn test_line_of_sight_unblocked_and_blocked() {
    let mut map = Map::simple_arena(10, 10);
    // Floor across (1, 5) to (5, 5) -> unblocked
    assert!(has_line_of_sight(
      &map,
      Position::new(1, 5),
      Position::new(5, 5)
    ));

    // Place a wall at (3, 5)
    map.set_tile(Position::new(3, 5), Tile::Wall);

    // Sight from (1, 5) to (5, 5) is now blocked by the wall
    assert!(!has_line_of_sight(
      &map,
      Position::new(1, 5),
      Position::new(5, 5)
    ));

    // Sight from (1, 5) to the wall at (3, 5) itself IS visible
    assert!(has_line_of_sight(
      &map,
      Position::new(1, 5),
      Position::new(3, 5)
    ));
  }

  #[test]
  fn test_compute_fov_with_wall_shadow() {
    let mut map = Map::simple_arena(15, 15);
    let origin = Position::new(5, 5);

    // Build a vertical wall barrier at x = 7 from y = 4 to y = 6
    for y in 4..=6 {
      map.set_tile(Position::new(7, y), Tile::Wall);
    }

    let fov = compute_fov(&map, origin, 5);

    // Origin is visible
    assert!(fov.contains(&origin));

    // Wall itself at (7, 5) is visible (catches light)
    assert!(fov.contains(&Position::new(7, 5)));
    assert!(fov.contains(&Position::new(7, 4)));
    assert!(fov.contains(&Position::new(7, 6)));

    // Tile directly behind wall at (8, 5) is occluded / in shadow!
    assert!(!fov.contains(&Position::new(8, 5)));
    assert!(!fov.contains(&Position::new(9, 5)));

    // Flanking floor cells around the barrier are visible
    assert!(fov.contains(&Position::new(7, 3)));
    assert!(fov.contains(&Position::new(7, 7)));
    assert!(fov.contains(&Position::new(8, 2)));
    assert!(fov.contains(&Position::new(8, 8)));
  }

  #[test]
  fn test_compute_fov_doors() {
    let mut map = Map::simple_arena(10, 10);
    let origin = Position::new(2, 5);
    let door_pos = Position::new(4, 5);
    let behind_door = Position::new(6, 5);

    // Closed door blocks sight behind it
    map.set_tile(door_pos, Tile::DoorClosed);
    let fov_closed = compute_fov(&map, origin, 6);
    assert!(fov_closed.contains(&door_pos));
    assert!(!fov_closed.contains(&behind_door));

    // Open door transmits sight
    map.set_tile(door_pos, Tile::DoorOpen);
    let fov_open = compute_fov(&map, origin, 6);
    assert!(fov_open.contains(&door_pos));
    assert!(fov_open.contains(&behind_door));
  }
}
