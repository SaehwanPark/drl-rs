//! End-to-end combat integration tests for DRL-Rust.

use drl_core::{Game, ReplayEngine};
use drl_protocol::{Command, CommandError, Direction, GameEvent, Position, ReplayLog, Turn};

#[test]
fn test_multi_turn_combat_encounter_and_death() {
  let mut game = Game::new(999, 12, 12, Position::new(2, 2)).expect("failed to init game");
  let monster_pos = Position::new(5, 2);

  let monster_id = game
    .world_mut()
    .spawn_monster(monster_pos, "Former Human", 10, 100, (2, 4))
    .expect("failed to spawn monster");

  // Initial turn 0: player fires ranged attack at monster at (5, 2)
  let events1 = game
    .step(Command::AttackRanged(monster_pos))
    .expect("ranged attack should succeed");

  assert!(events1.iter().any(|e| matches!(
    e,
    GameEvent::AttackResolved {
      target_id,
      is_ranged: true,
      ..
    } if *target_id == monster_id
  )));
  assert_eq!(game.turn(), Turn::new(1));

  // Monster takes its turn and moves towards player (from (5, 2) to (4, 2))
  let monster_current_pos = game
    .world()
    .get_actor(monster_id)
    .expect("monster exists")
    .position();
  assert_eq!(monster_current_pos, Position::new(4, 2));

  // Player steps closer (from (2, 2) to (3, 2))
  let events2 = game
    .step(Command::Move(Direction::East))
    .expect("move East should succeed");
  assert!(events2.iter().any(|e| matches!(
    e,
    GameEvent::EntityMoved {
      entity_id,
      from,
      to,
    } if *entity_id == game.world().player_id().unwrap() && *from == Position::new(2, 2) && *to == Position::new(3, 2)
  )));

  // Monster is now adjacent at (4, 2) vs player at (3, 2) and attacks player during monster turn
  assert!(events2.iter().any(|e| matches!(
    e,
    GameEvent::AttackResolved {
      attacker_id,
      target_id,
      is_ranged: false,
      ..
    } if *attacker_id == monster_id && *target_id == game.world().player_id().unwrap()
  )));

  // Player finishes monster with a melee bump attack moving East into (4, 2)
  let mut monster_dead = false;
  for _ in 0..10 {
    let events = game
      .step(Command::Move(Direction::East))
      .expect("melee attack should succeed");

    if events
      .iter()
      .any(|e| matches!(e, GameEvent::ActorDied { entity_id, .. } if *entity_id == monster_id))
    {
      monster_dead = true;
      break;
    }
  }

  assert!(
    monster_dead,
    "monster should eventually die from melee attacks"
  );

  // Dead monster no longer blocks movement -> Player can now step into (4, 2)
  let move_events = game
    .step(Command::Move(Direction::East))
    .expect("moving onto dead monster tile should succeed");

  assert!(move_events.iter().any(|e| matches!(
    e,
    GameEvent::EntityMoved {
      entity_id,
      to,
      ..
    } if *entity_id == game.world().player_id().unwrap() && *to == Position::new(4, 2)
  )));
}

#[test]
fn test_ranged_attack_error_cases() {
  let mut game = Game::new(1234, 15, 15, Position::new(2, 2)).expect("failed to init game");

  // Attack empty tile -> InvalidTarget
  let err = game
    .step(Command::AttackRanged(Position::new(3, 3)))
    .unwrap_err();
  assert_eq!(err, CommandError::InvalidTarget(Position::new(3, 3)));

  // Spawn monster far away at (12, 2) (distance = 10 > player ranged range 8)
  game
    .world_mut()
    .spawn_monster(Position::new(12, 2), "Distant Demon", 30, 100, (2, 4))
    .expect("failed to spawn");

  let err_range = game
    .step(Command::AttackRanged(Position::new(12, 2)))
    .unwrap_err();
  assert_eq!(
    err_range,
    CommandError::TargetOutOfRange(Position::new(12, 2))
  );
}

#[test]
fn test_combat_replay_determinism() {
  let seed = 777_888;
  let start_pos = Position::new(3, 3);
  let mut replay = ReplayLog::new(seed, 15, 15, start_pos);

  let commands = [
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Wait,
    Command::Move(Direction::South),
    Command::Move(Direction::NorthWest),
  ];

  for cmd in commands {
    replay.record_command(cmd);
  }

  let (game1, events1) = ReplayEngine::run(&replay).expect("replay run 1 failed");
  let (game2, events2) = ReplayEngine::run(&replay).expect("replay run 2 failed");

  assert_eq!(game1, game2);
  assert_eq!(events1, events2);
}
