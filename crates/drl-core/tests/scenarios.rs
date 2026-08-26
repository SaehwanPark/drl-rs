//! Integration tests for declarative scenario fixtures and ASCII map parsing.

use drl_core::grid::Tile;
use drl_core::replay::ReplayEngine;
use drl_core::scenario::{Scenario, ScenarioRunner};
use drl_protocol::{
  Command, Direction, GameEvent, ItemId, ItemSpawnKind, PlayerSpawnConfig, Position, RunOutcome,
  ScenarioFixture,
};

#[test]
fn test_ascii_scenario_complex_arena() {
  let ascii = r#"
############
#@..p...m..#
#.####.###.#
#...h...s..#
#.####.###.#
#...i...d..#
#........>.#
############
"#;

  let scenario = Scenario::from_ascii(
    "Gauntlet",
    "Multi-chamber gauntlet testing weapon pickups and multi-archetype combat",
    ascii,
  )
  .unwrap();

  assert_eq!(scenario.width, 12);
  assert_eq!(scenario.height, 8);
  assert_eq!(scenario.player_start, Position::new(1, 1));
  assert_eq!(scenario.stairs, Some(Position::new(9, 6)));
  assert_eq!(scenario.monsters.len(), 4);
  assert_eq!(scenario.items.len(), 2);

  let game = scenario.instantiate().unwrap();
  assert_eq!(
    game.world().player().unwrap().position(),
    Position::new(1, 1)
  );
  assert!(game.world().player().unwrap().is_alive());

  let monster_names: Vec<&str> = game
    .world()
    .actors()
    .values()
    .filter(|a| !a.is_player())
    .map(|a| a.name())
    .collect();
  assert!(monster_names.contains(&"Former Human"));
  assert!(monster_names.contains(&"Former Sergeant"));
  assert!(monster_names.contains(&"Imp"));
  assert!(monster_names.contains(&"Demon"));
}

#[test]
fn test_scenario_custom_player_spawn_config() {
  let ascii = r#"
#######
#@...h#
#######
"#;

  let mut scenario = Scenario::from_ascii("CustomHero", "Shotgun specialist", ascii).unwrap();
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 100,
    max_hp: 100,
    speed: 120,
    initial_items: vec![ItemSpawnKind::AmmoShells(25), ItemSpawnKind::LargeMedPack],
    equipped_weapon: Some(ItemSpawnKind::Shotgun),
    equipped_armor: Some(ItemSpawnKind::GreenArmor),
    equipped_armor_durability: None,
  });

  let game = scenario.instantiate().unwrap();
  let player = game.world().player().unwrap();
  assert_eq!(player.hp().current, 100);
  assert_eq!(player.hp().max, 100);
  assert_eq!(player.speed().as_u32(), 120);

  let weapon = player.equipment().weapon().unwrap();
  assert_eq!(weapon.name(), "Shotgun");
  let armor = player.equipment().armor().unwrap();
  assert_eq!(armor.name(), "Green Armor");

  assert_eq!(player.inventory().items().len(), 2);
}

#[test]
fn test_scenario_scripted_run_and_metrics_accumulation() {
  let ascii = r#"
#######
#@.p..>
#######
"#;

  let scenario = Scenario::from_ascii("LootAndExit", "Pickup pistol and descend", ascii).unwrap();
  let commands = vec![
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Pickup,
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Descend,
  ];

  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();

  assert_eq!(metrics.outcome, RunOutcome::Victory);
  assert_eq!(metrics.items_picked_up, 1);
  assert_eq!(metrics.level_reached.0, 2);
  assert_eq!(replay.commands.len(), 7);
  assert!(game.world().level_id().0 > 1);
  assert!(!events.is_empty());
}

#[test]
fn test_scenario_from_protocol_fixture() {
  let fixture = ScenarioFixture::new(
    "ProtocolFixture",
    "Fixture constructed via protocol domain types",
    8,
    8,
    Position::new(2, 2),
  )
  .with_stairs(Position::new(5, 5))
  .with_seed(9999);

  let scenario = Scenario::from_fixture(&fixture).unwrap();
  assert_eq!(scenario.name, "ProtocolFixture");
  assert_eq!(scenario.width, 8);
  assert_eq!(scenario.height, 8);
  assert_eq!(scenario.player_start, Position::new(2, 2));
  assert_eq!(scenario.stairs, Some(Position::new(5, 5)));
  assert_eq!(scenario.seed, 9999);

  let game = scenario.instantiate().unwrap();
  assert_eq!(
    game.world().player().unwrap().position(),
    Position::new(2, 2)
  );
}

