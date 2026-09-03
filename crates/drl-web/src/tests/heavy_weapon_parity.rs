//! Heavy/shotgun/BFG verticals projected through the boundary.

use super::*;

#[test]
fn plasma_shotgun_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoCells(10)],
    equipped_weapon: Some(ItemSpawnKind::PlasmaShotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_252, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    100,
    (1, 7),
  ));

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let player_id = initial.world().player_id().expect("player identity");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let ammo_id = ItemId::new(4);
  let plasma_shotgun_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(ammo_id)
      .expect("cell reserve")
      .count(),
    10
  );
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Plasma Shotgun")
      .id(),
    plasma_shotgun_id
  );

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct.step(command).expect("direct Plasma Shotgun command");
  let step = browser
    .submit(command)
    .expect("browser Plasma Shotgun command");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(
    expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == player_id && *event_target == target_id
      ))
      .count(),
    1
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Plasma Shotgun")
      .weapon_properties()
      .expect("Plasma Shotgun properties")
      .current_clip,
    27
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(ammo_id)
      .expect("cell reserve")
      .count(),
    10
  );

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn blaster_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoCells(10)],
    equipped_weapon: Some(ItemSpawnKind::Blaster),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_253, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    100,
    (1, 7),
  ));

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let player_id = initial.world().player_id().expect("player identity");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let cells_id = ItemId::new(4);
  let blaster_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(cells_id)
      .expect("cell reserve")
      .count(),
    10
  );
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Blaster")
      .id(),
    blaster_id
  );

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct.step(command).expect("direct Blaster command");
  let step = browser.submit(command).expect("browser Blaster command");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(
    expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == player_id && *event_target == target_id
      ))
      .count(),
    1
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Blaster")
      .weapon_properties()
      .expect("Blaster properties")
      .current_clip,
    9
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(cells_id)
      .expect("cell reserve")
      .count(),
    10
  );

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn blaster_aimed_fire_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoCells(6)],
    equipped_weapon: Some(ItemSpawnKind::Blaster),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_268, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    100,
    (1, 7),
  ));

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let command = Command::AttackRangedAimed(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct.step(command).expect("direct aimed Blaster command");
  let step = browser
    .submit(command)
    .expect("browser aimed Blaster command");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::ActionCostPaid {
        cost: drl_protocol::ActionCost(2_000),
        ..
      }
    )
  }));
  assert_eq!(direct.world().player().unwrap().weapon_recharge_timer(), 1);
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Blaster")
      .weapon_properties()
      .expect("Blaster properties")
      .current_clip,
    9
  );

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn plasma_rifle_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoCells(12)],
    equipped_weapon: Some(ItemSpawnKind::PlasmaRifle),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
      .with_ranged_combat((1, 4), 6, 65)
      .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let cells_id = ItemId::new(4);
  let plasma_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(cells_id)
      .expect("cell reserve")
      .count(),
    12
  );
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Plasma Rifle")
      .id(),
    plasma_id
  );

  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "PlasmaRifleCellVertical",
    "Plasma Rifle cell clip depletion and deterministic reload",
    "########\n#@.h...#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(player_config);
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let player_id = initial.world().player_id().expect("player identity");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let target = Position::new(3, 1);
  let commands = vec![Command::AttackRanged(target), Command::Reload];
  let ranged_attack = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::RangedAttack,
    start_tick: 0,
    duration_ticks: 2,
  };
  let hit = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Hit,
    start_tick: 2,
    duration_ticks: 1,
  };
  let reload = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Reload,
    start_tick: 0,
    duration_ticks: 3,
  };
  let expected_effects = [
    vec![
      ranged_attack,
      hit,
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 3,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 5,
        duration_ticks: 1,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 6,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 8,
        duration_ticks: 1,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 9,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 11,
        duration_ticks: 1,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 12,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 14,
        duration_ticks: 1,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 15,
        duration_ticks: 2,
      },
    ],
    vec![reload],
  ];
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut all_events = Vec::new();
  let mut reload_effects = Vec::new();

  for (index, command) in commands.iter().copied().enumerate() {
    let expected_events = direct.step(command).expect("direct Plasma Rifle command");
    let step = browser
      .submit(command)
      .expect("browser Plasma Rifle command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(step.effects, expected_effects[index]);
    if command == Command::Reload {
      reload_effects = step.effects;
    }
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    all_events.extend(expected_events);
  }

  assert_eq!(direct.world().player().unwrap().hp().current, 50);
  assert_eq!(
    direct.world().get_actor(target_id).unwrap().hp().current,
    480
  );
  let weapon = direct
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
    direct
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(cells_id)
      .unwrap()
      .count(),
    6
  );
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  assert_eq!(
    browser.observation().equipped_weapon.unwrap().clip,
    Some((6, 6))
  );
  assert_eq!(
    browser
      .observation()
      .inventory
      .iter()
      .find(|item| item.id == cells_id)
      .unwrap()
      .count,
    6
  );
  assert_eq!(reload_effects, vec![reload]);
  assert_eq!(
      all_events
        .iter()
        .filter(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { attacker_id, target_id: event_target, is_ranged: true, .. } if *attacker_id == player_id && *event_target == target_id))
        .count(),
      6
    );
  let reload_index = all_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 6,
          current_clip: 6,
          max_clip: 6,
        } if *entity_id == player_id
      )
    })
    .expect("reload event");
  assert_eq!(
    all_events[..reload_index]
      .iter()
      .filter(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::AttackResolved {
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
    all_events.get(reload_index + 1),
    Some(drl_protocol::GameEvent::ActionCostPaid {
      entity_id,
      cost: drl_protocol::ActionCost(1000),
    }) if *entity_id == player_id
  ));
  assert!(matches!(
    all_events.get(reload_index + 2),
    Some(drl_protocol::GameEvent::TurnEnded { .. })
  ));

  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, all_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn rocket_launcher_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoRockets(2)],
    equipped_weapon: Some(ItemSpawnKind::RocketLauncher),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
      .with_ranged_combat((1, 4), 6, 65)
      .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let rockets_id = ItemId::new(4);
  let launcher_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(rockets_id)
      .expect("rocket reserve")
      .count(),
    2
  );
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Rocket Launcher")
      .id(),
    launcher_id
  );

  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "RocketLauncherOneShotVertical",
    "Rocket Launcher one-shot clip depletion and deterministic reload",
    "########\n#@.h...#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(player_config);
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let player_id = initial.world().player_id().expect("player identity");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let target = Position::new(3, 1);
  let commands = vec![Command::AttackRanged(target), Command::Reload];
  let ranged_attack = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::RangedAttack,
    start_tick: 0,
    duration_ticks: 2,
  };
  let hit = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Hit,
    start_tick: 2,
    duration_ticks: 1,
  };
  let reload = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Reload,
    start_tick: 0,
    duration_ticks: 3,
  };
  let expected_effects = [
    vec![
      ranged_attack,
      hit,
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Knockback,
        start_tick: 3,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 5,
        duration_ticks: 1,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 6,
        duration_ticks: 1,
      },
    ],
    vec![reload],
  ];
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut all_events = Vec::new();
  let mut reload_effects = Vec::new();

  for (index, command) in commands.iter().copied().enumerate() {
    let expected_events = direct
      .step(command)
      .expect("direct Rocket Launcher command");
    let step = browser
      .submit(command)
      .expect("browser Rocket Launcher command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(step.effects, expected_effects[index]);
    if command == Command::Reload {
      reload_effects = step.effects;
    }
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    all_events.extend(expected_events);
  }

  assert_eq!(direct.world().player().unwrap().hp().current, 34);
  assert_eq!(
    direct.world().get_actor(target_id).unwrap().hp().current,
    445
  );
  let weapon = direct
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
    direct
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(rockets_id)
      .unwrap()
      .count(),
    1
  );
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  assert_eq!(
    browser.observation().equipped_weapon.unwrap().clip,
    Some((1, 1))
  );
  assert_eq!(
    browser
      .observation()
      .inventory
      .iter()
      .find(|item| item.id == rockets_id)
      .unwrap()
      .count,
    1
  );
  assert_eq!(reload_effects, vec![reload]);
  assert_eq!(
      all_events
        .iter()
        .filter(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { attacker_id, target_id: event_target, is_ranged: true, .. } if *attacker_id == player_id && *event_target == target_id))
        .count(),
      1
    );
  let reload_index = all_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 1,
          current_clip: 1,
          max_clip: 1,
        } if *entity_id == player_id
      )
    })
    .expect("reload event");
  assert_eq!(
    all_events[..reload_index]
      .iter()
      .filter(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::AttackResolved {
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
    all_events.get(reload_index + 1),
    Some(drl_protocol::GameEvent::ActionCostPaid {
      entity_id,
      cost: drl_protocol::ActionCost(1000),
    }) if *entity_id == player_id
  ));
  assert!(matches!(
    all_events.get(reload_index + 2),
    Some(drl_protocol::GameEvent::TurnEnded { .. })
  ));

  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, all_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn missile_launcher_single_shell_reload_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let target_position = Position::new(3, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoRockets(2)],
    equipped_weapon: Some(ItemSpawnKind::MissileLauncher),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let mut setup_replay =
    ReplayLog::new(1, 8, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Static Target", 1_000, 1, (2, 5))
      .with_ranged_combat((1, 4), 6, 65)
      .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
  );
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());

  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "MissileLauncherSingleShellVertical",
    "Missile Launcher single-shell reload after clip depletion",
    "########\n#@.h...#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 1;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 1_000;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(player_config);
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let commands = vec![
    Command::AttackRanged(target_position),
    Command::AttackRanged(target_position),
    Command::AttackRanged(target_position),
    Command::AttackRanged(target_position),
    Command::Reload,
    Command::Reload,
  ];
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut all_events = Vec::new();
  for command in commands.iter().copied() {
    let expected_events = direct
      .step(command)
      .expect("direct Missile Launcher command");
    let step = browser
      .submit(command)
      .expect("browser Missile Launcher command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    all_events.extend(expected_events);
  }

  let weapon = direct
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
    direct
      .world()
      .player()
      .unwrap()
      .inventory()
      .total_ammo(drl_protocol::AmmoType::Rocket),
    0
  );
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);

  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, all_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).unwrap());
}

