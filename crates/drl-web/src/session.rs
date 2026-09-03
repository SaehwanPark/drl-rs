//! `BrowserSession`: the browser-facing transactional session boundary over the
//! deterministic `Game`, its fair observations, presentation steps, particle
//! decals, and replay log.

use drl_core::item::Item;
use drl_core::{Game, Tile, chainfire_profile};
use drl_protocol::{
  Command, ItemArchetype, ItemSpawnKind, ItemSpawnSpec, MonsterKind, MonsterSpawnSpec,
  PlayerObservation, Position, ReplayLog,
};
use drl_render::{
  ParticleDecalSprite, ParticleDecalStorageError, ParticleDecalStore, PresentationStep,
  RenderScene, effect_timeline_for_observations,
};

use crate::persistence::{self, SnapshotError};

pub(crate) fn chainfire_ammo_cost(archetype: ItemArchetype, level: u8) -> Option<u32> {
  chainfire_profile(archetype, level).map(|burst| burst.ammo_cost())
}

/// Fixed deterministic content slice used by the first browser playthrough.
pub const M4_SEED: u64 = 0x4452_4c5f_4d34;
pub const M4_WIDTH: u32 = 24;
pub const M4_HEIGHT: u32 = 16;
pub const M4_START: Position = Position::new(4, 8);

/// A browser-facing simulation session with transactional command handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserSession {
  pub(crate) game: Game,
  last_error: Option<String>,
  pub(crate) commands: Vec<Command>,
  particle_decals: ParticleDecalStore,
  particle_decal_sprites: Vec<ParticleDecalSprite>,
}

impl BrowserSession {
  /// Creates the fixed M4 arena and its representative loot/combat content.
  pub fn new() -> Result<Self, drl_protocol::CommandError> {
    Ok(Self::from_game(Self::fixed_game()?))
  }

  /// Wraps an already-instantiated deterministic game at the browser boundary.
  ///
  /// The helper keeps browser presentation tests on the same authoritative
  /// `Game` state without adding a second scenario or replay representation.
  pub(crate) fn from_game(game: Game) -> Self {
    Self {
      game,
      last_error: None,
      commands: Vec::new(),
      particle_decals: ParticleDecalStore::new(256),
      particle_decal_sprites: Vec::new(),
    }
  }

  /// Builds the same fixed content for direct-core parity tests and tools.
  pub fn fixed_game() -> Result<Game, drl_protocol::CommandError> {
    let mut game = Game::new(M4_SEED, M4_WIDTH, M4_HEIGHT, M4_START)?;
    let stairs = Position::new(19, 8);
    game
      .world_mut()
      .map_mut()
      .set_tile(stairs, Tile::StairsDown);

    let loot_position = Position::new(7, 8);
    for kind in [
      drl_protocol::ItemSpawnKind::Shotgun,
      drl_protocol::ItemSpawnKind::GreenArmor,
      drl_protocol::ItemSpawnKind::SmallMedPack,
    ] {
      let id = game.world_mut().allocate_item_id();
      game
        .world_mut()
        .spawn_ground_item(loot_position, Item::from_spawn_kind(id, kind))?;
    }

    let monster_position = Position::new(13, 8);
    let id = game.world_mut().allocate_entity_id();
    let monster = drl_core::Actor::from_monster_kind(id, monster_position, MonsterKind::Imp);
    game.world_mut().actors_mut().insert(id, monster);
    Ok(game)
  }

  /// Returns the current fair player observation.
  #[must_use]
  pub fn observation(&self) -> PlayerObservation {
    self.game.observe_player()
  }

  /// Returns the current render scene derived from the fair observation.
  #[must_use]
  pub fn scene(&self) -> RenderScene {
    RenderScene::from_observation(&self.observation())
  }

  /// Returns retained presentation-only decal requests for the browser pass.
  #[must_use]
  pub fn particle_decal_store(&self) -> &ParticleDecalStore {
    &self.particle_decals
  }

  /// Returns the caller-owned opaque sprite-handle descriptor table.
  #[must_use]
  pub fn particle_decal_sprites(&self) -> &[ParticleDecalSprite] {
    &self.particle_decal_sprites
  }

