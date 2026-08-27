//! Integration tests for declarative scenario fixtures and ASCII map parsing.

use drl_core::grid::Tile;
use drl_core::replay::ReplayEngine;
use drl_core::scenario::{Scenario, ScenarioRunner};
use drl_protocol::{
  ActionCost, AttackOutcome, Command, DamageSource, Direction, GameEvent, ItemId, ItemSpawnKind,
  PlayerSpawnConfig, Position, RunOutcome, ScenarioFixture, TileKind,
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
fn nuclear_plasma_overload_hazard_scenario_preserves_nuke_order_and_replay() {
  let player_position = Position::new(1, 1);
  let mut scenario = Scenario::from_ascii(
    "NuclearPlasmaOverloadHazardVertical",
    "Confirmed Nuclear Plasma overload on an Acid tile",
    "########\n#@.....#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 794;
  scenario.tiles.insert(player_position, Tile::Acid);
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::NuclearPlasmaRifle),
    equipped_armor: None,
    equipped_armor_durability: None,
  });
  let plasma_id = scenario
    .instantiate()
    .unwrap()
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .id();

  let commands = [Command::AltReload {
    item_id: plasma_id,
    confirmed: true,
  }];
  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  let player_id = game.world().player_id().unwrap();
  let overload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::NuclearWeaponOverloaded {
          entity_id,
          item_id,
          countdown: 1,
          score_count_remaining: -1_000,
        } if *entity_id == player_id && *item_id == plasma_id
      )
    })
    .expect("hazard overload event must be emitted");
  let activate_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::NukeActivated { countdown: 1, .. }))
    .expect("hazard overload must arm a one-tick nuke");
  let level_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::LevelNuked { .. }))
    .expect("hazard overload must resolve the nuke");
  assert!(overload_index < activate_index);
  assert!(activate_index < level_index);
  assert!(game.is_game_over());
  assert!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .is_none()
  );
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn nuclear_bfg_overload_hazard_scenario_preserves_nuke_order_and_replay() {
  let player_position = Position::new(1, 1);
  let mut scenario = Scenario::from_ascii(
    "NuclearBfgOverloadHazardVertical",
    "Confirmed Nuclear BFG 9000 overload on an Acid tile",
    "########\n#@.....#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 795;
  scenario.tiles.insert(player_position, Tile::Acid);
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::NuclearBfg9000),
    equipped_armor: None,
    equipped_armor_durability: None,
  });
  let bfg_id = scenario
    .instantiate()
    .unwrap()
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .id();

  let commands = [Command::AltReload {
    item_id: bfg_id,
    confirmed: true,
  }];
  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  let player_id = game.world().player_id().unwrap();
  let overload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::NuclearWeaponOverloaded {
          entity_id,
          item_id,
          countdown: 1,
          score_count_remaining: -1_000,
        } if *entity_id == player_id && *item_id == bfg_id
      )
    })
    .expect("hazard BFG overload event must be emitted");
  let activate_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::NukeActivated { countdown: 1, .. }))
    .expect("hazard BFG overload must arm a one-tick nuke");
  let level_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::LevelNuked { .. }))
    .expect("hazard BFG overload must resolve the nuke");
  assert!(overload_index < activate_index);
  assert!(activate_index < level_index);
  assert!(game.is_game_over());
  assert!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .is_none()
  );
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn standard_bfg_exact_hit_vertical_scenario_preserves_replay() {
  let mut scenario = Scenario::from_ascii(
    "StandardBfgExactHitVertical",
    "Standard BFG 9000 bypasses to-hit sampling and consumes 40 cells",
    "########\n#@...h.#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Bfg9000),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let target = Position::new(5, 1);
  let commands = [Command::AttackRanged(target)];
  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  let player_id = game.world().player_id().unwrap();
  let target_id = game
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::AttackResolved {
        attacker_id,
        target_id: event_target,
        outcome: AttackOutcome::Hit { .. },
        is_ranged: true,
      } if *attacker_id == player_id && *event_target == target_id
    )
  }));
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActionCostPaid {
        entity_id,
        cost: ActionCost::RANGED_ATTACK,
      } if *entity_id == player_id
    )
  }));
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
    60
  );
  assert!(game.world().get_actor(target_id).unwrap().hp().current < 500);
  assert_eq!(replay.commands, commands);

  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn nuclear_bfg_exact_hit_vertical_scenario_preserves_replay() {
  let mut scenario = Scenario::from_ascii(
    "NuclearBfgExactHitVertical",
    "Nuclear BFG 9000 bypasses only to-hit sampling",
    "########\n#@...h.#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::NuclearBfg9000),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let target = Position::new(5, 1);
  let commands = [Command::AttackRanged(target)];
  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  let player_id = game.world().player_id().unwrap();
  let target_id = game
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::AttackResolved {
        attacker_id,
        target_id: event_target,
        outcome: AttackOutcome::Hit { .. },
        is_ranged: true,
      } if *attacker_id == player_id && *event_target == target_id
    )
  }));
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActionCostPaid {
        entity_id,
        cost: ActionCost::RANGED_ATTACK,
      } if *entity_id == player_id
    )
  }));
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
    0
  );
  assert!(game.world().get_actor(target_id).unwrap().hp().current < 500);
  assert_eq!(replay.commands, commands);

  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn revenants_launcher_exact_hit_vertical_scenario_preserves_replay() {
  let mut scenario = Scenario::from_ascii(
    "RevenantsLauncherExactHitVertical",
    "Revenant's Launcher bypasses only to-hit sampling",
    "########\n#@...h.#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::RevenantsLauncher),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let target = Position::new(5, 1);
  let commands = [Command::AttackRanged(target)];
  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  let player_id = game.world().player_id().unwrap();
  let target_id = game
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::AttackResolved {
        attacker_id,
        target_id: event_target,
        outcome: AttackOutcome::Hit { .. },
        is_ranged: true,
      } if *attacker_id == player_id && *event_target == target_id
    )
  }));
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
    0
  );
  assert!(game.world().get_actor(target_id).unwrap().hp().current < 500);
  assert_eq!(replay.commands, commands);

  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_exact_hit_vertical_scenario_preserves_replay() {
  let mut scenario = Scenario::from_ascii(
    "Bfg10kExactHitVertical",
    "BFG 10K bypasses only to-hit sampling",
    "########\n#@...h.#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Bfg10k),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let target = Position::new(5, 1);
  let commands = [Command::AttackRanged(target)];
  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  let player_id = game.world().player_id().unwrap();
  let target_id = game
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::AttackResolved {
        attacker_id,
        target_id: event_target,
        outcome: AttackOutcome::Hit { .. },
        is_ranged: true,
      } if *attacker_id == player_id && *event_target == target_id
    )
  }));
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
    49
  );
  assert!(game.world().get_actor(target_id).unwrap().hp().current < 500);
  assert_eq!(replay.commands, commands);

  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
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

