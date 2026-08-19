//! Deterministic replay execution engine.

use crate::game::Game;
use crate::item::Item;
use drl_protocol::{CommandError, GameEvent, ItemSpawnKind, ReplayLog};

/// Engine for replaying recorded game sessions deterministically.
pub struct ReplayEngine;

impl ReplayEngine {
  /// Executes a full replay log from its recorded seed and start configuration.
  ///
  /// Returns the final `Game` state and all accumulated `GameEvent`s.
  pub fn run(replay: &ReplayLog) -> Result<(Game, Vec<GameEvent>), CommandError> {
    let mut game = Game::new(
      replay.seed,
      replay.width,
      replay.height,
      replay.player_start,
    )?;

    if let Some(stairs_pos) = replay.initial_stairs {
      game
        .world_mut()
        .map_mut()
        .set_tile(stairs_pos, crate::grid::Tile::StairsDown);
    }

    for monster in &replay.initial_monsters {
      let id = game.world_mut().allocate_entity_id();
      let actor = crate::actor::Actor::new(id, monster.position, &monster.name, false).with_stats(
        drl_protocol::HitPoints::full(monster.hp),
        drl_protocol::Speed::new(monster.speed),
        monster.melee_damage,
        monster.ranged_damage,
        monster.ranged_range,
        monster.accuracy,
      );
      game.world_mut().actors_mut().insert(id, actor);
    }

    for item_spec in &replay.initial_items {
      let item_id = game.world_mut().allocate_item_id();
      let item = match item_spec.kind {
        ItemSpawnKind::Pistol => Item::pistol(item_id),
        ItemSpawnKind::Shotgun => Item::shotgun(item_id),
        ItemSpawnKind::CombatKnife => Item::combat_knife(item_id),
        ItemSpawnKind::Ammo9mm(count) => Item::ammo_9mm(item_id, count),
        ItemSpawnKind::AmmoShells(count) => Item::ammo_shells(item_id, count),
        ItemSpawnKind::SmallMedPack => Item::small_medpack(item_id),
        ItemSpawnKind::LargeMedPack => Item::large_medpack(item_id),
        ItemSpawnKind::GreenArmor => Item::green_armor(item_id),
        ItemSpawnKind::PhaseDevice => Item::phase_device(item_id),
      };
      game
        .world_mut()
        .spawn_ground_item(item_spec.position, item)?;
    }

    let mut all_events = Vec::new();
    for &cmd in &replay.commands {
      let step_events = game.step(cmd)?;
      all_events.extend(step_events);
    }

    Ok((game, all_events))
  }

  /// Runs a replay twice independently and verifies that the resulting game state
  /// and event logs are identical.
  pub fn verify_determinism(replay: &ReplayLog) -> Result<bool, CommandError> {
    let (game1, events1) = Self::run(replay)?;
    let (game2, events2) = Self::run(replay)?;

    Ok(game1 == game2 && events1 == events2)
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
}
