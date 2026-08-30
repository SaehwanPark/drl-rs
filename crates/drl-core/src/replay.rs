//! Deterministic replay execution engine and diagnostics.

use crate::game::Game;
use crate::generator::LevelGeneratorConfig;
use crate::item::Item;
use crate::scheduler::ACTION_THRESHOLD;
use drl_protocol::{
  Command, CommandError, EpisodeMetrics, EquipmentSlot, GameEvent, HitPoints, MonsterKind,
  Position, ReplayExecutionError, ReplayLog, RunOutcome, Speed, Turn,
};

/// Shared replay bounds enforced by the MCP decoder and direct core path.
const MIN_REPLAY_DIMENSION: u32 = 3;
const MAX_REPLAY_DIMENSION: u32 = 512;
const MAX_INITIAL_ENTITIES: usize = 4_096;
const MAX_CUSTOM_TILES: usize = 65_536;
const MAX_COMMANDS: usize = 100_000;
const MAX_PROCEDURAL_ROOMS: u32 = 64;
const MAX_ROOM_SIZE: u32 = 64;
const MAX_CONTENT_PER_ROOM: u32 = 64;

/// Engine for replaying recorded game sessions deterministically with rich diagnostics.
pub struct ReplayEngine;

impl ReplayEngine {
  /// Validates a replay log's schema headers, spatial/structural bounds, and
  /// consistency before execution.
  ///
  /// Custom tile overrides are checked here so execution cannot silently drop
  /// an out-of-bounds map mutation.
  pub fn validate(replay: &ReplayLog) -> Result<(), String> {
    if replay.version != drl_protocol::ReplayVersion::V2 {
      return Err(format!(
        "unsupported replay schema version {:?}; expected V2",
        replay.version
      ));
    }
    if replay.metadata.version != replay.version {
      return Err(format!(
        "replay schema version {:?} does not match metadata version {:?}",
        replay.version, replay.metadata.version
      ));
    }
    if replay.metadata.gameplay_semantics_version
      != drl_protocol::CURRENT_GAMEPLAY_SEMANTICS_VERSION
    {
      return Err(format!(
        "unsupported gameplay semantics version {}; expected {}",
        replay.metadata.gameplay_semantics_version,
        drl_protocol::CURRENT_GAMEPLAY_SEMANTICS_VERSION
      ));
    }
    if replay.metadata.rng_sampling_semantics_version
      != drl_protocol::CURRENT_RNG_SAMPLING_SEMANTICS_VERSION
    {
      return Err(format!(
        "unsupported RNG sampling semantics version {}; expected {}",
        replay.metadata.rng_sampling_semantics_version,
        drl_protocol::CURRENT_RNG_SAMPLING_SEMANTICS_VERSION
      ));
    }
    if replay.metadata.ruleset_id != drl_protocol::CURRENT_RULESET_ID
      && replay.metadata.ruleset_id != "drl-rust-ruleset-v1"
    {
      return Err(format!(
        "unsupported replay ruleset {:?}; expected {:?}",
        replay.metadata.ruleset_id,
        drl_protocol::CURRENT_RULESET_ID
      ));
    }
    if replay.procedural_config.is_some()
      && replay.metadata.generator_semantics_version
        != drl_protocol::CURRENT_GENERATOR_SEMANTICS_VERSION
    {
      return Err(format!(
        "unsupported generator semantics version {}; expected {}",
        replay.metadata.generator_semantics_version,
        drl_protocol::CURRENT_GENERATOR_SEMANTICS_VERSION
      ));
    }
    validate_replay_structure(replay)?;
    if !(MIN_REPLAY_DIMENSION..=MAX_REPLAY_DIMENSION).contains(&replay.width)
      || !(MIN_REPLAY_DIMENSION..=MAX_REPLAY_DIMENSION).contains(&replay.height)
    {
      return Err(format!(
        "Invalid map dimensions: {}x{}; expected {}..={}",
        replay.width, replay.height, MIN_REPLAY_DIMENSION, MAX_REPLAY_DIMENSION
      ));
    }

    let is_in_bounds = |pos: Position| -> bool {
      pos.x >= 0 && pos.x < replay.width as i32 && pos.y >= 0 && pos.y < replay.height as i32
    };

    if !is_in_bounds(replay.player_start) {
      return Err(format!(
        "Player start position {:?} is out of map bounds ({}x{})",
        replay.player_start, replay.width, replay.height
      ));
    }

    if let Some(stairs) = replay.initial_stairs
      && !is_in_bounds(stairs)
    {
      return Err(format!("Stairs position {:?} is out of map bounds", stairs));
    }

    for monster in &replay.initial_monsters {
      if !is_in_bounds(monster.position) {
        return Err(format!(
          "Monster '{}' position {:?} is out of map bounds",
          monster.name, monster.position
        ));
      }
    }

    for item in &replay.initial_items {
      if !is_in_bounds(item.position) {
        return Err(format!(
          "Item position {:?} is out of map bounds",
          item.position
        ));
      }
    }

    for &(position, _) in &replay.custom_tiles {
      if !is_in_bounds(position) {
        return Err(format!(
          "Custom tile position {:?} is out of map bounds",
          position
        ));
      }
    }

    Ok(())
  }

