//! Semantic command definitions and command failure errors.

use crate::types::{Direction, EntityId, Position};
use std::fmt;

/// Semantic player or actor command submitted to the simulation core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
  /// Move or attempt to step in a direction (initiates bump-attack if enemy present).
  Move(Direction),
  /// Direct melee attack in a direction.
  AttackMelee(Direction),
  /// Direct ranged attack targeting a grid position.
  AttackRanged(Position),
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
  /// Target position is out of range for the chosen weapon or action.
  TargetOutOfRange(Position),
  /// Target position contains no valid target.
  InvalidTarget(Position),
  /// Dead actor cannot perform actions.
  DeadActorCannotAct(EntityId),
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
      Self::TargetOutOfRange(pos) => {
        write!(f, "target position ({}, {}) is out of range", pos.x, pos.y)
      }
      Self::InvalidTarget(pos) => {
        write!(
          f,
          "target position ({}, {}) contains no valid target",
          pos.x, pos.y
        )
      }
      Self::DeadActorCannotAct(id) => {
        write!(f, "dead actor {} cannot perform actions", id.as_u64())
      }
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

    let dead = CommandError::DeadActorCannotAct(EntityId::new(7));
    assert_eq!(dead.to_string(), "dead actor 7 cannot perform actions");

    let out_range = CommandError::TargetOutOfRange(Position::new(10, 10));
    assert_eq!(
      out_range.to_string(),
      "target position (10, 10) is out of range"
    );
  }
}
