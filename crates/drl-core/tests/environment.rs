//! Integration tests for deterministic entered-cell environmental terrain rules.

use drl_core::game::Game;
use drl_core::grid::Tile;
use drl_core::item::Item;
use drl_core::replay::ReplayEngine;
use drl_core::scenario::{Scenario, ScenarioRunner};
use drl_protocol::{
  ActionCost, Command, DamageSource, Direction, GameEvent, Position, ReplayLog, TileKind,
};
use drl_protocol::{EquipmentSlot, ItemId};

#[test]
fn moving_onto_acid_applies_baseline_damage_without_rng_use() {
  let mut game = Game::new_arena(1_331, 12, 12).unwrap();
  let player_position = game.world().player().unwrap().position();
  let hazard_position = player_position + Direction::East;
  game
    .world_mut()
    .map_mut()
    .set_tile(hazard_position, Tile::Acid);
  let rng_before = game.rng().clone();

  let events = game.step(Command::Move(Direction::East)).unwrap();
  let move_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::EntityMoved { to, .. } if *to == hazard_position))
    .expect("movement event must be emitted");
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id,
          amount: 6,
          source: DamageSource::Environment,
          ..
        } if *target_id == game.world().player_id().unwrap()
      )
    })
    .expect("Acid contact damage event must be emitted");

  assert!(move_index < damage_index);
  assert_eq!(game.world().player().unwrap().hp().current, 44);
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::ActionCostPaid {
      entity_id,
      cost,
    } if *entity_id == game.world().player_id().unwrap() && *cost == ActionCost::new(1_250)
  )));
  assert_eq!(game.rng(), &rng_before);
  assert!(!game.is_game_over());
}

#[test]
fn moving_onto_floor_retains_standard_movement_cost() {
  let mut game = Game::new_arena(1_337, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let events = game.step(Command::Move(Direction::East)).unwrap();

  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::ActionCostPaid {
      entity_id,
      cost: ActionCost::MOVE,
    } if *entity_id == player_id
  )));
  assert!(
    !events
      .iter()
      .any(|event| matches!(event, GameEvent::DamageApplied { .. }))
  );
  assert_eq!(game.world().player().unwrap().hp().current, 50);
}

#[test]
fn moving_onto_water_uses_fluid_movement_cost_without_damage() {
  let mut game = Game::new_arena(1_338, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let water_position = game.world().player().unwrap().position() + Direction::East;
  game
    .world_mut()
    .map_mut()
    .set_tile(water_position, Tile::Water);

  let events = game.step(Command::Move(Direction::East)).unwrap();

  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::ActionCostPaid {
      entity_id,
      cost: ActionCost(1_250),
    } if *entity_id == player_id
  )));
  assert!(
    !events
      .iter()
      .any(|event| matches!(event, GameEvent::DamageApplied { .. }))
  );
  assert_eq!(game.world().player().unwrap().hp().current, 50);
}

#[test]
fn lethal_lava_contact_emits_environment_death_and_ends_game() {
  let mut game = Game::new_arena(1_332, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let player_position = game.world().player().unwrap().position();
  let hazard_position = player_position + Direction::East;
  game
    .world_mut()
    .map_mut()
    .set_tile(hazard_position, Tile::Lava);
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .hp_mut()
    .current = 12;
  let armor = Item::lava_armor(ItemId::new(1_332));
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Armor, armor)
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .armor_mut()
    .unwrap()
    .armor_properties_mut()
    .unwrap()
    .durability = 50;
  let monster_id = game
    .world_mut()
    .spawn_monster(Position::new(8, 8), "Waiting Monster", 10, 20, (1, 2))
    .unwrap();

  let events = game.step(Command::Move(Direction::East)).unwrap();
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::DamageApplied {
      target_id,
      amount: 12,
      remaining_hp: 0,
      source: DamageSource::Environment,
    } if *target_id == player_id
  )));
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::ActorDied {
      entity_id,
      cause: drl_protocol::DeathCause::Environment,
    } if *entity_id == player_id
  )));
  assert!(game.is_game_over());
  assert!(!game.world().player().unwrap().is_alive());
  let damage_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::DamageApplied { .. }))
    .unwrap();
  let death_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::ActorDied { .. }))
    .unwrap();
  let cost_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::ActionCostPaid { .. }))
    .unwrap();
  let ended_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::TurnEnded { .. }))
    .unwrap();
  assert!(damage_index < death_index);
  assert!(death_index < cost_index);
  assert!(cost_index < ended_index);
  assert!(matches!(
    events[cost_index],
    GameEvent::ActionCostPaid {
      cost: ActionCost(1_250),
      ..
    }
  ));
  assert!(
    !events
      .iter()
      .any(|event| matches!(event, GameEvent::EntityWaited { .. }))
  );
  assert!(
    !events
      .iter()
      .any(|event| matches!(event, GameEvent::LavaArmorRecharged { .. }))
  );
  assert!(!events.iter().any(|event| matches!(
    event,
    GameEvent::ActionCostPaid { entity_id, .. } if *entity_id == monster_id
  )));
}