  /// Executes a full replay log with step-by-step diagnostics and telemetry collection.
  pub fn run_with_diagnostics(
    replay: &ReplayLog,
  ) -> Result<(Game, Vec<GameEvent>, EpisodeMetrics), ReplayExecutionError> {
    Self::validate(replay).map_err(|msg| ReplayExecutionError {
      turn: Turn::zero(),
      command_index: 0,
      command: Command::Wait,
      error: CommandError::InvalidCommand(msg),
    })?;

    let mut game = if let Some(config) = &replay.procedural_config {
      Game::new_procedural(
        replay.seed,
        LevelGeneratorConfig {
          width: replay.width,
          height: replay.height,
          max_rooms: config.max_rooms,
          min_room_size: config.min_room_size,
          max_room_size: config.max_room_size,
          max_monsters_per_room: config.max_monsters_per_room,
          max_items_per_room: config.max_items_per_room,
        },
      )
    } else {
      Game::new(
        replay.seed,
        replay.width,
        replay.height,
        replay.player_start,
      )
    }
    .map_err(|err| ReplayExecutionError {
      turn: Turn::zero(),
      command_index: 0,
      command: Command::Wait,
      error: err,
    })?;

    let player_id = game
      .world()
      .player_id()
      .unwrap_or(drl_protocol::EntityId(1));

    for &(pos, kind) in &replay.custom_tiles {
      let tile = match kind {
        drl_protocol::TileKind::Wall => crate::grid::Tile::Wall,
        drl_protocol::TileKind::Floor => crate::grid::Tile::Floor,
        drl_protocol::TileKind::StairsDown => crate::grid::Tile::StairsDown,
        drl_protocol::TileKind::DoorClosed => crate::grid::Tile::DoorClosed,
        drl_protocol::TileKind::DoorOpen => crate::grid::Tile::DoorOpen,
        drl_protocol::TileKind::Lava => crate::grid::Tile::Lava,
        drl_protocol::TileKind::Acid => crate::grid::Tile::Acid,
        drl_protocol::TileKind::Water => crate::grid::Tile::Water,
        drl_protocol::TileKind::Mud => crate::grid::Tile::Mud,
      };
      game.world_mut().map_mut().set_tile(pos, tile);
    }

    if let Some(stairs_pos) = replay.initial_stairs {
      game
        .world_mut()
        .map_mut()
        .set_tile(stairs_pos, crate::grid::Tile::StairsDown);
    }

    if let Some(config) = &replay.player_config {
      if let Some(player) = game.world_mut().get_actor_mut(player_id) {
        let updated = player.clone().with_stats(
          HitPoints::new(config.hp, config.max_hp),
          Speed::new(config.speed),
          (2, 5),
          None,
          0,
          75,
        );
        *player = updated;
        player.set_energy(ACTION_THRESHOLD);
        *player.equipment_mut() = crate::inventory::Equipment::new();
        *player.inventory_mut() = crate::inventory::Inventory::new(10);
      }

      for &item_kind in &config.initial_items {
        let item_id = game.world_mut().allocate_item_id();
        let item = Item::from_spawn_kind(item_id, item_kind);
        if let Some(player) = game.world_mut().get_actor_mut(player_id) {
          let _ = player.inventory_mut().add_item(item);
        }
      }

      if let Some(weapon_kind) = config.equipped_weapon {
        let item_id = game.world_mut().allocate_item_id();
        let weapon = Item::from_spawn_kind(item_id, weapon_kind);
        if let Some(player) = game.world_mut().get_actor_mut(player_id) {
          let _ = player.equipment_mut().equip(EquipmentSlot::Weapon, weapon);
        }
      }

      if let Some(armor_kind) = config.equipped_armor {
        let item_id = game.world_mut().allocate_item_id();
        let armor = Item::from_spawn_kind(item_id, armor_kind);
        if let Some(player) = game.world_mut().get_actor_mut(player_id) {
          let _ = player.equipment_mut().equip(EquipmentSlot::Armor, armor);
          if let Some(durability) = config.equipped_armor_durability
            && let Some(properties) = player
              .equipment_mut()
              .armor_mut()
              .and_then(Item::armor_properties_mut)
          {
            properties.durability = durability.min(properties.max_durability);
          }
        }
      }
    }

    for monster in &replay.initial_monsters {
      let id = game.world_mut().allocate_entity_id();
      let actor = crate::actor::Actor::new(id, monster.position, &monster.name, false)
        .with_stats(
          HitPoints::full(monster.hp),
          Speed::new(monster.speed),
          monster.melee_damage,
          monster.ranged_damage,
          monster.ranged_range,
          monster.accuracy,
        )
        .with_death_drop(monster.death_drop)
        .with_boss(monster.is_boss);
      let actor = if let Some(kind) = MonsterKind::from_name(&monster.name) {
        actor.with_monster_kind(kind)
      } else {
        actor
      };
      game.world_mut().actors_mut().insert(id, actor);
    }

    for item_spec in &replay.initial_items {
      let item_id = game.world_mut().allocate_item_id();
      let item = Item::from_spawn_kind(item_id, item_spec.kind);
      game
        .world_mut()
        .spawn_ground_item(item_spec.position, item)
        .map_err(|err| ReplayExecutionError {
          turn: Turn::zero(),
          command_index: 0,
          command: Command::Wait,
          error: err,
        })?;
    }

    game.world_mut().update_visibility();

    let mut all_events = Vec::new();
    let mut metrics = EpisodeMetrics::new();

    for (idx, &cmd) in replay.commands.iter().enumerate() {
      let step_events = game.step(cmd).map_err(|err| ReplayExecutionError {
        turn: game.turn(),
        command_index: idx,
        command: cmd,
        error: err,
      })?;

      for event in &step_events {
        metrics.record_event(player_id, event);
      }
      all_events.extend(step_events);

      if game.world().level_id().0 > 1 {
        metrics.outcome = RunOutcome::Victory;
        break;
      }
      if let Some(player) = game.world().player()
        && !player.is_alive()
      {
        break;
      }
    }

    if metrics.outcome == RunOutcome::InProgress
      && let Some(player) = game.world().player()
      && player.is_alive()
      && game.world().level_id().0 > 1
    {
      metrics.outcome = RunOutcome::Victory;
    }

    Ok((game, all_events, metrics))
  }

