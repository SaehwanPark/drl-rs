//! Browser boundary versus direct `drl-core` presentation parity.

use super::*;

#[test]
fn browser_session_matches_direct_core_for_identical_commands() {
  let mut browser = BrowserSession::new().expect("fixed session");
  let mut direct = BrowserSession::fixed_game().expect("fixed core game");
  let commands = [
    Command::Wait,
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Move(Direction::East),
    Command::Pickup,
    Command::Pickup,
    Command::Pickup,
  ];
  for command in commands {
    let expected_events = direct.step(command).expect("direct command");
    let step = browser.submit(command).expect("browser command");
    assert_eq!(step.events, expected_events);
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events)
    );
    assert_eq!(step.after, direct.observe_player());
  }
  let replay = browser.replay_log();
  let (replayed, _) = drl_core::ReplayEngine::run(&replay).expect("replay browser run");
  let browser_observation = browser.observation();
  let replay_observation = replayed.observe_player();
  assert_eq!(browser_observation, replay_observation);
  assert!(drl_core::ReplayEngine::verify_determinism(&replay).expect("replay determinism"));
}

#[test]
fn subtle_knife_browser_boundary_matches_direct_core_presentation() {
  let mut setup_replay =
    ReplayLog::new(784, 30, 30, Position::new(15, 15)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::SubtleKnife),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  setup_replay.record_tile(Position::new(17, 15), TileKind::Wall);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    Position::new(16, 15),
    "Visible Imp",
    30,
    1,
    (1, 1),
  ));
  setup_replay.record_monster(MonsterSpawnSpec::new(
    Position::new(18, 15),
    "Occluded Imp",
    30,
    1,
    (1, 1),
  ));

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let command = Command::Invoke(ItemId::new(4));

  let expected_events = direct.step(command).expect("direct invoke");
  let step = browser.submit(command).expect("browser invoke");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed.observe_player(), direct.observe_player());
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn trigun_vertical_browser_boundary_matches_direct_core_presentation() {
  let mut setup_replay =
    ReplayLog::new(42, 12, 4, Position::new(1, 1)).with_player_config(PlayerSpawnConfig {
      hp: 20,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Trigun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  setup_replay.record_tile(Position::new(8, 1), TileKind::Wall);
  setup_replay.record_monster(
    MonsterSpawnSpec::new(Position::new(4, 1), "Imp", 20, 100, (3, 8))
      .with_ranged_combat((2, 5), 7, 70)
      .with_death_drop(Some(ItemSpawnKind::SmallMedPack)),
  );
  setup_replay.record_monster(
    MonsterSpawnSpec::new(Position::new(9, 1), "Imp", 20, 100, (3, 8))
      .with_ranged_combat((2, 5), 7, 70)
      .with_death_drop(Some(ItemSpawnKind::SmallMedPack)),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let initial_observation = initial.observe_player();
  let visible_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| actor.position() == Position::new(4, 1))
    .expect("visible actor")
    .id();
  let hidden_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| actor.position() == Position::new(9, 1))
    .expect("occluded actor")
    .id();
  let trigun_id = initial
    .world()
    .player()
    .expect("player")
    .equipment()
    .weapon()
    .expect("equipped Trigun")
    .id();
  assert!(
    initial_observation
      .visible_actors
      .iter()
      .any(|actor| actor.id == visible_id)
  );
  assert!(
    !initial_observation
      .visible_actors
      .iter()
      .any(|actor| actor.id == hidden_id)
  );

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let command = Command::AltReload {
    item_id: trigun_id,
    confirmed: true,
  };

  let expected_events = direct.step(command).expect("direct alternate reload");
  let step = browser.submit(command).expect("browser alternate reload");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert!(direct.is_game_over());
  assert!(browser.is_game_over());
  assert_eq!(
    direct.world().get_actor(visible_id).unwrap().hp().current,
    20
  );
  assert_eq!(
    direct.world().get_actor(hidden_id).unwrap().hp().current,
    20
  );

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed.observe_player(), direct.observe_player());
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn trigun_aimed_fire_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_268, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(6)],
      equipped_weapon: Some(ItemSpawnKind::Trigun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    100,
    (1, 7),
  ));

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("Trigun aimed replay setup");
  assert!(setup_events.is_empty());
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let command = Command::AttackRangedAimed(target_position);

  let expected_events = direct.step(command).expect("direct Trigun aimed command");
  let step = browser
    .submit(command)
    .expect("browser Trigun aimed command");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
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
    5
  );
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::ActionCostPaid {
        cost: drl_protocol::ActionCost(2_000),
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
    1
  );

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("Trigun aimed replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn anti_freak_jackal_aimed_fire_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_269, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(6)],
      equipped_weapon: Some(ItemSpawnKind::AntiFreakJackal),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    100,
    (1, 7),
  ));

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("Anti-Freak aimed replay setup");
  assert!(setup_events.is_empty());
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let command = Command::AttackRangedAimed(target_position);

  let expected_events = direct
    .step(command)
    .expect("direct Anti-Freak aimed command");
  let step = browser
    .submit(command)
    .expect("browser Anti-Freak aimed command");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
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
    5
  );
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::ActionCostPaid {
        cost: drl_protocol::ActionCost(2_000),
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
    1
  );

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("Anti-Freak aimed replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn anti_freak_jackal_browser_boundary_preserves_explosion_schedule() {
  let player_position = Position::new(2, 2);
  let target_position = Position::new(3, 2);
  let mut setup_replay =
    ReplayLog::new(0, 10, 6, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(6)],
      equipped_weapon: Some(ItemSpawnKind::AntiFreakJackal),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    1,
    (0, 0),
  ));
  setup_replay.record_monster(MonsterSpawnSpec::new(
    Position::new(4, 2),
    "Collateral Target",
    500,
    1,
    (0, 0),
  ));
  for position in [
    Position::new(3, 2),
    Position::new(3, 1),
    Position::new(4, 1),
    Position::new(4, 2),
    Position::new(4, 3),
    Position::new(3, 3),
    Position::new(2, 3),
    Position::new(2, 2),
    Position::new(2, 1),
  ] {
    setup_replay.record_item(ItemSpawnSpec::new(position, ItemSpawnKind::Ammo9mm(20)));
  }
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("Anti-Freak schedule setup");
  assert!(setup_events.is_empty());
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut all_events = Vec::new();
  for _ in 0..6 {
    let command = Command::AttackRangedAimed(target_position);
    let expected_events = direct
      .step(command)
      .expect("direct Anti-Freak schedule command");
    let step = browser
      .submit(command)
      .expect("browser Anti-Freak schedule command");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    all_events.extend(expected_events);
  }
  assert!(all_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::AntiFreakJackalExplosionScheduled {
        delay: 40,
        radius: 1,
        knockback: 8,
        ..
      }
    )
  }));
  assert!(
    all_events
      .iter()
      .any(|event| matches!(event, drl_protocol::GameEvent::ActorKnockedBack { .. }))
  );
  assert!(
    all_events
      .iter()
      .any(|event| matches!(event, drl_protocol::GameEvent::GroundItemDestroyed { .. }))
  );
  assert!(all_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::DamageApplied {
        source: drl_protocol::DamageSource::Environment,
        damage_type: Some(drl_protocol::DamageType::Fire),
        amount,
        ..
      } if (5..=15).contains(amount)
    )
  }));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands.len(), 6);
  let mut command_replay = setup_replay;
  for _ in 0..6 {
    command_replay.record_command(Command::AttackRangedAimed(target_position));
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("Anti-Freak schedule replay");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, all_events);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).unwrap());
}

