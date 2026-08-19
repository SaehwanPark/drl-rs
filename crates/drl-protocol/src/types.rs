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

  /// Returns the next sequential level ID.
  #[must_use]
  pub const fn next(self) -> Self {
    Self(self.0 + 1)
  }
}

/// Actor health points representing current and maximum durability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HitPoints {
  pub current: u32,
  pub max: u32,
}

impl HitPoints {
  /// Creates hit points with explicit current and maximum values.
  #[must_use]
  pub const fn new(current: u32, max: u32) -> Self {
    let cur = if current > max { max } else { current };
    Self { current: cur, max }
  }

  /// Creates full hit points with current equal to max.
  #[must_use]
  pub const fn full(max: u32) -> Self {
    Self { current: max, max }
  }

  /// Returns true if hit points have reached zero (dead).
  #[must_use]
  pub const fn is_dead(&self) -> bool {
    self.current == 0
  }

  /// Returns true if current hit points are at maximum.
  #[must_use]
  pub const fn is_full(&self) -> bool {
    self.current >= self.max
  }

  /// Deducts damage from current hit points, clamping to zero.
  /// Returns the actual damage deducted.
  pub fn take_damage(&mut self, amount: u32) -> u32 {
    let damage = if amount >= self.current {
      self.current
    } else {
      amount
    };
    self.current -= damage;
    damage
  }

  /// Restores health up to maximum. Returns the actual amount healed.
  pub fn heal(&mut self, amount: u32) -> u32 {
    let missing = self.max.saturating_sub(self.current);
    let healed = if amount > missing { missing } else { amount };
    self.current += healed;
    healed
  }
}

/// Relative movement and action speed (percentage modifier, standard = 100).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Speed(pub u32);

impl Speed {
  /// Standard normal actor speed (100%).
  pub const NORMAL: Self = Self(100);

  /// Creates a speed instance with a percentage rating.
  #[must_use]
  pub const fn new(percentage: u32) -> Self {
    Self(percentage)
  }

  /// Raw percentage value.
  #[must_use]
  pub const fn as_u32(self) -> u32 {
    self.0
  }
}

impl Default for Speed {
  fn default() -> Self {
    Self::NORMAL
  }
}

/// Time / energy cost required to execute an action (standard action = 1000).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActionCost(pub u32);

impl ActionCost {
  /// Standard action cost units (1000).
  pub const STANDARD: Self = Self(1000);
  /// Standard movement cost.
  pub const MOVE: Self = Self(1000);
  /// Standard wait cost.
  pub const WAIT: Self = Self(1000);
  /// Standard melee attack cost.
  pub const MELEE_ATTACK: Self = Self(1000);
  /// Standard ranged attack cost.
  pub const RANGED_ATTACK: Self = Self(1000);

  /// Creates an action cost with custom units.
  #[must_use]
  pub const fn new(units: u32) -> Self {
    Self(units)
  }

  /// Raw cost in energy/time units.
  #[must_use]
  pub const fn as_u32(self) -> u32 {
    self.0
  }
}

impl Default for ActionCost {
  fn default() -> Self {
    Self::STANDARD
  }
}

/// Damage type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DamageType {
  #[default]
  Physical,
  Plasma,
  Acid,
  Fire,
}

/// Source that caused damage or destruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DamageSource {
  /// Damage originated from an actor's direct action.
  Actor(EntityId),
  /// Damage originated from environmental hazard or terrain.
  Environment,
}

/// Cause of actor death.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeathCause {
  /// Killed by a melee strike from an actor.
  MeleeAttack { attacker_id: EntityId },
  /// Killed by a ranged attack from an actor.
  RangedAttack { attacker_id: EntityId },
  /// Killed by environmental hazard.
  Environment,
}

/// Result of an attack action resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttackOutcome {
  /// Attack connected and dealt damage.
  Hit { damage: u32, is_lethal: bool },
  /// Attack missed the target.
  Miss,
  /// Attack was blocked or absorbed completely.
  Blocked,
}

