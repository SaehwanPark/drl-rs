//! Semantic command definitions and command failure errors.

use crate::types::{Direction, EntityId, Position};
use std::fmt;

/// Semantic player or actor command submitted to the simulation core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
  /// Move or attempt to step in a direction.
  Move(Direction),
  /// Wait in place for one turn.
  Wait,
}

/// Errors returned when a command fails validation or execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
  /// Target cell is outside the map boundaries.
  OutOfBounds(Position),
  /// Target cell is blocked by impassable terrain (e.g. wall).
  BlockedByTerrain(Position),
  /// Target cell is blocked by an existing entity.
  BlockedByEntity {
    position: Position,
    entity_id: EntityId,
  },
  /// Entity submitting the command does not exist in the world.
  EntityNotFound(EntityId),
  /// Direction supplied is invalid for the requested action.
  InvalidDirection(Direction),
  /// Generic command validation failure.
  InvalidCommand(String),
}

impl fmt::Display for CommandError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::OutOfBounds(pos) => {
        write!(f, "target position ({}, {}) is out of bounds", pos.x, pos.y)
      }
      Self::BlockedByTerrain(pos) => {
        write!(f, "position ({}, {}) is blocked by terrain", pos.x, pos.y)
      }
      Self::BlockedByEntity {
        position,
        entity_id,
      } => {
        write!(
          f,
          "position ({}, {}) is blocked by entity {}",
          position.x,
          position.y,
          entity_id.as_u64()
        )
      }
      Self::EntityNotFound(id) => write!(f, "entity {} was not found", id.as_u64()),
      Self::InvalidDirection(dir) => write!(f, "invalid direction: {dir:?}"),
      Self::InvalidCommand(msg) => write!(f, "invalid command: {msg}"),
    }
  }
}

impl std::error::Error for CommandError {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_command_error_formatting() {
    let err = CommandError::OutOfBounds(Position::new(-1, 5));
    assert_eq!(err.to_string(), "target position (-1, 5) is out of bounds");

    let blocked = CommandError::BlockedByEntity {
      position: Position::new(3, 4),
      entity_id: EntityId::new(42),
    };
    assert_eq!(
      blocked.to_string(),
      "position (3, 4) is blocked by entity 42"
    );
  }
}