#[test]
fn missile_launcher_alt_reload_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let target_position = Position::new(3, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoRockets(4)],
    equipped_weapon: Some(ItemSpawnKind::MissileLauncher),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let mut setup_replay =
    ReplayLog::new(1, 8, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Static Target", 1_000, 1, (2, 5))
      .with_ranged_combat((1, 4), 6, 65)
      .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
  );
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());

  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "MissileLauncherAltReloadVertical",
    "Missile Launcher alternate full reload after clip depletion",
    "########\n#@.h...#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 1;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 1_000;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(player_config);
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let player_id = initial.world().player_id().expect("player identity");
  let weapon_id = initial
    .world()
    .player()
    .expect("player")
    .equipment()
    .weapon()
    .expect("Missile Launcher")
    .id();
  let commands = [
    Command::AttackRanged(target_position),
    Command::AttackRanged(target_position),
    Command::AttackRanged(target_position),
    Command::AttackRanged(target_position),
    Command::AltReload {
      item_id: weapon_id,
      confirmed: false,
    },
  ];
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut all_events = Vec::new();
  for command in commands {
    let expected_events = direct
      .step(command)
      .expect("direct Missile Launcher command");
    let step = browser
      .submit(command)
      .expect("browser Missile Launcher command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    all_events.extend(expected_events);
  }

  let weapon = direct
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
    direct
      .world()
      .player()
      .unwrap()
      .inventory()
      .total_ammo(drl_protocol::AmmoType::Rocket),
    0
  );
  let reload_index = all_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 4,
          current_clip: 4,
          max_clip: 4,
        } if *entity_id == player_id
      )
    })
    .expect("alternate reload event");
  assert!(matches!(
    all_events.get(reload_index + 1),
    Some(drl_protocol::GameEvent::ActionCostPaid {
      entity_id,
      cost: drl_protocol::ActionCost(2_500),
    }) if *entity_id == player_id
  ));
  assert!(matches!(
    all_events.get(reload_index + 2),
    Some(drl_protocol::GameEvent::TurnEnded { .. })
  ));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);

  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, all_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).unwrap());
}

