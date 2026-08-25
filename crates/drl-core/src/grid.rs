//! 2D tile grid maps and tile representations.

use drl_protocol::{Position, TileDefinition, TileKind, TileView};

/// Internal map tile kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tile {
  #[default]
  Floor,
  Wall,
  DoorClosed,
  DoorOpen,
  StairsDown,
  Lava,
  Acid,
  Water,
  Mud,
}

impl Tile {
  /// Returns the immutable semantic definition for this tile.
  #[must_use]
  pub const fn definition(self) -> TileDefinition {
    self.to_kind().definition()
  }

  /// Converts internal tile representation into protocol `TileKind`.
  #[must_use]
  pub const fn to_kind(self) -> TileKind {
    match self {
      Self::Floor => TileKind::Floor,
      Self::Wall => TileKind::Wall,
      Self::DoorClosed => TileKind::DoorClosed,
      Self::DoorOpen => TileKind::DoorOpen,
      Self::StairsDown => TileKind::StairsDown,
      Self::Lava => TileKind::Lava,
      Self::Acid => TileKind::Acid,
      Self::Water => TileKind::Water,
      Self::Mud => TileKind::Mud,
    }
  }

  /// Returns true if an entity can walk onto this tile.
  #[must_use]
  pub const fn is_walkable(self) -> bool {
    self.definition().is_walkable
  }

  /// Returns true if light/vision passes through this tile.
  #[must_use]
  pub const fn is_transparent(self) -> bool {
    self.definition().is_transparent
  }
}

/// 2D tile map representing a level's physical terrain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
  width: u32,
  height: u32,
  tiles: Vec<Tile>,
}

impl Map {
  /// Creates a new map filled with the given default tile.
  #[must_use]
  pub fn new(width: u32, height: u32, default_tile: Tile) -> Self {
    let size = (width as usize) * (height as usize);
    Self {
      width,
      height,
      tiles: vec![default_tile; size],
    }
  }

  /// Creates a simple bounded arena: floor cells surrounded by border walls.
  #[must_use]
  pub fn simple_arena(width: u32, height: u32) -> Self {
    let mut map = Self::new(width, height, Tile::Floor);
    for x in 0..width {
      map.set_tile(Position::new(x as i32, 0), Tile::Wall);
      map.set_tile(Position::new(x as i32, (height - 1) as i32), Tile::Wall);
    }
    for y in 0..height {
      map.set_tile(Position::new(0, y as i32), Tile::Wall);
      map.set_tile(Position::new((width - 1) as i32, y as i32), Tile::Wall);
    }
    map
  }

  /// Map width in cells.
  #[must_use]
  pub const fn width(&self) -> u32 {
    self.width
  }

  /// Map height in cells.
  #[must_use]
  pub const fn height(&self) -> u32 {
    self.height
  }

  /// Returns true if the position is within the map boundaries.
  #[must_use]
  pub const fn is_in_bounds(&self, pos: Position) -> bool {
    pos.x >= 0 && pos.x < (self.width as i32) && pos.y >= 0 && pos.y < (self.height as i32)
  }

  /// Converts a 2D position into a 1D tile slice index.
  #[must_use]
  pub const fn index_of(&self, pos: Position) -> Option<usize> {
    if !self.is_in_bounds(pos) {
      return None;
    }
    Some((pos.y as usize) * (self.width as usize) + (pos.x as usize))
  }

  /// Converts a 1D tile slice index into a 2D position.
  #[must_use]
  pub const fn pos_of(&self, index: usize) -> Position {
    let x = (index % (self.width as usize)) as i32;
    let y = (index / (self.width as usize)) as i32;
    Position::new(x, y)
  }

  /// Retrieves the tile at the given grid position.
  #[must_use]
  pub fn get_tile(&self, pos: Position) -> Option<Tile> {
    self.index_of(pos).map(|idx| self.tiles[idx])
  }

  /// Sets the tile at the given grid position. Returns false if out of bounds.
  pub fn set_tile(&mut self, pos: Position, tile: Tile) -> bool {
    if let Some(idx) = self.index_of(pos) {
      self.tiles[idx] = tile;
      true
    } else {
      false
    }
  }

  /// Checks whether a cell is in bounds and walkable.
  #[must_use]
  pub fn is_walkable(&self, pos: Position) -> bool {
    self.get_tile(pos).is_some_and(Tile::is_walkable)
  }

  /// Checks whether a cell is in bounds and transparent.
  #[must_use]
  pub fn is_transparent(&self, pos: Position) -> bool {
    self.get_tile(pos).is_some_and(Tile::is_transparent)
  }

  /// Converts a single tile position to a `TileView`.
  #[must_use]
  pub fn to_tile_view(&self, pos: Position, is_visible: bool) -> Option<TileView> {
    self.get_tile(pos).map(|tile| TileView {
      position: pos,
      kind: tile.to_kind(),
      is_walkable: tile.is_walkable(),
      is_transparent: tile.is_transparent(),
      is_visible,
    })
  }

  /// Exports all tile views for observation snapshots.
  #[must_use]
  pub fn to_tile_views(&self, is_visible: bool) -> Vec<TileView> {
    self
      .tiles
      .iter()
      .enumerate()
      .map(|(idx, &tile)| TileView {
        position: self.pos_of(idx),
        kind: tile.to_kind(),
        is_walkable: tile.is_walkable(),
        is_transparent: tile.is_transparent(),
        is_visible,
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tile_definitions_preserve_all_current_semantics() {
    let expected = [
      (Tile::Floor, TileKind::Floor, true, true),
      (Tile::Wall, TileKind::Wall, false, false),
      (Tile::DoorClosed, TileKind::DoorClosed, false, false),
      (Tile::DoorOpen, TileKind::DoorOpen, true, true),
      (Tile::StairsDown, TileKind::StairsDown, true, true),
      (Tile::Lava, TileKind::Lava, true, true),
      (Tile::Acid, TileKind::Acid, true, true),
      (Tile::Water, TileKind::Water, true, true),
      (Tile::Mud, TileKind::Mud, true, true),
    ];

    for (tile, kind, is_walkable, is_transparent) in expected {
      let definition = tile.definition();
      assert_eq!(definition.kind, kind);
      assert_eq!(definition.is_walkable, is_walkable);
      assert_eq!(definition.is_transparent, is_transparent);
      assert_eq!(tile.to_kind(), kind);
      assert_eq!(tile.is_walkable(), is_walkable);
      assert_eq!(tile.is_transparent(), is_transparent);
    }
  }

  #[test]
  fn test_map_bounds_and_arena() {
    let map = Map::simple_arena(10, 10);
    assert_eq!(map.width(), 10);
    assert_eq!(map.height(), 10);

    // Border walls
    assert_eq!(map.get_tile(Position::new(0, 0)), Some(Tile::Wall));
    assert_eq!(map.get_tile(Position::new(9, 9)), Some(Tile::Wall));
    assert!(!map.is_walkable(Position::new(0, 0)));

    // Inside floor
    assert_eq!(map.get_tile(Position::new(5, 5)), Some(Tile::Floor));
    assert!(map.is_walkable(Position::new(5, 5)));

    // Outside bounds
    assert_eq!(map.get_tile(Position::new(-1, 0)), None);
    assert_eq!(map.get_tile(Position::new(10, 5)), None);
    assert!(!map.is_walkable(Position::new(-1, 0)));
  }
}
