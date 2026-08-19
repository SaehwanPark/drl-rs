//! End-to-end headless simulation integration tests.

use drl_core::{Game, ReplayEngine};
use drl_protocol::{Command, CommandError, Direction, Position, ReplayLog, Turn};

#[test]
fn test_deterministic_multi_step_scenario() {
  let mut game = Game::new_arena(1337, 20, 20).expect("failed to init game arena");
  let start_pos = game.world().player().expect("player missing").position();
  assert_eq!(start_pos, Position::new(10, 10));

  let commands = [
    Command::Move(Direction::North),
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Wait,
    Command::Move(Direction::SouthWest),
  ];

  for cmd in commands {
    let events = game.step(cmd).expect("step should succeed");
    assert!(!events.is_empty());
  }

  assert_eq!(game.turn(), Turn::new(5));
  let final_pos = game.world().player().expect("player missing").position();
  assert_eq!(final_pos, Position::new(11, 10));

  // Verify player observation
  let obs = game.observe_player();
  assert_eq!(obs.player_position, final_pos);
  assert_eq!(obs.turn, Turn::new(5));

  // Verify omniscient observation
  let omni = game.observe_omniscient();
  assert_eq!(omni.turn, Turn::new(5));
  assert_eq!(omni.width, 20);
  assert_eq!(omni.height, 20);
  assert_eq!(omni.actors.len(), 1);
}

#[test]
fn test_replay_verification_identical_state() {
  let mut replay = ReplayLog::new(424242, 15, 15, Position::new(7, 7));
  let script = [
    Command::Move(Direction::North),
    Command::Move(Direction::NorthEast),
    Command::Move(Direction::East),
    Command::Wait,
    Command::Move(Direction::South),
    Command::Move(Direction::West),
  ];

  for cmd in script {
    replay.record_command(cmd);
  }

  let (game1, events1) = ReplayEngine::run(&replay).expect("replay run 1 failed");
  let (game2, events2) = ReplayEngine::run(&replay).expect("replay run 2 failed");

  assert_eq!(game1, game2, "world state must be bit-for-bit identical");
  assert_eq!(events1, events2, "event logs must be bit-for-bit identical");
}

#[test]
fn test_collision_with_monster_blocking() {
  let mut game = Game::new_arena(123, 10, 10).expect("failed to init arena");
  let player_pos = game.world().player().expect("player missing").position();
  let monster_pos = player_pos + Direction::East;

  let monster_id = game
    .world_mut()
    .spawn_actor(monster_pos, "Former Human", false)
    .expect("failed to spawn monster");

  // Attempt to move into the cell occupied by the monster
  let err = game.step(Command::Move(Direction::East)).unwrap_err();
  assert_eq!(
    err,
    CommandError::BlockedByEntity {
      position: monster_pos,
      entity_id: monster_id,
    }
  );

  // Turn should NOT advance
  assert_eq!(game.turn(), Turn::zero());
  assert_eq!(game.world().player().unwrap().position(), player_pos);
}