  /// Executes a full replay log from its recorded seed and start configuration.
  ///
  /// Returns the final `Game` state and all accumulated `GameEvent`s.
  pub fn run(replay: &ReplayLog) -> Result<(Game, Vec<GameEvent>), CommandError> {
    let (game, events, _) = Self::run_with_diagnostics(replay).map_err(|err| err.error)?;
    Ok((game, events))
  }

  /// Runs a replay twice independently and verifies that the resulting game state,
  /// event logs, and episode metrics are bit-exact identical.
  pub fn verify_determinism(replay: &ReplayLog) -> Result<bool, CommandError> {
    let (game1, events1, metrics1) = Self::run_with_diagnostics(replay).map_err(|err| err.error)?;
    let (game2, events2, metrics2) = Self::run_with_diagnostics(replay).map_err(|err| err.error)?;

    Ok(game1 == game2 && events1 == events2 && metrics1 == metrics2)
  }
}

fn validate_replay_structure(replay: &ReplayLog) -> Result<(), String> {
  if replay.initial_monsters.len() > MAX_INITIAL_ENTITIES {
    return Err(format!(
      "initial_monsters exceeds maximum of {MAX_INITIAL_ENTITIES} entries"
    ));
  }
  if replay.initial_items.len() > MAX_INITIAL_ENTITIES {
    return Err(format!(
      "initial_items exceeds maximum of {MAX_INITIAL_ENTITIES} entries"
    ));
  }
  if replay.custom_tiles.len() > MAX_CUSTOM_TILES {
    return Err(format!(
      "custom_tiles exceeds maximum of {MAX_CUSTOM_TILES} entries"
    ));
  }
  if replay.commands.len() > MAX_COMMANDS {
    return Err(format!(
      "commands exceeds maximum of {MAX_COMMANDS} entries"
    ));
  }
  if let Some(config) = &replay.player_config
    && config.initial_items.len() > MAX_INITIAL_ENTITIES
  {
    return Err(format!(
      "player_config.initial_items exceeds maximum of {MAX_INITIAL_ENTITIES} entries"
    ));
  }
  if let Some(config) = &replay.procedural_config
    && (config.max_rooms > MAX_PROCEDURAL_ROOMS
      || config.min_room_size == 0
      || config.min_room_size > config.max_room_size
      || config.max_room_size > MAX_ROOM_SIZE
      || config.max_monsters_per_room > MAX_CONTENT_PER_ROOM
      || config.max_items_per_room > MAX_CONTENT_PER_ROOM)
  {
    return Err("procedural replay configuration exceeds safe bounds".to_string());
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use drl_protocol::{Command, Direction, Position};

  #[test]
  fn test_replay_determinism() {
    let mut replay = ReplayLog::new(98765, 20, 20, Position::new(10, 10));
    replay.record_command(Command::Move(Direction::East));
    replay.record_command(Command::Move(Direction::East));
    replay.record_command(Command::Move(Direction::South));
    replay.record_command(Command::Wait);
    replay.record_command(Command::Move(Direction::NorthWest));

    let is_deterministic = ReplayEngine::verify_determinism(&replay).unwrap();
    assert!(is_deterministic);
  }

  #[test]
  fn test_replay_validation_failure() {
    let replay = ReplayLog::new(1234, 10, 10, Position::new(15, 15));
    assert!(ReplayEngine::validate(&replay).is_err());
  }

  #[test]
  fn test_replay_validation_rejects_incompatible_semantics() {
    let mut replay = ReplayLog::new(1234, 10, 10, Position::new(1, 1));
    // Version 101 predates BFG 10K's ninth-level chainfire effect and must not
    // be interpreted by the version-102 engine.
    assert_eq!(drl_protocol::CURRENT_GAMEPLAY_SEMANTICS_VERSION, 102);
    assert_eq!(
      drl_protocol::CURRENT_RNG_SAMPLING_SEMANTICS_VERSION,
      crate::rng::RNG_SAMPLING_SEMANTICS_VERSION
    );
    replay.metadata.gameplay_semantics_version = 75;
    let error = ReplayEngine::validate(&replay).unwrap_err();
    assert!(error.contains("unsupported gameplay semantics version"));

    let mut replay = ReplayLog::new(1234, 10, 10, Position::new(1, 1));
    replay.metadata.ruleset_id = "legacy-ruleset".to_string();
    let error = ReplayEngine::validate(&replay).unwrap_err();
    assert!(error.contains("unsupported replay ruleset"));

    let mut replay = ReplayLog::new(1234, 10, 10, Position::new(1, 1));
    replay.procedural_config = Some(drl_protocol::ProceduralGenerationConfig {
      max_rooms: 2,
      min_room_size: 4,
      max_room_size: 6,
      max_monsters_per_room: 1,
      max_items_per_room: 1,
    });
    replay.metadata.generator_semantics_version =
      drl_protocol::CURRENT_GENERATOR_SEMANTICS_VERSION - 1;
    let error = ReplayEngine::validate(&replay).unwrap_err();
    assert!(error.contains("unsupported generator semantics version"));

    let mut replay = ReplayLog::new(1234, 10, 10, Position::new(1, 1));
    replay.metadata.generator_semantics_version =
      drl_protocol::CURRENT_GENERATOR_SEMANTICS_VERSION - 1;
    assert!(ReplayEngine::validate(&replay).is_ok());
  }

  #[test]
  fn test_replay_diagnostics_error_context() {
    let mut replay = ReplayLog::new(1234, 10, 10, Position::new(1, 1));
    replay.record_command(Command::Move(Direction::West)); // Hits boundary wall

    let err = ReplayEngine::run_with_diagnostics(&replay).unwrap_err();
    assert_eq!(err.command_index, 0);
    assert_eq!(err.command, Command::Move(Direction::West));
  }
}