/// Target specification for actions, targeting queries, and line-of-fire checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
  /// Targeted grid position.
  Position(Position),
  /// Targeted actor entity.
  Entity(EntityId),
  /// Directional target line.
  Direction(Direction),
}

/// Standard representative monster archetypes in DRL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MonsterKind {
  /// Pistol-wielding former human soldier.
  FormerHuman,
  /// Shotgun-wielding former sergeant.
  FormerSergeant,
  /// Demonic imp hurling fireballs and slashing in melee.
  Imp,
  /// Fast, aggressive pinky demon rushing into melee.
  Demon,
}

impl MonsterKind {
  /// Display name of the monster archetype.
  #[must_use]
  pub const fn name(self) -> &'static str {
    match self {
      Self::FormerHuman => "Former Human",
      Self::FormerSergeant => "Former Sergeant",
      Self::Imp => "Imp",
      Self::Demon => "Demon",
    }
  }

  /// Default max hit points for this archetype.
  #[must_use]
  pub const fn default_hp(self) -> u32 {
    match self {
      Self::FormerHuman => 15,
      Self::FormerSergeant => 25,
      Self::Imp => 30,
      Self::Demon => 45,
    }
  }

  /// Default speed rating percentage for this archetype.
  #[must_use]
  pub const fn default_speed(self) -> u32 {
    match self {
      Self::FormerHuman => 100,
      Self::FormerSergeant => 90,
      Self::Imp => 100,
      Self::Demon => 130,
    }
  }

  /// Default melee damage range `(min, max)`.
  #[must_use]
  pub const fn default_melee_damage(self) -> (u32, u32) {
    match self {
      Self::FormerHuman => (2, 4),
      Self::FormerSergeant => (3, 6),
      Self::Imp => (4, 8),
      Self::Demon => (8, 16),
    }
  }

  /// Default ranged damage range `(min, max)`, range, and accuracy, if any.
  #[must_use]
  pub const fn default_ranged_stats(self) -> Option<((u32, u32), u32, i32)> {
    match self {
      Self::FormerHuman => Some(((4, 8), 7, 65)),
      Self::FormerSergeant => Some(((8, 14), 5, 60)),
      Self::Imp => Some(((5, 10), 8, 70)),
      Self::Demon => None,
    }
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

  #[test]
  fn test_hit_points_mechanics() {
    let mut hp = HitPoints::full(50);
    assert_eq!(hp.current, 50);
    assert_eq!(hp.max, 50);
    assert!(hp.is_full());
    assert!(!hp.is_dead());

    let taken = hp.take_damage(20);
    assert_eq!(taken, 20);
    assert_eq!(hp.current, 30);
    assert!(!hp.is_full());

    let healed = hp.heal(10);
    assert_eq!(healed, 10);
    assert_eq!(hp.current, 40);

    let over_healed = hp.heal(100);
    assert_eq!(over_healed, 10);
    assert_eq!(hp.current, 50);

    let lethal = hp.take_damage(100);
    assert_eq!(lethal, 50);
    assert_eq!(hp.current, 0);
    assert!(hp.is_dead());
  }

  #[test]
  fn test_speed_and_action_cost_defaults() {
    let speed = Speed::default();
    assert_eq!(speed.as_u32(), 100);

    let cost = ActionCost::default();
    assert_eq!(cost.as_u32(), 1000);
  }

  #[test]
  fn test_target_and_monster_kind() {
    let t_pos = Target::Position(Position::new(5, 5));
    let t_ent = Target::Entity(EntityId::new(42));
    let t_dir = Target::Direction(Direction::East);
    assert_ne!(t_pos, t_dir);
    assert_ne!(t_pos, t_ent);

    assert_eq!(MonsterKind::FormerHuman.name(), "Former Human");
    assert_eq!(MonsterKind::FormerHuman.default_hp(), 15);
    assert_eq!(MonsterKind::Demon.name(), "Demon");
    assert_eq!(MonsterKind::Demon.default_speed(), 130);
    assert!(MonsterKind::Demon.default_ranged_stats().is_none());
    assert!(MonsterKind::Imp.default_ranged_stats().is_some());
  }
}
