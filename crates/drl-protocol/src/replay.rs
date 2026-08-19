//! Replay log schema for deterministic recording and playback.

use crate::command::Command;
use crate::types::Position;

/// Serialized log of a game session sufficient to reproduce the run deterministically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayLog {
  /// RNG seed used to initialize the simulation.
  pub seed: u64,
  /// Initial level map width.
  pub width: u32,
  /// Initial level map height.
  pub height: u32,
  /// Player starting position.
  pub player_start: Position,
  /// Ordered sequence of commands executed by the player.
  pub commands: Vec<Command>,
}

impl ReplayLog {
  /// Creates a new replay log instance.
  #[must_use]
  pub fn new(seed: u64, width: u32, height: u32, player_start: Position) -> Self {
    Self {
      seed,
      width,
      height,
      player_start,
      commands: Vec::new(),
    }
  }

  /// Appends a command to the log.
  pub fn record_command(&mut self, command: Command) {
    self.commands.push(command);
  }
}
