//! Simulation game events emitted during turn processing.

use crate::types::{ActionCost, AttackOutcome, DamageSource, DeathCause, EntityId, Position, Turn};

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
  /// An attack was resolved between an attacker and target.
  AttackResolved {
    attacker_id: EntityId,
    target_id: EntityId,
    outcome: AttackOutcome,
    is_ranged: bool,
  },
  /// Damage was applied to an entity.
  DamageApplied {
    target_id: EntityId,
    amount: u32,
    remaining_hp: u32,
    source: DamageSource,
  },
  /// An actor was destroyed or killed.
  ActorDied {
    entity_id: EntityId,
    cause: DeathCause,
  },
  /// An actor paid action cost / energy.
  ActionCostPaid {
    entity_id: EntityId,
    cost: ActionCost,
  },
  /// The current turn completed.
  TurnEnded { turn: Turn },
}