#[test]
fn jackhammer_vertical_scenario_preserves_single_mode_and_replay() {
  let ascii = "########\n#@.i...#\n#......#\n########\n";
  let mut scenario = Scenario::from_ascii(
    "JackhammerVertical",
    "Single-mode Jackhammer encounter against a visible target",
    ascii,
  )
  .unwrap();
  scenario.seed = 3;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Jackhammer),
    equipped_armor: None,
    equipped_armor_durability: None,
  });
  scenario.monsters[0].name = "Single Target".to_string();
  scenario.monsters[0].hp = 100;
  scenario.monsters[0].speed = 1;

  let jackhammer_id = scenario
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
    item_id: jackhammer_id,
    confirmed: true,
  };
  let attack_command = Command::AttackRanged(target_position);
  let commands = [mode_command, attack_command];
  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  let player_id = game.world().player_id().unwrap();
  let target = game
    .world()
    .actors()
    .values()
    .find(|actor| actor.name() == "Single Target")
    .unwrap();
  let target_id = target.id();

  let mode_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::JackhammerFireModeChanged {
          entity_id,
          item_id,
          mode: drl_protocol::WeaponFireMode::Single,
          score_count_remaining: -1,
        } if *entity_id == player_id && *item_id == jackhammer_id
      )
    })
    .expect("mode toggle event must select Single");
  let attack_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id: resolved_target,
          outcome: drl_protocol::AttackOutcome::Hit { damage, .. },
          is_ranged: true,
        } if *attacker_id == player_id && *resolved_target == target_id && (8..=24).contains(damage)
      )
    })
    .expect("single-mode ranged attack must hit");
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id: resolved_target,
          amount,
          ..
        } if *resolved_target == target_id && (8..=24).contains(amount)
      )
    })
    .expect("single-mode attack must apply one shell's damage");
  let knockback_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ActorKnockedBack {
          entity_id,
          from,
          to,
        } if *entity_id == target_id
          && *from == target_position
          && *to == Position::new(4, 1)
      )
    })
    .expect("Jackhammer hit must preserve one-tile knockback");
  assert!(mode_index < attack_index);
  assert!(attack_index < damage_index);
  assert!(damage_index < knockback_index);
  assert_eq!(
    game.world().get_actor(target_id).unwrap().position(),
    Position::new(4, 1)
  );
  assert!(game.world().get_actor(target_id).unwrap().hp().current < 100);
  assert_eq!(game.world().player().unwrap().score_count(), -1);
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  assert_eq!(weapon.id(), jackhammer_id);
  assert_eq!(
    weapon.weapon_properties().unwrap().fire_mode,
    drl_protocol::WeaponFireMode::Single
  );
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 9);
  assert_eq!(replay.commands, commands);

  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replay_events, events);
  assert_eq!(replayed_game, game);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn lava_armor_vertical_scenario_preserves_recharge_and_replay() {
  let player_position = Position::new(1, 1);
  let mut scenario = Scenario::from_ascii(
    "LavaArmorVertical",
    "Lava Armor recharge encounter on a canonical Lava tile",
    "########\n#@=....#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 17;
  // The ASCII player marker supplies the spawn coordinate; the explicit tile
  // override records the canonical encounter's starting Lava under the player.
  scenario.tiles.insert(player_position, Tile::Lava);
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Pistol),
    equipped_armor: Some(ItemSpawnKind::LavaArmor),
    equipped_armor_durability: Some(97),
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let armor_id = initial
    .world()
    .player()
    .unwrap()
    .equipment()
    .armor()
    .unwrap()
    .id();
  assert_eq!(
    initial.world().map().get_tile(player_position),
    Some(Tile::Lava)
  );

  let commands = [Command::Wait; 5];
  let mut direct = initial.clone();
  let mut expected_events = Vec::new();
  for (index, command) in commands.iter().copied().enumerate() {
    let step_events = direct.step(command).unwrap();
    if index < 4 {
      assert_eq!(
        direct.world().player().unwrap().lava_recharge_timer(),
        (index + 1) as u32
      );
      assert!(
        !step_events
          .iter()
          .any(|event| matches!(event, GameEvent::LavaArmorRecharged { .. }))
      );
    } else {
      let recharge_index = step_events
        .iter()
        .position(|event| {
          matches!(
            event,
            GameEvent::LavaArmorRecharged {
              entity_id,
              item_id,
              durability_restored: 3,
              durability_remaining: 100,
              timer: 0,
            } if *entity_id == player_id && *item_id == armor_id
          )
        })
        .unwrap();
      assert!(matches!(
        step_events[recharge_index.saturating_sub(1)],
        GameEvent::EntityWaited { entity_id, .. } if entity_id == player_id
      ));
      assert!(matches!(
        step_events[recharge_index + 1],
        GameEvent::ActionCostPaid { entity_id, .. } if entity_id == player_id
      ));
      assert_eq!(direct.world().player().unwrap().lava_recharge_timer(), 0);
      assert_eq!(
        direct
          .world()
          .player()
          .unwrap()
          .equipment()
          .armor()
          .unwrap()
          .armor_properties()
          .unwrap()
          .durability,
        100
      );
    }
    expected_events.extend(step_events);
  }

  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(game, direct);
  assert_eq!(events, expected_events);
  assert_eq!(replay.commands, commands);
  let (scenario_replayed_game, scenario_replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(scenario_replayed_game, game);
  assert_eq!(scenario_replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());

  let mut setup_replay = drl_protocol::ReplayLog::new(17, 8, 4, player_position)
    .with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: Some(ItemSpawnKind::LavaArmor),
      equipped_armor_durability: Some(97),
    });
  setup_replay.record_tile(player_position, TileKind::Lava);
  setup_replay.record_tile(player_position + Direction::East, TileKind::Lava);
  for command in commands {
    setup_replay.record_command(command);
  }
  let (replayed_game, replay_events) = ReplayEngine::run(&setup_replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
}