#[test]
fn waiting_on_hazard_does_not_repeat_entered_cell_damage() {
  let mut game = Game::new_arena(1_333, 12, 12).unwrap();
  let player_position = game.world().player().unwrap().position();
  game
    .world_mut()
    .map_mut()
    .set_tile(player_position, Tile::Acid);

  let events = game.step(Command::Wait).unwrap();
  assert!(
    !events
      .iter()
      .any(|event| matches!(event, GameEvent::DamageApplied { .. }))
  );
  assert_eq!(game.world().player().unwrap().hp().current, 50);
}

#[test]
fn leaving_and_reentering_a_hazard_applies_contact_once_per_entry() {
  let mut game = Game::new_arena(1_335, 12, 12).unwrap();
  let player_position = game.world().player().unwrap().position();
  let acid_position = player_position + Direction::East;
  game
    .world_mut()
    .map_mut()
    .set_tile(acid_position, Tile::Acid);

  game.step(Command::Move(Direction::East)).unwrap();
  game.step(Command::Move(Direction::West)).unwrap();
  let events = game.step(Command::Move(Direction::East)).unwrap();

  assert_eq!(game.world().player().unwrap().hp().current, 38);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::DamageApplied {
          amount: 6,
          source: DamageSource::Environment,
          ..
        }
      ))
      .count(),
    1
  );
}

#[test]
fn replay_preserves_lava_contact_deterministically() {
  let start = Position::new(5, 5);
  let mut replay = ReplayLog::new(1_334, 12, 12, start);
  replay.record_tile(start + Direction::East, TileKind::Lava);
  replay.record_command(Command::Move(Direction::East));

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (game, events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(game.world().player().unwrap().hp().current, 38);
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::DamageApplied {
      amount: 12,
      source: DamageSource::Environment,
      ..
    }
  )));
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::ActionCostPaid {
      cost: ActionCost(1_250),
      ..
    }
  )));
}

#[test]
fn replay_preserves_water_fluid_movement_cost_deterministically() {
  let start = Position::new(5, 5);
  let mut replay = ReplayLog::new(1_339, 12, 12, start);
  replay.record_tile(start + Direction::East, TileKind::Water);
  replay.record_command(Command::Move(Direction::East));

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (game, events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(game.world().player().unwrap().hp().current, 50);
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::ActionCostPaid {
      cost: ActionCost(1_250),
      ..
    }
  )));
}

#[test]
fn rejected_move_preserves_hazard_game_state_and_rng() {
  let mut game = Game::new_arena(1_336, 12, 12).unwrap();
  let player_position = game.world().player().unwrap().position();
  game
    .world_mut()
    .map_mut()
    .set_tile(player_position + Direction::East, Tile::Wall);
  let before = game.clone();
  let rng_before = game.rng().clone();

  assert_eq!(
    game.step(Command::Move(Direction::East)),
    Err(drl_protocol::CommandError::BlockedByTerrain(
      player_position + Direction::East
    ))
  );
  assert_eq!(game, before);
  assert_eq!(game.rng(), &rng_before);
}

#[test]
fn ascii_scenario_replays_lava_contact_policy() {
  let scenario = Scenario::from_ascii(
    "LavaContact",
    "Player enters a Lava cell",
    "#####\n#@=.#\n#####\n",
  )
  .unwrap();
  let (game, events, _, _) =
    ScenarioRunner::run_commands(&scenario, &[Command::Move(Direction::East)]).unwrap();

  assert_eq!(game.world().player().unwrap().hp().current, 38);
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::DamageApplied {
      amount: 12,
      source: DamageSource::Environment,
      ..
    }
  )));
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::ActionCostPaid {
      cost: ActionCost(1_250),
      ..
    }
  )));
}

#[test]
fn ascii_scenario_replays_water_movement_cost_policy() {
  let scenario = Scenario::from_ascii(
    "WaterMovement",
    "Player enters a Water cell",
    "#####\n#@w.#\n#####\n",
  )
  .unwrap();
  let (game, events, _, _) =
    ScenarioRunner::run_commands(&scenario, &[Command::Move(Direction::East)]).unwrap();

  assert_eq!(game.world().player().unwrap().hp().current, 50);
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::ActionCostPaid {
      cost: ActionCost(1_250),
      ..
    }
  )));
}
