//! Weapon verticals projected through the browser boundary.

use super::*;

#[test]
fn laser_rifle_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoCells(10)],
    equipped_weapon: Some(ItemSpawnKind::LaserRifle),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_242, 8, 4, player_position).with_player_config(player_config);
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
  let laser_id = ItemId::new(5);
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
      .expect("Laser Rifle")
      .id(),
    laser_id
  );

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct.step(command).expect("direct Laser Rifle command");
  let step = browser
    .submit(command)
    .expect("browser Laser Rifle command");
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
    5
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Laser Rifle")
      .weapon_properties()
      .expect("Laser Rifle properties")
      .current_clip,
    35
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
fn minigun_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::Ammo9mm(10)],
    equipped_weapon: Some(ItemSpawnKind::Minigun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_243, 8, 4, player_position).with_player_config(player_config);
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
  let minigun_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(ammo_id)
      .expect("9mm reserve")
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
      .expect("Minigun")
      .id(),
    minigun_id
  );

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct.step(command).expect("direct Minigun command");
  let step = browser.submit(command).expect("browser Minigun command");
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
    8
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Minigun")
      .weapon_properties()
      .expect("Minigun properties")
      .current_clip,
    192
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(ammo_id)
      .expect("9mm reserve")
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
fn chaingun_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::Ammo9mm(10)],
    equipped_weapon: Some(ItemSpawnKind::Chaingun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_244, 8, 4, player_position).with_player_config(player_config);
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
  let chaingun_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(ammo_id)
      .expect("9mm reserve")
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
      .expect("Chaingun")
      .id(),
    chaingun_id
  );

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct.step(command).expect("direct Chaingun command");
  let step = browser.submit(command).expect("browser Chaingun command");
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
    4
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Chaingun")
      .weapon_properties()
      .expect("Chaingun properties")
      .current_clip,
    36
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(ammo_id)
      .expect("9mm reserve")
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
fn mega_buster_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::Ammo9mm(10)],
    equipped_weapon: Some(ItemSpawnKind::MegaBuster),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_245, 8, 4, player_position).with_player_config(player_config);
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
  let mega_buster_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(ammo_id)
      .expect("9mm reserve")
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
      .expect("Mega Buster")
      .id(),
    mega_buster_id
  );

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct.step(command).expect("direct Mega Buster command");
  let step = browser
    .submit(command)
    .expect("browser Mega Buster command");
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
    3
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Mega Buster")
      .weapon_properties()
      .expect("Mega Buster properties")
      .current_clip,
    51
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(ammo_id)
      .expect("9mm reserve")
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
fn super_shotgun_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoShells(10)],
    equipped_weapon: Some(ItemSpawnKind::SuperShotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_246, 8, 4, player_position).with_player_config(player_config);
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
      .expect("Super Shotgun")
      .id(),
    shotgun_id
  );

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct.step(command).expect("direct Super Shotgun command");
  let step = browser
    .submit(command)
    .expect("browser Super Shotgun command");
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
    2
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Super Shotgun")
      .weapon_properties()
      .expect("Super Shotgun properties")
      .current_clip,
    0
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(shells_id)
      .expect("shell reserve")
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
fn tristar_blaster_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoCells(20)],
    equipped_weapon: Some(ItemSpawnKind::TristarBlaster),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_247, 8, 4, player_position).with_player_config(player_config);
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
  let tristar_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(cells_id)
      .expect("cell reserve")
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
      .expect("Tristar Blaster")
      .id(),
    tristar_id
  );

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct
    .step(command)
    .expect("direct Tristar Blaster command");
  let step = browser
    .submit(command)
    .expect("browser Tristar Blaster command");
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
    3
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Tristar Blaster")
      .weapon_properties()
      .expect("Tristar Blaster properties")
      .current_clip,
    30
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
    20
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
fn railgun_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoCells(10)],
    equipped_weapon: Some(ItemSpawnKind::Railgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let intermediate_position = Position::new(2, 1);
  let mut setup_replay =
    ReplayLog::new(2_248, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    intermediate_position,
    "Intermediate Target",
    500,
    100,
    (1, 7),
  ));
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
  let cells_id = ItemId::new(4);
  let railgun_id = ItemId::new(5);
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
      .expect("Railgun")
      .id(),
    railgun_id
  );

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct.step(command).expect("direct Railgun command");
  let step = browser.submit(command).expect("browser Railgun command");
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
          is_ranged: true,
          ..
        } if *attacker_id == player_id
      ))
      .count(),
    2
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Railgun")
      .weapon_properties()
      .expect("Railgun properties")
      .current_clip,
    35
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
fn frag_shotgun_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::Ammo9mm(10)],
    equipped_weapon: Some(ItemSpawnKind::FragShotgun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_249, 8, 4, player_position).with_player_config(player_config);
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
  let shotgun_id = ItemId::new(5);
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(ammo_id)
      .expect("9mm reserve")
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
      .expect("Frag Shotgun")
      .id(),
    shotgun_id
  );

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct.step(command).expect("direct Frag Shotgun command");
  let step = browser
    .submit(command)
    .expect("browser Frag Shotgun command");
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
      .expect("Frag Shotgun")
      .weapon_properties()
      .expect("Frag Shotgun properties")
      .current_clip,
    14
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(ammo_id)
      .expect("9mm reserve")
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
fn combat_pistol_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::Ammo9mm(10)],
    equipped_weapon: Some(ItemSpawnKind::CombatPistol),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_250, 8, 4, player_position).with_player_config(player_config);
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
    10
  );
  assert_eq!(
    initial
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Combat Pistol")
      .id(),
    pistol_id
  );

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct.step(command).expect("direct Combat Pistol command");
  let step = browser
    .submit(command)
    .expect("browser Combat Pistol command");
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
      .expect("Combat Pistol")
      .weapon_properties()
      .expect("Combat Pistol properties")
      .current_clip,
    14
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(ammo_id)
      .expect("9mm reserve")
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
fn pistol_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::Ammo9mm(10)],
    equipped_weapon: Some(ItemSpawnKind::Pistol),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_251, 8, 4, player_position).with_player_config(player_config);
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
    10
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

  let command = Command::AttackRanged(target_position);
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let expected_events = direct.step(command).expect("direct Pistol command");
  let step = browser.submit(command).expect("browser Pistol command");
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
      .expect("Pistol")
      .weapon_properties()
      .expect("Pistol properties")
      .current_clip,
    9
  );
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .inventory()
      .get_item(ammo_id)
      .expect("9mm reserve")
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
fn pistol_aimed_fire_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::Ammo9mm(6)],
    equipped_weapon: Some(ItemSpawnKind::Pistol),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_263, 8, 4, player_position).with_player_config(player_config);
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
  let expected_events = direct.step(command).expect("direct aimed Pistol command");
  let step = browser
    .submit(command)
    .expect("browser aimed Pistol command");
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
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Pistol")
      .weapon_properties()
      .expect("Pistol properties")
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
fn combat_pistol_aimed_fire_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::Ammo9mm(6)],
    equipped_weapon: Some(ItemSpawnKind::CombatPistol),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_265, 8, 4, player_position).with_player_config(player_config);
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
  let expected_events = direct
    .step(command)
    .expect("direct aimed Combat Pistol command");
  let step = browser
    .submit(command)
    .expect("browser aimed Combat Pistol command");
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
  assert_eq!(
    direct
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Combat Pistol")
      .weapon_properties()
      .expect("Combat Pistol properties")
      .current_clip,
    14
  );

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}
