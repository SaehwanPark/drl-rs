//! Declarative scenario fixture runner and ASCII map parser.

use crate::actor::Actor;
use crate::agent::AgentPolicy;
use crate::game::Game;
use crate::grid::Tile;
use crate::item::Item;
use crate::scheduler::ACTION_THRESHOLD;
use drl_protocol::{
  Command, CommandError, EquipmentSlot, GameEvent, HitPoints, ItemSpawnKind, ItemSpawnSpec,
  MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayExecutionError, ReplayLog, RunOutcome,
  ScenarioFixture, ScenarioMap, Speed,
};
use std::collections::BTreeMap;

/// Concrete scenario definition with explicit map tiles, entity spawns, and seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
  /// Scenario title.
  pub name: String,
  /// Overview description.
  pub description: String,
  /// Grid width.
  pub width: u32,
  /// Grid height.
  pub height: u32,
  /// Explicit map tiles.
  pub tiles: BTreeMap<Position, Tile>,
  /// Initial player position.
  pub player_start: Position,
  /// Optional custom player stats/equipment configuration.
  pub player_config: Option<PlayerSpawnConfig>,
  /// Optional exit down-stairs coordinate.
  pub stairs: Option<Position>,
  /// Monsters to spawn before first action.
  pub monsters: Vec<MonsterSpawnSpec>,
  /// Ground items to spawn before first action.
  pub items: Vec<ItemSpawnSpec>,
  /// Initial RNG seed.
  pub seed: u64,
}

impl Scenario {
  /// Parses an ASCII map grid into a concrete `Scenario`.
  ///
  /// ASCII Legend:
  /// - `'#'`: Wall tile
  /// - `'.'`: Floor tile
  /// - `'@'`: Player start (sets floor tile under player)
  /// - `'>'`: Down stairs (sets `Tile::StairsDown`)
  /// - `'='`: Lava tile (sets `Tile::Lava`)
  /// - `'h'`: Former Human monster
  /// - `'s'`: Former Sergeant monster
  /// - `'i'`: Imp monster
  /// - `'d'`: Demon monster
  /// - `'p'`: Pistol item
  /// - `'S'`: Shotgun item
  /// - `'k'`: Combat Knife item
  /// - `'m'`: Small MedPack item
  /// - `'M'`: Large MedPack item
  /// - `'a'`: 9mm Ammo item (20 rounds)
  /// - `'A'`: Shotgun Shells item (10 rounds)
  /// - `'g'`: Green Armor item
  /// - `'P'`: Phase Device item
  /// - `' '`: Empty/Wall background
  pub fn from_ascii(
    name: impl Into<String>,
    description: impl Into<String>,
    ascii: &str,
  ) -> Result<Self, String> {
    let lines: Vec<&str> = ascii.lines().filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
      return Err("ASCII scenario layout cannot be empty".to_string());
    }

    let height = lines.len() as u32;
    let mut width = 0;
    for line in &lines {
      if line.chars().count() as u32 > width {
        width = line.chars().count() as u32;
      }
    }

    let mut tiles = BTreeMap::new();
    let mut player_start = None;
    let mut stairs = None;
    let mut monsters = Vec::new();
    let mut items = Vec::new();