#[test]
fn chainsaw_melee_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Chainsaw),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(2, 1);
  let mut setup_replay =
    ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (5, 10))
      .with_death_drop(Some(ItemSpawnKind::LargeMedPack)),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let chainsaw_id = ItemId::new(4);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Chainsaw")
      .id(),
    chainsaw_id
  );

  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "ChainsawMeleeVertical",
    "Chainsaw melee damage against a static Demon-profile target",
    "########\n#@d....#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(player_config);
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let player_id = initial.world().player_id().expect("player identity");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let command = Command::AttackMelee(Direction::East);
  let expected_effects = vec![
    drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::MeleeAttack,
      start_tick: 0,
      duration_ticks: 2,
    },
    drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Hit,
      start_tick: 2,
      duration_ticks: 1,
    },
  ];
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct.step(command).expect("direct Chainsaw command");
  let step = browser.submit(command).expect("browser Chainsaw command");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(step.effects, expected_effects);
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, vec![command]);
  assert_eq!(
    direct.world().get_actor(target_id).unwrap().hp().current,
    480
  );
  assert!(matches!(
    expected_events.get(1),
    Some(drl_protocol::GameEvent::AttackResolved {
      attacker_id,
      target_id: event_target,
      outcome: drl_protocol::AttackOutcome::Hit { damage: 20, is_lethal: false },
      is_ranged: false,
    }) if *attacker_id == player_id && *event_target == target_id
  ));
  assert!(matches!(
    expected_events.get(3),
    Some(drl_protocol::GameEvent::ActionCostPaid {
      entity_id,
      cost: drl_protocol::ActionCost(1000),
    }) if *entity_id == player_id
  ));

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, expected_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn shotgun_reload_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(2, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoShells(10)],
    equipped_weapon: Some(ItemSpawnKind::Shotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(7, 1);
  let mut setup_replay =
    ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
      .with_ranged_combat((1, 4), 6, 65)
      .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let shells_id = ItemId::new(4);
  let shotgun_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(shells_id)
      .expect("shell reserve")
      .count(),
    10
  );
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Shotgun")
      .id(),
    shotgun_id
  );

  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "ShotgunReloadVertical",
    "Shotgun shell clip depletion and deterministic reload",
    "#########\n#.@....h#\n#.......#\n#########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(player_config);
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let player_id = initial.world().player_id().expect("player identity");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let target = Position::new(7, 1);
  let mut commands = vec![Command::AttackRanged(target); 8];
  commands.push(Command::Reload);
  let ranged_attack = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::RangedAttack,
    start_tick: 0,
    duration_ticks: 2,
  };
  let hit = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Hit,
    start_tick: 2,
    duration_ticks: 1,
  };
  let reload = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Reload,
    start_tick: 0,
    duration_ticks: 3,
  };
  let expected_effects = [
    vec![ranged_attack],
    vec![ranged_attack],
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![ranged_attack],
    vec![ranged_attack, hit],
    vec![reload],
  ];
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut all_events = Vec::new();
  let mut reload_effects = Vec::new();

  for (index, command) in commands.iter().copied().enumerate() {
    let expected_events = direct.step(command).expect("direct Shotgun command");
    let step = browser.submit(command).expect("browser Shotgun command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(step.effects, expected_effects[index]);
    if command == Command::Reload {
      reload_effects = step.effects;
    }
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    all_events.extend(expected_events);
  }

  assert_eq!(direct.world().player().unwrap().hp().current, 50);
  assert_eq!(
    direct.world().get_actor(target_id).unwrap().hp().current,
    429
  );
  assert_eq!(
    direct.world().get_actor(target_id).unwrap().position(),
    target
  );
  assert!(!all_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::ActorKnockedBack { entity_id, .. }
        if *entity_id == target_id
    )
  }));
  let weapon = direct
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
    direct
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(shells_id)
      .unwrap()
      .count(),
    2
  );
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  assert_eq!(
    browser.observation().equipped_weapon.unwrap().clip,
    Some((8, 8))
  );
  assert_eq!(
    browser
      .observation()
      .inventory
      .iter()
      .find(|item| item.id == shells_id)
      .unwrap()
      .count,
    2
  );
  assert_eq!(reload_effects, vec![reload]);
  assert_eq!(
      all_events
        .iter()
        .filter(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { attacker_id, target_id: event_target, is_ranged: true, .. } if *attacker_id == player_id && *event_target == target_id))
        .count(),
      8
    );
  let reload_index = all_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 8,
          current_clip: 8,
          max_clip: 8,
        } if *entity_id == player_id
      )
    })
    .expect("reload event");
  assert_eq!(
    all_events[..reload_index]
      .iter()
      .filter(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::AttackResolved {
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
    all_events.get(reload_index + 1),
    Some(drl_protocol::GameEvent::ActionCostPaid {
      entity_id,
      cost: drl_protocol::ActionCost(1200),
    }) if *entity_id == player_id
  ));
  assert!(matches!(
    all_events.get(reload_index + 2),
    Some(drl_protocol::GameEvent::TurnEnded { .. })
  ));

  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, all_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn assault_shotgun_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(2, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoShells(8)],
    equipped_weapon: Some(ItemSpawnKind::AssaultShotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(7, 1);
  let mut setup_replay =
    ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
      .with_ranged_combat((1, 4), 6, 65)
      .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let shells_id = ItemId::new(4);
  let shotgun_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(shells_id)
      .expect("shell reserve")
      .count(),
    8
  );
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Assault Shotgun")
      .id(),
    shotgun_id
  );

  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "AssaultShotgunVertical",
    "Assault Shotgun shell clip depletion and deterministic reload",
    "#########\n#.@....h#\n#.......#\n#########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(player_config);
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let player_id = initial.world().player_id().expect("player identity");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let target = Position::new(7, 1);
  let mut commands = vec![Command::AttackRanged(target); 6];
  commands.push(Command::Reload);
  let ranged_attack = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::RangedAttack,
    start_tick: 0,
    duration_ticks: 2,
  };
  let hit = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Hit,
    start_tick: 2,
    duration_ticks: 1,
  };
  let reload = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Reload,
    start_tick: 0,
    duration_ticks: 3,
  };
  let expected_effects = [
    vec![ranged_attack],
    vec![ranged_attack],
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![reload],
  ];
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut all_events = Vec::new();
  let mut reload_effects = Vec::new();

  for (index, command) in commands.iter().copied().enumerate() {
    let expected_events = direct
      .step(command)
      .expect("direct Assault Shotgun command");
    let step = browser
      .submit(command)
      .expect("browser Assault Shotgun command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(step.effects, expected_effects[index]);
    if command == Command::Reload {
      reload_effects = step.effects;
    }
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    all_events.extend(expected_events);
  }

  assert_eq!(direct.world().player().unwrap().hp().current, 50);
  assert_eq!(
    direct.world().get_actor(target_id).unwrap().hp().current,
    433
  );
  assert_eq!(
    direct.world().get_actor(target_id).unwrap().position(),
    target
  );
  assert!(!all_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::ActorKnockedBack { entity_id, .. }
        if *entity_id == target_id
    )
  }));
  let weapon = direct
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
    direct
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(shells_id)
      .unwrap()
      .count(),
    7
  );
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  assert_eq!(
    browser.observation().equipped_weapon.unwrap().clip,
    Some((1, 6))
  );
  assert_eq!(
    browser
      .observation()
      .inventory
      .iter()
      .find(|item| item.id == shells_id)
      .unwrap()
      .count,
    7
  );
  assert_eq!(reload_effects, vec![reload]);
  assert_eq!(
      all_events
        .iter()
        .filter(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { attacker_id, target_id: event_target, is_ranged: true, .. } if *attacker_id == player_id && *event_target == target_id))
        .count(),
      6
    );
  let reload_index = all_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 1,
          current_clip: 1,
          max_clip: 6,
        } if *entity_id == player_id
      )
    })
    .expect("reload event");
  assert_eq!(
    all_events[..reload_index]
      .iter()
      .filter(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::AttackResolved {
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
    all_events.get(reload_index + 1),
    Some(drl_protocol::GameEvent::ActionCostPaid {
      entity_id,
      cost: drl_protocol::ActionCost(1000),
    }) if *entity_id == player_id
  ));
  assert!(matches!(
    all_events.get(reload_index + 2),
    Some(drl_protocol::GameEvent::TurnEnded { .. })
  ));

  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, all_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn assault_shotgun_alt_reload_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(2, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoShells(8)],
    equipped_weapon: Some(ItemSpawnKind::AssaultShotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(7, 1);
  let mut setup_replay =
    ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
      .with_ranged_combat((1, 4), 6, 65)
      .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let player_id = initial.world().player_id().expect("player identity");
  let weapon_id = initial
    .world()
    .player()
    .expect("player")
    .equipment()
    .weapon()
    .expect("Assault Shotgun")
    .id();
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();

  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "AssaultShotgunAltReloadVertical",
    "Assault Shotgun alternate full reload against a static target",
    "#########\n#.@....h#\n#.......#\n#########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(player_config);
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let target = Position::new(7, 1);
  let mut commands = vec![Command::AttackRanged(target); 6];
  commands.push(Command::AltReload {
    item_id: weapon_id,
    confirmed: false,
  });
  let ranged_attack = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::RangedAttack,
    start_tick: 0,
    duration_ticks: 2,
  };
  let hit = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Hit,
    start_tick: 2,
    duration_ticks: 1,
  };
  let reload = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Reload,
    start_tick: 0,
    duration_ticks: 3,
  };
  let expected_effects = [
    vec![ranged_attack],
    vec![ranged_attack],
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![reload],
  ];
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut all_events = Vec::new();

  for (index, command) in commands.iter().copied().enumerate() {
    let expected_events = direct
      .step(command)
      .expect("direct Assault Shotgun alternate reload command");
    let step = browser
      .submit(command)
      .expect("browser Assault Shotgun alternate reload command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(step.effects, expected_effects[index]);
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    all_events.extend(expected_events);
  }

  assert_eq!(
    direct.world().get_actor(target_id).unwrap().hp().current,
    433
  );
  assert_eq!(
    direct
      .world()
      .player()
      .unwrap()
      .inventory()
      .total_ammo(drl_protocol::AmmoType::Shells),
    2
  );
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
    6
  );
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  assert!(all_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(2_500),
      } if *entity_id == player_id
    )
  }));

  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, all_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn combat_shotgun_alt_reload_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(2, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoShells(10)],
    equipped_weapon: Some(ItemSpawnKind::CombatShotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(7, 1);
  let mut setup_replay =
    ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
      .with_ranged_combat((1, 4), 6, 65)
      .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
  );
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let player_id = initial.world().player_id().expect("player identity");
  let weapon_id = initial
    .world()
    .player()
    .expect("player")
    .equipment()
    .weapon()
    .expect("Combat Shotgun")
    .id();

  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "CombatShotgunAltReloadVertical",
    "Combat Shotgun alternate full reload directly chambers an empty chamber",
    "#########\n#.@....h#\n#.......#\n#########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(player_config);
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

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
  commands.push(Command::AttackRanged(target));

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut all_events = Vec::new();
  for command in commands.iter().copied() {
    let expected_events = direct
      .step(command)
      .expect("direct Combat Shotgun alternate reload command");
    let step = browser
      .submit(command)
      .expect("browser Combat Shotgun alternate reload command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    all_events.extend(expected_events);
  }

  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Combat Shotgun")
      .weapon_properties()
      .expect("weapon properties")
      .current_clip,
    4
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .inventory()
      .total_ammo(drl_protocol::AmmoType::Shells),
    5
  );
  assert!(all_events.iter().any(|event| matches!(
    event,
    drl_protocol::GameEvent::WeaponReloaded {
      entity_id,
      ammo_loaded: 5,
      current_clip: 5,
      max_clip: 5,
    } if *entity_id == player_id
  )));
  assert!(all_events.iter().any(|event| matches!(
    event,
    drl_protocol::GameEvent::ActionCostPaid {
      entity_id,
      cost: drl_protocol::ActionCost(2_500),
    } if *entity_id == player_id
  )));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);

  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, all_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn double_shotgun_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(2, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoShells(4)],
    equipped_weapon: Some(ItemSpawnKind::DoubleShotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(7, 1);
  let mut setup_replay =
    ReplayLog::new(1, 9, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
      .with_ranged_combat((1, 4), 6, 65)
      .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let shells_id = ItemId::new(4);
  let shotgun_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(shells_id)
      .expect("shell reserve")
      .count(),
    4
  );
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Double Shotgun")
      .id(),
    shotgun_id
  );

  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "DoubleShotgunVertical",
    "Double Shotgun clip depletion and deterministic reload",
    "#########\n#.@....h#\n#.......#\n#########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 1;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(player_config);
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let player_id = initial.world().player_id().expect("player identity");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let target = Position::new(7, 1);
  let mut commands = vec![Command::AttackRanged(target)];
  commands.push(Command::Reload);
  let ranged_attack = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::RangedAttack,
    start_tick: 0,
    duration_ticks: 2,
  };
  let ranged_attack_follow_up = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::RangedAttack,
    start_tick: 2,
    duration_ticks: 2,
  };
  let hit = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Hit,
    start_tick: 4,
    duration_ticks: 1,
  };
  let reload = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Reload,
    start_tick: 0,
    duration_ticks: 3,
  };
  let expected_effects = [
    vec![ranged_attack, ranged_attack_follow_up, hit],
    vec![reload],
  ];
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut all_events = Vec::new();
  let mut reload_effects = Vec::new();

  for (index, command) in commands.iter().copied().enumerate() {
    let expected_events = direct.step(command).expect("direct Double Shotgun command");
    let step = browser
      .submit(command)
      .expect("browser Double Shotgun command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(step.effects, expected_effects[index]);
    if command == Command::Reload {
      reload_effects = step.effects;
    }
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    all_events.extend(expected_events);
  }

  assert_eq!(direct.world().player().unwrap().hp().current, 50);
  assert_eq!(
    direct.world().get_actor(target_id).unwrap().hp().current,
    474
  );
  assert_eq!(
    direct.world().get_actor(target_id).unwrap().position(),
    target
  );
  assert!(!all_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::ActorKnockedBack { entity_id, .. }
        if *entity_id == target_id
    )
  }));
  let weapon = direct
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
    direct
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(shells_id)
      .unwrap()
      .count(),
    2
  );
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  assert_eq!(
    browser.observation().equipped_weapon.unwrap().clip,
    Some((2, 2))
  );
  assert_eq!(
    browser
      .observation()
      .inventory
      .iter()
      .find(|item| item.id == shells_id)
      .unwrap()
      .count,
    2
  );
  assert_eq!(reload_effects, vec![reload]);
  assert_eq!(
      all_events
        .iter()
        .filter(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { attacker_id, target_id: event_target, is_ranged: true, .. } if *attacker_id == player_id && *event_target == target_id))
        .count(),
      2
    );
  let mut attacks = Vec::new();
  let mut damages = Vec::new();
  for (index, event) in all_events.iter().enumerate() {
    match event {
      drl_protocol::GameEvent::AttackResolved {
        attacker_id: event_attacker,
        target_id: event_target,
        outcome,
        is_ranged: true,
      } if *event_attacker == player_id && *event_target == target_id => {
        attacks.push((index, *outcome));
      }
      drl_protocol::GameEvent::DamageApplied {
        target_id: event_target,
        amount,
        ..
      } if *event_target == target_id => damages.push((index, *amount)),
      _ => {}
    }
  }
  assert_eq!(attacks.len(), 2);
  assert_eq!(damages.len(), 1);
  let hit = attacks
    .iter()
    .find_map(|(index, outcome)| match outcome {
      drl_protocol::AttackOutcome::Hit { damage, .. } => Some((*index, *damage)),
      _ => None,
    })
    .expect("the seeded dual-shot fixture must contain one hit");
  assert_eq!(hit.1, 26);
  assert_eq!(damages[0].1, hit.1);
  assert_eq!(damages[0].0, hit.0 + 1);
  let reload_index = all_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 2,
          current_clip: 2,
          max_clip: 2,
        } if *entity_id == player_id
      )
    })
    .expect("reload event");
  assert_eq!(
    all_events[..reload_index]
      .iter()
      .filter(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::AttackResolved {
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
    all_events.get(reload_index + 1),
    Some(drl_protocol::GameEvent::ActionCostPaid {
      entity_id,
      cost: drl_protocol::ActionCost(1000),
    }) if *entity_id == player_id
  ));
  assert!(matches!(
    all_events.get(reload_index + 2),
    Some(drl_protocol::GameEvent::TurnEnded { .. })
  ));

  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, all_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn combat_pump_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(2, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoShells(10)],
    equipped_weapon: Some(ItemSpawnKind::CombatShotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(7, 1);
  let mut setup_replay =
    ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Static Target", 500, 1, (2, 5))
      .with_ranged_combat((1, 4), 6, 65)
      .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
  );
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let shells_id = ItemId::new(4);
  let weapon_id = ItemId::new(5);
  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "CombatPumpVertical",
    "Combat Shotgun pump cycles, shell reload, and deterministic replay",
    "#########\n#.@....h#\n#.......#\n#########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 0;
  scenario.monsters[0].name = "Static Target".to_string();
  scenario.monsters[0].hp = 500;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(player_config);
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let player_id = initial.world().player_id().expect("player identity");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let target = Position::new(7, 1);
  let mut commands = Vec::new();
  for index in 0..5 {
    commands.push(Command::AttackRanged(target));
    if index < 4 {
      commands.push(Command::Reload);
    }
  }
  commands.push(Command::Reload);
  let ranged_attack = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::RangedAttack,
    start_tick: 0,
    duration_ticks: 2,
  };
  let hit = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Hit,
    start_tick: 2,
    duration_ticks: 1,
  };
  let reload = drl_render::EffectSpan {
    effect: drl_render::PresentationEffect::Reload,
    start_tick: 0,
    duration_ticks: 3,
  };
  let expected_effects = [
    vec![ranged_attack],
    Vec::new(),
    vec![ranged_attack],
    Vec::new(),
    vec![ranged_attack, hit],
    Vec::new(),
    vec![ranged_attack, hit],
    Vec::new(),
    vec![ranged_attack, hit],
    vec![reload],
  ];
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut all_events = Vec::new();
  for (index, command) in commands.iter().copied().enumerate() {
    let expected_events = direct.step(command).expect("direct Combat Shotgun command");
    let step = browser
      .submit(command)
      .expect("browser Combat Shotgun command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    assert_eq!(step.effects, expected_effects[index]);
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    all_events.extend(expected_events);
  }

  assert_eq!(
    direct.world().get_actor(target_id).unwrap().hp().current,
    454
  );
  assert_eq!(
    direct.world().get_actor(target_id).unwrap().position(),
    target
  );
  assert!(!all_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::ActorKnockedBack { entity_id, .. }
        if *entity_id == target_id
    )
  }));
  let weapon_item = direct
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap();
  assert_eq!(weapon_item.id(), weapon_id);
  let weapon = weapon_item.weapon_properties().unwrap();
  assert_eq!(weapon.current_clip, 1);
  assert_eq!(
    direct
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(shells_id)
      .unwrap()
      .count(),
    9
  );
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  assert_eq!(
    browser.observation().equipped_weapon.unwrap().clip,
    Some((1, 5))
  );
  assert_eq!(
    browser
      .observation()
      .inventory
      .iter()
      .find(|item| item.id == shells_id)
      .unwrap()
      .count,
    9
  );
  assert_eq!(
    all_events
      .iter()
      .filter(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::ActionCostPaid {
            entity_id,
            cost: drl_protocol::ActionCost(200),
          } if *entity_id == player_id
        )
      })
      .count(),
    4
  );
  let reload_index = all_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 1,
          current_clip: 1,
          max_clip: 5,
        } if *entity_id == player_id
      )
    })
    .expect("reload event");
  assert!(matches!(
    all_events.get(reload_index + 1),
    Some(drl_protocol::GameEvent::ActionCostPaid {
      entity_id,
      cost: drl_protocol::ActionCost(1000),
    }) if *entity_id == player_id
  ));
  assert!(matches!(
    all_events.get(reload_index + 2),
    Some(drl_protocol::GameEvent::TurnEnded { .. })
  ));
  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, all_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn standard_bfg_exact_hit_browser_boundary_matches_direct_core() {
  let player_position = Position::new(2, 1);
  let target_position = Position::new(5, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Bfg9000),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let mut setup_replay = ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    1,
    (2, 4),
  ));
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("exact-hit replay setup");
  assert!(setup_events.is_empty());

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let expected_events = direct.step(command).expect("direct exact-hit command");
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::AttackResolved {
        outcome: drl_protocol::AttackOutcome::Hit { .. },
        is_ranged: true,
        ..
      }
    )
  }));
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
    60
  );
  let player_id = direct.world().player_id().unwrap();
  let target_id = direct
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  assert_standard_bfg_schedule_event(&expected_events, player_id, target_id);

  let mut browser = BrowserSession::from_game(initial);
  let step = browser.submit(command).expect("browser exact-hit command");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, vec![command]);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("exact-hit command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, expected_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn nuclear_bfg_exact_hit_browser_boundary_matches_direct_core() {
  let player_position = Position::new(2, 1);
  let target_position = Position::new(5, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::NuclearBfg9000),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let mut setup_replay = ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    1,
    (2, 4),
  ));
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("nuclear exact-hit replay setup");
  assert!(setup_events.is_empty());

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let expected_events = direct
    .step(command)
    .expect("direct nuclear exact-hit command");
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::AttackResolved {
        outcome: drl_protocol::AttackOutcome::Hit { .. },
        is_ranged: true,
        ..
      }
    )
  }));
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
    0
  );
  let player_id = direct.world().player_id().unwrap();
  let target_id = direct
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  assert_nuclear_bfg_schedule_event(&expected_events, player_id, target_id);

  let mut browser = BrowserSession::from_game(initial);
  let step = browser
    .submit(command)
    .expect("browser nuclear exact-hit command");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, vec![command]);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("nuclear exact-hit command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, expected_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn revenants_launcher_exact_hit_browser_boundary_matches_direct_core() {
  let player_position = Position::new(2, 1);
  let target_position = Position::new(5, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::RevenantsLauncher),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let mut setup_replay = ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    1,
    (2, 4),
  ));
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("Revenant exact-hit replay setup");
  assert!(setup_events.is_empty());

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let expected_events = direct
    .step(command)
    .expect("direct Revenant exact-hit command");
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::AttackResolved {
        outcome: drl_protocol::AttackOutcome::Hit { .. },
        is_ranged: true,
        ..
      }
    )
  }));
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
    0
  );

  let mut browser = BrowserSession::from_game(initial);
  let step = browser
    .submit(command)
    .expect("browser Revenant exact-hit command");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, vec![command]);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("Revenant exact-hit command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, expected_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn bfg10k_exact_hit_browser_boundary_matches_direct_core() {
  let player_position = Position::new(2, 1);
  let target_position = Position::new(5, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Bfg10k),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let mut setup_replay = ReplayLog::new(0, 9, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    1,
    (2, 4),
  ));
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("BFG 10K exact-hit replay setup");
  assert!(setup_events.is_empty());
  let player_id = initial.world().player_id().expect("player identity");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("BFG 10K target")
    .id();

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let expected_events = direct
    .step(command)
    .expect("direct BFG 10K exact-hit command");
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::AttackResolved {
        outcome: drl_protocol::AttackOutcome::Hit { .. },
        is_ranged: true,
        ..
      }
    )
  }));
  assert_eq!(
    expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    5
  );
  assert_bfg10k_volley_events(&expected_events, player_id, target_id);
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
    25
  );

  let mut browser = BrowserSession::from_game(initial);
  let step = browser
    .submit(command)
    .expect("browser BFG 10K exact-hit command");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, vec![command]);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("BFG 10K exact-hit command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, expected_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn bfg10k_shot_cost_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let target_position = Position::new(5, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Bfg10k),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let mut setup_replay = ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    1,
    (2, 4),
  ));
  setup_replay.record_item(ItemSpawnSpec::new(
    target_position,
    ItemSpawnKind::AmmoCells(2),
  ));
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("BFG 10K vertical replay setup");
  assert!(setup_events.is_empty());
  let player_id = initial.world().player_id().expect("player identity");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("BFG 10K target")
    .id();

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let expected_events = direct
    .step(command)
    .expect("direct BFG 10K shot-cost command");
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::AttackResolved {
        outcome: drl_protocol::AttackOutcome::Hit { .. },
        is_ranged: true,
        ..
      }
    )
  }));
  assert_eq!(
    expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    5
  );
  assert_bfg10k_volley_events(&expected_events, player_id, target_id);
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::GroundItemDestroyed { position, .. }
        if *position == target_position
    )
  }));
  assert!(
    expected_events
      .iter()
      .any(|event| { matches!(event, drl_protocol::GameEvent::ActionCostPaid { .. }) })
  );
  let attack_index = expected_events
    .iter()
    .position(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { .. }))
    .expect("accepted BFG 10K shot must resolve an attack");
  let cost_index = expected_events
    .iter()
    .position(|event| matches!(event, drl_protocol::GameEvent::ActionCostPaid { .. }))
    .expect("accepted BFG 10K shot must pay an action cost");
  let turn_end_index = expected_events
    .iter()
    .position(|event| matches!(event, drl_protocol::GameEvent::TurnEnded { .. }))
    .expect("accepted BFG 10K shot must end its turn");
  assert!(attack_index < cost_index);
  assert!(cost_index < turn_end_index);
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
    25
  );

  let mut browser = BrowserSession::from_game(initial);
  let step = browser
    .submit(command)
    .expect("browser BFG 10K shot-cost command");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, vec![command]);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("BFG 10K shot-cost command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, expected_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn standard_bfg_shot_cost_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let target_position = Position::new(5, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Bfg9000),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let mut setup_replay = ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    1,
    (2, 4),
  ));
  let splash_target_position = Position::new(6, 1);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    splash_target_position,
    "Splash Target",
    500,
    1,
    (2, 4),
  ));
  setup_replay.record_item(ItemSpawnSpec::new(
    target_position,
    ItemSpawnKind::SmallMedPack,
  ));
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("standard BFG vertical replay setup");
  assert!(setup_events.is_empty());

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let expected_events = direct
    .step(command)
    .expect("direct standard BFG shot-cost command");
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::AttackResolved {
        outcome: drl_protocol::AttackOutcome::Hit { .. },
        is_ranged: true,
        ..
      }
    )
  }));
  assert!(
    expected_events
      .iter()
      .any(|event| { matches!(event, drl_protocol::GameEvent::ActionCostPaid { .. }) })
  );
  let attack_index = expected_events
    .iter()
    .position(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { .. }))
    .expect("accepted standard BFG shot must resolve an attack");
  let cost_index = expected_events
    .iter()
    .position(|event| matches!(event, drl_protocol::GameEvent::ActionCostPaid { .. }))
    .expect("accepted standard BFG shot must pay an action cost");
  let turn_end_index = expected_events
    .iter()
    .position(|event| matches!(event, drl_protocol::GameEvent::TurnEnded { .. }))
    .expect("accepted standard BFG shot must end its turn");
  assert!(attack_index < cost_index);
  assert!(cost_index < turn_end_index);
  let player_id = direct.world().player_id().unwrap();
  let target_id = direct
    .world()
    .actors()
    .values()
    .find(|actor| actor.name() == "Static Target")
    .unwrap()
    .id();
  let splash_target_id = direct
    .world()
    .actors()
    .values()
    .find(|actor| actor.name() == "Splash Target")
    .unwrap()
    .id();
  assert_standard_bfg_schedule_event(&expected_events, player_id, target_id);
  assert_eq!(
    expected_events
      .iter()
      .filter(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::DamageApplied {
            target_id: event_target,
            source: drl_protocol::DamageSource::Environment,
            damage_type: Some(drl_protocol::DamageType::Plasma),
            ..
          } if *event_target == splash_target_id
        )
      })
      .count(),
    1,
    "browser standard BFG splash must damage the second actor exactly once"
  );
  let splash_damage_index = expected_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::DamageApplied {
          target_id: event_target,
          source: drl_protocol::DamageSource::Environment,
          damage_type: Some(drl_protocol::DamageType::Plasma),
          ..
        } if *event_target == target_id
      )
    })
    .expect("browser standard BFG center actor should receive splash damage");
  let destroyed_index = expected_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::GroundItemDestroyed { position, .. }
          if *position == target_position
      )
    })
    .expect("browser standard BFG should destroy the center ground item");
  assert!(splash_damage_index < destroyed_index);
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
    60
  );

  let mut browser = BrowserSession::from_game(initial);
  let step = browser
    .submit(command)
    .expect("browser standard BFG shot-cost command");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, vec![command]);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("standard BFG shot-cost command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, expected_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn nuclear_bfg_shot_cost_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let target_position = Position::new(5, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::NuclearBfg9000),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let mut setup_replay = ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    1,
    (2, 4),
  ));
  let splash_target_position = Position::new(6, 1);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    splash_target_position,
    "Splash Target",
    500,
    1,
    (2, 4),
  ));
  setup_replay.record_item(ItemSpawnSpec::new(
    target_position,
    ItemSpawnKind::SmallMedPack,
  ));
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("Nuclear BFG vertical replay setup");
  assert!(setup_events.is_empty());

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let expected_events = direct
    .step(command)
    .expect("direct Nuclear BFG shot-cost command");
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::AttackResolved {
        outcome: drl_protocol::AttackOutcome::Hit { .. },
        is_ranged: true,
        ..
      }
    )
  }));
  let attack_index = expected_events
    .iter()
    .position(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { .. }))
    .expect("accepted Nuclear BFG shot must resolve an attack");
  let cost_index = expected_events
    .iter()
    .position(|event| matches!(event, drl_protocol::GameEvent::ActionCostPaid { .. }))
    .expect("accepted Nuclear BFG shot must pay an action cost");
  let turn_end_index = expected_events
    .iter()
    .position(|event| matches!(event, drl_protocol::GameEvent::TurnEnded { .. }))
    .expect("accepted Nuclear BFG shot must end its turn");
  assert!(attack_index < cost_index);
  assert!(cost_index < turn_end_index);
  let player_id = direct.world().player_id().unwrap();
  let target_id = direct
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let splash_target_id = direct
    .world()
    .actors()
    .values()
    .find(|actor| actor.name() == "Splash Target")
    .unwrap()
    .id();
  assert_nuclear_bfg_schedule_event(&expected_events, player_id, target_id);
  assert_eq!(
    expected_events
      .iter()
      .filter(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::DamageApplied {
            target_id: event_target,
            source: drl_protocol::DamageSource::Environment,
            damage_type: Some(drl_protocol::DamageType::Plasma),
            ..
          } if *event_target == splash_target_id
        )
      })
      .count(),
    1,
    "browser Nuclear BFG splash must damage the second actor exactly once"
  );
  let splash_damage_index = expected_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::DamageApplied {
          target_id: event_target,
          source: drl_protocol::DamageSource::Environment,
          damage_type: Some(drl_protocol::DamageType::Plasma),
          ..
        } if *event_target == target_id
      )
    })
    .expect("browser Nuclear BFG center actor should receive splash damage");
  let destroyed_index = expected_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::GroundItemDestroyed { position, .. }
          if *position == target_position
      )
    })
    .expect("browser Nuclear BFG should destroy the center ground item");
  assert!(splash_damage_index < destroyed_index);
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
    0
  );

  let mut browser = BrowserSession::from_game(initial);
  let step = browser
    .submit(command)
    .expect("browser Nuclear BFG shot-cost command");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, vec![command]);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("Nuclear BFG shot-cost command replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, expected_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}
