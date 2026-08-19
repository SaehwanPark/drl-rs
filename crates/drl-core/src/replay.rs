//! Deterministic replay execution engine and diagnostics.

use crate::game::Game;
use crate::item::Item;
use crate::scheduler::ACTION_THRESHOLD;
use drl_protocol::{
  Command, CommandError, EpisodeMetrics, EquipmentSlot, GameEvent, HitPoints, Position,
  ReplayExecutionError, ReplayLog, RunOutcome, Speed, Turn,
};

/// Engine for replaying recorded game sessions deterministically with rich diagnostics.
pub struct ReplayEngine;

impl ReplayEngine {
  /// Validates a replay log's spatial bounds and structural consistency before execution.
  pub fn validate(replay: &ReplayLog) -> Result<(), String> {
    if replay.width == 0 || replay.height == 0 {
      return Err(format!(
        "Invalid map dimensions: {}x{}",
        replay.width, replay.height
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

    let mut game = Game::new(
      replay.seed,
      replay.width,
      replay.height,
      replay.player_start,
    )
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
        .with_death_drop(monster.death_drop);
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
  fn test_replay_diagnostics_error_context() {
    let mut replay = ReplayLog::new(1234, 10, 10, Position::new(1, 1));
    replay.record_command(Command::Move(Direction::West)); // Hits boundary wall

    let err = ReplayEngine::run_with_diagnostics(&replay).unwrap_err();
    assert_eq!(err.command_index, 0);
    assert_eq!(err.command, Command::Move(Direction::West));
  }
}