#[test]
fn test_ascii_fixture_preserves_acid_water_and_mud_tiles() {
  let fixture = ScenarioFixture::with_ascii_map(
    "AcidTerrain",
    "ASCII fixture exposes the terrain used by Acid Spitter reload",
    "#######\n#@xwu.#\n#######\n",
  );

  let scenario = Scenario::from_fixture(&fixture).unwrap();
  assert_eq!(scenario.tiles.get(&Position::new(2, 1)), Some(&Tile::Acid));
  assert_eq!(scenario.tiles.get(&Position::new(3, 1)), Some(&Tile::Water));
  assert_eq!(scenario.tiles.get(&Position::new(4, 1)), Some(&Tile::Mud));

  let game = scenario.instantiate().unwrap();
  assert_eq!(
    game.world().map().get_tile(Position::new(2, 1)),
    Some(Tile::Acid)
  );
  assert_eq!(
    game.world().map().get_tile(Position::new(3, 1)),
    Some(Tile::Water)
  );
  assert_eq!(
    game.world().map().get_tile(Position::new(4, 1)),
    Some(Tile::Mud)
  );
}

#[test]
fn subtle_knife_vertical_scenario_preserves_visibility_and_replay() {
  let ascii = "############\n#@..i...#i##\n#..........#\n############\n";
  let mut scenario = Scenario::from_ascii(
    "SubtleKnifeVertical",
    "Visible and occluded targets for the typed Subtle Knife encounter",
    ascii,
  )
  .unwrap();
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::SubtleKnife),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &[Command::Invoke(ItemId::new(4))]).unwrap();
  let visible_id = game
    .world()
    .actors()
    .values()
    .find(|actor| actor.position() == Position::new(4, 1))
    .unwrap()
    .id();
  let hidden_id = game
    .world()
    .actors()
    .values()
    .find(|actor| actor.position() == Position::new(9, 1))
    .unwrap()
    .id();

  let invoke_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::SubtleKnifeInvoked {
          item_id,
          targets,
          remaining_hp: 45,
          ..
        } if *item_id == ItemId::new(4) && targets == &[visible_id]
      )
    })
    .expect("vertical invoke event must name only the visible target");
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied { target_id, amount: 15, .. }
          if *target_id == visible_id
      )
    })
    .expect("visible target damage event must be emitted");
  let cost_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::ActionCostPaid { .. }))
    .expect("accepted invoke must pay its action cost");
  let turn_end_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::TurnEnded { .. }))
    .expect("accepted invoke must end its turn");

  assert!(invoke_index < damage_index);
  assert!(damage_index < cost_index);
  assert!(cost_index < turn_end_index);
  assert_eq!(game.world().get_actor(visible_id).unwrap().hp().current, 5);
  assert_eq!(game.world().get_actor(hidden_id).unwrap().hp().current, 20);
  assert_eq!(game.world().player().unwrap().hp().current, 45);
  assert!(game.world().player().unwrap().is_tired());
  assert_eq!(replay.commands, vec![Command::Invoke(ItemId::new(4))]);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn trigun_vertical_scenario_preserves_nuke_order_and_replay() {
  let ascii = "############\n#@..i...#i##\n#..........#\n############\n";
  let mut scenario = Scenario::from_ascii(
    "TrigunVertical",
    "Confirmed Trigun nuke encounter with visible and occluded actors",
    ascii,
  )
  .unwrap();
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 20,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Trigun),
    equipped_armor: None,
    equipped_armor_durability: None,
  });
  let trigun_id = scenario
    .instantiate()
    .unwrap()
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .id();

  let (game, events, _metrics, replay) = ScenarioRunner::run_commands(
    &scenario,
    &[Command::AltReload {
      item_id: trigun_id,
      confirmed: true,
    }],
  )
  .unwrap();
  let player_id = game.world().player_id().unwrap();
  let visible_id = game
    .world()
    .actors()
    .values()
    .find(|actor| actor.position() == Position::new(4, 1))
    .unwrap()
    .id();
  let hidden_id = game
    .world()
    .actors()
    .values()
    .find(|actor| actor.position() == Position::new(9, 1))
    .unwrap()
    .id();

  let reload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::TrigunAltReloaded {
          entity_id,
          item_id,
          remaining_hp: drl_protocol::HitPoints { current: 15, max: 45 },
          score_count_remaining: -1_000,
        } if *entity_id == player_id && *item_id == trigun_id
      )
    })
    .expect("vertical reload event must preserve typed costs");
  let activate_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::NukeActivated { countdown: 1, .. }))
    .expect("vertical reload must activate a one-tick nuke");
  let level_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::LevelNuked { .. }))
    .expect("vertical reload must resolve the nuke");
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id,
          amount: 15,
          remaining_hp: 0,
          source: drl_protocol::DamageSource::Environment,
          damage_type: None,
        } if *target_id == player_id
      )
    })
    .expect("nuke must apply terminal internal damage to the player");
  let death_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ActorDied {
          entity_id,
          cause: drl_protocol::DeathCause::Environment,
        } if *entity_id == player_id
      )
    })
    .expect("nuke must emit terminal death in order");

  assert!(reload_index < activate_index);
  assert!(activate_index < level_index);
  assert!(level_index < damage_index);
  assert!(damage_index < death_index);
  assert!(game.is_game_over());
  assert_eq!(game.world().player().unwrap().hp().current, 0);
  assert_eq!(game.world().get_actor(visible_id).unwrap().hp().current, 20);
  assert_eq!(game.world().get_actor(hidden_id).unwrap().hp().current, 20);
  assert_eq!(
    replay.commands,
    vec![Command::AltReload {
      item_id: trigun_id,
      confirmed: true,
    }]
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn acid_spitter_vertical_scenario_preserves_terrain_reload_and_replay() {
  let ascii = "########\n#@w....#\n#......#\n########\n";
  let mut scenario = Scenario::from_ascii(
    "AcidSpitterVertical",
    "Acid Spitter reload converts the current cell to Water",
    ascii,
  )
  .unwrap();
  let player_position = scenario.player_start;
  scenario.tiles.insert(player_position, Tile::Acid);
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::AcidSpitter),
    equipped_armor: None,
    equipped_armor_durability: None,
  });
  let acid_spitter_id = scenario
    .instantiate()
    .unwrap()
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .id();

  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &[Command::Reload]).unwrap();
  let player_id = game.world().player_id().unwrap();
  let reload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::AcidSpitterReloaded {
          entity_id,
          item_id,
          position,
          ammo_loaded: 1,
          current_clip: 1,
          max_clip: 10,
          score_count_remaining: -1_000,
        } if *entity_id == player_id
          && *item_id == acid_spitter_id
          && *position == player_position
      )
    })
    .expect("vertical reload event must preserve terrain-fed costs");
  let cost_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::ActionCostPaid { entity_id, .. } if *entity_id == player_id))
    .expect("accepted reload must pay an action cost");
  let turn_end_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::TurnEnded { .. }))
    .expect("accepted reload must end its turn");

  assert!(reload_index < cost_index);
  assert!(cost_index < turn_end_index);
  assert_eq!(
    game.world().map().get_tile(player_position),
    Some(Tile::Water)
  );
  assert_eq!(
    game
      .world()
      .map()
      .get_tile(player_position + Direction::East),
    Some(Tile::Water)
  );
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  assert_eq!(weapon.id(), acid_spitter_id);
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 1);
  assert_eq!(game.world().player().unwrap().score_count(), -1_000);
  assert_eq!(replay.commands, vec![Command::Reload]);
  let (replayed, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replay_events, events);
  assert_eq!(replayed, game);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn null_pointer_vertical_scenario_preserves_boss_hit_and_replay() {
  let ascii = "########\n#@i....#\n#......#\n########\n";
  let mut scenario = Scenario::from_ascii(
    "NullPointerVertical",
    "Boss target for the typed Null Pointer encounter",
    ascii,
  )
  .unwrap();
  scenario.seed = 25;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::NullPointer),
    equipped_armor: None,
    equipped_armor_durability: None,
  });
  scenario.monsters[0].name = "Boss Target".to_string();
  scenario.monsters[0].is_boss = true;

  let target_position = Position::new(2, 1);
  let command = Command::AttackRanged(target_position);
  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &[command]).unwrap();
  let player_id = game.world().player_id().unwrap();
  let target_id = game
    .world()
    .actors()
    .values()
    .find(|actor| actor.position() == target_position)
    .unwrap()
    .id();
  let item_id = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .id();

  let attack_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id: resolved_target,
          outcome: drl_protocol::AttackOutcome::Hit { damage: 0, .. },
          is_ranged: true,
        } if *attacker_id == player_id && *resolved_target == target_id
      )
    })
    .expect("Null Pointer ranged hit must resolve");
  let hit_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::NullPointerHit {
          entity_id,
          item_id: resolved_item,
          target_id: resolved_target,
          target_is_boss: true,
          score_count_remaining: 1000,
        } if *entity_id == player_id && *resolved_item == item_id && *resolved_target == target_id
      )
    })
    .expect("boss score branch must emit a typed hit event");
  let explosion_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::NullPointerExplosionScheduled {
          entity_id,
          target_id: resolved_target,
          delay: 50,
          radius: 1,
          damage: 10,
        } if *entity_id == player_id && *resolved_target == target_id
      )
    })
    .expect("deferred explosion must be scheduled");
  let cost_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::ActionCostPaid { .. }))
    .expect("accepted ranged attack must pay its action cost");
  let turn_end_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::TurnEnded { .. }))
    .expect("accepted ranged attack must end its turn");

  assert!(attack_index < hit_index);
  assert!(hit_index < explosion_index);
  assert!(explosion_index < cost_index);
  assert!(cost_index < turn_end_index);
  assert_eq!(
    game.world().get_actor(target_id).unwrap().score_count(),
    1000
  );
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    59
  );
  assert_eq!(replay.commands, vec![command]);

  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replay_events, events);
  assert_eq!(replayed_game, game);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn grammaton_vertical_scenario_preserves_burst_mode_and_replay() {
  let ascii = "########\n#@.i...#\n#......#\n########\n";
  let mut scenario = Scenario::from_ascii(
    "GrammatonVertical",
    "Burst-mode Grammaton encounter against a visible target",
    ascii,
  )
  .unwrap();
  scenario.seed = 4;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::GrammatonBeretta),
    equipped_armor: None,
    equipped_armor_durability: None,
  });
  scenario.monsters[0].name = "Burst Target".to_string();
  scenario.monsters[0].hp = 200;
  scenario.monsters[0].speed = 1;

  let grammaton_id = scenario
    .instantiate()
    .unwrap()
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .id();
  let target_position = Position::new(3, 1);
  let mode_command = Command::AltReload {
    item_id: grammaton_id,
    confirmed: true,
  };
  let attack_command = Command::AttackRanged(target_position);
  let commands = [mode_command, attack_command];
  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  let player_id = game.world().player_id().unwrap();
  let target_id = game
    .world()
    .actors()
    .values()
    .find(|actor| actor.position() == target_position)
    .unwrap()
    .id();

  let mode_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::GrammatonFireModeChanged {
          entity_id,
          item_id,
          mode: drl_protocol::WeaponFireMode::Burst,
          score_count_remaining: -200,
        } if *entity_id == player_id && *item_id == grammaton_id
      )
    })
    .expect("mode cycle event must select Burst");
  let attack_events: Vec<_> = events
    .iter()
    .enumerate()
    .filter(|(_, event)| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id: resolved_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == player_id && *resolved_target == target_id
      )
    })
    .collect();
  assert_eq!(attack_events.len(), 3);
  assert!(attack_events.iter().all(|(index, _)| *index > mode_index));
  assert_eq!(game.world().get_actor(target_id).unwrap().hp().current, 188);
  assert_eq!(game.world().player().unwrap().score_count(), -200);
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .fire_mode,
    drl_protocol::WeaponFireMode::Burst
  );
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    15
  );
  assert_eq!(replay.commands, commands);

  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replay_events, events);
  assert_eq!(replayed_game, game);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}
