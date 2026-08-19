//! Observation models for player frontends, debug tools, bots, and MCP.

use crate::types::{EntityId, Position, Turn};

/// High-level semantic tile classification for rendering and observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TileKind {
  #[default]
  Floor,
  Wall,
  DoorClosed,
  DoorOpen,
  StairsDown,
}

impl TileKind {
  /// Returns true if this tile can be stepped onto.
  #[must_use]
  pub const fn is_walkable(self) -> bool {
    match self {
      Self::Floor | Self::DoorOpen | Self::StairsDown => true,
      Self::Wall | Self::DoorClosed => false,
    }
  }

  /// Returns true if this tile transmits sight / light.
  #[must_use]
  pub const fn is_transparent(self) -> bool {
    match self {
      Self::Floor | Self::DoorOpen | Self::StairsDown => true,
      Self::Wall | Self::DoorClosed => false,
    }
  }
}

/// View of a single map cell in an observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileView {
  pub position: Position,
  pub kind: TileKind,
  pub is_walkable: bool,
  pub is_transparent: bool,
}

/// View of an actor in an observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorView {
  pub id: EntityId,
  pub position: Position,
  pub is_player: bool,
  pub name: String,
}

/// Player-centric observation containing only what the player perceives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerObservation {
  pub turn: Turn,
  pub player_position: Position,
  pub visible_tiles: Vec<TileView>,
  pub visible_actors: Vec<ActorView>,
}

/// Omniscient debug observation containing the complete world state snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmniscientObservation {
  pub turn: Turn,
  pub width: u32,
  pub height: u32,
  pub tiles: Vec<TileView>,
  pub actors: Vec<ActorView>,
}

/// Semantic observation delivered to observers, frontends, bots, and MCP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Observation {
  Player(PlayerObservation),
  Omniscient(OmniscientObservation),
}
