//! Application executable entry point and headless demo runner for DRL-Rust.

use drl_core::{Game, ReplayEngine};
use drl_protocol::{Command, Direction, Position, ReplayLog};

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
  let start_pos = Position::new(10, 5);

  println!("Starting headless simulation arena ({width}x{height}) with seed {seed}...");

  let mut game =
    Game::new(seed, width, height, start_pos).expect("failed to initialize game simulation");

  println!(
    "Turn {}: Player spawned at ({}, {})",
    game.turn().count,
    start_pos.x,
    start_pos.y
  );

  let commands = [
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Move(Direction::North),
    Command::Wait,
    Command::Move(Direction::SouthWest),
  ];

  let mut replay = ReplayLog::new(seed, width, height, start_pos);

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