#[test]
fn nuclear_plasma_overload_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let mut setup_replay =
    ReplayLog::new(794, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::NuclearPlasmaRifle),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  setup_replay.record_tile(player_position, TileKind::Acid);

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let plasma_id = initial
    .world()
    .player()
    .expect("player")
    .equipment()
    .weapon()
    .expect("equipped Nuclear Plasma Rifle")
    .id();
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let command = Command::AltReload {
    item_id: plasma_id,
    confirmed: true,
  };

  let expected_events = direct.step(command).expect("direct overload");
  let step = browser.submit(command).expect("browser overload");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert!(direct.is_game_over());
  assert!(browser.is_game_over());
  assert_eq!(direct.world().player().unwrap().equipment().weapon(), None);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn nuclear_bfg_overload_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let mut setup_replay =
    ReplayLog::new(795, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::NuclearBfg9000),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  setup_replay.record_tile(player_position, TileKind::Acid);

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let bfg_id = initial
    .world()
    .player()
    .expect("player")
    .equipment()
    .weapon()
    .expect("equipped Nuclear BFG 9000")
    .id();
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let command = Command::AltReload {
    item_id: bfg_id,
    confirmed: true,
  };

  let expected_events = direct.step(command).expect("direct overload");
  let step = browser.submit(command).expect("browser overload");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert!(direct.is_game_over());
  assert!(browser.is_game_over());
  assert_eq!(direct.world().player().unwrap().equipment().weapon(), None);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn acid_spitter_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let mut setup_replay =
    ReplayLog::new(42, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::AcidSpitter),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  setup_replay.record_tile(player_position, TileKind::Acid);
  setup_replay.record_tile(player_position + Direction::East, TileKind::Water);

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "AcidSpitterVertical",
    "Acid Spitter reload converts the current cell to Water",
    "########\n#@w....#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.tiles.insert(player_position, drl_core::Tile::Acid);
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::AcidSpitter),
    equipped_armor: None,
    equipped_armor_durability: None,
  });
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );
  let acid_spitter_id = initial
    .world()
    .player()
    .expect("player")
    .equipment()
    .weapon()
    .expect("equipped Acid Spitter")
    .id();
  assert_eq!(
    initial.world().map().get_tile(player_position),
    Some(Tile::Acid)
  );
  assert_eq!(
    initial
      .world()
      .map()
      .get_tile(player_position + Direction::East),
    Some(Tile::Water)
  );

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let command = Command::Reload;
  let expected_events = direct.step(command).expect("direct terrain reload");
  let step = browser.submit(command).expect("browser terrain reload");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(
    step.effects,
    vec![drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Reload,
      start_tick: 0,
      duration_ticks: 3,
    }]
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(
    direct.world().map().get_tile(player_position),
    Some(Tile::Water)
  );
  assert!(
    step
      .after
      .visible_tiles
      .iter()
      .any(|tile| { tile.position == player_position && tile.kind == TileKind::Water })
  );
  assert_eq!(browser.observation().player_position, player_position);
  assert_eq!(
    direct
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .id(),
    acid_spitter_id
  );

  let reload_index = expected_events
    .iter()
    .position(|event| matches!(event, drl_protocol::GameEvent::AcidSpitterReloaded { .. }))
    .expect("terrain reload event");
  let cost_index = expected_events
    .iter()
    .position(|event| matches!(event, drl_protocol::GameEvent::ActionCostPaid { .. }))
    .expect("reload action cost");
  let turn_end_index = expected_events
    .iter()
    .position(|event| matches!(event, drl_protocol::GameEvent::TurnEnded { .. }))
    .expect("reload turn end");
  assert!(reload_index < cost_index);
  assert!(cost_index < turn_end_index);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed.observe_player(), direct.observe_player());
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn null_pointer_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let target_position = Position::new(2, 1);
  let mut setup_replay =
    ReplayLog::new(25, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::NullPointer),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Boss Target", 20, 100, (3, 8))
      .with_ranged_combat((2, 5), 7, 70)
      .with_death_drop(Some(ItemSpawnKind::SmallMedPack))
      .with_boss(true),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "NullPointerVertical",
    "Boss target for the typed Null Pointer encounter",
    "########\n#@i....#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 25;
  scenario.monsters[0].name = "Boss Target".to_string();
  scenario.monsters[0].is_boss = true;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::NullPointer),
    equipped_armor: None,
    equipped_armor_durability: None,
  });
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let player_id = initial.world().player_id().expect("player");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| actor.position() == target_position)
    .expect("boss target")
    .id();
  let item_id = initial
    .world()
    .player()
    .expect("player")
    .equipment()
    .weapon()
    .expect("equipped Null Pointer")
    .id();
  assert!(
    initial
      .observe_player()
      .visible_actors
      .iter()
      .any(|actor| actor.id == target_id)
  );

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let command = Command::AttackRanged(target_position);
  let expected_events = direct.step(command).expect("direct ranged hit");
  let step = browser.submit(command).expect("browser ranged hit");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    expected_events
      .iter()
      .filter(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::DamageApplied {
            source: drl_protocol::DamageSource::Environment,
            damage_type: Some(drl_protocol::DamageType::Plasma),
            amount: 10,
            ..
          }
        )
      })
      .count(),
    2
  );
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(
    step.effects,
    vec![
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 0,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 2,
        duration_ticks: 1,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 3,
        duration_ticks: 1,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::MeleeAttack,
        start_tick: 4,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 6,
        duration_ticks: 1,
      },
    ]
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(
    direct.world().get_actor(target_id).unwrap().score_count(),
    1000
  );
  assert_eq!(
    direct
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .id(),
    item_id
  );
  assert_eq!(direct.world().player().unwrap().id(), player_id);

  let attack_index = expected_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      )
    })
    .expect("ranged attack event");
  let hit_index = expected_events
    .iter()
    .position(|event| matches!(event, drl_protocol::GameEvent::NullPointerHit { .. }))
    .expect("Null Pointer hit event");
  let explosion_index = expected_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::NullPointerExplosionScheduled { .. }
      )
    })
    .expect("deferred explosion event");
  assert!(attack_index < hit_index);
  assert!(hit_index < explosion_index);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert_eq!(replayed.observe_player(), direct.observe_player());
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn grammaton_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(4, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::GrammatonBeretta),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Burst Target", 200, 1, (3, 8))
      .with_ranged_combat((2, 5), 7, 70)
      .with_death_drop(Some(ItemSpawnKind::SmallMedPack)),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "GrammatonVertical",
    "Burst-mode Grammaton encounter against a visible target",
    "########\n#@.i...#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 4;
  scenario.monsters[0].name = "Burst Target".to_string();
  scenario.monsters[0].hp = 200;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::GrammatonBeretta),
    equipped_armor: None,
    equipped_armor_durability: None,
  });
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let grammaton_id = initial
    .world()
    .player()
    .expect("player")
    .equipment()
    .weapon()
    .expect("equipped Grammaton")
    .id();
  let mode_command = Command::AltReload {
    item_id: grammaton_id,
    confirmed: true,
  };
  let attack_command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);

  let mode_events = direct.step(mode_command).expect("direct mode cycle");
  let mode_step = browser.submit(mode_command).expect("browser mode cycle");
  assert_eq!(mode_step.events, mode_events);
  assert_eq!(mode_step.after, direct.observe_player());
  assert!(mode_step.effects.is_empty());
  assert!(mode_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::GrammatonFireModeChanged {
        item_id,
        mode: drl_protocol::WeaponFireMode::Burst,
        score_count_remaining: -200,
        ..
      } if *item_id == grammaton_id
    )
  }));

  let expected_events = direct.step(attack_command).expect("direct burst attack");
  let step = browser
    .submit(attack_command)
    .expect("browser burst attack");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(
    step.effects,
    vec![
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 0,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 2,
        duration_ticks: 1,
      },
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
    ]
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("weapon")
      .weapon_properties()
      .expect("weapon properties")
      .current_clip,
    15
  );

  let mut command_replay = setup_replay;
  command_replay.record_command(mode_command);
  command_replay.record_command(attack_command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  let mut expected_full_events = mode_events;
  expected_full_events.extend(expected_events);
  assert_eq!(replay_events, expected_full_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn jackhammer_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(3, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Jackhammer),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Single Target", 100, 1, (3, 8))
      .with_ranged_combat((2, 5), 7, 70)
      .with_death_drop(Some(ItemSpawnKind::SmallMedPack)),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "JackhammerVertical",
    "Single-mode Jackhammer encounter against a visible target",
    "########\n#@.i...#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 3;
  scenario.monsters[0].name = "Single Target".to_string();
  scenario.monsters[0].hp = 100;
  scenario.monsters[0].speed = 1;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Jackhammer),
    equipped_armor: None,
    equipped_armor_durability: None,
  });
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let jackhammer_id = initial
    .world()
    .player()
    .expect("player")
    .equipment()
    .weapon()
    .expect("equipped Jackhammer")
    .id();
  let mode_command = Command::AltReload {
    item_id: jackhammer_id,
    confirmed: true,
  };
  let attack_command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);

  let mode_events = direct.step(mode_command).expect("direct mode toggle");
  let mode_step = browser.submit(mode_command).expect("browser mode toggle");
  assert_eq!(mode_step.events, mode_events);
  assert_eq!(mode_step.after, direct.observe_player());
  assert!(mode_step.effects.is_empty());
  assert!(mode_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::JackhammerFireModeChanged {
        item_id,
        mode: drl_protocol::WeaponFireMode::Single,
        score_count_remaining: -1,
        ..
      } if *item_id == jackhammer_id
    )
  }));

  let expected_events = direct.step(attack_command).expect("direct single attack");
  let step = browser
    .submit(attack_command)
    .expect("browser single attack");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(
    step.effects,
    vec![
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 0,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 2,
        duration_ticks: 1,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Knockback,
        start_tick: 3,
        duration_ticks: 2,
      },
    ]
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("weapon")
      .weapon_properties()
      .expect("weapon properties")
      .current_clip,
    9
  );
  assert_eq!(
    direct
      .world()
      .actors()
      .values()
      .find(|actor| actor.name() == "Single Target")
      .expect("target")
      .position(),
    Position::new(4, 1)
  );

  let mut command_replay = setup_replay;
  command_replay.record_command(mode_command);
  command_replay.record_command(attack_command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  let mut expected_full_events = mode_events;
  expected_full_events.extend(expected_events);
  assert_eq!(replay_events, expected_full_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn lava_armor_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let mut setup_replay =
    ReplayLog::new(17, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: Some(ItemSpawnKind::LavaArmor),
      equipped_armor_durability: Some(97),
    });
  setup_replay.record_tile(player_position, drl_protocol::TileKind::Lava);
  setup_replay.record_tile(
    player_position + Direction::East,
    drl_protocol::TileKind::Lava,
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "LavaArmorVertical",
    "Lava Armor recharge encounter on a canonical Lava tile",
    "########\n#@=....#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 17;
  scenario.tiles.insert(player_position, drl_core::Tile::Lava);
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Pistol),
    equipped_armor: Some(ItemSpawnKind::LavaArmor),
    equipped_armor_durability: Some(97),
  });
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let commands = [Command::Wait; 5];
  let mut expected_events = Vec::new();
  for (index, command) in commands.iter().copied().enumerate() {
    let direct_events = direct.step(command).expect("direct wait");
    let step = browser.submit(command).expect("browser wait");
    assert_eq!(step.events, direct_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(step.effects, Vec::new());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &direct_events,)
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    if index < 4 {
      assert_eq!(
        direct.world().player().unwrap().lava_recharge_timer(),
        (index + 1) as u32
      );
    } else {
      assert!(direct_events.iter().any(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::LavaArmorRecharged {
            durability_restored: 3,
            durability_remaining: 100,
            timer: 0,
            ..
          }
        )
      }));
      assert_eq!(direct.world().player().unwrap().lava_recharge_timer(), 0);
    }
    expected_events.extend(direct_events);
  }

  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn blaster_recharge_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let target_position = Position::new(2, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Blaster),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target =
    drl_protocol::MonsterSpawnSpec::new(target_position, "Recharge Target", 1_000, 1, (0, 0));
  let mut setup_replay =
    ReplayLog::new(31, 8, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(target.clone());

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "BlasterRechargeVertical",
    "Blaster recharge after an accepted-command interval",
    "########\n#@i....#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 31;
  scenario.monsters[0] = target;
  scenario.player_config = Some(player_config);
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut commands = Vec::with_capacity(40);
  commands.push(Command::AttackRanged(target_position));
  commands.extend(std::iter::repeat_n(Command::Wait, 39));
  let mut expected_events = Vec::new();
  for (index, command) in commands.iter().copied().enumerate() {
    let direct_events = direct.step(command).expect("direct command");
    let step = browser.submit(command).expect("browser command");
    assert_eq!(step.events, direct_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &direct_events,)
    );
    if index > 0 {
      assert_eq!(step.effects, Vec::new());
    }
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    if index < 39 {
      assert!(
        !direct_events
          .iter()
          .any(|event| matches!(event, drl_protocol::GameEvent::WeaponRecharged { .. }))
      );
    } else {
      assert!(direct_events.iter().any(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::WeaponRecharged {
            ammo_recharged: 1,
            current_clip: 10,
            max_clip: 10,
            timer: 30,
            ..
          }
        )
      }));
    }
    expected_events.extend(direct_events);
  }

  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn nuclear_plasma_recharge_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let target_position = Position::new(2, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoCells(6)],
    equipped_weapon: Some(ItemSpawnKind::NuclearPlasmaRifle),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target =
    drl_protocol::MonsterSpawnSpec::new(target_position, "Recharge Target", 1_000, 1, (0, 0));
  let mut setup_replay =
    ReplayLog::new(32, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(target);
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut commands = Vec::with_capacity(42);
  commands.push(Command::AttackRanged(target_position));
  commands.extend(std::iter::repeat_n(Command::Wait, 41));
  let mut expected_events = Vec::new();
  for (index, command) in commands.iter().copied().enumerate() {
    let direct_events = direct.step(command).expect("direct command");
    let step = browser.submit(command).expect("browser command");
    assert_eq!(step.events, direct_events);
    assert_eq!(step.after, direct.observe_player());
    if index < 41 {
      assert!(
        !direct_events
          .iter()
          .any(|event| matches!(event, drl_protocol::GameEvent::WeaponRecharged { .. }))
      );
    } else {
      assert!(direct_events.iter().any(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::WeaponRecharged {
            ammo_recharged: 1,
            current_clip: 19,
            max_clip: 24,
            timer: 40,
            ..
          }
        )
      }));
    }
    expected_events.extend(direct_events);
  }

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
    6
  );

  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).unwrap());
}

