//! Replay log schema for deterministic recording and playback.

use crate::command::{Command, CommandError};
use crate::item::ItemArchetype;
pub use crate::item::ItemSpawnKind;
use crate::observation::TileKind;
use crate::types::{Position, Turn};

/// Supported version identifiers for replay log schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReplayVersion {
  /// Initial stable schema format.
  V1 = 1,
  /// Schema format with explicit RNG sampling semantics metadata.
  V2 = 2,
}

/// Version of the bounded RNG sampling algorithms required by a replay.
pub const CURRENT_RNG_SAMPLING_SEMANTICS_VERSION: u32 = 1;

/// Gameplay semantics identifier expected by the current replay engine.
///
/// This advances independently from the wire/schema and RNG-sampling versions
/// when other deterministic simulation rules change. Version `130` includes
/// Blue Armor's typed Plasma mitigation on actor splash; version `129` includes
/// the bounded Rocket Launcher radius-4 actor splash; version `128` includes
/// the unified whole-rule chainfire state model for all six supported families;
/// version `127` includes
/// Chaingun's typed fourteenth-level six-projectile chainfire burst; version `126`
/// includes Chaingun's typed thirteenth-level six-projectile chainfire burst;
/// version `125` includes Chaingun's typed twelfth-level six-projectile chainfire burst;
/// version `124` includes Chaingun's typed eleventh-level six-projectile chainfire burst;
/// version `123` includes Chaingun's typed tenth-level six-projectile chainfire burst;
/// version `122` includes Chaingun's typed ninth-level six-projectile chainfire burst;
/// version `121` includes Chaingun's typed eighth-level six-projectile chainfire burst;
/// version `120` includes Chaingun's typed seventh-level six-projectile chainfire burst;
/// version `119` includes Chaingun's typed sixth-level six-projectile chainfire
/// burst; version `118`
/// includes Chaingun's typed fifth-level six-projectile chainfire burst;
/// version `117` includes Chaingun's typed fourth-level six-projectile
/// chainfire burst; version `116` includes Laser Rifle's typed seventh-level
/// seven-projectile chainfire burst;
/// version `115` includes Nuclear Plasma Rifle's typed seventh-level
/// nine-projectile chainfire burst; version `114` includes BFG 10K's typed twenty-first-level
/// seven-projectile chainfire burst; version `113`
/// includes BFG 10K's typed twentieth-level seven-projectile chainfire burst; version `112`
/// includes BFG 10K's typed nineteenth-level seven-projectile chainfire burst; version `111`
/// includes BFG 10K's typed eighteenth-level seven-projectile chainfire burst; version `110`
/// includes BFG 10K's typed seventeenth-level seven-projectile chainfire burst; version `109`
/// includes BFG 10K's typed sixteenth-level seven-projectile chainfire burst; version `108`
/// includes BFG 10K's typed fifteenth-level seven-projectile chainfire burst; version `107`
/// includes BFG 10K's typed fourteenth-level seven-projectile chainfire burst; version `106`
/// includes BFG 10K's typed thirteenth-level seven-projectile chainfire burst; version `105`
/// includes BFG 10K's typed twelfth-level seven-projectile chainfire burst; version `104`
/// includes BFG 10K's typed eleventh-level seven-projectile chainfire burst; version `103`
/// includes BFG 10K's typed tenth-level seven-projectile chainfire burst; version `102`
/// includes BFG 10K's typed ninth-level seven-projectile chainfire burst;
/// version `101` includes BFG 10K's typed eighth-level seven-projectile chainfire burst;
/// version `100` includes BFG 10K's typed seventh-level seven-projectile chainfire burst;
/// version `99` includes BFG 10K's typed sixth-level seven-projectile chainfire burst;
/// version `98` includes Nuclear Plasma Rifle's typed sixth-level
/// nine-projectile chainfire burst; version `97` includes Laser Rifle's typed
/// sixth-level seven-projectile chainfire burst; version `96` includes Laser
/// Rifle's typed fifth-level
/// seven-projectile chainfire burst; version `95` includes BFG 10K's typed fifth-level seven-projectile
/// chainfire burst; version `94` includes Nuclear Plasma Rifle's typed fifth-level
/// nine-projectile chainfire burst; version `93` includes Nuclear Plasma
/// Rifle's typed fourth-level nine-projectile chainfire burst; version `92`
/// includes BFG 10K's typed fourth-level seven-projectile chainfire burst;
/// version `91` includes Laser Rifle's typed fourth-level seven-projectile
/// chainfire burst; version `90` includes Laser Rifle's
/// typed third-level seven-projectile chainfire burst; version `89` includes
/// Laser Rifle's typed second-level five-projectile chainfire burst; version
/// `88` includes Plasma Rifle's typed
/// second-level six-projectile chainfire burst; version `87` includes
/// Minigun's typed third-level twelve-projectile chainfire burst; version `86`
/// includes Minigun's typed second-level eight-projectile chainfire burst;
/// version `85` includes Chaingun's typed third-level six-projectile chainfire burst; version `84`
/// includes Chaingun's typed second-level four-projectile chainfire burst; version `83`
/// includes Nuclear Plasma Rifle's typed third-level nine-projectile chainfire burst;
/// version `82` includes Nuclear Plasma Rifle's typed second-level
/// six-projectile chainfire burst; version `81` includes BFG 10K's typed third-level seven-projectile
/// chainfire burst; version `80` includes the typed second-level
/// five-projectile chainfire burst; version `79`
/// Nuclear BFG 9000's typed radius-8 ground-item destruction; version `78`
/// includes Standard BFG 9000's typed radius-8 ground-item destruction;
/// version `77` includes Nuclear BFG 9000's typed radius-8 actor-only splash;
/// version `76` includes Standard BFG 9000's typed radius-8 actor-only splash;
/// version `75` includes BFG 10K's typed radius-2 loose-ammo destruction;
/// version `74` includes its
/// typed radius-2 actor splash; version `73` includes its typed
/// first-level four-projectile chainfire burst; version `72` includes Nuclear
/// Plasma Rifle's typed first-level four-projectile chainfire
/// burst; version `71` includes Laser Rifle's typed first-level four-projectile
/// chainfire burst; version `70` includes Plasma Rifle's typed first-level
/// four-projectile chainfire burst; version `69` includes Minigun's typed
/// first-level six-projectile chainfire burst;
/// version `68` includes Chaingun's deterministic three-outcome completion
/// after lethal targets; version `67` includes its typed first-level
/// three-projectile chainfire burst, version `66` includes
/// Null Pointer's typed actor-only radius-1 splash, version `65`'s Railgun
/// typed ray/piercing traversal, version `64`'s Anti-Freak Jackal
/// ground-item destruction, version `63`'s typed splash knockback, version
/// `62`'s typed radius-1 splash fanout, version `61`'s typed
/// delayed-explosion schedule, version `60`'s typed
/// aimed-fire command (+3 accuracy, doubled action cost), version `59`'s
/// Nuclear Plasma Rifle typed six-projectile ordinary-fire volley and
/// six-cell aggregate cost, version `58`'s Trigun typed aimed-fire command
/// (+3 accuracy, doubled action cost), version `57`'s Plasma Rifle
/// six-projectile ordinary-fire volley and six-cell aggregate cost, and
/// version `56`'s Pistol, Combat Pistol, and
/// Blaster typed aimed-fire command (+3 accuracy, doubled action cost),
/// version `55`'s Combat Pistol aimed-fire command,
/// version `53`'s Laser Rifle five-projectile/five-cell ordinary-fire volley,
/// version 46's Null Pointer ten-cell ordinary-fire cost, version 45's Railgun
/// five-cell ordinary-fire cost, the version 44 Frag Shotgun's
/// two-round ordinary-fire cost, the version 43 Plasma Shotgun's three-cell
/// ordinary-fire cost, the version 42 Double Shotgun's two-projectile dual-shot
/// policy, version 41's typed exact-hit policies, BFG 10K's five-projectile
/// volley and delayed explosion schedules, standard and Nuclear BFG 9000's
/// forty-cell shot costs and delayed explosion schedules, Nuclear BFG 9000's
/// recharge/overload, Revenant's Launcher exact-hit resolution, Nuclear Plasma
/// first- through seventh-level chainfire plus alternate overload/recharge, the
/// `IF_NORELOAD` policy, Blaster recharge, Laser Rifle's first- through
/// seventh-level chainfire, the prior Malek's Armor, Missile Launcher, and
/// Combat Shotgun policies, and the typed ordinary-fire cost policies through
/// Laser Rifle.
pub const CURRENT_GAMEPLAY_SEMANTICS_VERSION: u32 = 130;

