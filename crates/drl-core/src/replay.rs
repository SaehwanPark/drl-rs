//! Deterministic replay execution engine.

use crate::game::Game;
use drl_protocol::{CommandError, GameEvent, ReplayLog};

/// Engine for replaying recorded game sessions deterministically.
pub struct ReplayEngine;

impl ReplayEngine {
  /// Executes a full replay log from its recorded seed and start configuration.
  ///
  /// Returns the final `Game` state and all accumulated `GameEvent`s.
  pub fn run(replay: &ReplayLog) -> Result<(Game, Vec<GameEvent>), CommandError> {
    let mut game = Game::new(
      replay.seed,
      replay.width,
      replay.height,
      replay.player_start,
    )?;

    let mut all_events = Vec::new();
    for &cmd in &replay.commands {
      let step_events = game.step(cmd)?;
      all_events.extend(step_events);
    }

    Ok((game, all_events))
  }

  /// Runs a replay twice independently and verifies that the resulting game state
  /// and event logs are identical.
  pub fn verify_determinism(replay: &ReplayLog) -> Result<bool, CommandError> {
    let (game1, events1) = Self::run(replay)?;
    let (game2, events2) = Self::run(replay)?;

    Ok(game1 == game2 && events1 == events2)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use drl_protocol::{Command, Direction, Position};

  #[test]
  fn test_replay_determinism() {
    let mut replay = ReplayLog::new(98765, 20, 20, Position::new(10, 10));
    replay.record_command(Command::Move(Direction::East));
    replay.record_command(Command::Move(Direction::East));
    replay.record_command(Command::Move(Direction::South));
    replay.record_command(Command::Wait);
    replay.record_command(Command::Move(Direction::NorthWest));

    let is_deterministic = ReplayEngine::verify_determinism(&replay).unwrap();
    assert!(is_deterministic);
  }
}
