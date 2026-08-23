//! Replay log schema for deterministic recording and playback.

use crate::command::{Command, CommandError};
use crate::observation::TileKind;
use crate::types::{Position, Turn};

/// Supported version identifiers for replay log schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReplayVersion {
  /// Initial stable schema format.
  V1 = 1,
}

/// Metadata header describing engine environment and replay context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayMetadata {
  /// Schema format version.
  pub version: ReplayVersion,
  /// Engine name string.
  pub engine_name: String,
  /// Engine crate version string.
  pub engine_version: String,
}

impl Default for ReplayMetadata {
  fn default() -> Self {
    Self {
      version: ReplayVersion::V1,
      engine_name: "DRL-Rust".to_string(),
      engine_version: env!("CARGO_PKG_VERSION").to_string(),
    }
  }
}

/// Configuration for player character starting stats and equipment in replays/scenarios.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerSpawnConfig {
  /// Current hit points.
  pub hp: u32,
  /// Maximum hit points.
  pub max_hp: u32,
  /// Movement/action speed.
  pub speed: u32,
  /// Initial items in backpack inventory.
  pub initial_items: Vec<ItemSpawnKind>,
  /// Weapon equipped in active weapon slot.
  pub equipped_weapon: Option<ItemSpawnKind>,
  /// Armor equipped in active armor slot.
  pub equipped_armor: Option<ItemSpawnKind>,
}

/// Procedural generator parameters needed to reconstruct an MCP replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProceduralGenerationConfig {
  /// Maximum number of rooms to generate.
  pub max_rooms: u32,
  /// Minimum generated room dimension.
  pub min_room_size: u32,
  /// Maximum generated room dimension.
  pub max_room_size: u32,
  /// Maximum monsters generated per room.
  pub max_monsters_per_room: u32,
  /// Maximum items generated per room.
  pub max_items_per_room: u32,
}

impl Default for PlayerSpawnConfig {
  fn default() -> Self {
    Self {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: None,
    }
  }
}

/// Rich diagnostic error capturing execution failure with turn and command index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayExecutionError {
  /// Turn counter when the command failed.
  pub turn: Turn,
  /// 0-based index of the command within `ReplayLog.commands`.
  pub command_index: usize,
  /// The command that produced the failure.
  pub command: Command,
  /// The underlying simulation error.
  pub error: CommandError,
}

impl core::fmt::Display for ReplayExecutionError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "Replay execution failed at turn {} (command #{} {:?}): {}",
      self.turn.count, self.command_index, self.command, self.error
    )
  }
}

impl std::error::Error for ReplayExecutionError {}

/// Specification for representative item spawns in replays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSpawnKind {
  Pistol,
  Shotgun,
  CombatKnife,
  Ammo9mm(u32),
  AmmoShells(u32),
  AmmoRockets(u32),
  AmmoCells(u32),
  SmallMedPack,
  LargeMedPack,
  GreenArmor,
  PhaseDevice,
}

/// Initial item spawn specification recorded in a replay log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSpawnSpec {
  pub position: Position,
  pub kind: ItemSpawnKind,
}

impl ItemSpawnSpec {
  /// Creates a new item spawn specification.
  #[must_use]
  pub const fn new(position: Position, kind: ItemSpawnKind) -> Self {
    Self { position, kind }
  }
}

/// Initial monster spawn specification recorded in a replay log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterSpawnSpec {
  pub position: Position,
  pub name: String,
  pub hp: u32,
  pub speed: u32,
  pub melee_damage: (u32, u32),
  pub ranged_damage: Option<(u32, u32)>,
  pub ranged_range: u32,
  pub accuracy: i32,
  pub death_drop: Option<ItemSpawnKind>,
}

impl MonsterSpawnSpec {
  /// Creates a new monster spawn specification with default melee combat stats.
  #[must_use]
  pub fn new(
    position: Position,
    name: impl Into<String>,
    hp: u32,
    speed: u32,
    melee_damage: (u32, u32),
  ) -> Self {
    Self {
      position,
      name: name.into(),
      hp,
      speed,
      melee_damage,
      ranged_damage: None,
      ranged_range: 0,
      accuracy: 65,
      death_drop: None,
    }
  }

