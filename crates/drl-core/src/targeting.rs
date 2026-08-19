//! Targeting validation, target queries, and target selection utilities.

use drl_protocol::{CommandError, EntityId, Position, Target};

use crate::world::World;

/// Pure targeting query and validation routines.
pub struct TargetingSystem;

impl TargetingSystem {
  /// Validates a target specification against world bounds, range, and line of sight.
  ///
  /// Returns the resolved target `Position` on success.
  pub fn validate_target(
    world: &World,
    origin: Position,
    target: Target,
    max_range: u32,
  ) -> Result<Position, CommandError> {
    let target_pos = match target {
      Target::Position(pos) => pos,
      Target::Entity(entity_id) => {
        let actor = world
          .get_actor(entity_id)
          .ok_or(CommandError::EntityNotFound(entity_id))?;
        if !actor.is_alive() {
          return Err(CommandError::InvalidTarget(actor.position()));
        }
        actor.position()
      }
      Target::Direction(dir) => origin + dir,
    };

    if !world.map().is_in_bounds(target_pos) {
      return Err(CommandError::OutOfBounds(target_pos));
    }

    let dist = origin.distance_chebyshev(target_pos);
    if dist > max_range {
      return Err(CommandError::TargetOutOfRange(target_pos));
    }

    if !world.has_line_of_sight(origin, target_pos) {
      return Err(CommandError::LineOfSightBlocked(target_pos));
    }

    Ok(target_pos)
  }

  /// Finds and sorts all living non-player targets in line of sight within `max_range`.
  ///
  /// Returns a list of `(EntityId, Position, ChebyshevDistance)` sorted by distance (closest first).
  #[must_use]
  pub fn find_visible_targets(
    world: &World,
    observer_pos: Position,
    max_range: u32,
  ) -> Vec<(EntityId, Position, u32)> {
    let mut targets = Vec::new();

    for actor in world.actors().values() {
      if actor.is_player() || !actor.is_alive() {
        continue;
      }

      let pos = actor.position();
      let dist = observer_pos.distance_chebyshev(pos);
      if dist > 0 && dist <= max_range && world.has_line_of_sight(observer_pos, pos) {
        targets.push((actor.id(), pos, dist));
      }
    }

    // Sort deterministically: distance first, then EntityId
    targets.sort_by(|a, b| a.2.cmp(&b.2).then_with(|| a.0.cmp(&b.0)));
    targets
  }

  /// Returns the nearest living non-player target in line of sight within `max_range`, if any.
  #[must_use]
  pub fn find_nearest_target(
    world: &World,
    observer_pos: Position,
    max_range: u32,
  ) -> Option<EntityId> {
    Self::find_visible_targets(world, observer_pos, max_range)
      .first()
      .map(|(id, _, _)| *id)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::grid::Map;
  use drl_protocol::{Direction, LevelId};

  #[test]
  fn test_validate_target_position_and_direction() {
    let map = Map::simple_arena(20, 20);
    let mut world = World::new(LevelId::new(1), map);
    let player_pos = Position::new(5, 5);
    world.spawn_player(player_pos, "Marine").unwrap();

    let target_pos = Position::new(8, 5);
    let validated =
      TargetingSystem::validate_target(&world, player_pos, Target::Position(target_pos), 8)
        .unwrap();
    assert_eq!(validated, target_pos);

    let dir_target =
      TargetingSystem::validate_target(&world, player_pos, Target::Direction(Direction::East), 8)
        .unwrap();
    assert_eq!(dir_target, Position::new(6, 5));
  }

  #[test]
  fn test_validate_target_out_of_range_and_blocked() {
    let map = Map::simple_arena(20, 20);
    let mut world = World::new(LevelId::new(1), map);
    let player_pos = Position::new(5, 5);
    world.spawn_player(player_pos, "Marine").unwrap();

    // Out of range check
    let far_pos = Position::new(18, 18);
    let err = TargetingSystem::validate_target(&world, player_pos, Target::Position(far_pos), 5)
      .unwrap_err();
    assert_eq!(err, CommandError::TargetOutOfRange(far_pos));

    // Blocked by wall check
    world
      .map_mut()
      .set_tile(Position::new(6, 5), crate::grid::Tile::Wall);
    let blocked_pos = Position::new(8, 5);
    let err2 =
      TargetingSystem::validate_target(&world, player_pos, Target::Position(blocked_pos), 8)
        .unwrap_err();
    assert_eq!(err2, CommandError::LineOfSightBlocked(blocked_pos));
  }

  #[test]
  fn test_find_visible_and_nearest_targets() {
    let map = Map::simple_arena(20, 20);
    let mut world = World::new(LevelId::new(1), map);
    let player_pos = Position::new(5, 5);
    world.spawn_player(player_pos, "Marine").unwrap();

    let m1 = world
      .spawn_monster(Position::new(8, 5), "Imp", 20, 100, (2, 4))
      .unwrap();
    let m2 = world
      .spawn_monster(Position::new(6, 5), "Former Human", 15, 100, (2, 4))
      .unwrap();

    let targets = TargetingSystem::find_visible_targets(&world, player_pos, 8);
    assert_eq!(targets.len(), 2);
    // Closest is m2 at distance 1
    assert_eq!(targets[0].0, m2);
    assert_eq!(targets[0].2, 1);
    // Next is m1 at distance 3
    assert_eq!(targets[1].0, m1);
    assert_eq!(targets[1].2, 3);

    let nearest = TargetingSystem::find_nearest_target(&world, player_pos, 8);
    assert_eq!(nearest, Some(m2));
  }
}
