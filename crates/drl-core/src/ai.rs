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

    // 3. Navigate towards the player by choosing the best walkable direction
    let best_dir = Direction::ALL_8WAY
      .into_iter()
      .filter(|&dir| {
        let target = m_pos + dir;
        world.map().is_in_bounds(target) && !world.is_cell_blocked(target)
      })
      .min_by_key(|&dir| {
        let target = m_pos + dir;
        (
          target.distance_chebyshev(p_pos),
          target.distance_squared(p_pos),
        )
      });

    if let Some(dir) = best_dir {
      MonsterAction::Move(dir)
    } else {
      MonsterAction::Wait
    }
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
}
