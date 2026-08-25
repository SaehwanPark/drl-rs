//! Observation models for player frontends, debug tools, bots, and MCP.

use crate::item::{GroundItemView, ItemView};
use crate::types::{EntityId, HitPoints, Position, Speed, Turn};

/// High-level semantic tile classification for rendering and observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TileKind {
  #[default]
  Floor,
  Wall,
  DoorClosed,
  DoorOpen,
  StairsDown,
  Lava,
}

/// Immutable semantic metadata for one current tile kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileDefinition {
  pub kind: TileKind,
  pub name: &'static str,
  pub is_walkable: bool,
  pub is_transparent: bool,
}

const TILE_DEFINITIONS: [TileDefinition; 6] = [
  TileDefinition {
    kind: TileKind::Floor,
    name: "Floor",
    is_walkable: true,
    is_transparent: true,
  },
  TileDefinition {
    kind: TileKind::Wall,
    name: "Wall",
    is_walkable: false,
    is_transparent: false,
  },
  TileDefinition {
    kind: TileKind::DoorClosed,
    name: "Door Closed",
    is_walkable: false,
    is_transparent: false,
  },
  TileDefinition {
    kind: TileKind::DoorOpen,
    name: "Door Open",
    is_walkable: true,
    is_transparent: true,
  },
  TileDefinition {
    kind: TileKind::StairsDown,
    name: "Stairs Down",
    is_walkable: true,
    is_transparent: true,
  },
  TileDefinition {
    kind: TileKind::Lava,
    name: "Lava",
    is_walkable: true,
    is_transparent: true,
  },
];

impl TileKind {
  /// Returns the immutable semantic definition for this tile kind.
  #[must_use]
  pub const fn definition(self) -> TileDefinition {
    match self {
      Self::Floor => TILE_DEFINITIONS[0],
      Self::Wall => TILE_DEFINITIONS[1],
      Self::DoorClosed => TILE_DEFINITIONS[2],
      Self::DoorOpen => TILE_DEFINITIONS[3],
      Self::StairsDown => TILE_DEFINITIONS[4],
      Self::Lava => TILE_DEFINITIONS[5],
    }
  }

  /// Returns true if this tile can be stepped onto.
  #[must_use]
  pub const fn is_walkable(self) -> bool {
    self.definition().is_walkable
  }

  /// Returns true if this tile transmits sight / light.
  #[must_use]
  pub const fn is_transparent(self) -> bool {
    self.definition().is_transparent
  }
}

/// View of a single map cell in an observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileView {
  pub position: Position,
  pub kind: TileKind,
  pub is_walkable: bool,
  pub is_transparent: bool,
  pub is_visible: bool,
}

/// View of an actor in an observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorView {
  pub id: EntityId,
  pub position: Position,
  pub is_player: bool,
  pub name: String,
  pub hp: Option<HitPoints>,
  pub is_alive: bool,
  pub speed: Speed,
  /// Stable monster classification, when this actor is not the player.
  pub monster_kind: Option<crate::types::MonsterKind>,
}

/// Player-centric observation containing only what the player perceives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerObservation {
  pub turn: Turn,
  /// Complete map dimensions needed to lay out a fair board renderer.
  pub map_width: u32,
  pub map_height: u32,
  pub player_position: Position,
  /// Player HP is exposed explicitly so a frontend never needs `World` access.
  pub player_hp: Option<HitPoints>,
  pub visible_tiles: Vec<TileView>,
  pub visible_actors: Vec<ActorView>,
  pub inventory: Vec<ItemView>,
  pub equipped_weapon: Option<ItemView>,
  pub equipped_armor: Option<ItemView>,
  pub ground_items: Vec<GroundItemView>,
}

/// Omniscient debug observation containing the complete world state snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmniscientObservation {
  pub turn: Turn,
  pub width: u32,
  pub height: u32,
  pub tiles: Vec<TileView>,
  pub actors: Vec<ActorView>,
  pub ground_items: Vec<GroundItemView>,
}

/// Semantic observation delivered to observers, frontends, bots, and MCP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Observation {
  Player(Box<PlayerObservation>),
  Omniscient(OmniscientObservation),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tile_definitions_preserve_current_semantics() {
    let expected = [
      (TileKind::Floor, "Floor", true, true),
      (TileKind::Wall, "Wall", false, false),
      (TileKind::DoorClosed, "Door Closed", false, false),
      (TileKind::DoorOpen, "Door Open", true, true),
      (TileKind::StairsDown, "Stairs Down", true, true),
      (TileKind::Lava, "Lava", true, true),
    ];

    for (kind, name, is_walkable, is_transparent) in expected {
      let definition = kind.definition();
      assert_eq!(definition.kind, kind);
      assert_eq!(definition.name, name);
      assert_eq!(definition.is_walkable, is_walkable);
      assert_eq!(definition.is_transparent, is_transparent);
      assert_eq!(kind.is_walkable(), is_walkable);
      assert_eq!(kind.is_transparent(), is_transparent);
    }
  }
}
