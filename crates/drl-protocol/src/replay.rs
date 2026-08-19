//! Replay log schema for deterministic recording and playback.

use crate::command::Command;
use crate::types::Position;

/// Specification for representative item spawns in replays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSpawnKind {
  Pistol,
  Shotgun,
  CombatKnife,
  Ammo9mm(u32),
  AmmoShells(u32),
  SmallMedPack,
  LargeMedPack,
  GreenArmor,
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
}

impl MonsterSpawnSpec {
  /// Creates a new monster spawn specification.
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
    }
  }
}

/// Serialized log of a game session sufficient to reproduce the run deterministically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayLog {
  /// RNG seed used to initialize the simulation.
  pub seed: u64,
  /// Initial level map width.
  pub width: u32,
  /// Initial level map height.
  pub height: u32,
  /// Player starting position.
  pub player_start: Position,
  /// Initial monsters spawned in the level prior to command execution.
  pub initial_monsters: Vec<MonsterSpawnSpec>,
  /// Initial items spawned on the ground prior to command execution.
  pub initial_items: Vec<ItemSpawnSpec>,
  /// Ordered sequence of commands executed by the player.
  pub commands: Vec<Command>,
}

impl ReplayLog {
  /// Creates a new replay log instance.
  #[must_use]
  pub fn new(seed: u64, width: u32, height: u32, player_start: Position) -> Self {
    Self {
      seed,
      width,
      height,
      player_start,
      initial_monsters: Vec::new(),
      initial_items: Vec::new(),
      commands: Vec::new(),
    }
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
