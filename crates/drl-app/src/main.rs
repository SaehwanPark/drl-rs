//! Application executable entry point and headless demo runner for DRL-Rust.

use drl_core::{Game, ReplayEngine};
use drl_protocol::{Command, Direction, MonsterSpawnSpec, Position, ReplayLog};

fn main() {
  println!(
    "DRL-Rust ({}, protocol {}) initialized.",
    drl_core::engine_name(),
    drl_protocol::protocol_version()
  );

  run_headless_demo();
}

fn run_headless_demo() {
  let seed = 42;
  let width = 20;
  let height = 10;
  let start_pos = Position::new(5, 5);

  println!("Starting headless simulation arena ({width}x{height}) with seed {seed}...");

  let mut game =
    Game::new(seed, width, height, start_pos).expect("failed to initialize game simulation");

  // Spawn representative monster (Former Human) at (8, 5)
  let monster_pos = Position::new(8, 5);
  let _monster_id = game
    .world_mut()
    .spawn_monster(monster_pos, "Former Human", 15, 100, (2, 4))
    .expect("failed to spawn monster");

  println!(
    "Turn {}: Player spawned at ({}, {}), Former Human spawned at ({}, {})",
    game.turn().count,
    start_pos.x,
    start_pos.y,
    monster_pos.x,
    monster_pos.y
  );

  let commands = [
    Command::AttackRanged(monster_pos),
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Move(Direction::East), // Melee bump attack against monster
    Command::Move(Direction::East), // Finish monster
    Command::Move(Direction::East), // Step onto defeated monster tile
  ];

  let mut replay = ReplayLog::new(seed, width, height, start_pos);
  replay.record_monster(MonsterSpawnSpec::new(
    monster_pos,
    "Former Human",
    15,
    100,
    (2, 4),
  ));

  for cmd in commands {
    replay.record_command(cmd);
    match game.step(cmd) {
      Ok(events) => {
        let p_pos = game
          .world()
          .player()
          .map_or(Position::new(0, 0), |p| p.position());
        println!(
          "Turn {}: Executed {:?} -> Player at ({}, {}), emitted {} event(s)",
          game.turn().count,
          cmd,
          p_pos.x,
          p_pos.y,
          events.len()
        );
        for event in &events {
          match event {
            drl_protocol::GameEvent::AttackResolved {
              attacker_id,
              target_id,
              outcome,
              is_ranged,
            } => {
              println!(
                "  -> Combat: Actor {} attacked Actor {} (ranged: {}) -> outcome: {:?}",
                attacker_id.as_u64(),
                target_id.as_u64(),
                is_ranged,
                outcome
              );
            }
            drl_protocol::GameEvent::DamageApplied {
              target_id,
              amount,
              remaining_hp,
              ..
            } => {
              println!(
                "  -> Damage: Actor {} took {} damage (remaining HP: {})",
                target_id.as_u64(),
                amount,
                remaining_hp
              );
            }
            drl_protocol::GameEvent::ActorDied { entity_id, cause } => {
              println!(
                "  -> Death: Actor {} died (cause: {:?})",
                entity_id.as_u64(),
                cause
              );
            }
            _ => {}
          }
        }
      }
      Err(err) => {
        println!("Command {:?} rejected: {err}", cmd);
      }
    }
  }

  println!("Verifying replay determinism from recorded command log...");
  let is_deterministic =
    ReplayEngine::verify_determinism(&replay).expect("replay verification failed");

  if is_deterministic {
    println!("Simulation determinism check PASSED: Replay yielded bit-for-bit identical state.");
  } else {
    eprintln!("Simulation determinism check FAILED!");
    std::process::exit(1);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_app_initialization() {
    assert_eq!(drl_core::engine_name(), "drl-core");
    assert_eq!(drl_protocol::protocol_version(), "0.1.0");
  }

  #[test]
  fn test_headless_demo_execution() {
    let seed = 123;
    let mut game = Game::new_arena(seed, 10, 10).unwrap();
    let events = game.step(Command::Move(Direction::North)).unwrap();
    assert!(!events.is_empty());
    assert_eq!(game.turn().count, 1);
  }
}
