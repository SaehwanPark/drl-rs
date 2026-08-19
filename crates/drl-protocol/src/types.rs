//! Core domain types: coordinates, directions, IDs, and turns.

use std::ops::Add;

/// 2D integer grid position in level coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Position {
  pub x: i32,
  pub y: i32,
}

impl Position {
  /// Creates a new grid position.
  #[must_use]
  pub const fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }

  /// Calculates the Chebyshev (8-way / chessboard) distance to another position.
  #[must_use]
  pub const fn distance_chebyshev(self, other: Self) -> u32 {
    let dx = (self.x - other.x).unsigned_abs();
    let dy = (self.y - other.y).unsigned_abs();
    if dx > dy { dx } else { dy }
  }

  /// Calculates the Manhattan (4-way / taxicab) distance to another position.
  #[must_use]
  pub const fn distance_manhattan(self, other: Self) -> u32 {
    let dx = (self.x - other.x).unsigned_abs();
    let dy = (self.y - other.y).unsigned_abs();
    dx + dy
  }

  /// Calculates the squared Euclidean distance to another position.
  #[must_use]
  pub const fn distance_squared(self, other: Self) -> u64 {
    let dx = (self.x - other.x) as i64;
    let dy = (self.y - other.y) as i64;
    (dx * dx + dy * dy) as u64
  }

  /// Offsets this position by delta x and delta y.
  #[must_use]
  pub const fn offset(self, dx: i32, dy: i32) -> Self {
    Self {
      x: self.x + dx,
      y: self.y + dy,
    }
  }

  /// Offsets this position by a directional step.
  #[must_use]
  pub const fn apply_direction(self, dir: Direction) -> Self {
    self.offset(dir.dx(), dir.dy())
  }
}

impl Add<Direction> for Position {
  type Output = Self;

  fn add(self, rhs: Direction) -> Self::Output {
    self.apply_direction(rhs)
  }
}

/// 8-way directional movement plus stationary `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Direction {
  #[default]
  None,
  North,
  NorthEast,
  East,
  SouthEast,
  South,
  SouthWest,
  West,
  NorthWest,
}

impl Direction {
  /// All eight standard movement directions.
  pub const ALL_8WAY: [Self; 8] = [
    Self::North,
    Self::NorthEast,
    Self::East,
    Self::SouthEast,
    Self::South,
    Self::SouthWest,
    Self::West,
    Self::NorthWest,
  ];

  /// The four cardinal directions.
  pub const ALL_CARDINAL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

  /// The four diagonal directions.
  pub const ALL_DIAGONAL: [Self; 4] = [
    Self::NorthEast,
    Self::SouthEast,
    Self::SouthWest,
    Self::NorthWest,
  ];

  /// Returns the X offset for this direction.
  #[must_use]
  pub const fn dx(self) -> i32 {
    match self {
      Self::None | Self::North | Self::South => 0,
      Self::East | Self::NorthEast | Self::SouthEast => 1,
      Self::West | Self::NorthWest | Self::SouthWest => -1,
    }
  }

  /// Returns the Y offset for this direction (North is -1 in 2D top-left grid coordinates).
  #[must_use]
  pub const fn dy(self) -> i32 {
    match self {
      Self::None | Self::East | Self::West => 0,
      Self::North | Self::NorthEast | Self::NorthWest => -1,
      Self::South | Self::SouthEast | Self::SouthWest => 1,
    }
  }

  /// Returns true if this is one of the four cardinal directions.
  #[must_use]
  pub const fn is_cardinal(self) -> bool {
    matches!(self, Self::North | Self::East | Self::South | Self::West)
  }

  /// Returns true if this is one of the four diagonal directions.
  #[must_use]
  pub const fn is_diagonal(self) -> bool {
    matches!(
      self,
      Self::NorthEast | Self::SouthEast | Self::SouthWest | Self::NorthWest
    )
  }

  /// Constructs a direction from normalized delta coordinates (-1, 0, 1).
  #[must_use]
  pub const fn from_delta(dx: i32, dy: i32) -> Option<Self> {
    let nx = if dx > 0 {
      1
    } else if dx < 0 {
      -1
    } else {
      0
    };
    let ny = if dy > 0 {
      1
    } else if dy < 0 {
      -1
    } else {
      0
    };
    match (nx, ny) {
      (0, 0) => Some(Self::None),
      (0, -1) => Some(Self::North),
      (1, -1) => Some(Self::NorthEast),
      (1, 0) => Some(Self::East),
      (1, 1) => Some(Self::SouthEast),
      (0, 1) => Some(Self::South),
      (-1, 1) => Some(Self::SouthWest),
      (-1, 0) => Some(Self::West),
      (-1, -1) => Some(Self::NorthWest),
      _ => None,
    }
  }
}

/// Simulation turn counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Turn {
  pub count: u64,
}

impl Turn {
  /// Creates a turn instance with a given counter value.
  #[must_use]
  pub const fn new(count: u64) -> Self {
    Self { count }
  }

  /// Turn 0 (initial turn).
  #[must_use]
  pub const fn zero() -> Self {
    Self { count: 0 }
  }

  /// Returns the next sequential turn.
  #[must_use]
  pub const fn next(self) -> Self {
    Self {
      count: self.count + 1,
    }
  }
}

/// Unique identifier for an entity/actor in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub u64);

impl EntityId {
  /// Creates a new entity ID.
  #[must_use]
  pub const fn new(id: u64) -> Self {
    Self(id)
  }

  /// Returns the raw u64 ID.
  #[must_use]
  pub const fn as_u64(self) -> u64 {
    self.0
  }
}

/// Unique identifier for an item instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId(pub u64);

impl ItemId {
  /// Creates a new item ID.
  #[must_use]
  pub const fn new(id: u64) -> Self {
    Self(id)
  }

  /// Returns the raw u64 ID.
  #[must_use]
  pub const fn as_u64(self) -> u64 {
    self.0
  }
}

/// Unique identifier for a dungeon level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LevelId(pub u32);

impl LevelId {
  /// Creates a new level ID.
  #[must_use]
  pub const fn new(id: u32) -> Self {
    Self(id)
  }

  /// Returns the raw u32 ID.
  #[must_use]
  pub const fn as_u32(self) -> u32 {
    self.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_position_arithmetic_and_distances() {
    let p1 = Position::new(2, 3);
    let p2 = Position::new(5, 7);

    assert_eq!(p1.distance_chebyshev(p2), 4);
    assert_eq!(p1.distance_manhattan(p2), 7);
    assert_eq!(p1.distance_squared(p2), 25);

    let moved = p1 + Direction::East;
    assert_eq!(moved, Position::new(3, 3));
    let moved_diag = p1 + Direction::NorthWest;
    assert_eq!(moved_diag, Position::new(1, 2));
  }

  #[test]
  fn test_direction_conversions() {
    assert_eq!(Direction::from_delta(0, -1), Some(Direction::North));
    assert_eq!(Direction::from_delta(1, 1), Some(Direction::SouthEast));
    assert_eq!(Direction::from_delta(0, 0), Some(Direction::None));
    assert_eq!(Direction::North.dx(), 0);
    assert_eq!(Direction::North.dy(), -1);
    assert!(Direction::North.is_cardinal());
    assert!(!Direction::North.is_diagonal());
    assert!(Direction::NorthEast.is_diagonal());
  }

  #[test]
  fn test_turn_counter() {
    let t0 = Turn::zero();
    let t1 = t0.next();
    assert_eq!(t0.count, 0);
    assert_eq!(t1.count, 1);
  }
}
