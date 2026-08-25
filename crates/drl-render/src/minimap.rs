//! Fair-observation minimap projection.
//!
//! The projection deliberately contains only explored tiles and actors already
//! present in the player's observation. It is renderer-neutral: a browser,
//! native frontend, or accessibility surface may choose its own geometry and
//! palette without gaining access to hidden simulation state.

use drl_protocol::{PlayerObservation, Position, TileKind};
use std::collections::BTreeSet;

/// Marker rendered on top of an explored minimap tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinimapMarker {
  /// The player's current position.
  Player,
  /// A non-player actor currently visible to the player.
  VisibleActor,
}

/// One explored minimap cell and its optional fair-observation marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimapCell {
  pub position: Position,
  pub tile_kind: TileKind,
  pub is_visible: bool,
  pub marker: Option<MinimapMarker>,
}

/// Deterministic renderer-neutral minimap data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimapState {
  pub map_width: u32,
  pub map_height: u32,
  pub cells: Vec<MinimapCell>,
}

impl MinimapState {
  /// Projects explored topology and currently visible actor markers.
  ///
  /// Duplicate tile records are resolved deterministically in favor of a
  /// visible record, then by tile-kind order. Entries outside the advertised
  /// map dimensions are ignored so malformed observations cannot produce
  /// impossible cells in a frontend.
  #[must_use]
  pub fn from_observation(observation: &PlayerObservation) -> Self {
    let mut cells: Vec<MinimapCell> = observation
      .visible_tiles
      .iter()
      .filter(|tile| in_bounds(tile.position, observation.map_width, observation.map_height))
      .map(|tile| MinimapCell {
        position: tile.position,
        tile_kind: tile.kind,
        is_visible: tile.is_visible,
        marker: None,
      })
      .collect();

    cells.sort_unstable_by(|left, right| {
      left
        .position
        .cmp(&right.position)
        .then_with(|| right.is_visible.cmp(&left.is_visible))
        .then_with(|| tile_kind_order(left.tile_kind).cmp(&tile_kind_order(right.tile_kind)))
    });
    cells.dedup_by_key(|cell| cell.position);

    let visible_actor_positions: BTreeSet<Position> = observation
      .visible_actors
      .iter()
      .filter(|actor| {
        !actor.is_player
          && in_bounds(
            actor.position,
            observation.map_width,
            observation.map_height,
          )
      })
      .map(|actor| actor.position)
      .collect();

    for cell in &mut cells {
      cell.marker = if cell.position == observation.player_position {
        Some(MinimapMarker::Player)
      } else if visible_actor_positions.contains(&cell.position) {
        Some(MinimapMarker::VisibleActor)
      } else {
        None
      };
    }

    Self {
      map_width: observation.map_width,
      map_height: observation.map_height,
      cells,
    }
  }
}

const fn in_bounds(position: Position, map_width: u32, map_height: u32) -> bool {
  position.x >= 0
    && position.y >= 0
    && (position.x as u64) < map_width as u64
    && (position.y as u64) < map_height as u64
}

const fn tile_kind_order(kind: TileKind) -> u8 {
  match kind {
    TileKind::Floor => 0,
    TileKind::Wall => 1,
    TileKind::DoorClosed => 2,
    TileKind::DoorOpen => 3,
    TileKind::StairsDown => 4,
    TileKind::Lava => 5,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use drl_protocol::{ActorView, EntityId, Speed, TileView, Turn};

  fn observation(
    map_width: u32,
    map_height: u32,
    player_position: Position,
    visible_tiles: Vec<TileView>,
    visible_actors: Vec<ActorView>,
  ) -> PlayerObservation {
    PlayerObservation {
      turn: Turn::zero(),
      map_width,
      map_height,
      player_position,
      player_hp: None,
      visible_tiles,
      visible_actors,
      inventory: Vec::new(),
      equipped_weapon: None,
      equipped_armor: None,
      ground_items: Vec::new(),
    }
  }

  fn tile(position: Position, kind: TileKind, is_visible: bool) -> TileView {
    let definition = kind.definition();
    TileView {
      position,
      kind,
      is_walkable: definition.is_walkable,
      is_transparent: definition.is_transparent,
      is_visible,
    }
  }

  fn actor(position: Position, is_player: bool) -> ActorView {
    ActorView {
      id: EntityId::new(position.x as u64 + position.y as u64 * 100),
      position,
      is_player,
      name: if is_player { "player" } else { "imp" }.to_string(),
      hp: None,
      is_alive: true,
      speed: Speed::new(100),
      monster_kind: None,
    }
  }

  #[test]
  fn projection_is_sorted_deduplicated_and_bounds_checked() {
    let state = MinimapState::from_observation(&observation(
      4,
      3,
      Position::new(1, 1),
      vec![
        tile(Position::new(3, 1), TileKind::Wall, false),
        tile(Position::new(1, 1), TileKind::Floor, false),
        tile(Position::new(1, 1), TileKind::Floor, true),
        tile(Position::new(-1, 0), TileKind::Wall, true),
        tile(Position::new(4, 0), TileKind::Wall, true),
      ],
      Vec::new(),
    ));

    assert_eq!(state.map_width, 4);
    assert_eq!(state.map_height, 3);
    assert_eq!(state.cells.len(), 2);
    assert_eq!(state.cells[0].position, Position::new(1, 1));
    assert!(state.cells[0].is_visible);
    assert_eq!(state.cells[0].marker, Some(MinimapMarker::Player));
    assert_eq!(state.cells[1].position, Position::new(3, 1));
    assert!(!state.cells[1].is_visible);
  }

  #[test]
  fn projection_marks_only_visible_actors_and_player_wins_precedence() {
    let state = MinimapState::from_observation(&observation(
      3,
      2,
      Position::new(1, 0),
      vec![
        tile(Position::new(0, 0), TileKind::Floor, true),
        tile(Position::new(1, 0), TileKind::Floor, true),
        tile(Position::new(2, 0), TileKind::Floor, true),
      ],
      vec![
        actor(Position::new(0, 0), false),
        actor(Position::new(1, 0), false),
        actor(Position::new(2, 1), false),
      ],
    ));

    assert_eq!(state.cells[0].marker, Some(MinimapMarker::VisibleActor));
    assert_eq!(state.cells[1].marker, Some(MinimapMarker::Player));
    assert_eq!(state.cells[2].marker, None);
  }
}
