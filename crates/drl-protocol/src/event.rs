//! Simulation game events emitted during turn processing.

use crate::types::{EntityId, Position, Turn};

/// Game event emitted deterministically by the simulation core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
  /// A new turn has started.
  TurnStarted { turn: Turn },
  /// An entity moved from one cell to another.
  EntityMoved {
    entity_id: EntityId,
    from: Position,
    to: Position,
  },
  /// An entity waited in place for a turn.
  EntityWaited {
    entity_id: EntityId,
    position: Position,
  },
  /// The current turn completed.
  TurnEnded { turn: Turn },
}
