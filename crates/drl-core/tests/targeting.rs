//! Integration tests for target validation and visible target queries.

use drl_core::game::Game;
use drl_core::grid::Tile;
use drl_core::targeting::TargetingSystem;
use drl_protocol::{CommandError, Direction, MonsterKind, Position, Target};

#[test]
fn test_targeting_system_end_to_end_validation() {
  let mut game = Game::new(3333, 20, 20, Position::new(5, 5)).unwrap();
  let player_pos = Position::new(5, 5);

  let imp_id = game
    .world_mut()
    .spawn_monster_kind(Position::new(9, 5), MonsterKind::Imp)
    .unwrap();

  // Validate by Entity Target
  let pos =
    TargetingSystem::validate_target(game.world(), player_pos, Target::Entity(imp_id), 10).unwrap();
  assert_eq!(pos, Position::new(9, 5));

  // Validate by Position Target
  let pos2 = TargetingSystem::validate_target(
    game.world(),
    player_pos,
    Target::Position(Position::new(9, 5)),
    10,
  )
  .unwrap();
  assert_eq!(pos2, Position::new(9, 5));

  // Validate by Direction Target
  let pos3 = TargetingSystem::validate_target(
    game.world(),
    player_pos,
    Target::Direction(Direction::East),
    10,
  )
  .unwrap();
  assert_eq!(pos3, Position::new(6, 5));
}

#[test]
fn test_targeting_system_error_handling() {
  let mut game = Game::new(3333, 20, 20, Position::new(5, 5)).unwrap();
  let player_pos = Position::new(5, 5);

  // 1. Out of bounds
  let out_pos = Position::new(25, 25);
  let err_oob =
    TargetingSystem::validate_target(game.world(), player_pos, Target::Position(out_pos), 30)
      .unwrap_err();
  assert_eq!(err_oob, CommandError::OutOfBounds(out_pos));

  // 2. Out of range
  let far_pos = Position::new(15, 5);
  let err_range =
    TargetingSystem::validate_target(game.world(), player_pos, Target::Position(far_pos), 5)
      .unwrap_err();
  assert_eq!(err_range, CommandError::TargetOutOfRange(far_pos));

  // 3. Blocked by wall
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(7, 5), Tile::Wall);
  let blocked_pos = Position::new(9, 5);
  let err_blocked =
    TargetingSystem::validate_target(game.world(), player_pos, Target::Position(blocked_pos), 10)
      .unwrap_err();
  assert_eq!(err_blocked, CommandError::LineOfSightBlocked(blocked_pos));
}

#[test]
fn test_query_visible_targets_ordering() {
  let mut game = Game::new(3333, 20, 20, Position::new(5, 5)).unwrap();
  let player_pos = Position::new(5, 5);

  let m_far = game
    .world_mut()
    .spawn_monster_kind(Position::new(10, 5), MonsterKind::FormerHuman)
    .unwrap();
  let m_near = game
    .world_mut()
    .spawn_monster_kind(Position::new(7, 5), MonsterKind::Demon)
    .unwrap();
  let m_mid = game
    .world_mut()
    .spawn_monster_kind(Position::new(8, 5), MonsterKind::Imp)
    .unwrap();

  let visible_targets = TargetingSystem::find_visible_targets(game.world(), player_pos, 10);
  assert_eq!(visible_targets.len(), 3);
  // Nearest first: m_near (dist 2) -> m_mid (dist 3) -> m_far (dist 5)
  assert_eq!(visible_targets[0].0, m_near);
  assert_eq!(visible_targets[0].2, 2);
  assert_eq!(visible_targets[1].0, m_mid);
  assert_eq!(visible_targets[1].2, 3);
  assert_eq!(visible_targets[2].0, m_far);
  assert_eq!(visible_targets[2].2, 5);

  let nearest = TargetingSystem::find_nearest_target(game.world(), player_pos, 10);
  assert_eq!(nearest, Some(m_near));
}