/// Procedural-generation semantics identifier expected for replays that carry
/// a procedural generation configuration. Version 2 includes the exact
/// integer-ratio room-connection branch introduced in 0.2.107.
pub const CURRENT_GENERATOR_SEMANTICS_VERSION: u32 = 2;

/// Ruleset/content identity expected by the current replay engine.
pub const CURRENT_RULESET_ID: &str = "drl-rs-ruleset-v1";

/// Fixed browser-session content identity expected by the current snapshot
/// decoder. This identifies the deterministic M4 arena and its initial
/// content, independently of the general ruleset identity.
pub const CURRENT_FIXED_CONTENT_ID: &str = "fixed-m4-v1";

/// Metadata header describing engine environment and replay context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayMetadata {
  /// Schema format version.
  pub version: ReplayVersion,
  /// Engine name string.
  pub engine_name: String,
  /// Engine crate version string.
  pub engine_version: String,
  /// Gameplay semantics version required to interpret the command history.
  /// Version 130 includes Blue Armor's typed Plasma mitigation on actor
  /// splash; version 129 includes the bounded Rocket Launcher radius-4 actor
  /// splash; version 128 includes the unified whole-rule chainfire state model
  /// for all
  /// six supported families; version 127 includes Chaingun's typed
  /// fourteenth-level six-projectile
  /// chainfire burst; version 126 includes Chaingun's typed thirteenth-level
  /// six-projectile chainfire burst; version 125 includes Chaingun's typed twelfth-level
  /// six-projectile chainfire burst; version 124 includes Chaingun's typed eleventh-level
  /// six-projectile chainfire burst; version 123 includes Chaingun's typed tenth-level
  /// six-projectile chainfire burst; version 122 includes Chaingun's typed ninth-level
  /// six-projectile chainfire burst; version 121 includes Chaingun's typed eighth-level
  /// six-projectile chainfire burst; version 120 includes Chaingun's typed seventh-level
  /// six-projectile chainfire burst; version 119 includes Chaingun's typed sixth-level
  /// six-projectile chainfire burst; version 118 includes Chaingun's typed fifth-level
  /// six-projectile chainfire burst; version 117 includes Chaingun's typed
  /// fourth-level six-projectile chainfire burst; version 116 includes Laser
  /// Rifle's typed seventh-level seven-projectile chainfire burst; version 115 includes Nuclear Plasma
  /// Rifle's typed seventh-level nine-projectile chainfire burst; version 97 includes Laser
  /// Rifle's typed sixth-level seven-projectile chainfire burst; version 96
  /// includes Laser Rifle's typed fifth-level seven-projectile chainfire burst;
  /// version 95 includes BFG 10K's typed
  /// fifth-level seven-projectile chainfire burst; version 94 includes Nuclear
  /// Plasma Rifle's typed fifth-level nine-projectile chainfire burst; version 93
  /// includes Nuclear Plasma Rifle's typed fourth-level nine-projectile
  /// chainfire burst; version 92 includes BFG 10K's typed fourth-level
  /// seven-projectile chainfire burst; version 91 includes Laser Rifle's typed
  /// fourth-level seven-projectile chainfire burst; version 90
  /// includes Laser Rifle's typed third-level seven-projectile chainfire burst;
  /// version 89 includes Laser Rifle's typed second-level five-projectile
  /// chainfire burst; version 88 includes Plasma Rifle's typed second-level
  /// six-projectile chainfire
  /// burst; version 87 includes Minigun's typed third-level twelve-projectile
  /// chainfire burst; version 86 includes Minigun's typed second-level
  /// eight-projectile chainfire burst; version 85 includes Chaingun's typed
  /// third-level six-projectile chainfire burst; version 84 includes Chaingun's typed second-level four-projectile
  /// chainfire burst; version 83 includes Nuclear Plasma Rifle's typed
  /// third-level nine-projectile chainfire burst; version 82 includes Nuclear Plasma
  /// Rifle's typed second-level six-projectile chainfire burst; version 81 includes BFG 10K's typed
  /// third-level seven-projectile chainfire burst; version 80 includes the
  /// typed second-level five-projectile chainfire burst; version 79 includes Nuclear BFG 9000's
  /// typed radius-8 ground-item destruction; version 78 includes Standard BFG 9000's typed radius-8
  /// ground-item destruction; version 77 includes Nuclear BFG 9000's typed
  /// radius-8 actor-only splash; version 76 includes Standard BFG 9000's typed
  /// radius-8 actor-only splash; version 75 includes BFG 10K's typed radius-2 loose-ammo
  /// destruction; version 74 includes its typed radius-2 actor splash; version 73 includes
  /// its typed first-level four-projectile chainfire burst; version 72 includes
  /// Nuclear Plasma Rifle's typed first-level four-projectile
  /// chainfire burst; version 71 includes Laser Rifle's typed
  /// first-level four-projectile chainfire burst; version 70 includes Plasma
  /// Rifle's typed first-level four-projectile chainfire burst; version 69
  /// includes Minigun's typed first-level six-projectile chainfire burst;
  /// version 68 includes Chaingun's
  /// deterministic three-outcome completion after lethal targets; version 67
  /// includes its typed first-level three-projectile chainfire burst; version
  /// 66 includes Null Pointer's typed actor-only radius-1 splash,
  /// version 65 includes Railgun's typed ray/piercing traversal, version 64
  /// includes Anti-Freak Jackal's typed ground-item destruction, version 63
  /// includes its typed splash knockback, version 62 includes its typed
  /// radius-1 splash fanout, version 61 includes its typed
  /// delayed-explosion schedule, version 60
  /// includes its typed aimed-fire command (+3 accuracy, doubled
  /// action cost), version 59 includes Nuclear Plasma Rifle's typed
  /// six-projectile ordinary-fire volley and six-cell aggregate cost, version
  /// 58 includes
  /// Trigun's typed aimed-fire command (+3 accuracy, doubled action cost),
  /// version 57's Plasma Rifle six-projectile ordinary-fire volley and
  /// six-cell aggregate cost, and version 56 includes the Pistol, Combat Pistol,
  /// and Blaster typed aimed-fire
  /// command (+3 accuracy, doubled action cost), version 55's Combat Pistol
  /// aimed-fire command, version 54's Pistol aimed-fire command, and version
  /// 53's Laser Rifle five-projectile/five-cell
  /// ordinary-fire volley, version 52's Chaingun four-projectile/four-round
  /// ordinary-fire volley, version 51's Minigun eight-projectile/eight-round
  /// ordinary-fire volley, version 50's Super Shotgun two-projectile/
  /// two-shell ordinary-fire volley, version 49's Mega Buster three-projectile/
  /// three-round ordinary-fire volley, version 48's Acid Spitter ten-rocket
  /// ordinary-fire cost, version 47's Tristar Blaster three-projectile/five-cell
  /// ordinary-fire volley, version 46's Null Pointer ten-cell ordinary-fire
  /// cost, version 45's Railgun five-cell ordinary-fire cost, the version 44
  /// Frag Shotgun's two-round ordinary-fire cost, the version 43 Plasma
  /// Shotgun's three-cell ordinary-fire cost, the version 42 Double
  /// Shotgun's two-projectile dual-shot policy, plus typed exact-hit policies,
  /// BFG 10K's five-projectile volley and delayed explosion schedules,
  /// standard and Nuclear BFG 9000's forty-cell shot costs and delayed
  /// explosion schedules, Nuclear BFG 9000's recharge/overload, Revenant's
  /// Launcher exact-hit resolution, Nuclear Plasma alternate overload/recharge,
  /// the `IF_NORELOAD` policy, Blaster recharge, and the prior Malek's Armor,
  /// Missile Launcher, and Combat Shotgun policies.
  pub gameplay_semantics_version: u32,
  /// RNG sampling semantics required to reproduce bounded random choices.
  pub rng_sampling_semantics_version: u32,
  /// Procedural-generation semantics required when reconstructing generated maps.
  pub generator_semantics_version: u32,
  /// Ruleset/content identity required to reconstruct initial state and policy.
  pub ruleset_id: String,
}