  /// Retains one presentation-only decal request without touching gameplay.
  pub fn try_insert_particle_decal(
    &mut self,
    insertion: drl_render::ParticleDecalInsertion,
  ) -> Result<(), ParticleDecalStorageError> {
    self.particle_decals.try_insert(insertion)
  }

  /// Replaces the caller-owned descriptor table used by decal rendering.
  pub fn set_particle_decal_sprites(&mut self, sprites: Vec<ParticleDecalSprite>) {
    self.particle_decal_sprites = sprites;
  }

  /// Returns the most recent rejected-command message, if any.
  #[must_use]
  pub fn last_error(&self) -> Option<&str> {
    self.last_error.as_deref()
  }

  /// Returns true after the deterministic session reaches player death.
  #[must_use]
  pub fn is_game_over(&self) -> bool {
    self.game.is_game_over()
  }

  /// Submits one semantic command. Core `Game::step` owns simulation rollback;
  /// this boundary only records presentation state after an accepted command
  /// and the most recent error after a rejection.
  pub fn submit(&mut self, command: Command) -> Result<PresentationStep, String> {
    let before = self.observation();
    match self.game.step(command) {
      Ok(events) => {
        self.last_error = None;
        self.commands.push(command);
        let after = self.observation();
        let effects = effect_timeline_for_observations(&before, &after, &events);
        Ok(PresentationStep {
          before,
          command,
          events,
          effects,
          after,
        })
      }
      Err(error) => {
        let message = error.to_string();
        self.last_error = Some(message.clone());
        Err(message)
      }
    }
  }

  /// Restarts the deterministic M4 session.
  pub fn restart(&mut self) -> Result<(), drl_protocol::CommandError> {
    *self = Self::new()?;
    Ok(())
  }

  /// Encodes successful fixed-session commands into a versioned save token.
  pub fn snapshot_token(&self) -> Result<String, SnapshotError> {
    persistence::encode_snapshot(&self.commands)
  }

  /// Rebuilds this session from a versioned token without exposing game state.
  pub fn restore_snapshot(&mut self, token: &str) -> Result<(), SnapshotError> {
    self.restore_snapshot_with_format(token).map(|_| ())
  }

  pub(crate) fn restore_snapshot_with_format(
    &mut self,
    token: &str,
  ) -> Result<persistence::SnapshotFormat, SnapshotError> {
    let decoded = persistence::decode_snapshot_with_format(token)?;
    let mut restored =
      Self::new().map_err(|error| SnapshotError::Initialization(error.to_string()))?;
    for command in decoded.commands {
      restored
        .submit(command)
        .map_err(SnapshotError::CommandRejected)?;
    }
    let format = decoded.format;
    *self = restored;
    Ok(format)
  }

  /// Returns a replay-schema representation of the fixed browser session.
  ///
  /// The log uses the existing versioned replay schema; it does not create a
  /// browser-specific wire format or expose authoritative state to JavaScript.
  #[must_use]
  pub fn replay_log(&self) -> ReplayLog {
    let mut replay = ReplayLog::new(M4_SEED, M4_WIDTH, M4_HEIGHT, M4_START);
    replay.record_stairs(Position::new(19, 8));
    replay.record_monster(
      MonsterSpawnSpec::new(
        Position::new(13, 8),
        "Imp",
        MonsterKind::Imp.default_hp(),
        MonsterKind::Imp.default_speed(),
        MonsterKind::Imp.default_melee_damage(),
      )
      .with_ranged_combat((5, 10), 8, 70)
      .with_death_drop(Some(ItemSpawnKind::SmallMedPack)),
    );
    let loot_position = Position::new(7, 8);
    replay.record_item(ItemSpawnSpec::new(loot_position, ItemSpawnKind::Shotgun));
    replay.record_item(ItemSpawnSpec::new(loot_position, ItemSpawnKind::GreenArmor));
    replay.record_item(ItemSpawnSpec::new(
      loot_position,
      ItemSpawnKind::SmallMedPack,
    ));
    for command in &self.commands {
      replay.record_command(*command);
    }
    replay
  }
}
