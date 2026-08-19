//! Shared semantic protocol contracts for DRL-Rust.
//!
//! Defines command, observation, and event schemas shared across
//! frontends, test agents, MCP, and the simulation core.

pub mod command;
pub mod event;
pub mod item;
pub mod observation;
pub mod replay;
pub mod types;

pub use command::{Command, CommandError};
pub use event::GameEvent;
pub use item::{AmmoType, EquipmentSlot, GroundItemView, ItemCategory, ItemView};
pub use observation::{
  ActorView, Observation, OmniscientObservation, PlayerObservation, TileKind, TileView,
};
pub use replay::{ItemSpawnKind, ItemSpawnSpec, MonsterSpawnSpec, ReplayLog};
pub use types::{
  ActionCost, AttackOutcome, DamageSource, DamageType, DeathCause, Direction, EntityId, HitPoints,
  ItemId, LevelId, MonsterKind, Position, Speed, Target, Turn,
};

/// Returns the protocol schema version for DRL-Rust.
#[must_use]
pub fn protocol_version() -> &'static str {
  "0.1.0"
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_protocol_version() {
    assert_eq!(protocol_version(), "0.1.0");
  }

  #[test]
  fn test_replay_log_recording() {
    let mut replay = ReplayLog::new(42, 80, 25, Position::new(10, 10));
    replay.record_command(Command::Move(Direction::East));
    replay.record_command(Command::Wait);
    assert_eq!(replay.commands.len(), 2);
    assert_eq!(replay.commands[0], Command::Move(Direction::East));
    assert_eq!(replay.commands[1], Command::Wait);
  }
}