#[test]
fn nuclear_bfg_recharge_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let target_position = Position::new(2, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::NuclearBfg9000),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target =
    drl_protocol::MonsterSpawnSpec::new(target_position, "Recharge Target", 1_000, 1, (0, 0));
  let mut setup_replay =
    ReplayLog::new(33, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(target);
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut commands = Vec::with_capacity(5);
  commands.push(Command::AttackRanged(target_position));
  commands.extend(std::iter::repeat_n(Command::Wait, 4));
  let mut expected_events = Vec::new();
  for (index, command) in commands.iter().copied().enumerate() {
    let direct_events = direct.step(command).expect("direct command");
    let step = browser.submit(command).expect("browser command");
    assert_eq!(step.events, direct_events);
    assert_eq!(step.after, direct.observe_player());
    if index < 4 {
      assert!(
        !direct_events
          .iter()
          .any(|event| matches!(event, drl_protocol::GameEvent::WeaponRecharged { .. }))
      );
    } else {
      assert!(direct_events.iter().any(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::WeaponRecharged {
            ammo_recharged: 1,
            current_clip: 1,
            max_clip: 40,
            timer: 0,
            ..
          }
        )
      }));
    }
    expected_events.extend(direct_events);
  }

  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).unwrap());
}

#[test]
fn if_noreload_denial_browser_boundary_matches_direct_core() {
  for kind in [
    ItemSpawnKind::Blaster,
    ItemSpawnKind::NuclearPlasmaRifle,
    ItemSpawnKind::NuclearBfg9000,
  ] {
    let replay =
      ReplayLog::new(1_764, 8, 4, Position::new(1, 1)).with_player_config(PlayerSpawnConfig {
        hp: 50,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(kind),
        equipped_armor: None,
        equipped_armor_durability: None,
      });
    let (initial, setup_events) = drl_core::ReplayEngine::run(&replay).expect("replay setup");
    assert!(setup_events.is_empty());

    let mut direct = initial.clone();
    let before = direct.clone();
    assert!(matches!(
      direct.step(Command::Reload),
      Err(drl_protocol::CommandError::CannotReload(_))
    ));
    assert_eq!(direct, before);

    let mut browser = BrowserSession::from_game(initial);
    let observation_before = browser.observation();
    let replay_before = browser.replay_log().clone();
    let weapon_id = observation_before
      .equipped_weapon
      .as_ref()
      .expect("configured weapon")
      .id;
    let error = browser.submit(Command::Reload).unwrap_err();
    assert_eq!(
      error,
      drl_protocol::CommandError::CannotReload(weapon_id).to_string()
    );
    assert_eq!(browser.observation(), observation_before);
    assert_eq!(browser.replay_log(), replay_before);
  }
}

