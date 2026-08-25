//! Tactical monster AI decision making.

use drl_protocol::{Direction, EntityId, Position};

use crate::actor::Actor;
use crate::world::World;

/// Tactical action decided by monster AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterAction {
  /// Melee attack the target entity.
  Melee(EntityId),
  /// Ranged attack targeting a grid position.
  Ranged(Position),
  /// Step in a specific direction.
  Move(Direction),
  /// Wait in place.
  Wait,
}

/// Pure AI decision engine for monster turns.
pub struct MonsterAi;

impl MonsterAi {
  /// Evaluates world state and decides the optimal action for a monster against the player.
  #[must_use]
  pub fn decide_action(
    monster: &Actor,
    world: &World,
    target_player_id: EntityId,
  ) -> MonsterAction {
    let Some(player) = world.get_actor(target_player_id) else {
      return MonsterAction::Wait;
    };
    if !player.is_alive() {
      return MonsterAction::Wait;
    }

    let m_pos = monster.position();
    let p_pos = player.position();
    let dist = m_pos.distance_chebyshev(p_pos);

    // 1. If adjacent to player -> Melee attack
    if dist == 1 {
      return MonsterAction::Melee(target_player_id);
    }

    // 2. If monster has ranged capability and clear line of fire within range -> Ranged attack
    if monster.ranged_damage().is_some() {
      let range = monster.ranged_range();
      if dist <= range && world.has_line_of_sight(m_pos, p_pos) {
        return MonsterAction::Ranged(p_pos);
      }
    }

    // 3. Try the smoothed one-step direction first. Strongly skewed deltas
    // become cardinal before the legacy path retries the raw diagonal.
    let dx = p_pos.x - m_pos.x;
    let dy = p_pos.y - m_pos.y;
    let Some(preferred_dir) = Self::smooth_direction(dx, dy) else {
      return MonsterAction::Wait;
    };
    if Self::is_open_step(world, m_pos, preferred_dir) {
      return MonsterAction::Move(preferred_dir);
    }

    // A blocked smoothed step retries the raw sign direction, then tries the
    // horizontal and vertical cardinal components in that order. No broader
    // pathfinding search is performed.
    let Some(raw_dir) = Direction::from_delta(dx, dy) else {
      return MonsterAction::Wait;
    };
    if raw_dir != preferred_dir && Self::is_open_step(world, m_pos, raw_dir) {
      return MonsterAction::Move(raw_dir);
    }
    for fallback_dir in [Direction::from_delta(dx, 0), Direction::from_delta(0, dy)]
      .into_iter()
      .flatten()
      .filter(|&direction| direction != Direction::None && direction != raw_dir)
    {
      if Self::is_open_step(world, m_pos, fallback_dir) {
        return MonsterAction::Move(fallback_dir);
      }
    }

    MonsterAction::Wait
  }

  fn smooth_direction(dx: i32, dy: i32) -> Option<Direction> {
    if dx == 0 && dy == 0 {
      return None;
    }
    let raw = Direction::from_delta(dx, dy)?;
    if dx == 0 || dy == 0 {
      return Some(raw);
    }

    let abs_dx = i64::from(dx).abs();
    let abs_dy = i64::from(dy).abs();
    if abs_dx * 10 >= abs_dy * 19 {
      Direction::from_delta(dx, 0)
    } else if abs_dy * 10 >= abs_dx * 19 {
      Direction::from_delta(0, dy)
    } else {
      Some(raw)
    }
  }