  /// Sets ranged combat stats on this monster spawn specification.
  #[must_use]
  pub fn with_ranged_combat(mut self, damage: (u32, u32), range: u32, accuracy: i32) -> Self {
    self.ranged_damage = Some(damage);
    self.ranged_range = range;
    self.accuracy = accuracy;
    self
  }

  /// Sets the death loot drop specification.
  #[must_use]
  pub fn with_death_drop(mut self, drop: Option<ItemSpawnKind>) -> Self {
    self.death_drop = drop;
    self
  }
}

/// Serialized log of a game session sufficient to reproduce the run deterministically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayLog {
  /// Schema format version.
  pub version: ReplayVersion,
  /// Replay metadata header.
  pub metadata: ReplayMetadata,
  /// Optional custom player spawn configuration.
  pub player_config: Option<PlayerSpawnConfig>,
  /// Optional procedural generator parameters; absent for arena/scenario logs.
  pub procedural_config: Option<ProceduralGenerationConfig>,
  /// Optional MCP session turn limit captured for terminal-session restoration.
  pub max_turns: Option<u64>,
  /// RNG seed used to initialize the simulation.
  pub seed: u64,
  /// Initial level map width.
  pub width: u32,
  /// Initial level map height.
  pub height: u32,
  /// Player starting position.
  pub player_start: Position,
  /// Optional stairs down position placed on initial level.
  pub initial_stairs: Option<Position>,
  /// Initial monsters spawned in the level prior to command execution.
  pub initial_monsters: Vec<MonsterSpawnSpec>,
  /// Initial items spawned on the ground prior to command execution.
  pub initial_items: Vec<ItemSpawnSpec>,
  /// Optional explicit tile overrides (e.g. for custom scenario fixtures).
  pub custom_tiles: Vec<(Position, TileKind)>,
  /// Ordered sequence of commands executed by the player.
  pub commands: Vec<Command>,
}

impl ReplayLog {
  /// Creates a new replay log instance.
  #[must_use]
  pub fn new(seed: u64, width: u32, height: u32, player_start: Position) -> Self {
    Self {
      version: ReplayVersion::V1,
      metadata: ReplayMetadata::default(),
      player_config: None,
      procedural_config: None,
      max_turns: None,
      seed,
      width,
      height,
      player_start,
      initial_stairs: None,
      initial_monsters: Vec::new(),
      initial_items: Vec::new(),
      custom_tiles: Vec::new(),
      commands: Vec::new(),
    }
  }

  /// Sets custom player spawn configuration.
  #[must_use]
  pub fn with_player_config(mut self, config: PlayerSpawnConfig) -> Self {
    self.player_config = Some(config);
    self
  }

  /// Marks this replay as originating from procedural generation.
  #[must_use]
  pub fn with_procedural_config(mut self, config: ProceduralGenerationConfig) -> Self {
    self.procedural_config = Some(config);
    self
  }

  /// Records the optional MCP session turn limit for deterministic restore.
  #[must_use]
  pub fn with_max_turns(mut self, max_turns: Option<u64>) -> Self {
    self.max_turns = max_turns;
    self
  }

  /// Sets custom replay metadata header.
  #[must_use]
  pub fn with_metadata(mut self, metadata: ReplayMetadata) -> Self {
    self.metadata = metadata;
    self
  }

  /// Records an explicit custom tile override in the replay.
  pub fn record_tile(&mut self, position: Position, kind: TileKind) {
    self.custom_tiles.push((position, kind));
  }

  /// Records an initial down-stairs position in the replay.
  pub fn record_stairs(&mut self, position: Position) {
    self.initial_stairs = Some(position);
  }

  /// Records an initial monster spawn in the replay.
  pub fn record_monster(&mut self, monster: MonsterSpawnSpec) {
    self.initial_monsters.push(monster);
  }

  /// Records an initial ground item spawn in the replay.
  pub fn record_item(&mut self, item: ItemSpawnSpec) {
    self.initial_items.push(item);
  }

  /// Appends a command to the log.
  pub fn record_command(&mut self, command: Command) {
    self.commands.push(command);
  }
}