#[test]
fn medical_powerarmor_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let setup_replay =
    ReplayLog::new(23, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
      hp: 20,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: Some(ItemSpawnKind::MedicalPowerarmor),
      equipped_armor_durability: Some(100),
    });

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let scenario = drl_core::scenario::Scenario::from_ascii(
    "MedicalPowerarmorVertical",
    "Medical Powerarmor periodic repair encounter",
    "########\n#@.....#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  let mut scenario = scenario;
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
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let commands = [Command::Wait; 30];
  let mut expected_events = Vec::new();
  for (index, command) in commands.iter().copied().enumerate() {
    let direct_events = direct.step(command).expect("direct wait");
    let step = browser.submit(command).expect("browser wait");
    assert_eq!(step.events, direct_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(step.effects, Vec::new());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &direct_events,)
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    if index < 29 {
      assert_eq!(
        direct.world().player().unwrap().medical_repair_timer(),
        (index + 1) as u32
      );
    } else {
      assert!(direct_events.iter().any(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::MedicalPowerarmorRepaired {
            healed: 1,
            remaining_hp: 21,
            durability_remaining: 99,
            timer: 20,
            ..
          }
        )
      }));
      assert_eq!(direct.world().player().unwrap().medical_repair_timer(), 20);
    }
    expected_events.extend(direct_events);
  }

  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn maleks_armor_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let setup_replay =
    ReplayLog::new(24, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: Some(ItemSpawnKind::MaleksArmor),
      equipped_armor_durability: Some(99),
    });

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "MalekArmorVertical",
    "Malek's Armor periodic durability recharge encounter",
    "########\n#@.....#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
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
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let commands = [Command::Wait; 56];
  let mut expected_events = Vec::new();
  for (index, command) in commands.iter().copied().enumerate() {
    let direct_events = direct.step(command).expect("direct wait");
    let step = browser.submit(command).expect("browser wait");
    assert_eq!(step.events, direct_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(step.effects, Vec::new());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &direct_events,)
    );
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    if index < 54 {
      assert_eq!(
        direct.world().player().unwrap().malek_recharge_timer(),
        (index + 1) as u32
      );
    } else if index == 54 {
      assert!(direct_events.iter().any(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::MalekArmorRecharged {
            durability_restored: 1,
            durability_remaining: 100,
            timer: 50,
            ..
          }
        )
      }));
      assert_eq!(direct.world().player().unwrap().malek_recharge_timer(), 50);
    } else {
      assert!(
        !direct_events
          .iter()
          .any(|event| matches!(event, drl_protocol::GameEvent::MalekArmorRecharged { .. }))
      );
      assert_eq!(direct.world().player().unwrap().malek_recharge_timer(), 50);
    }
    expected_events.extend(direct_events);
  }

  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn former_human_profile_progression_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let setup_replay =
    ReplayLog::new(0, 8, 4, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let mut setup_replay = setup_replay;
  setup_replay.record_stairs(Position::new(5, 1));
  setup_replay.record_monster(
    MonsterSpawnSpec::new(Position::new(4, 1), "Progression Target", 10, 100, (2, 5))
      .with_ranged_combat((1, 4), 6, 65)
      .with_death_drop(Some(ItemSpawnKind::Ammo9mm(10))),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let scenario = drl_core::scenario::Scenario::from_ascii(
    "FormerHumanProfileProgressionVertical",
    "Pistol progression through a Former Human profile, dropped ammunition, and stairs",
    "########\n#@..h>.#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  let mut scenario = scenario;
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
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let monster_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("Former Human identity")
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

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut expected_events = Vec::new();
  let expected_effects = [
    vec![
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Move,
        start_tick: 0,
        duration_ticks: 1,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 1,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 3,
        duration_ticks: 1,
      },
    ],
    vec![
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 0,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 2,
        duration_ticks: 1,
      },
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
    ],
    vec![
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 0,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 2,
        duration_ticks: 1,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Death,
        start_tick: 3,
        duration_ticks: 4,
      },
    ],
    vec![drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Move,
      start_tick: 0,
      duration_ticks: 1,
    }],
    vec![drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Move,
      start_tick: 0,
      duration_ticks: 1,
    }],
    vec![drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Pickup,
      start_tick: 0,
      duration_ticks: 2,
    }],
    vec![drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Move,
      start_tick: 0,
      duration_ticks: 1,
    }],
    vec![drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::LevelTransition,
      start_tick: 0,
      duration_ticks: 4,
    }],
  ];
  for (command, literal_effects) in commands.iter().copied().zip(expected_effects) {
    let direct_events = direct.step(command).expect("progression command");
    let step = browser
      .submit(command)
      .expect("browser progression command");
    assert_eq!(step.events, direct_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &direct_events,)
    );
    assert_eq!(step.effects, literal_effects);
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    expected_events.extend(direct_events);
  }

  assert_eq!(direct.world().level_id().0, 2);
  assert_eq!(direct.world().player().unwrap().hp().current, 44);
  assert!(
    direct
      .world()
      .player()
      .unwrap()
      .inventory()
      .has_ammo(drl_protocol::AmmoType::Ammo9mm, 10)
  );
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::ActorDied { entity_id, .. } if *entity_id == monster_id
    )
  }));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);

  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn phase_device_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let mut setup_replay = ReplayLog::new(9999, 8, 4, player_position);
  setup_replay.record_item(ItemSpawnSpec::new(
    Position::new(2, 1),
    ItemSpawnKind::PhaseDevice,
  ));

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let scenario = drl_core::scenario::Scenario::from_ascii(
    "PhaseDeviceVertical",
    "Phase Device escape from a fixed arena",
    "########\n#@P....#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  let mut scenario = scenario;
  scenario.seed = 9999;
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let player_id = initial.world().player_id().expect("player identity");
  let device_id = initial
    .world()
    .ground_items()
    .keys()
    .next()
    .copied()
    .expect("phase device identity");
  let commands = vec![
    Command::Move(Direction::East),
    Command::Pickup,
    Command::Use(device_id),
  ];
  let expected_effects = [
    vec![drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Move,
      start_tick: 0,
      duration_ticks: 1,
    }],
    vec![drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Pickup,
      start_tick: 0,
      duration_ticks: 2,
    }],
    vec![
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Teleport,
        start_tick: 0,
        duration_ticks: 4,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Use,
        start_tick: 4,
        duration_ticks: 2,
      },
    ],
  ];

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut expected_events = Vec::new();
  for (command, literal_effects) in commands.iter().copied().zip(expected_effects) {
    let direct_events = direct.step(command).expect("phase device command");
    let step = browser
      .submit(command)
      .expect("browser phase device command");
    assert_eq!(step.events, direct_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &direct_events,)
    );
    assert_eq!(step.effects, literal_effects);
    assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
    expected_events.extend(direct_events);
  }

  assert_eq!(
    direct.world().player().unwrap().position(),
    Position::new(6, 2)
  );
  assert!(direct.world().is_explored(Position::new(6, 2)));
  assert!(
    direct
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(device_id)
      .is_none()
  );
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);

  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("phase device command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::PlayerTeleported { from, to }
        if *from == Position::new(2, 1) && *to == Position::new(6, 2)
    )
  }));
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::ItemUsed { entity_id, item_id, .. }
        if *entity_id == player_id && *item_id == device_id
    )
  }));
}