#[test]
fn blaster_periodic_recharge_vertical_scenario_preserves_replay() {
  let target_position = Position::new(2, 1);
  let mut scenario = Scenario::from_ascii(
    "BlasterRechargeVertical",
    "Blaster recharge after an accepted-command interval",
    "########\n#@i....#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 31;
  // Keep the target inert while the 40-command recharge interval elapses.
  scenario.monsters[0].name = "Recharge Target".to_string();
  scenario.monsters[0].hp = 1_000;
  scenario.monsters[0].speed = 1;
  scenario.monsters[0].ranged_damage = None;
  scenario.monsters[0].ranged_range = 0;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Blaster),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let weapon_id = initial
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .id();
  let weapon = initial
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap();
  assert_eq!(weapon.archetype(), drl_protocol::ItemArchetype::Blaster);
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 10);
  assert_eq!(initial.world().player().unwrap().weapon_recharge_timer(), 0);

  let mut commands = Vec::with_capacity(40);
  commands.push(Command::AttackRanged(target_position));
  commands.extend(std::iter::repeat_n(Command::Wait, 39));

  let mut direct = initial.clone();
  let mut expected_events = Vec::new();
  let attack_events = direct.step(commands[0]).unwrap();
  assert_eq!(
    direct
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    9
  );
  assert_eq!(direct.world().player().unwrap().weapon_recharge_timer(), 1);
  assert!(
    !attack_events
      .iter()
      .any(|event| matches!(event, GameEvent::WeaponRecharged { .. }))
  );
  expected_events.extend(attack_events);

  for (index, command) in commands.iter().copied().enumerate().skip(1) {
    let step_events = direct.step(command).unwrap();
    if index < 39 {
      assert_eq!(
        direct.world().player().unwrap().weapon_recharge_timer(),
        (index + 1) as u32
      );
      assert!(
        !step_events
          .iter()
          .any(|event| matches!(event, GameEvent::WeaponRecharged { .. }))
      );
    } else {
      assert!(step_events.iter().any(|event| {
        matches!(
          event,
          GameEvent::WeaponRecharged {
            entity_id,
            item_id,
            ammo_recharged: 1,
            current_clip: 10,
            max_clip: 10,
            timer: 30,
          } if *entity_id == player_id && *item_id == weapon_id
        )
      }));
      assert_eq!(direct.world().player().unwrap().weapon_recharge_timer(), 30);
      assert_eq!(
        direct
          .world()
          .player()
          .unwrap()
          .equipment()
          .weapon()
          .unwrap()
          .weapon_properties()
          .unwrap()
          .current_clip,
        10
      );
    }
    expected_events.extend(step_events);
  }

  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(game, direct);
  assert_eq!(events, expected_events);
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn nuclear_plasma_periodic_recharge_vertical_scenario_preserves_replay() {
  let target_position = Position::new(2, 1);
  let mut scenario = Scenario::from_ascii(
    "NuclearPlasmaRechargeVertical",
    "Nuclear Plasma Rifle recharge after an accepted-command interval",
    "########\n#@i....#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 32;
  scenario.monsters[0].name = "Recharge Target".to_string();
  scenario.monsters[0].hp = 1_000;
  scenario.monsters[0].speed = 1;
  scenario.monsters[0].ranged_damage = None;
  scenario.monsters[0].ranged_range = 0;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::NuclearPlasmaRifle),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let weapon = initial
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap();
  let weapon_id = weapon.id();
  assert_eq!(
    weapon.archetype(),
    drl_protocol::ItemArchetype::NuclearPlasmaRifle
  );
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 24);
  assert_eq!(initial.world().player().unwrap().weapon_recharge_timer(), 0);

  let mut commands = Vec::with_capacity(42);
  commands.push(Command::AttackRanged(target_position));
  commands.extend(std::iter::repeat_n(Command::Wait, 41));

  let mut direct = initial.clone();
  let mut expected_events = Vec::new();
  for (index, command) in commands.iter().copied().enumerate() {
    let step_events = direct.step(command).unwrap();
    if index < 41 {
      assert!(
        !step_events
          .iter()
          .any(|event| matches!(event, GameEvent::WeaponRecharged { .. }))
      );
    } else {
      assert!(step_events.iter().any(|event| {
        matches!(
          event,
          GameEvent::WeaponRecharged {
            entity_id,
            item_id,
            ammo_recharged: 1,
            current_clip: 24,
            max_clip: 24,
            timer: 40,
          } if *entity_id == player_id && *item_id == weapon_id
        )
      }));
      assert_eq!(direct.world().player().unwrap().weapon_recharge_timer(), 40);
      assert_eq!(
        direct
          .world()
          .player()
          .unwrap()
          .equipment()
          .weapon()
          .unwrap()
          .weapon_properties()
          .unwrap()
          .current_clip,
        24
      );
    }
    expected_events.extend(step_events);
  }

  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(game, direct);
  assert_eq!(events, expected_events);
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn nuclear_bfg_periodic_recharge_vertical_scenario_preserves_replay() {
  let target_position = Position::new(2, 1);
  let mut scenario = Scenario::from_ascii(
    "NuclearBfgRechargeVertical",
    "Nuclear BFG 9000 recharge after an accepted-command interval",
    "########\n#@i....#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 33;
  scenario.monsters[0].name = "Recharge Target".to_string();
  scenario.monsters[0].hp = 1_000;
  scenario.monsters[0].speed = 1;
  scenario.monsters[0].ranged_damage = None;
  scenario.monsters[0].ranged_range = 0;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::NuclearBfg9000),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let weapon = initial
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap();
  let weapon_id = weapon.id();
  assert_eq!(
    weapon.archetype(),
    drl_protocol::ItemArchetype::NuclearBfg9000
  );
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 40);

  let mut commands = Vec::with_capacity(5);
  commands.push(Command::AttackRanged(target_position));
  commands.extend(std::iter::repeat_n(Command::Wait, 4));
  let mut direct = initial.clone();
  let mut expected_events = Vec::new();
  for (index, command) in commands.iter().copied().enumerate() {
    let step_events = direct.step(command).unwrap();
    if index < 4 {
      assert!(
        !step_events
          .iter()
          .any(|event| matches!(event, GameEvent::WeaponRecharged { .. }))
      );
    } else {
      assert!(step_events.iter().any(|event| {
        matches!(
          event,
          GameEvent::WeaponRecharged {
            entity_id,
            item_id,
            ammo_recharged: 1,
            current_clip: 1,
            max_clip: 40,
            timer: 0,
          } if *entity_id == player_id && *item_id == weapon_id
        )
      }));
      assert_eq!(direct.world().player().unwrap().weapon_recharge_timer(), 0);
    }
    expected_events.extend(step_events);
  }

  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(game, direct);
  assert_eq!(events, expected_events);
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn medical_powerarmor_vertical_scenario_preserves_repair_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "MedicalPowerarmorVertical",
    "Medical Powerarmor periodic repair encounter",
    "########\n#@.....#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 23;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 20,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Pistol),
    equipped_armor: Some(ItemSpawnKind::MedicalPowerarmor),
    equipped_armor_durability: Some(100),
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let armor_id = initial
    .world()
    .player()
    .unwrap()
    .equipment()
    .armor()
    .unwrap()
    .id();
  assert_eq!(initial.world().player().unwrap().hp().current, 20);

  let commands = [Command::Wait; 30];
  let mut direct = initial.clone();
  let mut expected_events = Vec::new();
  for (index, command) in commands.iter().copied().enumerate() {
    let step_events = direct.step(command).unwrap();
    if index < 29 {
      assert_eq!(
        direct.world().player().unwrap().medical_repair_timer(),
        (index + 1) as u32
      );
      assert_eq!(direct.world().player().unwrap().hp().current, 20);
      assert!(
        !step_events
          .iter()
          .any(|event| matches!(event, GameEvent::MedicalPowerarmorRepaired { .. }))
      );
    } else {
      let repair_index = step_events
        .iter()
        .position(|event| {
          matches!(
            event,
            GameEvent::MedicalPowerarmorRepaired {
              entity_id,
              item_id,
              healed: 1,
              remaining_hp: 21,
              durability_remaining: 99,
              timer: 20,
            } if *entity_id == player_id && *item_id == armor_id
          )
        })
        .unwrap();
      assert_eq!(repair_index, 2);
      assert!(matches!(step_events[0], GameEvent::TurnStarted { .. }));
      assert!(matches!(step_events[1], GameEvent::EntityWaited { .. }));
      assert!(matches!(step_events[3], GameEvent::ActionCostPaid { .. }));
      assert!(matches!(step_events[4], GameEvent::TurnEnded { .. }));
      assert_eq!(direct.world().player().unwrap().medical_repair_timer(), 20);
      assert_eq!(direct.world().player().unwrap().hp().current, 21);
      assert_eq!(
        direct
          .world()
          .player()
          .unwrap()
          .equipment()
          .armor()
          .unwrap()
          .armor_properties()
          .unwrap()
          .durability,
        99
      );
    }
    expected_events.extend(step_events);
  }

  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(game, direct);
  assert_eq!(events, expected_events);
  assert_eq!(replay.commands, commands);
  let (scenario_replayed_game, scenario_replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(scenario_replayed_game, game);
  assert_eq!(scenario_replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn maleks_armor_vertical_scenario_preserves_recharge_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "MalekArmorVertical",
    "Malek's Armor periodic durability recharge encounter",
    "########\n#@.....#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 24;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Pistol),
    equipped_armor: Some(ItemSpawnKind::MaleksArmor),
    equipped_armor_durability: Some(99),
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let armor_id = initial
    .world()
    .player()
    .unwrap()
    .equipment()
    .armor()
    .unwrap()
    .id();
  let commands = [Command::Wait; 56];
  let mut direct = initial.clone();
  let mut expected_events = Vec::new();

  for (index, command) in commands.iter().copied().enumerate() {
    let step_events = direct.step(command).unwrap();
    if index < 54 {
      assert_eq!(
        direct.world().player().unwrap().malek_recharge_timer(),
        (index + 1) as u32
      );
      assert!(
        !step_events
          .iter()
          .any(|event| matches!(event, GameEvent::MalekArmorRecharged { .. }))
      );
    } else if index == 54 {
      let recharge_index = step_events
        .iter()
        .position(|event| {
          matches!(
            event,
            GameEvent::MalekArmorRecharged {
              entity_id,
              item_id,
              durability_restored: 1,
              durability_remaining: 100,
              timer: 50,
            } if *entity_id == player_id && *item_id == armor_id
          )
        })
        .expect("Malek's Armor recharge event must be emitted");
      assert_eq!(recharge_index, 2);
      assert!(matches!(step_events[0], GameEvent::TurnStarted { .. }));
      assert!(matches!(step_events[1], GameEvent::EntityWaited { .. }));
      assert!(matches!(step_events[3], GameEvent::ActionCostPaid { .. }));
      assert!(matches!(step_events[4], GameEvent::TurnEnded { .. }));
      assert_eq!(direct.world().player().unwrap().malek_recharge_timer(), 50);
      assert_eq!(
        direct
          .world()
          .player()
          .unwrap()
          .equipment()
          .armor()
          .unwrap()
          .armor_properties()
          .unwrap()
          .durability,
        100
      );
    } else {
      assert_eq!(direct.world().player().unwrap().malek_recharge_timer(), 50);
      assert!(
        !step_events
          .iter()
          .any(|event| matches!(event, GameEvent::MalekArmorRecharged { .. }))
      );
    }
    expected_events.extend(step_events);
  }

  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(game, direct);
  assert_eq!(events, expected_events);
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn former_human_profile_progression_vertical_scenario_preserves_combat_pickup_and_descent() {
  let mut scenario = Scenario::from_ascii(
    "FormerHumanProfileProgressionVertical",
    "Pistol progression through a Former Human profile, dropped ammunition, and stairs",
    "########\n#@..h>.#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Progression Target".to_string();
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Pistol),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let monster_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let target = Position::new(4, 1);
  let commands = vec![
    Command::Move(Direction::East),
    Command::AttackRanged(target),
    Command::AttackRanged(target),
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Pickup,
    Command::Move(Direction::East),
    Command::Descend,
  ];

  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::Victory);
  assert_eq!(game.world().level_id().0, 2);
  assert_eq!(game.world().player().unwrap().hp().current, 44);
  assert!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .has_ammo(drl_protocol::AmmoType::Ammo9mm, 10)
  );

  let player_shots = events
    .iter()
    .filter(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id,
          is_ranged: true,
          ..
        } if *attacker_id == player_id && *target_id == monster_id
      )
    })
    .count();
  let monster_shots = events
    .iter()
    .filter(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id,
          is_ranged: true,
          ..
        } if *attacker_id == monster_id && *target_id == player_id
      )
    })
    .count();
  assert_eq!(player_shots, 2);
  assert_eq!(monster_shots, 2);

  let death_index = events
    .iter()
    .position(
      |event| matches!(event, GameEvent::ActorDied { entity_id, .. } if *entity_id == monster_id),
    )
    .expect("Former Human-profile target death event");
  let dropped_item_id = events
    .iter()
    .find_map(|event| match event {
      GameEvent::ItemDropped {
        entity_id, item_id, ..
      } if *entity_id == monster_id => Some(*item_id),
      _ => None,
    })
    .expect("Former Human ammunition drop");
  let pickup_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ItemPickedUp { entity_id, item_id, .. }
          if *entity_id == player_id && *item_id == dropped_item_id
      )
    })
    .expect("dropped ammunition pickup");
  let transition_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::LevelTransitioned {
          from_level,
          to_level,
        } if from_level.0 == 1 && to_level.0 == 2
      )
    })
    .expect("stairs descent event");
  assert!(death_index < pickup_index);
  assert!(pickup_index < transition_index);
  assert_eq!(replay.commands, commands);

  let (scenario_replayed_game, scenario_replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(scenario_replayed_game, game);
  assert_eq!(scenario_replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn phase_device_vertical_scenario_preserves_teleport_pickup_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "PhaseDeviceVertical",
    "Phase Device escape from a fixed arena",
    "########\n#@P....#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 9999;

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let initial_position = initial.world().player().unwrap().position();
  let device_id = initial
    .world()
    .ground_items()
    .keys()
    .next()
    .copied()
    .unwrap();
  let commands = vec![
    Command::Move(Direction::East),
    Command::Pickup,
    Command::Use(device_id),
  ];

  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  let final_position = game.world().player().unwrap().position();
  assert_eq!(metrics.items_picked_up, 1);
  assert_eq!(game.world().level_id().0, 1);
  assert_ne!(final_position, initial_position);
  assert_eq!(final_position, Position::new(6, 2));
  assert!(game.world().map().is_walkable(final_position));
  assert!(game.world().is_explored(final_position));
  assert!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(device_id)
      .is_none()
  );

  let pickup_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ItemPickedUp {
          entity_id,
          item_id,
          ..
        } if *entity_id == player_id && *item_id == device_id
      )
    })
    .expect("phase device pickup event");
  let teleport_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::PlayerTeleported { from, to }
          if *from == Position::new(2, 1) && *to == final_position
      )
    })
    .expect("phase device teleport event");
  let used_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ItemUsed {
          entity_id,
          item_id,
          item_name,
        } if *entity_id == player_id
          && *item_id == device_id
          && item_name == "Phase Device"
      )
    })
    .expect("phase device use event");
  assert!(pickup_index < teleport_index);
  assert!(teleport_index < used_index);
  assert_eq!(replay.commands, commands);

  let (scenario_replayed_game, scenario_replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(scenario_replayed_game, game);
  assert_eq!(scenario_replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn shotgun_knockback_vertical_scenario_preserves_response_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "ShotgunKnockbackVertical",
    "Shotgun knockback against a Former Sergeant profile",
    "########\n#@.s...#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Knockback Target".to_string();
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Shotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let target = Position::new(3, 1);
  let commands = vec![Command::AttackRanged(target)];

  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(
    game.world().player().unwrap().position(),
    Position::new(1, 1)
  );
  assert_eq!(game.world().player().unwrap().hp().current, 47);
  let target_actor = game.world().get_actor(target_id).unwrap();
  assert_eq!(target_actor.position(), Position::new(4, 1));
  assert_eq!(target_actor.hp().current, 3);

  let player_attack_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: AttackOutcome::Hit { damage: 12, is_lethal: false },
          is_ranged: true,
        } if *attacker_id == player_id && *event_target == target_id
      )
    })
    .expect("Shotgun hit event");
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id: event_target,
          amount: 12,
          remaining_hp: 3,
          source: DamageSource::Actor(source_id),
          ..
        } if *event_target == target_id && *source_id == player_id
      )
    })
    .expect("Shotgun damage event");
  let knockback_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ActorKnockedBack { entity_id, from, to }
          if *entity_id == target_id
            && *from == Position::new(3, 1)
            && *to == Position::new(4, 1)
      )
    })
    .expect("Shotgun knockback event");
  let response_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == target_id && *event_target == player_id
      )
    })
    .expect("Former Sergeant response event");
  let response_damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id: event_target,
          amount: 3,
          remaining_hp: 47,
          source: DamageSource::Actor(source_id),
          ..
        } if *event_target == player_id && *source_id == target_id
      )
    })
    .expect("Former Sergeant response damage event");
  assert!(player_attack_index < damage_index);
  assert!(damage_index < knockback_index);
  assert!(knockback_index < response_index);
  assert!(response_index < response_damage_index);
  assert_eq!(replay.commands, commands);

  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn green_armor_protection_vertical_scenario_preserves_mitigation_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "GreenArmorProtectionVertical",
    "Green Armor mitigation against a Former Sergeant profile",
    "########\n#@.s...#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 4;
  scenario.monsters[0].name = "Armor Target".to_string();
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Pistol),
    equipped_armor: Some(ItemSpawnKind::GreenArmor),
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  assert_eq!(initial.world().player().unwrap().armor_protection(), 5);
  let commands = vec![Command::Wait];

  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(game.world().player().unwrap().hp().current, 49);
  assert_eq!(
    game.world().get_actor(target_id).unwrap().position(),
    Position::new(3, 1)
  );

  let response_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: AttackOutcome::Hit { damage: 3, is_lethal: false },
          is_ranged: true,
        } if *attacker_id == target_id && *event_target == player_id
      )
    })
    .expect("Former Sergeant-profile response event");
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id: event_target,
          amount: 1,
          remaining_hp: 49,
          source: DamageSource::Actor(source_id),
          ..
        } if *event_target == player_id && *source_id == target_id
      )
    })
    .expect("Green Armor mitigated damage event");
  assert!(response_index < damage_index);
  assert_eq!(replay.commands, commands);

  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn small_medpack_recovery_vertical_scenario_preserves_cap_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "SmallMedPackRecoveryVertical",
    "Small MedPack recovery at the health cap",
    "########\n#@.....#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 2;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 45,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::SmallMedPack],
    equipped_weapon: None,
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let medpack_id = *initial
    .world()
    .player()
    .unwrap()
    .inventory()
    .items()
    .keys()
    .next()
    .unwrap();
  assert_eq!(medpack_id, ItemId::new(4));
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(medpack_id)
      .unwrap()
      .name(),
    "Small MedPack"
  );
  let commands = vec![Command::Use(medpack_id)];

  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.items_used, 1);
  assert_eq!(game.world().player().unwrap().hp().current, 50);
  assert!(game.world().player().unwrap().inventory().is_empty());

  let turn_started = events
    .iter()
    .position(|event| matches!(event, GameEvent::TurnStarted { .. }))
    .unwrap();
  let item_used = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ItemUsed { entity_id, item_id, item_name }
          if *entity_id == player_id
            && *item_id == medpack_id
            && item_name == "Small MedPack"
      )
    })
    .unwrap();
  let action_cost = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::ActionCostPaid { entity_id, cost }
          if *entity_id == player_id && *cost == ActionCost(1000)
      )
    })
    .unwrap();
  let turn_ended = events
    .iter()
    .position(|event| matches!(event, GameEvent::TurnEnded { .. }))
    .unwrap();
  assert!(turn_started < item_used);
  assert!(item_used < action_cost);
  assert!(action_cost < turn_ended);
  assert_eq!(replay.commands, commands);

  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn demon_medpack_recovery_vertical_scenario_preserves_ai_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "DemonMedPackRecoveryVertical",
    "Demon melee pressure around Small MedPack recovery",
    "########\n#@d....#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Rush Demon".to_string();
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 46,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::SmallMedPack],
    equipped_weapon: None,
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let medpack_id = *initial
    .world()
    .player()
    .unwrap()
    .inventory()
    .items()
    .keys()
    .next()
    .unwrap();
  assert_eq!(medpack_id, ItemId::new(4));
  assert_eq!(
    initial.world().get_actor(target_id).unwrap().name(),
    "Rush Demon"
  );
  let commands = vec![Command::Wait, Command::Use(medpack_id)];

  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.turns_survived, 1);
  assert_eq!(metrics.damage_taken, 15);
  assert_eq!(metrics.items_used, 1);
  assert_eq!(game.world().player().unwrap().hp().current, 41);
  assert!(game.world().player().unwrap().inventory().is_empty());
  assert_eq!(events.len(), 14);
  assert!(matches!(events[0], GameEvent::TurnStarted { .. }));
  assert!(matches!(events[1], GameEvent::EntityWaited { entity_id, .. } if entity_id == player_id));
  assert!(
    matches!(events[2], GameEvent::ActionCostPaid { entity_id, cost: ActionCost(1000) } if entity_id == player_id)
  );
  assert!(matches!(
    events[3],
    GameEvent::AttackResolved {
      attacker_id,
      target_id: event_target,
      outcome: AttackOutcome::Hit { damage: 6, is_lethal: false },
      is_ranged: false,
    } if attacker_id == target_id && event_target == player_id
  ));
  assert!(matches!(
    events[4],
    GameEvent::DamageApplied {
      target_id: event_target,
      amount: 6,
      remaining_hp: 40,
      source: DamageSource::Actor(source_id),
      ..
    } if event_target == player_id && source_id == target_id
  ));
  assert!(
    matches!(events[5], GameEvent::ActionCostPaid { entity_id, cost: ActionCost(1000) } if entity_id == target_id)
  );
  assert!(matches!(events[6], GameEvent::TurnEnded { .. }));
  assert!(matches!(events[7], GameEvent::TurnStarted { .. }));
  assert!(matches!(
    &events[8],
    GameEvent::ItemUsed { entity_id, item_id, item_name }
      if *entity_id == player_id && *item_id == medpack_id && item_name == "Small MedPack"
  ));
  assert!(
    matches!(events[9], GameEvent::ActionCostPaid { entity_id, cost: ActionCost(1000) } if entity_id == player_id)
  );
  assert!(matches!(
    events[10],
    GameEvent::AttackResolved {
      attacker_id,
      target_id: event_target,
      outcome: AttackOutcome::Hit { damage: 9, is_lethal: false },
      is_ranged: false,
    } if attacker_id == target_id && event_target == player_id
  ));
  assert!(matches!(
    events[11],
    GameEvent::DamageApplied {
      target_id: event_target,
      amount: 9,
      remaining_hp: 41,
      source: DamageSource::Actor(source_id),
      ..
    } if event_target == player_id && source_id == target_id
  ));
  assert!(
    matches!(events[12], GameEvent::ActionCostPaid { entity_id, cost: ActionCost(1000) } if entity_id == target_id)
  );
  assert!(matches!(events[13], GameEvent::TurnEnded { .. }));
  assert_eq!(replay.commands, commands);

  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn pistol_reload_vertical_scenario_preserves_ammo_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "PistolReloadVertical",
    "Pistol clip depletion and deterministic reload",
    "########\n#@.h...#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::Ammo9mm(20)],
    equipped_weapon: Some(ItemSpawnKind::Pistol),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let ammo_id = ItemId::new(4);
  let pistol_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(ammo_id)
      .unwrap()
      .count(),
    20
  );
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    10
  );
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .id(),
    pistol_id
  );
  let target_position = Position::new(3, 1);
  let mut commands = vec![Command::AttackRanged(target_position); 10];
  commands.push(Command::Reload);

  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.turns_survived, 10);
  assert_eq!(metrics.shots_fired, 10);
  assert_eq!(metrics.shots_hit, 7);
  assert_eq!(metrics.damage_dealt, 42);
  assert_eq!(game.world().get_actor(target_id).unwrap().hp().current, 458);
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
    10
  );
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(ammo_id)
      .unwrap()
      .count(),
    10
  );

  let reload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 10,
          current_clip: 10,
          max_clip: 10,
        } if *entity_id == player_id
      )
    })
    .unwrap();
  assert!(reload_index > 0);
  assert!(matches!(
    events.get(reload_index + 1),
    Some(GameEvent::ActionCostPaid { entity_id, cost: ActionCost(1000) })
      if *entity_id == player_id
  ));
  assert!(matches!(
    events.get(reload_index + 2),
    Some(GameEvent::TurnEnded { .. })
  ));
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn plasma_rifle_vertical_scenario_preserves_cell_reload_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "PlasmaRifleCellVertical",
    "Plasma Rifle cell clip depletion and deterministic reload",
    "########\n#@.h...#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoCells(12)],
    equipped_weapon: Some(ItemSpawnKind::PlasmaRifle),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let cells_id = ItemId::new(4);
  let plasma_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(cells_id)
      .unwrap()
      .count(),
    12
  );
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .id(),
    plasma_id
  );
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    6
  );

  let target = Position::new(3, 1);
  let mut commands = vec![Command::AttackRanged(target); 6];
  commands.push(Command::Reload);
  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.turns_survived, 6);
  assert_eq!(metrics.shots_fired, 6);
  assert_eq!(metrics.shots_hit, 5);
  assert_eq!(metrics.damage_dealt, 20);
  assert_eq!(game.world().get_actor(target_id).unwrap().hp().current, 480);
  let weapon = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon.current_clip, 6);
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(cells_id)
      .unwrap()
      .count(),
    6
  );

  let reload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 6,
          current_clip: 6,
          max_clip: 6,
        } if *entity_id == player_id
      )
    })
    .unwrap();
  assert_eq!(
    events[..reload_index]
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        )
      })
      .count(),
    6
  );
  assert!(matches!(
    events.get(reload_index + 1),
    Some(GameEvent::ActionCostPaid { entity_id, cost: ActionCost(1000) })
      if *entity_id == player_id
  ));
  assert!(matches!(
    events.get(reload_index + 2),
    Some(GameEvent::TurnEnded { .. })
  ));
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn rocket_launcher_vertical_scenario_preserves_one_shot_reload_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "RocketLauncherOneShotVertical",
    "Rocket Launcher one-shot clip depletion and deterministic reload",
    "########\n#@.h...#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoRockets(2)],
    equipped_weapon: Some(ItemSpawnKind::RocketLauncher),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let rockets_id = ItemId::new(4);
  let launcher_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(rockets_id)
      .unwrap()
      .count(),
    2
  );
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .id(),
    launcher_id
  );

  let target = Position::new(3, 1);
  let commands = vec![Command::AttackRanged(target), Command::Reload];
  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.turns_survived, 1);
  assert_eq!(metrics.shots_fired, 1);
  assert_eq!(metrics.shots_hit, 1);
  assert_eq!(metrics.damage_dealt, 29);
  assert_eq!(game.world().get_actor(target_id).unwrap().hp().current, 471);
  let weapon = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon.current_clip, 1);
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(rockets_id)
      .unwrap()
      .count(),
    1
  );

  let reload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 1,
          current_clip: 1,
          max_clip: 1,
        } if *entity_id == player_id
      )
    })
    .unwrap();
  assert_eq!(
    events[..reload_index]
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        )
      })
      .count(),
    1
  );
  assert!(matches!(
    events.get(reload_index + 1),
    Some(GameEvent::ActionCostPaid { entity_id, cost: ActionCost(1000) })
      if *entity_id == player_id
  ));
  assert!(matches!(
    events.get(reload_index + 2),
    Some(GameEvent::TurnEnded { .. })
  ));
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn missile_launcher_vertical_scenario_preserves_single_shell_reload_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "MissileLauncherSingleShellVertical",
    "Missile Launcher single-shell reload after deterministic clip depletion",
    "########\n#@.h...#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 1;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 1_000;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoRockets(2)],
    equipped_weapon: Some(ItemSpawnKind::MissileLauncher),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target = Position::new(3, 1);
  let commands = vec![
    Command::AttackRanged(target),
    Command::AttackRanged(target),
    Command::AttackRanged(target),
    Command::AttackRanged(target),
    Command::Reload,
    Command::Reload,
  ];
  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();

  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.shots_fired, 4);
  assert!(metrics.shots_hit <= metrics.shots_fired);
  let weapon = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon.current_clip, 2);
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .total_ammo(drl_protocol::AmmoType::Rocket),
    0
  );

  let reloads: Vec<_> = events
    .iter()
    .enumerate()
    .filter(|(_, event)| {
      matches!(
        event,
        GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 1,
          current_clip: 1..=2,
          max_clip: 4,
        } if *entity_id == player_id
      )
    })
    .collect();
  assert_eq!(reloads.len(), 2);
  for (index, _) in reloads {
    assert!(matches!(
      events.get(index + 1),
      Some(GameEvent::ActionCostPaid { entity_id, cost: ActionCost(1000) })
        if *entity_id == player_id
    ));
    assert!(matches!(
      events.get(index + 2),
      Some(GameEvent::TurnEnded { .. })
    ));
  }

  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn missile_launcher_alt_reload_vertical_scenario_preserves_full_reload_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "MissileLauncherAltReloadVertical",
    "Missile Launcher alternate full reload after clip depletion",
    "########\n#@.h...#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 1;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 1_000;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoRockets(4)],
    equipped_weapon: Some(ItemSpawnKind::MissileLauncher),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let weapon_id = initial
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .id();
  let target = Position::new(3, 1);
  let mut commands = vec![Command::AttackRanged(target); 4];
  commands.push(Command::AltReload {
    item_id: weapon_id,
    confirmed: false,
  });
  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();

  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.shots_fired, 4);
  let weapon = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon.current_clip, 4);
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .total_ammo(drl_protocol::AmmoType::Rocket),
    0
  );
  let reload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 4,
          current_clip: 4,
          max_clip: 4,
        } if *entity_id == player_id
      )
    })
    .expect("alternate reload event");
  assert!(matches!(
    events.get(reload_index + 1),
    Some(GameEvent::ActionCostPaid { entity_id, cost: ActionCost(2_500) })
      if *entity_id == player_id
  ));
  assert!(matches!(
    events.get(reload_index + 2),
    Some(GameEvent::TurnEnded { .. })
  ));

  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn chainsaw_melee_vertical_scenario_preserves_damage_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "ChainsawMeleeVertical",
    "Chainsaw melee damage against a static Demon-profile target",
    "########\n#@d....#\n#......#\n########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Chainsaw),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let chainsaw_id = ItemId::new(4);
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .id(),
    chainsaw_id
  );

  let commands = vec![Command::AttackMelee(Direction::East)];
  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.turns_survived, 0);
  assert_eq!(metrics.damage_dealt, 20);
  assert_eq!(metrics.damage_taken, 0);
  assert_eq!(game.world().get_actor(target_id).unwrap().hp().current, 480);
  assert!(matches!(
    events.first(),
    Some(GameEvent::TurnStarted { .. })
  ));
  assert!(matches!(
    events.get(1),
    Some(GameEvent::AttackResolved {
      attacker_id,
      target_id: event_target,
      outcome: AttackOutcome::Hit { damage: 20, is_lethal: false },
      is_ranged: false,
    }) if *attacker_id == player_id && *event_target == target_id
  ));
  assert!(matches!(
    events.get(2),
    Some(GameEvent::DamageApplied {
      target_id: event_target,
      amount: 20,
      remaining_hp: 480,
      source: DamageSource::Actor(source_id),
      ..
    }) if *event_target == target_id && *source_id == player_id
  ));
  assert!(matches!(
    events.get(3),
    Some(GameEvent::ActionCostPaid { entity_id, cost: ActionCost(1000) })
      if *entity_id == player_id
  ));
  assert!(matches!(events.get(4), Some(GameEvent::TurnEnded { .. })));
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn shotgun_reload_vertical_scenario_preserves_shells_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "ShotgunReloadVertical",
    "Shotgun shell clip depletion and deterministic reload",
    "#########\n#.@....h#\n#.......#\n#########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoShells(10)],
    equipped_weapon: Some(ItemSpawnKind::Shotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let shells_id = ItemId::new(4);
  let shotgun_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(shells_id)
      .unwrap()
      .count(),
    10
  );
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .id(),
    shotgun_id
  );
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    8
  );

  let target = Position::new(7, 1);
  let mut commands = vec![Command::AttackRanged(target); 8];
  commands.push(Command::Reload);
  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.turns_survived, 8);
  assert_eq!(metrics.shots_fired, 8);
  assert_eq!(metrics.shots_hit, 5);
  assert_eq!(metrics.damage_dealt, 71);
  assert_eq!(game.world().get_actor(target_id).unwrap().hp().current, 429);
  assert_eq!(
    game.world().get_actor(target_id).unwrap().position(),
    target
  );
  assert!(!events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActorKnockedBack { entity_id, .. } if *entity_id == target_id
    )
  }));
  let weapon = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon.current_clip, 8);
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(shells_id)
      .unwrap()
      .count(),
    2
  );

  let reload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 8,
          current_clip: 8,
          max_clip: 8,
        } if *entity_id == player_id
      )
    })
    .unwrap();
  assert_eq!(
    events[..reload_index]
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        )
      })
      .count(),
    8
  );
  assert!(matches!(
    events.get(reload_index + 1),
    Some(GameEvent::ActionCostPaid { entity_id, cost: ActionCost(1200) })
      if *entity_id == player_id
  ));
  assert!(matches!(
    events.get(reload_index + 2),
    Some(GameEvent::TurnEnded { .. })
  ));
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn assault_shotgun_vertical_scenario_preserves_shells_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "AssaultShotgunVertical",
    "Assault Shotgun shell clip depletion and deterministic reload",
    "#########\n#.@....h#\n#.......#\n#########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoShells(8)],
    equipped_weapon: Some(ItemSpawnKind::AssaultShotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let shells_id = ItemId::new(4);
  let shotgun_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(shells_id)
      .unwrap()
      .count(),
    8
  );
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .id(),
    shotgun_id
  );
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    6
  );

  let target = Position::new(7, 1);
  let mut commands = vec![Command::AttackRanged(target); 6];
  commands.push(Command::Reload);
  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.turns_survived, 6);
  assert_eq!(metrics.shots_fired, 6);
  assert_eq!(metrics.shots_hit, 4);
  assert_eq!(metrics.damage_dealt, 67);
  assert_eq!(game.world().get_actor(target_id).unwrap().hp().current, 433);
  assert_eq!(
    game.world().get_actor(target_id).unwrap().position(),
    target
  );
  assert!(!events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActorKnockedBack { entity_id, .. } if *entity_id == target_id
    )
  }));
  let weapon = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon.current_clip, 1);
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(shells_id)
      .unwrap()
      .count(),
    7
  );

  let reload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 1,
          current_clip: 1,
          max_clip: 6,
        } if *entity_id == player_id
      )
    })
    .unwrap();
  assert_eq!(
    events[..reload_index]
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        )
      })
      .count(),
    6
  );
  assert!(matches!(
    events.get(reload_index + 1),
    Some(GameEvent::ActionCostPaid { entity_id, cost: ActionCost(1000) })
      if *entity_id == player_id
  ));
  assert!(matches!(
    events.get(reload_index + 2),
    Some(GameEvent::TurnEnded { .. })
  ));
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn assault_shotgun_alt_reload_vertical_scenario_preserves_full_reload_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "AssaultShotgunAltReloadVertical",
    "Assault Shotgun alternate full reload against a static target",
    "#########\n#.@....h#\n#.......#\n#########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoShells(8)],
    equipped_weapon: Some(ItemSpawnKind::AssaultShotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let weapon_id = initial
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .id();
  let target = Position::new(7, 1);
  let mut commands = vec![Command::AttackRanged(target); 6];
  commands.push(Command::AltReload {
    item_id: weapon_id,
    confirmed: false,
  });

  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.turns_survived, 6);
  assert_eq!(metrics.shots_fired, 6);
  assert_eq!(metrics.shots_hit, 4);
  assert_eq!(metrics.damage_dealt, 67);
  assert_eq!(game.world().get_actor(target_id).unwrap().hp().current, 433);
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .total_ammo(drl_protocol::AmmoType::Shells),
    2
  );

  let reload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 6,
          current_clip: 6,
          max_clip: 6,
        } if *entity_id == player_id
      )
    })
    .expect("alternate reload event");
  assert!(matches!(
    events.get(reload_index + 1),
    Some(GameEvent::ActionCostPaid { entity_id, cost: ActionCost(2_500) })
      if *entity_id == player_id
  ));
  assert!(matches!(
    events.get(reload_index + 2),
    Some(GameEvent::TurnEnded { .. })
  ));
  assert_eq!(replay.commands, commands);

  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn double_shotgun_vertical_scenario_preserves_shells_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "DoubleShotgunVertical",
    "Double Shotgun clip depletion and deterministic reload",
    "#########\n#.@....h#\n#.......#\n#########\n",
  )
  .unwrap();
  scenario.seed = 1;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoShells(4)],
    equipped_weapon: Some(ItemSpawnKind::DoubleShotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let shells_id = ItemId::new(4);
  let shotgun_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(shells_id)
      .unwrap()
      .count(),
    4
  );
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .id(),
    shotgun_id
  );
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    2
  );

  let target = Position::new(7, 1);
  let mut commands = vec![Command::AttackRanged(target); 2];
  commands.push(Command::Reload);
  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.turns_survived, 2);
  assert_eq!(metrics.shots_fired, 2);
  assert_eq!(metrics.shots_hit, 1);
  assert_eq!(metrics.damage_dealt, 26);
  assert_eq!(game.world().get_actor(target_id).unwrap().hp().current, 474);
  assert_eq!(
    game.world().get_actor(target_id).unwrap().position(),
    target
  );
  assert!(!events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActorKnockedBack { entity_id, .. } if *entity_id == target_id
    )
  }));
  let weapon = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon.current_clip, 2);
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(shells_id)
      .unwrap()
      .count(),
    2
  );

  let reload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 2,
          current_clip: 2,
          max_clip: 2,
        } if *entity_id == player_id
      )
    })
    .unwrap();
  assert_eq!(
    events[..reload_index]
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        )
      })
      .count(),
    2
  );
  assert!(matches!(
    events.get(reload_index + 1),
    Some(GameEvent::ActionCostPaid { entity_id, cost: ActionCost(1000) })
      if *entity_id == player_id
  ));
  assert!(matches!(
    events.get(reload_index + 2),
    Some(GameEvent::TurnEnded { .. })
  ));
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn combat_pump_vertical_scenario_preserves_shells_and_replay() {
  let mut scenario = Scenario::from_ascii(
    "CombatPumpVertical",
    "Combat Shotgun pump cycles, shell reload, and deterministic replay",
    "#########\n#.@....h#\n#.......#\n#########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoShells(10)],
    equipped_weapon: Some(ItemSpawnKind::CombatShotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let shells_id = ItemId::new(4);
  let shotgun_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(shells_id)
      .unwrap()
      .count(),
    10
  );
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .id(),
    shotgun_id
  );
  assert_eq!(
    initial
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    5
  );

  let target = Position::new(7, 1);
  let mut commands = Vec::new();
  for index in 0..5 {
    commands.push(Command::AttackRanged(target));
    if index < 4 {
      commands.push(Command::Reload);
    }
  }
  commands.push(Command::Reload);
  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.turns_survived, 9);
  assert_eq!(metrics.shots_fired, 5);
  assert_eq!(metrics.shots_hit, 3);
  assert_eq!(metrics.damage_dealt, 46);
  assert_eq!(game.world().get_actor(target_id).unwrap().hp().current, 454);
  assert_eq!(
    game.world().get_actor(target_id).unwrap().position(),
    target
  );
  assert!(!events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActorKnockedBack { entity_id, .. } if *entity_id == target_id
    )
  }));
  let weapon = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon.current_clip, 1);
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(shells_id)
      .unwrap()
      .count(),
    9
  );
  let reload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 1,
          current_clip: 1,
          max_clip: 5,
        } if *entity_id == player_id
      )
    })
    .unwrap();
  assert_eq!(
    events[..reload_index]
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        )
      })
      .count(),
    5
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::ActionCostPaid {
            entity_id,
            cost: ActionCost(200),
          } if *entity_id == player_id
        )
      })
      .count(),
    4
  );
  assert!(matches!(
    events.get(reload_index + 1),
    Some(GameEvent::ActionCostPaid { entity_id, cost: ActionCost(1000) })
      if *entity_id == player_id
  ));
  assert!(matches!(
    events.get(reload_index + 2),
    Some(GameEvent::TurnEnded { .. })
  ));
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn combat_shotgun_alt_reload_vertical_scenario_resets_chamber_and_replays() {
  let mut scenario = Scenario::from_ascii(
    "CombatShotgunAltReloadVertical",
    "Combat Shotgun alternate full reload directly chambers an empty chamber",
    "#########\n#.@....h#\n#.......#\n#########\n",
  )
  .unwrap();
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoShells(10)],
    equipped_weapon: Some(ItemSpawnKind::CombatShotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  });

  let initial = scenario.instantiate().unwrap();
  let player_id = initial.world().player_id().unwrap();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let weapon_id = initial
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .id();
  let target = Position::new(7, 1);
  let mut commands = Vec::new();
  for index in 0..5 {
    commands.push(Command::AttackRanged(target));
    if index < 4 {
      commands.push(Command::Reload);
    }
  }
  commands.push(Command::AltReload {
    item_id: weapon_id,
    confirmed: false,
  });
  // This attack is intentionally immediate: the alternate reload must have
  // reset the empty chamber without a separate 200-unit pump command.
  commands.push(Command::AttackRanged(target));

  let (game, events, metrics, replay) = ScenarioRunner::run_commands(&scenario, &commands).unwrap();
  assert_eq!(metrics.outcome, RunOutcome::InProgress);
  assert_eq!(metrics.shots_fired, 6);
  assert_eq!(
    game.world().get_actor(target_id).unwrap().position(),
    target
  );
  let weapon = game
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(weapon.current_clip, 4);
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .total_ammo(drl_protocol::AmmoType::Shells),
    5
  );
  let reload_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 5,
          current_clip: 5,
          max_clip: 5,
        } if *entity_id == player_id
      )
    })
    .expect("alternate reload event");
  assert!(matches!(
    events.get(reload_index + 1),
    Some(GameEvent::ActionCostPaid { entity_id, cost: ActionCost(2_500) })
      if *entity_id == player_id
  ));
  assert_eq!(
    events[reload_index + 2..]
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        )
      })
      .count(),
    1
  );
  assert_eq!(replay.commands, commands);
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(replayed_game, game);
  assert_eq!(replay_events, events);
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}