    for (y, line) in lines.iter().enumerate() {
      for (x, ch) in line.chars().enumerate() {
        let pos = Position::new(x as i32, y as i32);
        match ch {
          '#' | ' ' => {
            tiles.insert(pos, Tile::Wall);
          }
          '.' => {
            tiles.insert(pos, Tile::Floor);
          }
          '@' => {
            tiles.insert(pos, Tile::Floor);
            player_start = Some(pos);
          }
          '>' => {
            tiles.insert(pos, Tile::StairsDown);
            stairs = Some(pos);
          }
          '=' => {
            tiles.insert(pos, Tile::Lava);
          }
          'h' => {
            tiles.insert(pos, Tile::Floor);
            monsters.push(
              MonsterSpawnSpec::new(pos, "Former Human", 10, 100, (2, 5))
                .with_ranged_combat((1, 4), 6, 65)
                .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
            );
          }
          's' => {
            tiles.insert(pos, Tile::Floor);
            monsters.push(
              MonsterSpawnSpec::new(pos, "Former Sergeant", 15, 100, (3, 6))
                .with_ranged_combat((2, 6), 5, 60)
                .with_death_drop(Some(ItemSpawnKind::AmmoShells(10))),
            );
          }
          'i' => {
            tiles.insert(pos, Tile::Floor);
            monsters.push(
              MonsterSpawnSpec::new(pos, "Imp", 20, 100, (3, 8))
                .with_ranged_combat((2, 5), 7, 70)
                .with_death_drop(Some(ItemSpawnKind::SmallMedPack)),
            );
          }
          'd' => {
            tiles.insert(pos, Tile::Floor);
            monsters.push(
              MonsterSpawnSpec::new(pos, "Demon", 30, 140, (5, 10))
                .with_death_drop(Some(ItemSpawnKind::LargeMedPack)),
            );
          }
          'p' => {
            tiles.insert(pos, Tile::Floor);
            items.push(ItemSpawnSpec::new(pos, ItemSpawnKind::Pistol));
          }
          'S' => {
            tiles.insert(pos, Tile::Floor);
            items.push(ItemSpawnSpec::new(pos, ItemSpawnKind::Shotgun));
          }
          'k' => {
            tiles.insert(pos, Tile::Floor);
            items.push(ItemSpawnSpec::new(pos, ItemSpawnKind::CombatKnife));
          }
          'm' => {
            tiles.insert(pos, Tile::Floor);
            items.push(ItemSpawnSpec::new(pos, ItemSpawnKind::SmallMedPack));
          }
          'M' => {
            tiles.insert(pos, Tile::Floor);
            items.push(ItemSpawnSpec::new(pos, ItemSpawnKind::LargeMedPack));
          }
          'a' => {
            tiles.insert(pos, Tile::Floor);
            items.push(ItemSpawnSpec::new(pos, ItemSpawnKind::Ammo9mm(20)));
          }
          'A' => {
            tiles.insert(pos, Tile::Floor);
            items.push(ItemSpawnSpec::new(pos, ItemSpawnKind::AmmoShells(10)));
          }
          'g' => {
            tiles.insert(pos, Tile::Floor);
            items.push(ItemSpawnSpec::new(pos, ItemSpawnKind::GreenArmor));
          }
          'P' => {
            tiles.insert(pos, Tile::Floor);
            items.push(ItemSpawnSpec::new(pos, ItemSpawnKind::PhaseDevice));
          }
          _ => {
            tiles.insert(pos, Tile::Floor);
          }
        }
      }
    }

    let start_pos = player_start.unwrap_or_else(|| Position::new(1, 1));