  fn is_open_step(world: &World, from: Position, direction: Direction) -> bool {
    let target = from + direction;
    world.map().is_in_bounds(target) && !world.is_cell_blocked(target)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::grid::{Map, Tile};
  use drl_protocol::LevelId;

  #[test]
  fn test_adjacent_monster_decides_melee() {
    let map = Map::simple_arena(20, 20);
    let mut world = World::new(LevelId::new(1), map);
    let player_id = world.spawn_player(Position::new(5, 5), "Marine").unwrap();

    let monster = Actor::demon(EntityId::new(10), Position::new(6, 5));
    let action = MonsterAi::decide_action(&monster, &world, player_id);
    assert_eq!(action, MonsterAction::Melee(player_id));
  }

  #[test]
  fn test_ranged_monster_decides_ranged_attack() {
    let map = Map::simple_arena(20, 20);
    let mut world = World::new(LevelId::new(1), map);
    let player_id = world.spawn_player(Position::new(2, 2), "Marine").unwrap();

    let imp = Actor::imp(EntityId::new(10), Position::new(6, 2));
    let action = MonsterAi::decide_action(&imp, &world, player_id);
    assert_eq!(action, MonsterAction::Ranged(Position::new(2, 2)));
  }

  #[test]
  fn test_blocked_ranged_monster_moves_closer() {
    let mut map = Map::simple_arena(20, 20);
    // Wall blocking direct LOS at (4, 2)
    map.set_tile(Position::new(4, 2), Tile::Wall);
    let mut world = World::new(LevelId::new(1), map);
    let player_id = world.spawn_player(Position::new(2, 2), "Marine").unwrap();

    let imp = Actor::imp(EntityId::new(10), Position::new(6, 2));
    let action = MonsterAi::decide_action(&imp, &world, player_id);

    // Cannot shoot because LOS is blocked, so moves towards player
    assert!(matches!(action, MonsterAction::Move(_)));
  }

  #[test]
  fn test_melee_demon_moves_towards_player() {
    let map = Map::simple_arena(20, 20);
    let mut world = World::new(LevelId::new(1), map);
    let player_id = world.spawn_player(Position::new(2, 2), "Marine").unwrap();

    let demon = Actor::demon(EntityId::new(10), Position::new(6, 2));
    let action = MonsterAi::decide_action(&demon, &world, player_id);
    assert_eq!(action, MonsterAction::Move(Direction::West));
  }

  #[test]
  fn test_blocked_diagonal_uses_horizontal_fallback_first() {
    let mut map = Map::simple_arena(20, 20);
    map.set_tile(Position::new(5, 5), Tile::Wall);
    let mut world = World::new(LevelId::new(1), map);
    let player_id = world.spawn_player(Position::new(4, 4), "Marine").unwrap();

    let demon = Actor::demon(EntityId::new(10), Position::new(6, 6));
    let action = MonsterAi::decide_action(&demon, &world, player_id);

    assert_eq!(action, MonsterAction::Move(Direction::West));
  }

  #[test]
  fn test_blocked_horizontal_fallback_uses_vertical_fallback() {
    let mut map = Map::simple_arena(20, 20);
    map.set_tile(Position::new(5, 5), Tile::Wall);
    map.set_tile(Position::new(5, 6), Tile::Wall);
    let mut world = World::new(LevelId::new(1), map);
    let player_id = world.spawn_player(Position::new(4, 4), "Marine").unwrap();

    let demon = Actor::demon(EntityId::new(10), Position::new(6, 6));
    let action = MonsterAi::decide_action(&demon, &world, player_id);

    assert_eq!(action, MonsterAction::Move(Direction::North));
  }

  #[test]
  fn test_blocked_smoothed_cardinal_retries_raw_diagonal() {
    let mut map = Map::simple_arena(20, 20);
    map.set_tile(Position::new(7, 6), Tile::Wall);
    let mut world = World::new(LevelId::new(1), map);
    let player_id = world.spawn_player(Position::new(4, 5), "Marine").unwrap();

    let demon = Actor::demon(EntityId::new(10), Position::new(8, 6));
    let action = MonsterAi::decide_action(&demon, &world, player_id);

    assert_eq!(action, MonsterAction::Move(Direction::NorthWest));
  }

  #[test]
  fn test_blocked_diagonal_and_cardinal_fallbacks_wait_without_searching() {
    let mut map = Map::simple_arena(20, 20);
    map.set_tile(Position::new(5, 5), Tile::Wall);
    map.set_tile(Position::new(6, 5), Tile::Wall);
    map.set_tile(Position::new(5, 6), Tile::Wall);
    let mut world = World::new(LevelId::new(1), map);
    let player_id = world.spawn_player(Position::new(4, 4), "Marine").unwrap();

    let demon = Actor::demon(EntityId::new(10), Position::new(6, 6));
    let action = MonsterAi::decide_action(&demon, &world, player_id);

    assert_eq!(action, MonsterAction::Wait);
  }

  #[test]
  fn test_same_position_target_waits_without_move_none() {
    let map = Map::simple_arena(20, 20);
    let mut world = World::new(LevelId::new(1), map);
    let player_id = world.spawn_player(Position::new(6, 6), "Marine").unwrap();

    let demon = Actor::demon(EntityId::new(10), Position::new(6, 6));
    let action = MonsterAi::decide_action(&demon, &world, player_id);

    assert_eq!(action, MonsterAction::Wait);
  }
}