impl Default for ReplayMetadata {
  fn default() -> Self {
    Self {
      version: ReplayVersion::V2,
      engine_name: "drl-rs".to_string(),
      engine_version: env!("CARGO_PKG_VERSION").to_string(),
      gameplay_semantics_version: CURRENT_GAMEPLAY_SEMANTICS_VERSION,
      rng_sampling_semantics_version: CURRENT_RNG_SAMPLING_SEMANTICS_VERSION,
      generator_semantics_version: CURRENT_GENERATOR_SEMANTICS_VERSION,
      ruleset_id: CURRENT_RULESET_ID.to_string(),
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
  /// Optional initial durability for the equipped armor in deterministic
  /// fixtures; omitted means the factory's default durability.
  pub equipped_armor_durability: Option<u32>,
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
      equipped_armor_durability: None,
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

impl ItemSpawnKind {
  /// Returns the explicit stack count carried by a loose-ammo spawn.
  #[must_use]
  pub const fn stack_count(self) -> Option<u32> {
    match self {
      Self::Ammo9mm(count)
      | Self::AmmoShells(count)
      | Self::AmmoRockets(count)
      | Self::AmmoCells(count) => Some(count),
      _ => None,
    }
  }

  /// Reconstructs a spawn family from its stable archetype and optional ammo count.
  ///
  /// Ordinary families are resolved from [`Self::ALL`]. Loose-ammo families
  /// retain explicit count-sensitive branches because their payload is not
  /// represented by the normalized catalog value.
  #[must_use]
  pub fn from_archetype(archetype: ItemArchetype, count: Option<u32>) -> Option<Self> {
    match archetype {
      ItemArchetype::Ammo9mm => count.map(Self::Ammo9mm),
      ItemArchetype::AmmoShells => count.map(Self::AmmoShells),
      ItemArchetype::AmmoRockets => count.map(Self::AmmoRockets),
      ItemArchetype::AmmoCells => count.map(Self::AmmoCells),
      archetype => Self::ALL
        .iter()
        .copied()
        .find(|kind| kind.archetype() == archetype),
    }
  }
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
  /// Whether this target is a boss for target-dependent item behavior.
  pub is_boss: bool,
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
      is_boss: false,
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

  /// Marks this monster as a boss for target-dependent item behavior.
  #[must_use]
  pub const fn with_boss(mut self, is_boss: bool) -> Self {
    self.is_boss = is_boss;
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
      version: ReplayVersion::V2,
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn item_spawn_projection_preserves_stable_names_and_counts() {
    assert_eq!(ItemSpawnKind::Pistol.archetype().to_string(), "pistol");
    assert_eq!(ItemSpawnKind::Bfg10k.archetype().to_string(), "bfg_10k");
    assert_eq!(
      ItemSpawnKind::AmmoCells(20).archetype().to_string(),
      "ammo_cells"
    );
    assert_eq!(ItemSpawnKind::AmmoCells(20).stack_count(), Some(20));
    assert_eq!(ItemSpawnKind::AmmoPackCells.stack_count(), None);
    assert_eq!(ItemSpawnKind::PhaseDevice.stack_count(), None);
    assert_eq!(
      ItemSpawnKind::from_archetype(ItemArchetype::AmmoCells, Some(20)),
      Some(ItemSpawnKind::AmmoCells(20))
    );
    assert_eq!(
      ItemSpawnKind::from_archetype(ItemArchetype::AmmoCells, None),
      None
    );
    assert_eq!(
      ItemSpawnKind::from_archetype(ItemArchetype::Pistol, Some(20)),
      Some(ItemSpawnKind::Pistol)
    );
    for archetype in [
      ItemArchetype::Ammo9mm,
      ItemArchetype::AmmoShells,
      ItemArchetype::AmmoRockets,
      ItemArchetype::AmmoCells,
    ] {
      assert_eq!(ItemSpawnKind::from_archetype(archetype, None), None);
    }
    assert_eq!(
      ItemSpawnKind::from_archetype(ItemArchetype::Unknown, Some(20)),
      None
    );
  }

  #[test]
  fn spawn_catalog_covers_each_known_archetype_once() {
    let mut archetypes = Vec::with_capacity(ItemSpawnKind::ALL.len());
    for &kind in ItemSpawnKind::ALL {
      let archetype = kind.archetype();
      assert_ne!(archetype, ItemArchetype::Unknown);
      assert_eq!(
        archetype.requires_stack_count(),
        kind.stack_count().is_some(),
        "catalog stack-shape mismatch for {archetype:?}"
      );
      assert!(
        !archetypes.contains(&archetype),
        "duplicate spawn catalog archetype: {archetype:?}"
      );
      assert_eq!(
        ItemSpawnKind::from_archetype(archetype, kind.stack_count()),
        Some(kind)
      );
      archetypes.push(archetype);
    }
    assert_eq!(archetypes.len(), ItemArchetype::ALL.len() - 1);
  }

  #[test]
  fn spawn_catalog_preserves_stable_archetype_order() {
    assert_eq!(
      ItemArchetype::ALL.first().copied(),
      Some(ItemArchetype::Unknown)
    );
    assert_eq!(ItemSpawnKind::ALL.len() + 1, ItemArchetype::ALL.len());

    for (index, &kind) in ItemSpawnKind::ALL.iter().enumerate() {
      assert_eq!(
        kind.archetype(),
        ItemArchetype::ALL[index + 1],
        "spawn/archetype catalog order diverged at index {index}"
      );
    }
  }
}