    Ok(Self {
      name: name.into(),
      description: description.into(),
      width,
      height,
      tiles,
      player_start: start_pos,
      player_config: None,
      stairs,
      monsters,
      items,
      seed: 42,
    })
  }

  /// Converts a protocol `ScenarioFixture` into an executable `Scenario`.
  pub fn from_fixture(fixture: &ScenarioFixture) -> Result<Self, String> {
    match &fixture.map {
      ScenarioMap::Ascii(ascii) => {
        let mut sc = Self::from_ascii(&fixture.name, &fixture.description, ascii)?;
        sc.player_config = fixture.player_config.clone();
        sc.seed = fixture.seed;
        if let Some(st) = fixture.stairs {
          sc.stairs = Some(st);
        }
        for m in &fixture.monsters {
          sc.monsters.push(m.clone());
        }
        for i in &fixture.items {
          sc.items.push(i.clone());
        }
        Ok(sc)
      }
      ScenarioMap::Dimensions { width, height } => {
        let mut tiles = BTreeMap::new();
        for y in 0..*height {
          for x in 0..*width {
            let pos = Position::new(x as i32, y as i32);
            if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
              tiles.insert(pos, Tile::Wall);
            } else {
              tiles.insert(pos, Tile::Floor);
            }
          }
        }
        Ok(Self {
          name: fixture.name.clone(),
          description: fixture.description.clone(),
          width: *width,
          height: *height,
          tiles,
          player_start: fixture.player_start,
          player_config: fixture.player_config.clone(),
          stairs: fixture.stairs,
          monsters: fixture.monsters.clone(),
          items: fixture.items.clone(),
          seed: fixture.seed,
        })
      }
    }
  }

  /// Instantiates a new initialized `Game` from this scenario definition.
  pub fn instantiate(&self) -> Result<Game, CommandError> {
    let mut game = Game::new(self.seed, self.width, self.height, self.player_start)?;

    // Apply custom map tiles
    for (&pos, &tile) in &self.tiles {
      game.world_mut().map_mut().set_tile(pos, tile);
    }

    if let Some(st_pos) = self.stairs {
      game
        .world_mut()
        .map_mut()
        .set_tile(st_pos, Tile::StairsDown);
    }

    // Apply custom player configuration if specified
    if let Some(config) = &self.player_config {
      let player_id = game.world().player_id().ok_or_else(|| {
        CommandError::InvalidCommand("no player initialized in scenario".to_string())
      })?;
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

        // Clear default equipment/inventory
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

    // Spawn monsters
    for monster in &self.monsters {
      let id = game.world_mut().allocate_entity_id();
      let actor = Actor::new(id, monster.position, &monster.name, false)
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

    // Spawn ground items
    for item_spec in &self.items {
      let item_id = game.world_mut().allocate_item_id();
      let item = Item::from_spawn_kind(item_id, item_spec.kind);
      game
        .world_mut()
        .spawn_ground_item(item_spec.position, item)?;
    }

    game.world_mut().update_visibility();
    Ok(game)
  }
}

/// Test runner executing scenarios with command sequences or automated agent policies.
pub struct ScenarioRunner;

impl ScenarioRunner {
  /// Executes a scripted command sequence on a scenario and collects telemetry.
  pub fn run_commands(
    scenario: &Scenario,
    commands: &[Command],
  ) -> Result<
    (
      Game,
      Vec<GameEvent>,
      drl_protocol::EpisodeMetrics,
      ReplayLog,
    ),
    ReplayExecutionError,
  > {
    let mut game = scenario.instantiate().map_err(|err| ReplayExecutionError {
      turn: drl_protocol::Turn::zero(),
      command_index: 0,
      command: Command::Wait,
      error: err,
    })?;

    let player_id = game
      .world()
      .player_id()
      .unwrap_or(drl_protocol::EntityId(1));
    let mut replay = ReplayLog::new(
      scenario.seed,
      scenario.width,
      scenario.height,
      scenario.player_start,
    );
    if let Some(st) = scenario.stairs {
      replay.record_stairs(st);
    }
    for (&pos, &tile) in &scenario.tiles {
      let kind = match tile {
        Tile::Wall => drl_protocol::TileKind::Wall,
        Tile::Floor => drl_protocol::TileKind::Floor,
        Tile::StairsDown => drl_protocol::TileKind::StairsDown,
        Tile::DoorClosed => drl_protocol::TileKind::DoorClosed,
        Tile::DoorOpen => drl_protocol::TileKind::DoorOpen,
        Tile::Lava => drl_protocol::TileKind::Lava,
      };
      replay.record_tile(pos, kind);
    }
    for m in &scenario.monsters {
      replay.record_monster(m.clone());
    }
    for i in &scenario.items {
      replay.record_item(i.clone());
    }
    if let Some(cfg) = &scenario.player_config {
      replay = replay.with_player_config(cfg.clone());
    }

    let mut all_events = Vec::new();
    let mut metrics = drl_protocol::EpisodeMetrics::new();

    for (idx, &cmd) in commands.iter().enumerate() {
      replay.record_command(cmd);
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

    Ok((game, all_events, metrics, replay))
  }

  /// Executes an automated agent policy on a scenario up to `max_turns`.
  pub fn run_policy(
    scenario: &Scenario,
    policy: &mut dyn AgentPolicy,
    max_turns: u64,
  ) -> Result<
    (
      Game,
      Vec<GameEvent>,
      drl_protocol::EpisodeMetrics,
      ReplayLog,
    ),
    CommandError,
  > {
    let mut game = scenario.instantiate()?;
    let player_id = game
      .world()
      .player_id()
      .unwrap_or(drl_protocol::EntityId(1));

    let mut replay = ReplayLog::new(
      scenario.seed,
      scenario.width,
      scenario.height,
      scenario.player_start,
    );
    if let Some(st) = scenario.stairs {
      replay.record_stairs(st);
    }
    for (&pos, &tile) in &scenario.tiles {
      let kind = match tile {
        Tile::Wall => drl_protocol::TileKind::Wall,
        Tile::Floor => drl_protocol::TileKind::Floor,
        Tile::StairsDown => drl_protocol::TileKind::StairsDown,
        Tile::DoorClosed => drl_protocol::TileKind::DoorClosed,
        Tile::DoorOpen => drl_protocol::TileKind::DoorOpen,
        Tile::Lava => drl_protocol::TileKind::Lava,
      };
      replay.record_tile(pos, kind);
    }
    for m in &scenario.monsters {
      replay.record_monster(m.clone());
    }
    for i in &scenario.items {
      replay.record_item(i.clone());
    }
    if let Some(cfg) = &scenario.player_config {
      replay = replay.with_player_config(cfg.clone());
    }

    let mut all_events = Vec::new();
    let mut metrics = drl_protocol::EpisodeMetrics::new();

    for _ in 0..max_turns {
      if game.world().level_id().0 > 1 {
        metrics.outcome = RunOutcome::Victory;
        break;
      }
      if let Some(player) = game.world().player() {
        if !player.is_alive() {
          break;
        }
      } else {
        break;
      }

      let obs = game.observe_player();
      let cmd = match policy.decide_action(&obs) {
        Some(c) => c,
        None => {
          metrics.outcome = RunOutcome::Stalled;
          break;
        }
      };

      replay.record_command(cmd);
      let step_events = game.step(cmd)?;
      for event in &step_events {
        metrics.record_event(player_id, event);
      }
      all_events.extend(step_events);
    }

    if metrics.outcome == RunOutcome::InProgress
      && let Some(player) = game.world().player()
      && player.is_alive()
    {
      if game.world().level_id().0 > 1 {
        metrics.outcome = RunOutcome::Victory;
      } else {
        metrics.outcome = RunOutcome::TurnLimitReached;
      }
    }

    Ok((game, all_events, metrics, replay))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use drl_protocol::Direction;

  #[test]
  fn test_scenario_ascii_parsing_and_instantiation() {
    let ascii = r#"
#####
#@.h#
#..>#
#####
"#;
    let scenario = Scenario::from_ascii("MiniMap", "Simple test room", ascii).unwrap();
    assert_eq!(scenario.width, 5);
    assert_eq!(scenario.height, 4);
    assert_eq!(scenario.player_start, Position::new(1, 1));
    assert_eq!(scenario.stairs, Some(Position::new(3, 2)));
    assert_eq!(scenario.monsters.len(), 1);
    assert_eq!(scenario.monsters[0].name, "Former Human");

    let game = scenario.instantiate().unwrap();
    assert_eq!(
      game.world().player().unwrap().position(),
      Position::new(1, 1)
    );
    let living_monsters = game
      .world()
      .actors()
      .values()
      .filter(|a| !a.is_player() && a.is_alive())
      .count();
    assert_eq!(living_monsters, 1);
  }

  #[test]
  fn test_scenario_runner_with_commands() {
    let ascii = r#"
#####
#@..#
#...>
#####
"#;
    let scenario = Scenario::from_ascii("Descent", "Reach stairs", ascii).unwrap();
    let commands = vec![
      Command::Move(Direction::East),
      Command::Move(Direction::East),
      Command::Move(Direction::South),
      Command::Move(Direction::East),
      Command::Descend,
    ];

    let (game, events, metrics, replay) =
      ScenarioRunner::run_commands(&scenario, &commands).unwrap();
    assert_eq!(metrics.outcome, RunOutcome::Victory);
    assert!(game.world().level_id().0 > 1);
    assert_eq!(replay.commands.len(), 5);
    assert!(!events.is_empty());
  }
}