#[test]
fn shotgun_knockback_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Shotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Knockback Target", 15, 100, (3, 6))
      .with_ranged_combat((2, 6), 5, 60)
      .with_death_drop(Some(ItemSpawnKind::AmmoShells(10))),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "ShotgunKnockbackVertical",
    "Shotgun knockback against a Former Sergeant profile",
    "########\n#@.s...#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 0;
  scenario.monsters[0].name = "Knockback Target".to_string();
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
    .expect("knockback target")
    .id();
  let attack_command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);

  let expected_events = direct
    .step(attack_command)
    .expect("direct Shotgun knockback attack");
  let step = browser
    .submit(attack_command)
    .expect("browser Shotgun knockback attack");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(
    step.effects,
    vec![
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 0,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 2,
        duration_ticks: 1,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Knockback,
        start_tick: 3,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 5,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 7,
        duration_ticks: 1,
      },
    ]
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, vec![attack_command]);
  assert_eq!(
    direct.world().player().unwrap().hp().current,
    47,
    "Former Sergeant profile response should hit once"
  );
  assert_eq!(
    direct.world().get_actor(target_id).unwrap().position(),
    Position::new(4, 1)
  );
  assert_eq!(direct.world().get_actor(target_id).unwrap().hp().current, 3);
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::ActorKnockedBack { entity_id, from, to }
        if *entity_id == target_id
          && *from == target_position
          && *to == Position::new(4, 1)
    )
  }));
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::AttackResolved {
        attacker_id,
        target_id: event_target,
        is_ranged: true,
        ..
      } if *attacker_id == target_id && *event_target == player_id
    )
  }));

  let mut command_replay = setup_replay;
  command_replay.record_command(attack_command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn green_armor_protection_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::Pistol),
    equipped_armor: Some(ItemSpawnKind::GreenArmor),
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(4, 8, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Armor Target", 15, 100, (3, 6))
      .with_ranged_combat((2, 6), 5, 60)
      .with_death_drop(Some(ItemSpawnKind::AmmoShells(10))),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "GreenArmorProtectionVertical",
    "Green Armor mitigation against a Former Sergeant profile",
    "########\n#@.s...#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 4;
  scenario.monsters[0].name = "Armor Target".to_string();
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
    .expect("armor target")
    .id();
  let command = Command::Wait;
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);

  let expected_events = direct.step(command).expect("direct armor response");
  let step = browser.submit(command).expect("browser armor response");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(
    step.effects,
    vec![
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::RangedAttack,
        start_tick: 0,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 2,
        duration_ticks: 1,
      },
    ]
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, vec![command]);
  let armor = step.after.equipped_armor.as_ref().expect("Green Armor");
  assert_eq!(armor.name, "Green Armor");
  assert_eq!(armor.armor_value, Some(5));
  assert_eq!(direct.world().player().unwrap().hp().current, 49);
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::AttackResolved {
        attacker_id,
        target_id: event_target,
        outcome: drl_protocol::AttackOutcome::Hit { damage: 3, is_lethal: false },
        is_ranged: true,
      } if *attacker_id == target_id && *event_target == player_id
    )
  }));
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::DamageApplied {
        target_id: event_target,
        amount: 1,
        remaining_hp: 49,
        source: drl_protocol::DamageSource::Actor(source_id),
        ..
      } if *event_target == player_id && *source_id == target_id
    )
  }));

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn small_medpack_recovery_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 45,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::SmallMedPack],
    equipped_weapon: None,
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let setup_replay =
    ReplayLog::new(2, 8, 4, player_position).with_player_config(player_config.clone());
  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let medpack_id = *initial
    .world()
    .player()
    .expect("player")
    .inventory()
    .items()
    .keys()
    .next()
    .expect("Small MedPack");
  assert_eq!(medpack_id, drl_protocol::ItemId::new(4));

  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "SmallMedPackRecoveryVertical",
    "Small MedPack recovery at the health cap",
    "########\n#@.....#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 2;
  scenario.player_config = Some(player_config);
  assert_eq!(
    scenario.instantiate().expect("scenario initial state"),
    initial
  );

  let player_id = initial.world().player_id().expect("player identity");
  let command = Command::Use(medpack_id);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);

  let expected_events = direct.step(command).expect("direct medpack use");
  let step = browser.submit(command).expect("browser medpack use");
  assert_eq!(step.events, expected_events);
  assert_eq!(step.after, direct.observe_player());
  assert_eq!(
    step.effects,
    drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
  );
  assert_eq!(
    step.effects,
    vec![drl_render::EffectSpan {
      effect: drl_render::PresentationEffect::Use,
      start_tick: 0,
      duration_ticks: 2,
    }]
  );
  assert_eq!(browser.scene(), RenderScene::from_observation(&step.after));
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, vec![command]);
  assert_eq!(step.after.player_hp.unwrap().current, 50);
  assert!(step.after.inventory.is_empty());
  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::ItemUsed { entity_id, item_id, item_name }
        if *entity_id == player_id
          && *item_id == medpack_id
          && item_name == "Small MedPack"
    )
  }));

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn demon_medpack_recovery_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 46,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::SmallMedPack],
    equipped_weapon: None,
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(2, 1);
  let mut setup_replay =
    ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config.clone());
  setup_replay.record_monster(
    MonsterSpawnSpec::new(target_position, "Rush Demon", 30, 140, (5, 10))
      .with_death_drop(Some(ItemSpawnKind::LargeMedPack)),
  );

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let medpack_id = *initial
    .world()
    .player()
    .expect("player")
    .inventory()
    .items()
    .keys()
    .next()
    .expect("Small MedPack");
  assert_eq!(medpack_id, ItemId::new(4));

  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "DemonMedPackRecoveryVertical",
    "Demon melee pressure around Small MedPack recovery",
    "########\n#@d....#\n#......#\n########\n",
  )
  .expect("vertical scenario fixture");
  scenario.seed = 0;
  scenario.monsters[0].name = "Rush Demon".to_string();
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
    .expect("rush demon")
    .id();
  let commands = [Command::Wait, Command::Use(medpack_id)];
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut all_events = Vec::new();
  let mut all_effects = Vec::new();
  let mut effect_offset = 0;

  for command in commands {
    let expected_events = direct.step(command).expect("direct demon encounter");
    let step = browser.submit(command).expect("browser demon encounter");
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step.effects,
      drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events,)
    );
    let step_duration = step
      .effects
      .iter()
      .map(|span| u32::from(span.duration_ticks))
      .sum::<u32>();
    all_events.extend(expected_events);
    all_effects.extend(step.effects.into_iter().map(|span| drl_render::EffectSpan {
      start_tick: span.start_tick + effect_offset,
      ..span
    }));
    effect_offset += step_duration;
  }

  assert_eq!(direct.world().player().unwrap().hp().current, 41);
  assert!(direct.world().player().unwrap().inventory().is_empty());
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&direct.observe_player())
  );
  assert_eq!(browser.replay_log().commands, commands);
  assert_eq!(all_events.len(), 14);
  assert!(all_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::AttackResolved {
        attacker_id,
        target_id: event_target,
        outcome: drl_protocol::AttackOutcome::Hit { damage: 6, is_lethal: false },
        is_ranged: false,
      } if *attacker_id == target_id && *event_target == player_id
    )
  }));
  assert!(all_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::ItemUsed { entity_id, item_id, item_name }
        if *entity_id == player_id
          && *item_id == medpack_id
          && item_name == "Small MedPack"
    )
  }));
  assert_eq!(
    all_effects,
    vec![
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
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Use,
        start_tick: 3,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::MeleeAttack,
        start_tick: 5,
        duration_ticks: 2,
      },
      drl_render::EffectSpan {
        effect: drl_render::PresentationEffect::Hit,
        start_tick: 7,
        duration_ticks: 1,
      },
    ]
  );

  let mut command_replay = setup_replay;
  for command in commands {
    command_replay.record_command(command);
  }
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, all_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn pistol_reload_vertical_browser_boundary_matches_direct_core_presentation() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::Ammo9mm(20)],
    equipped_weapon: Some(ItemSpawnKind::Pistol),
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
  let ammo_id = ItemId::new(4);
  let pistol_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(ammo_id)
      .expect("9mm reserve")
      .count(),
    20
  );
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Pistol")
      .id(),
    pistol_id
  );

  let mut scenario = drl_core::scenario::Scenario::from_ascii(
    "PistolReloadVertical",
    "Pistol clip depletion and deterministic reload",
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
  let mut commands = vec![Command::AttackRanged(target); 10];
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
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![ranged_attack],
    vec![ranged_attack, hit],
    vec![ranged_attack, hit],
    vec![ranged_attack],
    vec![ranged_attack],
    vec![reload],
  ];
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut all_events = Vec::new();
  let mut reload_effects = Vec::new();

  for (index, command) in commands.iter().copied().enumerate() {
    let expected_events = direct.step(command).expect("direct pistol command");
    let step = browser.submit(command).expect("browser pistol command");
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
    458
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
  assert_eq!(weapon.current_clip, 10);
  assert_eq!(
    direct
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(ammo_id)
      .unwrap()
      .count(),
    10
  );
  assert_eq!(browser.observation(), direct.observe_player());
  assert_eq!(browser.replay_log().commands, commands);
  assert_eq!(
    browser.observation().equipped_weapon.unwrap().clip,
    Some((10, 10))
  );
  assert_eq!(
    browser
      .observation()
      .inventory
      .iter()
      .find(|item| item.id == ammo_id)
      .unwrap()
      .count,
    10
  );
  assert_eq!(reload_effects, vec![reload]);
  assert_eq!(
      all_events
        .iter()
        .filter(|event| matches!(event, drl_protocol::GameEvent::AttackResolved { attacker_id, target_id: event_target, is_ranged: true, .. } if *attacker_id == player_id && *event_target == target_id))
        .count(),
      10
    );
  assert!(all_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::WeaponReloaded {
        entity_id,
        ammo_loaded: 10,
        current_clip: 10,
        max_clip: 10,
      } if *entity_id == player_id
    )
  }));
  let reload_index = all_events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::WeaponReloaded {
          entity_id,
          ammo_loaded: 10,
          current_clip: 10,
          max_clip: 10,
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
  assert_eq!(replay_events, all_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}
