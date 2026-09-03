//! Chainfire weapon verticals projected through the browser boundary.

use super::*;

#[test]
fn chaingun_chainfire_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::Ammo9mm(80)],
    equipped_weapon: Some(ItemSpawnKind::Chaingun),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let mut setup_replay =
    ReplayLog::new(2_245, 8, 4, player_position).with_player_config(player_config);
  let target_position = Position::new(3, 1);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    10_000,
    0,
    (1, 7),
  ));

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let command = Command::AttackRangedChainfire(target_position);
  let mut expected_events = direct
    .step(command)
    .expect("direct Chaingun chainfire command");
  let step = browser
    .submit(command)
    .expect("browser Chaingun chainfire command");
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
          is_ranged: true,
          ..
        }
      ))
      .count(),
    3
  );
  assert_eq!(
    step
      .after
      .equipped_weapon
      .expect("Chaingun")
      .chainfire_level,
    1
  );

  let second_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &direct.observe_player()),
    Some(second_command)
  );
  let second_expected_events = direct
    .step(second_command)
    .expect("direct second Chaingun chainfire command");
  let second_step = browser
    .submit(second_command)
    .expect("browser second Chaingun chainfire command");
  assert_eq!(second_step.events, second_expected_events);
  assert_eq!(second_step.after, direct.observe_player());
  assert_eq!(
    second_step.effects,
    effect_timeline_for_observations(
      &second_step.before,
      &second_step.after,
      &second_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&second_step.after)
  );
  assert_eq!(
    second_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    4
  );
  let chaingun = second_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Chaingun");
  assert_eq!(chaingun.chainfire_level, 2);
  assert_eq!(chaingun.clip, Some((33, 40)));
  let third_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &second_step.after),
    Some(third_command)
  );
  expected_events.extend(second_expected_events);
  let third_expected_events = direct
    .step(third_command)
    .expect("direct third Chaingun chainfire command");
  let third_step = browser
    .submit(third_command)
    .expect("browser third Chaingun chainfire command");
  assert_eq!(third_step.events, third_expected_events);
  assert_eq!(third_step.after, direct.observe_player());
  assert_eq!(
    third_step.effects,
    effect_timeline_for_observations(
      &third_step.before,
      &third_step.after,
      &third_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&third_step.after)
  );
  assert_eq!(
    third_expected_events
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
  let chaingun = third_step.after.equipped_weapon.as_ref().expect("Chaingun");
  assert_eq!(chaingun.chainfire_level, 3);
  assert_eq!(chaingun.clip, Some((27, 40)));
  let fourth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &third_step.after),
    Some(fourth_command)
  );
  let fourth_expected_events = direct
    .step(fourth_command)
    .expect("direct fourth Chaingun chainfire command");
  let fourth_step = browser
    .submit(fourth_command)
    .expect("browser fourth Chaingun chainfire command");
  assert_eq!(fourth_step.events, fourth_expected_events);
  assert_eq!(fourth_step.after, direct.observe_player());
  assert_eq!(
    fourth_step.effects,
    effect_timeline_for_observations(
      &fourth_step.before,
      &fourth_step.after,
      &fourth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fourth_step.after)
  );
  assert_eq!(
    fourth_expected_events
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
  let chaingun = fourth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Chaingun");
  assert_eq!(chaingun.chainfire_level, 4);
  assert_eq!(chaingun.clip, Some((21, 40)));
  let fifth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &fourth_step.after),
    Some(fifth_command)
  );
  let fifth_expected_events = direct
    .step(fifth_command)
    .expect("direct fifth Chaingun chainfire command");
  let fifth_step = browser
    .submit(fifth_command)
    .expect("browser fifth Chaingun chainfire command");
  assert_eq!(fifth_step.events, fifth_expected_events);
  assert_eq!(fifth_step.after, direct.observe_player());
  assert_eq!(
    fifth_step.effects,
    effect_timeline_for_observations(
      &fifth_step.before,
      &fifth_step.after,
      &fifth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fifth_step.after)
  );
  assert_eq!(
    fifth_expected_events
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
  let chaingun = fifth_step.after.equipped_weapon.as_ref().expect("Chaingun");
  assert_eq!(chaingun.chainfire_level, 5);
  assert_eq!(chaingun.clip, Some((15, 40)));
  let sixth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &fifth_step.after),
    Some(sixth_command)
  );
  let sixth_expected_events = direct
    .step(sixth_command)
    .expect("direct sixth Chaingun chainfire command");
  let sixth_step = browser
    .submit(sixth_command)
    .expect("browser sixth Chaingun chainfire command");
  assert_eq!(sixth_step.events, sixth_expected_events);
  assert_eq!(sixth_step.after, direct.observe_player());
  assert_eq!(
    sixth_step.effects,
    effect_timeline_for_observations(
      &sixth_step.before,
      &sixth_step.after,
      &sixth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&sixth_step.after)
  );
  assert_eq!(
    sixth_expected_events
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
  let chaingun = sixth_step.after.equipped_weapon.as_ref().expect("Chaingun");
  assert_eq!(chaingun.chainfire_level, 6);
  assert_eq!(chaingun.clip, Some((9, 40)));
  let seventh_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &sixth_step.after),
    Some(seventh_command)
  );
  let seventh_expected_events = direct
    .step(seventh_command)
    .expect("direct seventh Chaingun chainfire command");
  let seventh_step = browser
    .submit(seventh_command)
    .expect("browser seventh Chaingun chainfire command");
  assert_eq!(seventh_step.events, seventh_expected_events);
  assert_eq!(seventh_step.after, direct.observe_player());
  assert_eq!(
    seventh_step.effects,
    effect_timeline_for_observations(
      &seventh_step.before,
      &seventh_step.after,
      &seventh_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&seventh_step.after)
  );
  assert_eq!(
    seventh_expected_events
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
  let chaingun = seventh_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Chaingun");
  assert_eq!(chaingun.chainfire_level, 7);
  assert_eq!(chaingun.clip, Some((3, 40)));
  let reload_command = Command::Reload;
  assert_eq!(
    BrowserSession::command_for_key("r", &seventh_step.after),
    Some(reload_command)
  );
  let reload_expected_events = direct
    .step(reload_command)
    .expect("direct Chaingun chainfire reload");
  let reload_step = browser
    .submit(reload_command)
    .expect("browser Chaingun chainfire reload");
  assert_eq!(reload_step.events, reload_expected_events);
  assert_eq!(reload_step.after, direct.observe_player());
  assert_eq!(
    reload_step.effects,
    effect_timeline_for_observations(
      &reload_step.before,
      &reload_step.after,
      &reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&reload_step.after)
  );
  assert!(reload_expected_events.iter().any(|event| matches!(
    event,
    drl_protocol::GameEvent::WeaponReloaded {
      ammo_loaded: 37,
      current_clip: 40,
      max_clip: 40,
      ..
    }
  )));
  let eighth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &reload_step.after),
    Some(eighth_command)
  );
  let eighth_expected_events = direct
    .step(eighth_command)
    .expect("direct eighth Chaingun chainfire command");
  let eighth_step = browser
    .submit(eighth_command)
    .expect("browser eighth Chaingun chainfire command");
  assert_eq!(eighth_step.events, eighth_expected_events);
  assert_eq!(eighth_step.after, direct.observe_player());
  assert_eq!(
    eighth_step.effects,
    effect_timeline_for_observations(
      &eighth_step.before,
      &eighth_step.after,
      &eighth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&eighth_step.after)
  );
  assert_eq!(
    eighth_expected_events
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
  let chaingun = eighth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Chaingun");
  assert_eq!(chaingun.chainfire_level, 8);
  assert_eq!(chaingun.clip, Some((34, 40)));
  let ninth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &eighth_step.after),
    Some(ninth_command)
  );
  let ninth_expected_events = direct
    .step(ninth_command)
    .expect("direct ninth Chaingun chainfire command");
  let ninth_step = browser
    .submit(ninth_command)
    .expect("browser ninth Chaingun chainfire command");
  assert_eq!(ninth_step.events, ninth_expected_events);
  assert_eq!(ninth_step.after, direct.observe_player());
  assert_eq!(
    ninth_step.effects,
    effect_timeline_for_observations(
      &ninth_step.before,
      &ninth_step.after,
      &ninth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&ninth_step.after)
  );
  assert_eq!(
    ninth_expected_events
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
  let chaingun = ninth_step.after.equipped_weapon.as_ref().expect("Chaingun");
  assert_eq!(chaingun.chainfire_level, 9);
  assert_eq!(chaingun.clip, Some((28, 40)));
  let tenth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &ninth_step.after),
    Some(tenth_command)
  );
  let tenth_expected_events = direct
    .step(tenth_command)
    .expect("direct tenth Chaingun chainfire command");
  let tenth_step = browser
    .submit(tenth_command)
    .expect("browser tenth Chaingun chainfire command");
  assert_eq!(tenth_step.events, tenth_expected_events);
  assert_eq!(tenth_step.after, direct.observe_player());
  assert_eq!(
    tenth_step.effects,
    effect_timeline_for_observations(
      &tenth_step.before,
      &tenth_step.after,
      &tenth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&tenth_step.after)
  );
  assert_eq!(
    tenth_expected_events
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
  let chaingun = tenth_step.after.equipped_weapon.as_ref().expect("Chaingun");
  assert_eq!(chaingun.chainfire_level, 10);
  assert_eq!(chaingun.clip, Some((22, 40)));
  let eleventh_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &tenth_step.after),
    Some(eleventh_command)
  );
  let eleventh_expected_events = direct
    .step(eleventh_command)
    .expect("direct eleventh Chaingun chainfire command");
  let eleventh_step = browser
    .submit(eleventh_command)
    .expect("browser eleventh Chaingun chainfire command");
  assert_eq!(eleventh_step.events, eleventh_expected_events);
  assert_eq!(eleventh_step.after, direct.observe_player());
  assert_eq!(
    eleventh_step.effects,
    effect_timeline_for_observations(
      &eleventh_step.before,
      &eleventh_step.after,
      &eleventh_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&eleventh_step.after)
  );
  assert_eq!(
    eleventh_expected_events
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
  let chaingun = eleventh_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Chaingun");
  assert_eq!(chaingun.chainfire_level, 11);
  assert_eq!(chaingun.clip, Some((16, 40)));
  let twelfth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &eleventh_step.after),
    Some(twelfth_command)
  );
  let twelfth_expected_events = direct
    .step(twelfth_command)
    .expect("direct twelfth Chaingun chainfire command");
  let twelfth_step = browser
    .submit(twelfth_command)
    .expect("browser twelfth Chaingun chainfire command");
  assert_eq!(twelfth_step.events, twelfth_expected_events);
  assert_eq!(twelfth_step.after, direct.observe_player());
  assert_eq!(
    twelfth_step.effects,
    effect_timeline_for_observations(
      &twelfth_step.before,
      &twelfth_step.after,
      &twelfth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&twelfth_step.after)
  );
  assert_eq!(
    twelfth_expected_events
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
  let chaingun = twelfth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Chaingun");
  assert_eq!(chaingun.chainfire_level, 12);
  assert_eq!(chaingun.clip, Some((10, 40)));
  let thirteenth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &twelfth_step.after),
    Some(thirteenth_command)
  );
  let thirteenth_expected_events = direct
    .step(thirteenth_command)
    .expect("direct thirteenth Chaingun chainfire command");
  let thirteenth_step = browser
    .submit(thirteenth_command)
    .expect("browser thirteenth Chaingun chainfire command");
  assert_eq!(thirteenth_step.events, thirteenth_expected_events);
  assert_eq!(thirteenth_step.after, direct.observe_player());
  assert_eq!(
    thirteenth_step.effects,
    effect_timeline_for_observations(
      &thirteenth_step.before,
      &thirteenth_step.after,
      &thirteenth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&thirteenth_step.after)
  );
  assert_eq!(
    thirteenth_expected_events
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
  let chaingun = thirteenth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Chaingun");
  assert_eq!(chaingun.chainfire_level, 13);
  assert_eq!(chaingun.clip, Some((4, 40)));
  let second_reload_command = Command::Reload;
  assert_eq!(
    BrowserSession::command_for_key("r", &thirteenth_step.after),
    Some(second_reload_command)
  );
  let second_reload_expected_events = direct
    .step(second_reload_command)
    .expect("direct second Chaingun chainfire reload");
  let second_reload_step = browser
    .submit(second_reload_command)
    .expect("browser second Chaingun chainfire reload");
  assert_eq!(second_reload_step.events, second_reload_expected_events);
  assert_eq!(second_reload_step.after, direct.observe_player());
  assert_eq!(
    second_reload_step.effects,
    effect_timeline_for_observations(
      &second_reload_step.before,
      &second_reload_step.after,
      &second_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&second_reload_step.after)
  );
  assert!(second_reload_expected_events.iter().any(|event| matches!(
    event,
    drl_protocol::GameEvent::WeaponReloaded {
      ammo_loaded: 36,
      current_clip: 40,
      max_clip: 40,
      ..
    }
  )));
  let fourteenth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &second_reload_step.after),
    Some(fourteenth_command)
  );
  let fourteenth_expected_events = direct
    .step(fourteenth_command)
    .expect("direct fourteenth Chaingun chainfire command");
  let fourteenth_step = browser
    .submit(fourteenth_command)
    .expect("browser fourteenth Chaingun chainfire command");
  assert_eq!(fourteenth_step.events, fourteenth_expected_events);
  assert_eq!(fourteenth_step.after, direct.observe_player());
  assert_eq!(
    fourteenth_step.effects,
    effect_timeline_for_observations(
      &fourteenth_step.before,
      &fourteenth_step.after,
      &fourteenth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fourteenth_step.after)
  );
  assert_eq!(
    fourteenth_expected_events
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
  let chaingun = fourteenth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Chaingun");
  assert_eq!(chaingun.chainfire_level, 14);
  assert_eq!(chaingun.clip, Some((34, 40)));
  assert_eq!(
    BrowserSession::command_for_key("C", &fourteenth_step.after),
    Some(Command::AttackRangedChainfire(target_position))
  );
  expected_events.extend(third_expected_events);
  expected_events.extend(fourth_expected_events);
  expected_events.extend(fifth_expected_events);
  expected_events.extend(sixth_expected_events);
  expected_events.extend(seventh_expected_events);
  expected_events.extend(reload_expected_events);
  expected_events.extend(eighth_expected_events);
  expected_events.extend(ninth_expected_events);
  expected_events.extend(tenth_expected_events);
  expected_events.extend(eleventh_expected_events);
  expected_events.extend(twelfth_expected_events);
  expected_events.extend(thirteenth_expected_events);
  expected_events.extend(second_reload_expected_events);
  expected_events.extend(fourteenth_expected_events);
  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  command_replay.record_command(second_command);
  command_replay.record_command(third_command);
  command_replay.record_command(fourth_command);
  command_replay.record_command(fifth_command);
  command_replay.record_command(sixth_command);
  command_replay.record_command(seventh_command);
  command_replay.record_command(reload_command);
  command_replay.record_command(eighth_command);
  command_replay.record_command(ninth_command);
  command_replay.record_command(tenth_command);
  command_replay.record_command(eleventh_command);
  command_replay.record_command(twelfth_command);
  command_replay.record_command(thirteenth_command);
  command_replay.record_command(second_reload_command);
  command_replay.record_command(fourteenth_command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn minigun_chainfire_vertical_browser_boundary_matches_direct_core() {
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
    ReplayLog::new(2_246, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    10_000,
    0,
    (1, 7),
  ));

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &initial.observe_player()),
    Some(command)
  );

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut expected_events = direct
    .step(command)
    .expect("direct Minigun chainfire command");
  let step = browser
    .submit(command)
    .expect("browser Minigun chainfire command");
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
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    6
  );
  assert_eq!(
    step
      .after
      .equipped_weapon
      .as_ref()
      .expect("Minigun")
      .chainfire_level,
    1
  );
  let second_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &step.after),
    Some(second_command)
  );
  let second_expected_events = direct
    .step(second_command)
    .expect("direct second Minigun chainfire command");
  let second_step = browser
    .submit(second_command)
    .expect("browser second Minigun chainfire command");
  assert_eq!(second_step.events, second_expected_events);
  assert_eq!(second_step.after, direct.observe_player());
  assert_eq!(
    second_step.effects,
    effect_timeline_for_observations(
      &second_step.before,
      &second_step.after,
      &second_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&second_step.after)
  );
  assert_eq!(
    second_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    8
  );
  let minigun = second_step.after.equipped_weapon.as_ref().expect("Minigun");
  assert_eq!(minigun.chainfire_level, 2);
  assert_eq!(minigun.clip, Some((186, 200)));
  let third_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &second_step.after),
    Some(third_command)
  );
  let third_expected_events = direct
    .step(third_command)
    .expect("direct third Minigun chainfire command");
  let third_step = browser
    .submit(third_command)
    .expect("browser third Minigun chainfire command");
  assert_eq!(third_step.events, third_expected_events);
  assert_eq!(third_step.after, direct.observe_player());
  assert_eq!(
    third_step.effects,
    effect_timeline_for_observations(
      &third_step.before,
      &third_step.after,
      &third_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&third_step.after)
  );
  assert_eq!(
    third_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    12
  );
  let minigun = third_step.after.equipped_weapon.as_ref().expect("Minigun");
  assert_eq!(minigun.chainfire_level, 3);
  assert_eq!(minigun.clip, Some((174, 200)));
  assert_eq!(
    BrowserSession::command_for_key("C", &third_step.after),
    Some(Command::AttackRangedChainfire(target_position))
  );
  expected_events.extend(second_expected_events);
  expected_events.extend(third_expected_events);
  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  command_replay.record_command(second_command);
  command_replay.record_command(third_command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn plasma_rifle_chainfire_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoCells(4)],
    equipped_weapon: Some(ItemSpawnKind::PlasmaRifle),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_447, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    0,
    (1, 7),
  ));

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &initial.observe_player()),
    Some(command)
  );

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut expected_events = direct
    .step(command)
    .expect("direct Plasma Rifle chainfire command");
  let step = browser
    .submit(command)
    .expect("browser Plasma Rifle chainfire command");
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
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    4
  );
  assert_eq!(
    step
      .after
      .equipped_weapon
      .expect("Plasma Rifle")
      .chainfire_level,
    1
  );

  let reload_command = Command::Reload;
  let reload_expected_events = direct
    .step(reload_command)
    .expect("direct Plasma Rifle chainfire reload");
  let reload_step = browser
    .submit(reload_command)
    .expect("browser Plasma Rifle chainfire reload");
  assert_eq!(reload_step.events, reload_expected_events);
  assert_eq!(reload_step.after, direct.observe_player());
  assert_eq!(
    reload_step.effects,
    effect_timeline_for_observations(
      &reload_step.before,
      &reload_step.after,
      &reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&reload_step.after)
  );
  let reloaded = reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Plasma Rifle");
  assert_eq!(reloaded.chainfire_level, 1);
  assert_eq!(reloaded.clip, Some((6, 6)));
  expected_events.extend(reload_expected_events);

  let second_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &reload_step.after),
    Some(second_command)
  );
  let second_expected_events = direct
    .step(second_command)
    .expect("direct second Plasma Rifle chainfire command");
  let second_step = browser
    .submit(second_command)
    .expect("browser second Plasma Rifle chainfire command");
  assert_eq!(second_step.events, second_expected_events);
  assert_eq!(second_step.after, direct.observe_player());
  assert_eq!(
    second_step.effects,
    effect_timeline_for_observations(
      &second_step.before,
      &second_step.after,
      &second_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&second_step.after)
  );
  assert_eq!(
    second_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    6
  );
  let second_plasma = second_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Plasma Rifle");
  assert_eq!(second_plasma.chainfire_level, 2);
  assert_eq!(second_plasma.clip, Some((0, 6)));
  assert_eq!(
    BrowserSession::command_for_key("C", &second_step.after),
    None
  );
  expected_events.extend(second_expected_events);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  command_replay.record_command(reload_command);
  command_replay.record_command(second_command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn laser_rifle_chainfire_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoCells(7)],
    equipped_weapon: Some(ItemSpawnKind::LaserRifle),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_549, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    0,
    (1, 7),
  ));

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &initial.observe_player()),
    Some(command)
  );

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut expected_events = direct
    .step(command)
    .expect("direct Laser Rifle chainfire command");
  let step = browser
    .submit(command)
    .expect("browser Laser Rifle chainfire command");
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
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    4
  );
  assert_eq!(
    step
      .after
      .equipped_weapon
      .as_ref()
      .expect("Laser Rifle")
      .chainfire_level,
    1
  );

  let second_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &step.after),
    Some(second_command)
  );
  let second_expected_events = direct
    .step(second_command)
    .expect("direct second Laser Rifle chainfire command");
  let second_step = browser
    .submit(second_command)
    .expect("browser second Laser Rifle chainfire command");
  assert_eq!(second_step.events, second_expected_events);
  assert_eq!(second_step.after, direct.observe_player());
  assert_eq!(
    second_step.effects,
    effect_timeline_for_observations(
      &second_step.before,
      &second_step.after,
      &second_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&second_step.after)
  );
  assert_eq!(
    second_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    5
  );
  let second_laser = second_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Laser Rifle");
  assert_eq!(second_laser.chainfire_level, 2);
  assert_eq!(second_laser.clip, Some((31, 40)));
  let third_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &second_step.after),
    Some(third_command)
  );
  let third_expected_events = direct
    .step(third_command)
    .expect("direct third Laser Rifle chainfire command");
  let third_step = browser
    .submit(third_command)
    .expect("browser third Laser Rifle chainfire command");
  assert_eq!(third_step.events, third_expected_events);
  assert_eq!(third_step.after, direct.observe_player());
  assert_eq!(
    third_step.effects,
    effect_timeline_for_observations(
      &third_step.before,
      &third_step.after,
      &third_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&third_step.after)
  );
  assert_eq!(
    third_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  let third_laser = third_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Laser Rifle");
  assert_eq!(third_laser.chainfire_level, 3);
  assert_eq!(third_laser.clip, Some((24, 40)));
  let fourth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &third_step.after),
    Some(fourth_command)
  );
  let fourth_expected_events = direct
    .step(fourth_command)
    .expect("direct fourth Laser Rifle chainfire command");
  let fourth_step = browser
    .submit(fourth_command)
    .expect("browser fourth Laser Rifle chainfire command");
  assert_eq!(fourth_step.events, fourth_expected_events);
  assert_eq!(fourth_step.after, direct.observe_player());
  assert_eq!(
    fourth_step.effects,
    effect_timeline_for_observations(
      &fourth_step.before,
      &fourth_step.after,
      &fourth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fourth_step.after)
  );
  assert_eq!(
    fourth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  let fourth_laser = fourth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Laser Rifle");
  assert_eq!(fourth_laser.chainfire_level, 4);
  assert_eq!(fourth_laser.clip, Some((17, 40)));
  let fifth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &fourth_step.after),
    Some(fifth_command)
  );
  let fifth_expected_events = direct
    .step(fifth_command)
    .expect("direct fifth Laser Rifle chainfire command");
  let fifth_step = browser
    .submit(fifth_command)
    .expect("browser fifth Laser Rifle chainfire command");
  assert_eq!(fifth_step.events, fifth_expected_events);
  assert_eq!(fifth_step.after, direct.observe_player());
  assert_eq!(
    fifth_step.effects,
    effect_timeline_for_observations(
      &fifth_step.before,
      &fifth_step.after,
      &fifth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fifth_step.after)
  );
  assert_eq!(
    fifth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  let fifth_laser = fifth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Laser Rifle");
  assert_eq!(fifth_laser.chainfire_level, 5);
  assert_eq!(fifth_laser.clip, Some((10, 40)));
  let sixth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &fifth_step.after),
    Some(sixth_command)
  );
  let sixth_expected_events = direct
    .step(sixth_command)
    .expect("direct sixth Laser Rifle chainfire command");
  let sixth_step = browser
    .submit(sixth_command)
    .expect("browser sixth Laser Rifle chainfire command");
  assert_eq!(sixth_step.events, sixth_expected_events);
  assert_eq!(sixth_step.after, direct.observe_player());
  assert_eq!(
    sixth_step.effects,
    effect_timeline_for_observations(
      &sixth_step.before,
      &sixth_step.after,
      &sixth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&sixth_step.after)
  );
  assert_eq!(
    sixth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  let sixth_laser = sixth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Laser Rifle");
  assert_eq!(sixth_laser.chainfire_level, 6);
  assert_eq!(sixth_laser.clip, Some((3, 40)));
  assert_eq!(
    BrowserSession::command_for_key("C", &sixth_step.after),
    None
  );
  let reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct Laser Rifle reload");
  let reload_step = browser
    .submit(Command::Reload)
    .expect("browser Laser Rifle reload");
  assert_eq!(reload_step.events, reload_expected_events);
  assert_eq!(reload_step.after, direct.observe_player());
  assert_eq!(
    reload_step.effects,
    effect_timeline_for_observations(
      &reload_step.before,
      &reload_step.after,
      &reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&reload_step.after)
  );
  assert_eq!(
    reload_step
      .after
      .equipped_weapon
      .as_ref()
      .expect("Laser Rifle")
      .clip,
    Some((10, 40))
  );
  let seventh_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &reload_step.after),
    Some(seventh_command)
  );
  let seventh_expected_events = direct
    .step(seventh_command)
    .expect("direct seventh Laser Rifle chainfire command");
  let seventh_step = browser
    .submit(seventh_command)
    .expect("browser seventh Laser Rifle chainfire command");
  assert_eq!(seventh_step.events, seventh_expected_events);
  assert_eq!(seventh_step.after, direct.observe_player());
  assert_eq!(
    seventh_step.effects,
    effect_timeline_for_observations(
      &seventh_step.before,
      &seventh_step.after,
      &seventh_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&seventh_step.after)
  );
  assert_eq!(
    seventh_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  let seventh_laser = seventh_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Laser Rifle");
  assert_eq!(seventh_laser.chainfire_level, 7);
  assert_eq!(seventh_laser.clip, Some((3, 40)));
  assert_eq!(
    BrowserSession::command_for_key("C", &seventh_step.after),
    None
  );
  expected_events.extend(second_expected_events);
  expected_events.extend(third_expected_events);
  expected_events.extend(fourth_expected_events);
  expected_events.extend(fifth_expected_events);
  expected_events.extend(sixth_expected_events);
  expected_events.extend(reload_expected_events);
  expected_events.extend(seventh_expected_events);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  command_replay.record_command(second_command);
  command_replay.record_command(third_command);
  command_replay.record_command(fourth_command);
  command_replay.record_command(fifth_command);
  command_replay.record_command(sixth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(seventh_command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn bfg10k_chainfire_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoCells(710)],
    equipped_weapon: Some(ItemSpawnKind::Bfg10k),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(7, 1);
  let mut setup_replay =
    ReplayLog::new(2_707, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    10_000,
    0,
    (1, 7),
  ));

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &initial.observe_player()),
    Some(command)
  );

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut expected_events = direct
    .step(command)
    .expect("direct BFG 10K chainfire command");
  let step = browser
    .submit(command)
    .expect("browser BFG 10K chainfire command");
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
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    4
  );
  assert_eq!(
    expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    4
  );
  let bfg10k = step.after.equipped_weapon.expect("BFG 10K");
  assert_eq!(bfg10k.chainfire_level, 1);
  assert_eq!(bfg10k.clip, Some((30, 50)));

  let second_target = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive")
    .position();
  let second_command = Command::AttackRangedChainfire(second_target);
  assert_eq!(
    BrowserSession::command_for_key("C", &direct.observe_player()),
    Some(second_command)
  );
  let second_expected_events = direct
    .step(second_command)
    .expect("direct second BFG 10K chainfire command");
  let second_step = browser
    .submit(second_command)
    .expect("browser second BFG 10K chainfire command");
  assert_eq!(second_step.events, second_expected_events);
  assert_eq!(second_step.after, direct.observe_player());
  assert_eq!(
    second_step.effects,
    effect_timeline_for_observations(
      &second_step.before,
      &second_step.after,
      &second_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&second_step.after)
  );
  assert_eq!(
    second_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    5
  );
  assert_eq!(
    second_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    5
  );
  let bfg10k = second_step.after.equipped_weapon.as_ref().expect("BFG 10K");
  assert_eq!(bfg10k.chainfire_level, 2);
  assert_eq!(bfg10k.clip, Some((5, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &second_step.after),
    None
  );
  expected_events.extend(second_expected_events);

  let reload_command = Command::Reload;
  let reload_expected_events = direct
    .step(reload_command)
    .expect("direct BFG 10K chainfire reload");
  let reload_step = browser
    .submit(reload_command)
    .expect("browser BFG 10K chainfire reload");
  assert_eq!(reload_step.events, reload_expected_events);
  assert_eq!(reload_step.after, direct.observe_player());
  assert_eq!(
    reload_step.effects,
    effect_timeline_for_observations(
      &reload_step.before,
      &reload_step.after,
      &reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&reload_step.after)
  );
  expected_events.extend(reload_expected_events);
  let reloaded = reload_step.after.equipped_weapon.as_ref().expect("BFG 10K");
  assert_eq!(reloaded.chainfire_level, 2);
  assert_eq!(reloaded.clip, Some((50, 50)));

  let third_target = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive reload")
    .position();
  let third_command = Command::AttackRangedChainfire(third_target);
  assert_eq!(
    BrowserSession::command_for_key("C", &direct.observe_player()),
    Some(third_command)
  );
  let third_expected_events = direct
    .step(third_command)
    .expect("direct third BFG 10K chainfire command");
  let third_step = browser
    .submit(third_command)
    .expect("browser third BFG 10K chainfire command");
  assert_eq!(third_step.events, third_expected_events);
  assert_eq!(third_step.after, direct.observe_player());
  assert_eq!(
    third_step.effects,
    effect_timeline_for_observations(
      &third_step.before,
      &third_step.after,
      &third_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&third_step.after)
  );
  assert_eq!(
    third_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    third_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  expected_events.extend(third_expected_events);
  let bfg10k = third_step.after.equipped_weapon.as_ref().expect("BFG 10K");
  assert_eq!(bfg10k.chainfire_level, 3);
  assert_eq!(bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &third_step.after),
    None
  );

  let second_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct second BFG 10K chainfire reload");
  let second_reload_step = browser
    .submit(Command::Reload)
    .expect("browser second BFG 10K chainfire reload");
  assert_eq!(second_reload_step.events, second_reload_expected_events);
  assert_eq!(second_reload_step.after, direct.observe_player());
  assert_eq!(
    second_reload_step.effects,
    effect_timeline_for_observations(
      &second_reload_step.before,
      &second_reload_step.after,
      &second_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&second_reload_step.after)
  );
  expected_events.extend(second_reload_expected_events);
  let second_reloaded = second_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(second_reloaded.chainfire_level, 3);
  assert_eq!(second_reloaded.clip, Some((50, 50)));

  let fourth_target = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive second reload")
    .position();
  let fourth_command = Command::AttackRangedChainfire(fourth_target);
  assert_eq!(
    BrowserSession::command_for_key("C", &second_reload_step.after),
    Some(fourth_command)
  );
  let fourth_expected_events = direct
    .step(fourth_command)
    .expect("direct fourth BFG 10K chainfire command");
  let fourth_step = browser
    .submit(fourth_command)
    .expect("browser fourth BFG 10K chainfire command");
  assert_eq!(fourth_step.events, fourth_expected_events);
  assert_eq!(fourth_step.after, direct.observe_player());
  assert_eq!(
    fourth_step.effects,
    effect_timeline_for_observations(
      &fourth_step.before,
      &fourth_step.after,
      &fourth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fourth_step.after)
  );
  assert_eq!(
    fourth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    fourth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  expected_events.extend(fourth_expected_events);
  let fourth_bfg10k = fourth_step.after.equipped_weapon.as_ref().expect("BFG 10K");
  assert_eq!(fourth_bfg10k.chainfire_level, 4);
  assert_eq!(fourth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &fourth_step.after),
    None
  );

  let third_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct third BFG 10K chainfire reload");
  let third_reload_step = browser
    .submit(Command::Reload)
    .expect("browser third BFG 10K chainfire reload");
  assert_eq!(third_reload_step.events, third_reload_expected_events);
  assert_eq!(third_reload_step.after, direct.observe_player());
  assert_eq!(
    third_reload_step.effects,
    effect_timeline_for_observations(
      &third_reload_step.before,
      &third_reload_step.after,
      &third_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&third_reload_step.after)
  );
  expected_events.extend(third_reload_expected_events);
  let third_reloaded = third_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(third_reloaded.chainfire_level, 4);
  assert_eq!(third_reloaded.clip, Some((50, 50)));

  let fifth_target = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive third reload")
    .position();
  let fifth_command = Command::AttackRangedChainfire(fifth_target);
  assert_eq!(
    BrowserSession::command_for_key("C", &direct.observe_player()),
    Some(fifth_command)
  );
  let fifth_expected_events = direct
    .step(fifth_command)
    .expect("direct fifth BFG 10K chainfire command");
  let fifth_step = browser
    .submit(fifth_command)
    .expect("browser fifth BFG 10K chainfire command");
  assert_eq!(fifth_step.events, fifth_expected_events);
  assert_eq!(fifth_step.after, direct.observe_player());
  assert_eq!(
    fifth_step.effects,
    effect_timeline_for_observations(
      &fifth_step.before,
      &fifth_step.after,
      &fifth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fifth_step.after)
  );
  assert_eq!(
    fifth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    fifth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let fifth_bfg10k = fifth_step.after.equipped_weapon.as_ref().expect("BFG 10K");
  assert_eq!(fifth_bfg10k.chainfire_level, 5);
  assert_eq!(fifth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &fifth_step.after),
    None
  );
  expected_events.extend(fifth_expected_events);

  let fourth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct fourth BFG 10K chainfire reload");
  let fourth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser fourth BFG 10K chainfire reload");
  assert_eq!(fourth_reload_step.events, fourth_reload_expected_events);
  assert_eq!(fourth_reload_step.after, direct.observe_player());
  assert_eq!(
    fourth_reload_step.effects,
    effect_timeline_for_observations(
      &fourth_reload_step.before,
      &fourth_reload_step.after,
      &fourth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fourth_reload_step.after)
  );
  expected_events.extend(fourth_reload_expected_events);
  let fourth_reloaded = fourth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(fourth_reloaded.chainfire_level, 5);
  assert_eq!(fourth_reloaded.clip, Some((50, 50)));

  let sixth_target = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive fourth reload")
    .position();
  let sixth_command = Command::AttackRangedChainfire(sixth_target);
  assert_eq!(
    BrowserSession::command_for_key("C", &fourth_reload_step.after),
    Some(sixth_command)
  );
  let sixth_expected_events = direct
    .step(sixth_command)
    .expect("direct sixth BFG 10K chainfire command");
  let sixth_step = browser
    .submit(sixth_command)
    .expect("browser sixth BFG 10K chainfire command");
  assert_eq!(sixth_step.events, sixth_expected_events);
  assert_eq!(sixth_step.after, direct.observe_player());
  assert_eq!(
    sixth_step.effects,
    effect_timeline_for_observations(
      &sixth_step.before,
      &sixth_step.after,
      &sixth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&sixth_step.after)
  );
  assert_eq!(
    sixth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    sixth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let sixth_bfg10k = sixth_step.after.equipped_weapon.as_ref().expect("BFG 10K");
  assert_eq!(sixth_bfg10k.chainfire_level, 6);
  assert_eq!(sixth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &sixth_step.after),
    None
  );
  expected_events.extend(sixth_expected_events);

  let fifth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct fifth BFG 10K chainfire reload");
  let fifth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser fifth BFG 10K chainfire reload");
  assert_eq!(fifth_reload_step.events, fifth_reload_expected_events);
  assert_eq!(fifth_reload_step.after, direct.observe_player());
  assert_eq!(
    fifth_reload_step.effects,
    effect_timeline_for_observations(
      &fifth_reload_step.before,
      &fifth_reload_step.after,
      &fifth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fifth_reload_step.after)
  );
  expected_events.extend(fifth_reload_expected_events);
  let fifth_reloaded = fifth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(fifth_reloaded.chainfire_level, 6);
  assert_eq!(fifth_reloaded.clip, Some((50, 50)));

  let seventh_target = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive fifth reload")
    .position();
  let seventh_command = Command::AttackRangedChainfire(seventh_target);
  assert_eq!(
    BrowserSession::command_for_key("C", &fifth_reload_step.after),
    Some(seventh_command)
  );
  let seventh_expected_events = direct
    .step(seventh_command)
    .expect("direct seventh BFG 10K chainfire command");
  let seventh_step = browser
    .submit(seventh_command)
    .expect("browser seventh BFG 10K chainfire command");
  assert_eq!(seventh_step.events, seventh_expected_events);
  assert_eq!(seventh_step.after, direct.observe_player());
  assert_eq!(
    seventh_step.effects,
    effect_timeline_for_observations(
      &seventh_step.before,
      &seventh_step.after,
      &seventh_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&seventh_step.after)
  );
  assert_eq!(
    seventh_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    seventh_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let seventh_bfg10k = seventh_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(seventh_bfg10k.chainfire_level, 7);
  assert_eq!(seventh_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &seventh_step.after),
    None
  );
  expected_events.extend(seventh_expected_events);

  let sixth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct sixth BFG 10K chainfire reload");
  let sixth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser sixth BFG 10K chainfire reload");
  assert_eq!(sixth_reload_step.events, sixth_reload_expected_events);
  assert_eq!(sixth_reload_step.after, direct.observe_player());
  assert_eq!(
    sixth_reload_step.effects,
    effect_timeline_for_observations(
      &sixth_reload_step.before,
      &sixth_reload_step.after,
      &sixth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&sixth_reload_step.after)
  );
  expected_events.extend(sixth_reload_expected_events);
  let sixth_reloaded = sixth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(sixth_reloaded.chainfire_level, 7);
  assert_eq!(sixth_reloaded.clip, Some((50, 50)));

  let eighth_target = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive sixth reload")
    .position();
  let eighth_command = Command::AttackRangedChainfire(eighth_target);
  assert_eq!(
    BrowserSession::command_for_key("C", &sixth_reload_step.after),
    Some(eighth_command)
  );
  let eighth_expected_events = direct
    .step(eighth_command)
    .expect("direct eighth BFG 10K chainfire command");
  let eighth_step = browser
    .submit(eighth_command)
    .expect("browser eighth BFG 10K chainfire command");
  assert_eq!(eighth_step.events, eighth_expected_events);
  assert_eq!(eighth_step.after, direct.observe_player());
  assert_eq!(
    eighth_step.effects,
    effect_timeline_for_observations(
      &eighth_step.before,
      &eighth_step.after,
      &eighth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&eighth_step.after)
  );
  assert_eq!(
    eighth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    eighth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let eighth_bfg10k = eighth_step.after.equipped_weapon.as_ref().expect("BFG 10K");
  assert_eq!(eighth_bfg10k.chainfire_level, 8);
  assert_eq!(eighth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &eighth_step.after),
    None
  );
  expected_events.extend(eighth_expected_events);

  let seventh_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct seventh BFG 10K chainfire reload");
  let seventh_reload_step = browser
    .submit(Command::Reload)
    .expect("browser seventh BFG 10K chainfire reload");
  assert_eq!(seventh_reload_step.events, seventh_reload_expected_events);
  assert_eq!(seventh_reload_step.after, direct.observe_player());
  assert_eq!(
    seventh_reload_step.effects,
    effect_timeline_for_observations(
      &seventh_reload_step.before,
      &seventh_reload_step.after,
      &seventh_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&seventh_reload_step.after)
  );
  expected_events.extend(seventh_reload_expected_events);
  let seventh_reloaded = seventh_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(seventh_reloaded.chainfire_level, 8);
  assert_eq!(seventh_reloaded.clip, Some((50, 50)));

  let ninth_target = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive seventh reload")
    .position();
  let ninth_command = Command::AttackRangedChainfire(ninth_target);
  assert_eq!(
    BrowserSession::command_for_key("C", &seventh_reload_step.after),
    Some(ninth_command)
  );
  let ninth_expected_events = direct
    .step(ninth_command)
    .expect("direct ninth BFG 10K chainfire command");
  let ninth_step = browser
    .submit(ninth_command)
    .expect("browser ninth BFG 10K chainfire command");
  assert_eq!(ninth_step.events, ninth_expected_events);
  assert_eq!(ninth_step.after, direct.observe_player());
  assert_eq!(
    ninth_step.effects,
    effect_timeline_for_observations(
      &ninth_step.before,
      &ninth_step.after,
      &ninth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&ninth_step.after)
  );
  assert_eq!(
    ninth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    ninth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let ninth_bfg10k = ninth_step.after.equipped_weapon.as_ref().expect("BFG 10K");
  assert_eq!(ninth_bfg10k.chainfire_level, 9);
  assert_eq!(ninth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &ninth_step.after),
    None
  );
  expected_events.extend(ninth_expected_events);

  let ninth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct ninth BFG 10K chainfire reload");
  let ninth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser ninth BFG 10K chainfire reload");
  assert_eq!(ninth_reload_step.events, ninth_reload_expected_events);
  assert_eq!(ninth_reload_step.after, direct.observe_player());
  assert_eq!(
    ninth_reload_step.effects,
    effect_timeline_for_observations(
      &ninth_reload_step.before,
      &ninth_reload_step.after,
      &ninth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&ninth_reload_step.after)
  );
  expected_events.extend(ninth_reload_expected_events);
  let ninth_reloaded = ninth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(ninth_reloaded.chainfire_level, 9);
  assert_eq!(ninth_reloaded.clip, Some((50, 50)));

  let tenth_target_position = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive ninth reload")
    .position();
  let tenth_command = Command::AttackRangedChainfire(tenth_target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &ninth_reload_step.after),
    Some(tenth_command)
  );
  let tenth_expected_events = direct
    .step(tenth_command)
    .expect("direct tenth BFG 10K chainfire command");
  let tenth_step = browser
    .submit(tenth_command)
    .expect("browser tenth BFG 10K chainfire command");
  assert_eq!(tenth_step.events, tenth_expected_events);
  assert_eq!(tenth_step.after, direct.observe_player());
  assert_eq!(
    tenth_step.effects,
    effect_timeline_for_observations(
      &tenth_step.before,
      &tenth_step.after,
      &tenth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&tenth_step.after)
  );
  assert_eq!(
    tenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    tenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let tenth_bfg10k = tenth_step.after.equipped_weapon.as_ref().expect("BFG 10K");
  assert_eq!(tenth_bfg10k.chainfire_level, 10);
  assert_eq!(tenth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &tenth_step.after),
    None
  );
  expected_events.extend(tenth_expected_events);

  let tenth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct tenth BFG 10K chainfire reload");
  let tenth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser tenth BFG 10K chainfire reload");
  assert_eq!(tenth_reload_step.events, tenth_reload_expected_events);
  assert_eq!(tenth_reload_step.after, direct.observe_player());
  assert_eq!(
    tenth_reload_step.effects,
    effect_timeline_for_observations(
      &tenth_reload_step.before,
      &tenth_reload_step.after,
      &tenth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&tenth_reload_step.after)
  );
  expected_events.extend(tenth_reload_expected_events);
  let tenth_reloaded = tenth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(tenth_reloaded.chainfire_level, 10);
  assert_eq!(tenth_reloaded.clip, Some((50, 50)));

  let eleventh_target_position = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive tenth reload")
    .position();
  let eleventh_command = Command::AttackRangedChainfire(eleventh_target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &tenth_reload_step.after),
    Some(eleventh_command)
  );
  let eleventh_expected_events = direct
    .step(eleventh_command)
    .expect("direct eleventh BFG 10K chainfire command");
  let eleventh_step = browser
    .submit(eleventh_command)
    .expect("browser eleventh BFG 10K chainfire command");
  assert_eq!(eleventh_step.events, eleventh_expected_events);
  assert_eq!(eleventh_step.after, direct.observe_player());
  assert_eq!(
    eleventh_step.effects,
    effect_timeline_for_observations(
      &eleventh_step.before,
      &eleventh_step.after,
      &eleventh_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&eleventh_step.after)
  );
  assert_eq!(
    eleventh_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    eleventh_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let eleventh_bfg10k = eleventh_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(eleventh_bfg10k.chainfire_level, 11);
  assert_eq!(eleventh_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &eleventh_step.after),
    None
  );
  expected_events.extend(eleventh_expected_events);

  let twelfth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct twelfth BFG 10K chainfire reload");
  let twelfth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser twelfth BFG 10K chainfire reload");
  assert_eq!(twelfth_reload_step.events, twelfth_reload_expected_events);
  assert_eq!(twelfth_reload_step.after, direct.observe_player());
  assert_eq!(
    twelfth_reload_step.effects,
    effect_timeline_for_observations(
      &twelfth_reload_step.before,
      &twelfth_reload_step.after,
      &twelfth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&twelfth_reload_step.after)
  );
  expected_events.extend(twelfth_reload_expected_events);
  let twelfth_reloaded = twelfth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(twelfth_reloaded.chainfire_level, 11);
  assert_eq!(twelfth_reloaded.clip, Some((50, 50)));

  let twelfth_target_position = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive eleventh reload")
    .position();
  let twelfth_command = Command::AttackRangedChainfire(twelfth_target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &twelfth_reload_step.after),
    Some(twelfth_command)
  );
  let twelfth_expected_events = direct
    .step(twelfth_command)
    .expect("direct twelfth BFG 10K chainfire command");
  let twelfth_step = browser
    .submit(twelfth_command)
    .expect("browser twelfth BFG 10K chainfire command");
  assert_eq!(twelfth_step.events, twelfth_expected_events);
  assert_eq!(twelfth_step.after, direct.observe_player());
  assert_eq!(
    twelfth_step.effects,
    effect_timeline_for_observations(
      &twelfth_step.before,
      &twelfth_step.after,
      &twelfth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&twelfth_step.after)
  );
  assert_eq!(
    twelfth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    twelfth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let twelfth_bfg10k = twelfth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(twelfth_bfg10k.chainfire_level, 12);
  assert_eq!(twelfth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &twelfth_step.after),
    None
  );
  expected_events.extend(twelfth_expected_events);

  let thirteenth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct thirteenth BFG 10K chainfire reload");
  let thirteenth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser thirteenth BFG 10K chainfire reload");
  assert_eq!(
    thirteenth_reload_step.events,
    thirteenth_reload_expected_events
  );
  assert_eq!(thirteenth_reload_step.after, direct.observe_player());
  assert_eq!(
    thirteenth_reload_step.effects,
    effect_timeline_for_observations(
      &thirteenth_reload_step.before,
      &thirteenth_reload_step.after,
      &thirteenth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&thirteenth_reload_step.after)
  );
  expected_events.extend(thirteenth_reload_expected_events);
  let thirteenth_reloaded = thirteenth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(thirteenth_reloaded.chainfire_level, 12);
  assert_eq!(thirteenth_reloaded.clip, Some((50, 50)));

  let thirteenth_target_position = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive twelfth reload")
    .position();
  let thirteenth_command = Command::AttackRangedChainfire(thirteenth_target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &thirteenth_reload_step.after),
    Some(thirteenth_command)
  );
  let thirteenth_expected_events = direct
    .step(thirteenth_command)
    .expect("direct thirteenth BFG 10K chainfire command");
  let thirteenth_step = browser
    .submit(thirteenth_command)
    .expect("browser thirteenth BFG 10K chainfire command");
  assert_eq!(thirteenth_step.events, thirteenth_expected_events);
  assert_eq!(thirteenth_step.after, direct.observe_player());
  assert_eq!(
    thirteenth_step.effects,
    effect_timeline_for_observations(
      &thirteenth_step.before,
      &thirteenth_step.after,
      &thirteenth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&thirteenth_step.after)
  );
  assert_eq!(
    thirteenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    thirteenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let thirteenth_bfg10k = thirteenth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(thirteenth_bfg10k.chainfire_level, 13);
  assert_eq!(thirteenth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &thirteenth_step.after),
    None
  );
  expected_events.extend(thirteenth_expected_events);

  let fourteenth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct fourteenth BFG 10K chainfire reload");
  let fourteenth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser fourteenth BFG 10K chainfire reload");
  assert_eq!(
    fourteenth_reload_step.events,
    fourteenth_reload_expected_events
  );
  assert_eq!(fourteenth_reload_step.after, direct.observe_player());
  assert_eq!(
    fourteenth_reload_step.effects,
    effect_timeline_for_observations(
      &fourteenth_reload_step.before,
      &fourteenth_reload_step.after,
      &fourteenth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fourteenth_reload_step.after)
  );
  expected_events.extend(fourteenth_reload_expected_events);
  let fourteenth_reloaded = fourteenth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(fourteenth_reloaded.chainfire_level, 13);
  assert_eq!(fourteenth_reloaded.clip, Some((50, 50)));

  let fourteenth_target_position = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive thirteenth reload")
    .position();
  let fourteenth_command = Command::AttackRangedChainfire(fourteenth_target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &fourteenth_reload_step.after),
    Some(fourteenth_command)
  );
  let fourteenth_expected_events = direct
    .step(fourteenth_command)
    .expect("direct fourteenth BFG 10K chainfire command");
  let fourteenth_step = browser
    .submit(fourteenth_command)
    .expect("browser fourteenth BFG 10K chainfire command");
  assert_eq!(fourteenth_step.events, fourteenth_expected_events);
  assert_eq!(fourteenth_step.after, direct.observe_player());
  assert_eq!(
    fourteenth_step.effects,
    effect_timeline_for_observations(
      &fourteenth_step.before,
      &fourteenth_step.after,
      &fourteenth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fourteenth_step.after)
  );
  assert_eq!(
    fourteenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    fourteenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let fourteenth_bfg10k = fourteenth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(fourteenth_bfg10k.chainfire_level, 14);
  assert_eq!(fourteenth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &fourteenth_step.after),
    None
  );
  expected_events.extend(fourteenth_expected_events);

  let fifteenth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct fifteenth BFG 10K chainfire reload");
  let fifteenth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser fifteenth BFG 10K chainfire reload");
  assert_eq!(
    fifteenth_reload_step.events,
    fifteenth_reload_expected_events
  );
  assert_eq!(fifteenth_reload_step.after, direct.observe_player());
  assert_eq!(
    fifteenth_reload_step.effects,
    effect_timeline_for_observations(
      &fifteenth_reload_step.before,
      &fifteenth_reload_step.after,
      &fifteenth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fifteenth_reload_step.after)
  );
  expected_events.extend(fifteenth_reload_expected_events);
  let fifteenth_reloaded = fifteenth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(fifteenth_reloaded.chainfire_level, 14);
  assert_eq!(fifteenth_reloaded.clip, Some((50, 50)));

  let fifteenth_target_position = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive fourteenth reload")
    .position();
  let fifteenth_command = Command::AttackRangedChainfire(fifteenth_target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &fifteenth_reload_step.after),
    Some(fifteenth_command)
  );
  let fifteenth_expected_events = direct
    .step(fifteenth_command)
    .expect("direct fifteenth BFG 10K chainfire command");
  let fifteenth_step = browser
    .submit(fifteenth_command)
    .expect("browser fifteenth BFG 10K chainfire command");
  assert_eq!(fifteenth_step.events, fifteenth_expected_events);
  assert_eq!(fifteenth_step.after, direct.observe_player());
  assert_eq!(
    fifteenth_step.effects,
    effect_timeline_for_observations(
      &fifteenth_step.before,
      &fifteenth_step.after,
      &fifteenth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fifteenth_step.after)
  );
  assert_eq!(
    fifteenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    fifteenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let fifteenth_bfg10k = fifteenth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(fifteenth_bfg10k.chainfire_level, 15);
  assert_eq!(fifteenth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &fifteenth_step.after),
    None
  );
  expected_events.extend(fifteenth_expected_events);

  let sixteenth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct sixteenth BFG 10K chainfire reload");
  let sixteenth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser sixteenth BFG 10K chainfire reload");
  assert_eq!(
    sixteenth_reload_step.events,
    sixteenth_reload_expected_events
  );
  assert_eq!(sixteenth_reload_step.after, direct.observe_player());
  assert_eq!(
    sixteenth_reload_step.effects,
    effect_timeline_for_observations(
      &sixteenth_reload_step.before,
      &sixteenth_reload_step.after,
      &sixteenth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&sixteenth_reload_step.after)
  );
  expected_events.extend(sixteenth_reload_expected_events);
  let sixteenth_reloaded = sixteenth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(sixteenth_reloaded.chainfire_level, 15);
  assert_eq!(sixteenth_reloaded.clip, Some((50, 50)));

  let sixteenth_target_position = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive fifteenth reload")
    .position();
  let sixteenth_command = Command::AttackRangedChainfire(sixteenth_target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &sixteenth_reload_step.after),
    Some(sixteenth_command)
  );
  let sixteenth_expected_events = direct
    .step(sixteenth_command)
    .expect("direct sixteenth BFG 10K chainfire command");
  let sixteenth_step = browser
    .submit(sixteenth_command)
    .expect("browser sixteenth BFG 10K chainfire command");
  assert_eq!(sixteenth_step.events, sixteenth_expected_events);
  assert_eq!(sixteenth_step.after, direct.observe_player());
  assert_eq!(
    sixteenth_step.effects,
    effect_timeline_for_observations(
      &sixteenth_step.before,
      &sixteenth_step.after,
      &sixteenth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&sixteenth_step.after)
  );
  assert_eq!(
    sixteenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    sixteenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let sixteenth_bfg10k = sixteenth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(sixteenth_bfg10k.chainfire_level, 16);
  assert_eq!(sixteenth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &sixteenth_step.after),
    None
  );
  expected_events.extend(sixteenth_expected_events);

  let seventeenth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct seventeenth BFG 10K chainfire reload");
  let seventeenth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser seventeenth BFG 10K chainfire reload");
  assert_eq!(
    seventeenth_reload_step.events,
    seventeenth_reload_expected_events
  );
  assert_eq!(seventeenth_reload_step.after, direct.observe_player());
  assert_eq!(
    seventeenth_reload_step.effects,
    effect_timeline_for_observations(
      &seventeenth_reload_step.before,
      &seventeenth_reload_step.after,
      &seventeenth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&seventeenth_reload_step.after)
  );
  expected_events.extend(seventeenth_reload_expected_events);
  let seventeenth_reloaded = seventeenth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(seventeenth_reloaded.chainfire_level, 16);
  assert_eq!(seventeenth_reloaded.clip, Some((50, 50)));

  let seventeenth_target_position = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive sixteenth reload")
    .position();
  let seventeenth_command = Command::AttackRangedChainfire(seventeenth_target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &seventeenth_reload_step.after),
    Some(seventeenth_command)
  );
  let seventeenth_expected_events = direct
    .step(seventeenth_command)
    .expect("direct seventeenth BFG 10K chainfire command");
  let seventeenth_step = browser
    .submit(seventeenth_command)
    .expect("browser seventeenth BFG 10K chainfire command");
  assert_eq!(seventeenth_step.events, seventeenth_expected_events);
  assert_eq!(seventeenth_step.after, direct.observe_player());
  assert_eq!(
    seventeenth_step.effects,
    effect_timeline_for_observations(
      &seventeenth_step.before,
      &seventeenth_step.after,
      &seventeenth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&seventeenth_step.after)
  );
  assert_eq!(
    seventeenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    seventeenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let seventeenth_bfg10k = seventeenth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(seventeenth_bfg10k.chainfire_level, 17);
  assert_eq!(seventeenth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &seventeenth_step.after),
    None
  );
  expected_events.extend(seventeenth_expected_events);

  let eighteenth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct eighteenth BFG 10K chainfire reload");
  let eighteenth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser eighteenth BFG 10K chainfire reload");
  assert_eq!(
    eighteenth_reload_step.events,
    eighteenth_reload_expected_events
  );
  assert_eq!(eighteenth_reload_step.after, direct.observe_player());
  assert_eq!(
    eighteenth_reload_step.effects,
    effect_timeline_for_observations(
      &eighteenth_reload_step.before,
      &eighteenth_reload_step.after,
      &eighteenth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&eighteenth_reload_step.after)
  );
  expected_events.extend(eighteenth_reload_expected_events);
  let eighteenth_reloaded = eighteenth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(eighteenth_reloaded.chainfire_level, 17);
  assert_eq!(eighteenth_reloaded.clip, Some((50, 50)));

  let eighteenth_target_position = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive seventeenth reload")
    .position();
  let eighteenth_command = Command::AttackRangedChainfire(eighteenth_target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &eighteenth_reload_step.after),
    Some(eighteenth_command)
  );
  let eighteenth_expected_events = direct
    .step(eighteenth_command)
    .expect("direct eighteenth BFG 10K chainfire command");
  let eighteenth_step = browser
    .submit(eighteenth_command)
    .expect("browser eighteenth BFG 10K chainfire command");
  assert_eq!(eighteenth_step.events, eighteenth_expected_events);
  assert_eq!(eighteenth_step.after, direct.observe_player());
  assert_eq!(
    eighteenth_step.effects,
    effect_timeline_for_observations(
      &eighteenth_step.before,
      &eighteenth_step.after,
      &eighteenth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&eighteenth_step.after)
  );
  assert_eq!(
    eighteenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    eighteenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let eighteenth_bfg10k = eighteenth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(eighteenth_bfg10k.chainfire_level, 18);
  assert_eq!(eighteenth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &eighteenth_step.after),
    None
  );
  expected_events.extend(eighteenth_expected_events);

  let nineteenth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct nineteenth BFG 10K chainfire reload");
  let nineteenth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser nineteenth BFG 10K chainfire reload");
  assert_eq!(
    nineteenth_reload_step.events,
    nineteenth_reload_expected_events
  );
  assert_eq!(nineteenth_reload_step.after, direct.observe_player());
  assert_eq!(
    nineteenth_reload_step.effects,
    effect_timeline_for_observations(
      &nineteenth_reload_step.before,
      &nineteenth_reload_step.after,
      &nineteenth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&nineteenth_reload_step.after)
  );
  expected_events.extend(nineteenth_reload_expected_events);
  let nineteenth_reloaded = nineteenth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(nineteenth_reloaded.chainfire_level, 18);
  assert_eq!(nineteenth_reloaded.clip, Some((50, 50)));

  let nineteenth_target_position = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K target should survive eighteenth reload")
    .position();
  let nineteenth_command = Command::AttackRangedChainfire(nineteenth_target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &nineteenth_reload_step.after),
    Some(nineteenth_command)
  );
  let nineteenth_expected_events = direct
    .step(nineteenth_command)
    .expect("direct nineteenth BFG 10K chainfire command");
  let nineteenth_step = browser
    .submit(nineteenth_command)
    .expect("browser nineteenth BFG 10K chainfire command");
  assert_eq!(nineteenth_step.events, nineteenth_expected_events);
  assert_eq!(nineteenth_step.after, direct.observe_player());
  assert_eq!(
    nineteenth_step.effects,
    effect_timeline_for_observations(
      &nineteenth_step.before,
      &nineteenth_step.after,
      &nineteenth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&nineteenth_step.after)
  );
  assert_eq!(
    nineteenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    nineteenth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let nineteenth_bfg10k = nineteenth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(nineteenth_bfg10k.chainfire_level, 19);
  assert_eq!(nineteenth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &nineteenth_step.after),
    None
  );
  expected_events.extend(nineteenth_expected_events);

  let twentieth_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct twentieth BFG 10K chainfire reload");
  let twentieth_reload_step = browser
    .submit(Command::Reload)
    .expect("browser twentieth BFG 10K chainfire reload");
  assert_eq!(
    twentieth_reload_step.events,
    twentieth_reload_expected_events
  );
  assert_eq!(twentieth_reload_step.after, direct.observe_player());
  assert_eq!(
    twentieth_reload_step.effects,
    effect_timeline_for_observations(
      &twentieth_reload_step.before,
      &twentieth_reload_step.after,
      &twentieth_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&twentieth_reload_step.after)
  );
  expected_events.extend(twentieth_reload_expected_events);
  let twentieth_reloaded = twentieth_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(twentieth_reloaded.chainfire_level, 19);
  assert_eq!(twentieth_reloaded.clip, Some((50, 50)));

  let twentieth_target_position = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K chainfire target should survive nineteenth reload")
    .position();
  let twentieth_command = Command::AttackRangedChainfire(twentieth_target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &twentieth_reload_step.after),
    Some(twentieth_command)
  );
  let twentieth_expected_events = direct
    .step(twentieth_command)
    .expect("direct twentieth BFG 10K chainfire command");
  let twentieth_step = browser
    .submit(twentieth_command)
    .expect("browser twentieth BFG 10K chainfire command");
  assert_eq!(twentieth_step.events, twentieth_expected_events);
  assert_eq!(twentieth_step.after, direct.observe_player());
  assert_eq!(
    twentieth_step.effects,
    effect_timeline_for_observations(
      &twentieth_step.before,
      &twentieth_step.after,
      &twentieth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&twentieth_step.after)
  );
  assert_eq!(
    twentieth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    twentieth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let twentieth_bfg10k = twentieth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(twentieth_bfg10k.chainfire_level, 20);
  assert_eq!(twentieth_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &twentieth_step.after),
    None
  );
  expected_events.extend(twentieth_expected_events);

  let twenty_first_reload_expected_events = direct
    .step(Command::Reload)
    .expect("direct twenty-first BFG 10K chainfire reload");
  let twenty_first_reload_step = browser
    .submit(Command::Reload)
    .expect("browser twenty-first BFG 10K chainfire reload");
  assert_eq!(
    twenty_first_reload_step.events,
    twenty_first_reload_expected_events
  );
  assert_eq!(twenty_first_reload_step.after, direct.observe_player());
  assert_eq!(
    twenty_first_reload_step.effects,
    effect_timeline_for_observations(
      &twenty_first_reload_step.before,
      &twenty_first_reload_step.after,
      &twenty_first_reload_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&twenty_first_reload_step.after)
  );
  expected_events.extend(twenty_first_reload_expected_events);
  let twenty_first_reloaded = twenty_first_reload_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(twenty_first_reloaded.chainfire_level, 20);
  assert_eq!(twenty_first_reloaded.clip, Some((50, 50)));

  let twenty_first_target_position = direct
    .world()
    .get_actor(target_id)
    .expect("BFG 10K chainfire target should survive twentieth reload")
    .position();
  let twenty_first_command = Command::AttackRangedChainfire(twenty_first_target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &twenty_first_reload_step.after),
    Some(twenty_first_command)
  );
  let twenty_first_expected_events = direct
    .step(twenty_first_command)
    .expect("direct twenty-first BFG 10K chainfire command");
  let twenty_first_step = browser
    .submit(twenty_first_command)
    .expect("browser twenty-first BFG 10K chainfire command");
  assert_eq!(twenty_first_step.events, twenty_first_expected_events);
  assert_eq!(twenty_first_step.after, direct.observe_player());
  assert_eq!(
    twenty_first_step.effects,
    effect_timeline_for_observations(
      &twenty_first_step.before,
      &twenty_first_step.after,
      &twenty_first_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&twenty_first_step.after)
  );
  assert_eq!(
    twenty_first_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    7
  );
  assert_eq!(
    twenty_first_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      ))
      .count(),
    7
  );
  let twenty_first_bfg10k = twenty_first_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("BFG 10K");
  assert_eq!(twenty_first_bfg10k.chainfire_level, 21);
  assert_eq!(twenty_first_bfg10k.clip, Some((15, 50)));
  assert_eq!(
    BrowserSession::command_for_key("C", &twenty_first_step.after),
    None
  );
  expected_events.extend(twenty_first_expected_events);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  command_replay.record_command(second_command);
  command_replay.record_command(reload_command);
  command_replay.record_command(third_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(fourth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(fifth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(sixth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(seventh_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(eighth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(ninth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(tenth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(eleventh_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(twelfth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(thirteenth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(fourteenth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(fifteenth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(sixteenth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(seventeenth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(eighteenth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(nineteenth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(twentieth_command);
  command_replay.record_command(Command::Reload);
  command_replay.record_command(twenty_first_command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}

#[test]
fn nuclear_plasma_chainfire_vertical_browser_boundary_matches_direct_core() {
  let player_position = Position::new(1, 1);
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoCells(10)],
    equipped_weapon: Some(ItemSpawnKind::NuclearPlasmaRifle),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let target_position = Position::new(3, 1);
  let mut setup_replay =
    ReplayLog::new(2_649, 8, 4, player_position).with_player_config(player_config);
  setup_replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Static Target",
    500,
    0,
    (1, 7),
  ));

  let (initial, setup_events) =
    drl_core::ReplayEngine::run(&setup_replay).expect("vertical replay setup");
  assert!(setup_events.is_empty());
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("static target")
    .id();
  let command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &initial.observe_player()),
    Some(command)
  );

  let mut direct = initial.clone();
  let mut browser = BrowserSession::from_game(initial);
  let mut expected_events = direct
    .step(command)
    .expect("direct Nuclear Plasma chainfire command");
  let step = browser
    .submit(command)
    .expect("browser Nuclear Plasma chainfire command");
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
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    4
  );
  assert_eq!(
    step
      .after
      .equipped_weapon
      .expect("Nuclear Plasma Rifle")
      .chainfire_level,
    1
  );
  let second_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &direct.observe_player()),
    Some(second_command)
  );
  let second_expected_events = direct
    .step(second_command)
    .expect("direct second Nuclear Plasma chainfire command");
  let second_step = browser
    .submit(second_command)
    .expect("browser second Nuclear Plasma chainfire command");
  assert_eq!(second_step.events, second_expected_events);
  assert_eq!(second_step.after, direct.observe_player());
  assert_eq!(
    second_step.effects,
    effect_timeline_for_observations(
      &second_step.before,
      &second_step.after,
      &second_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&second_step.after)
  );
  assert_eq!(
    second_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    6
  );
  let nuclear_plasma = second_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Nuclear Plasma Rifle");
  assert_eq!(nuclear_plasma.chainfire_level, 2);
  assert_eq!(nuclear_plasma.clip, Some((14, 24)));
  assert_eq!(
    BrowserSession::command_for_key("C", &second_step.after),
    Some(Command::AttackRangedChainfire(target_position))
  );
  let third_command = Command::AttackRangedChainfire(target_position);
  let third_expected_events = direct
    .step(third_command)
    .expect("direct third Nuclear Plasma chainfire command");
  let third_step = browser
    .submit(third_command)
    .expect("browser third Nuclear Plasma chainfire command");
  assert_eq!(third_step.events, third_expected_events);
  assert_eq!(third_step.after, direct.observe_player());
  assert_eq!(
    third_step.effects,
    effect_timeline_for_observations(
      &third_step.before,
      &third_step.after,
      &third_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&third_step.after)
  );
  assert_eq!(
    third_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    9
  );
  let nuclear_plasma = third_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Nuclear Plasma Rifle");
  assert_eq!(nuclear_plasma.chainfire_level, 3);
  assert_eq!(nuclear_plasma.clip, Some((5, 24)));
  expected_events.extend(second_expected_events);
  expected_events.extend(third_expected_events);

  for _ in 0..47 {
    let wait_expected_events = direct
      .step(Command::Wait)
      .expect("direct Nuclear Plasma recharge wait");
    let wait_step = browser
      .submit(Command::Wait)
      .expect("browser Nuclear Plasma recharge wait");
    assert_eq!(wait_step.events, wait_expected_events);
    assert_eq!(wait_step.after, direct.observe_player());
    assert_eq!(
      wait_step.effects,
      effect_timeline_for_observations(&wait_step.before, &wait_step.after, &wait_expected_events,)
    );
    assert_eq!(
      browser.scene(),
      RenderScene::from_observation(&wait_step.after)
    );
    expected_events.extend(wait_expected_events);
  }
  let recharged_observation = direct.observe_player();
  let recharged = recharged_observation
    .equipped_weapon
    .as_ref()
    .expect("Nuclear Plasma Rifle after recharge");
  assert_eq!(recharged.clip, Some((9, 24)));
  let fourth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &direct.observe_player()),
    Some(fourth_command)
  );
  let fourth_expected_events = direct
    .step(fourth_command)
    .expect("direct fourth Nuclear Plasma chainfire command");
  let fourth_step = browser
    .submit(fourth_command)
    .expect("browser fourth Nuclear Plasma chainfire command");
  assert_eq!(fourth_step.events, fourth_expected_events);
  assert_eq!(fourth_step.after, direct.observe_player());
  assert_eq!(
    fourth_step.effects,
    effect_timeline_for_observations(
      &fourth_step.before,
      &fourth_step.after,
      &fourth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fourth_step.after)
  );
  assert_eq!(
    fourth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    9
  );
  let nuclear_plasma = fourth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Nuclear Plasma Rifle");
  assert_eq!(nuclear_plasma.chainfire_level, 4);
  assert_eq!(nuclear_plasma.clip, Some((0, 24)));
  assert_eq!(
    BrowserSession::command_for_key("C", &fourth_step.after),
    None
  );
  expected_events.extend(fourth_expected_events);

  for _ in 0..57 {
    let wait_expected_events = direct
      .step(Command::Wait)
      .expect("direct Nuclear Plasma fifth-level recharge wait");
    let wait_step = browser
      .submit(Command::Wait)
      .expect("browser Nuclear Plasma fifth-level recharge wait");
    assert_eq!(wait_step.events, wait_expected_events);
    assert_eq!(wait_step.after, direct.observe_player());
    assert_eq!(
      wait_step.effects,
      effect_timeline_for_observations(&wait_step.before, &wait_step.after, &wait_expected_events,)
    );
    assert_eq!(
      browser.scene(),
      RenderScene::from_observation(&wait_step.after)
    );
    expected_events.extend(wait_expected_events);
  }
  let recharged_observation = direct.observe_player();
  let recharged = recharged_observation
    .equipped_weapon
    .as_ref()
    .expect("Nuclear Plasma Rifle after fifth-level recharge");
  assert_eq!(recharged.clip, Some((9, 24)));

  let fifth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &direct.observe_player()),
    Some(fifth_command)
  );
  let fifth_expected_events = direct
    .step(fifth_command)
    .expect("direct fifth Nuclear Plasma chainfire command");
  let fifth_step = browser
    .submit(fifth_command)
    .expect("browser fifth Nuclear Plasma chainfire command");
  assert_eq!(fifth_step.events, fifth_expected_events);
  assert_eq!(fifth_step.after, direct.observe_player());
  assert_eq!(
    fifth_step.effects,
    effect_timeline_for_observations(
      &fifth_step.before,
      &fifth_step.after,
      &fifth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&fifth_step.after)
  );
  assert_eq!(
    fifth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    9
  );
  let nuclear_plasma = fifth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Nuclear Plasma Rifle");
  assert_eq!(nuclear_plasma.chainfire_level, 5);
  assert_eq!(nuclear_plasma.clip, Some((0, 24)));
  expected_events.extend(fifth_expected_events);

  for _ in 0..57 {
    let wait_expected_events = direct
      .step(Command::Wait)
      .expect("direct Nuclear Plasma sixth-level recharge wait");
    let wait_step = browser
      .submit(Command::Wait)
      .expect("browser Nuclear Plasma sixth-level recharge wait");
    assert_eq!(wait_step.events, wait_expected_events);
    assert_eq!(wait_step.after, direct.observe_player());
    assert_eq!(
      wait_step.effects,
      effect_timeline_for_observations(&wait_step.before, &wait_step.after, &wait_expected_events,)
    );
    assert_eq!(
      browser.scene(),
      RenderScene::from_observation(&wait_step.after)
    );
    expected_events.extend(wait_expected_events);
  }
  let recharged_observation = direct.observe_player();
  let recharged = recharged_observation
    .equipped_weapon
    .as_ref()
    .expect("Nuclear Plasma Rifle after sixth-level recharge");
  assert_eq!(recharged.clip, Some((9, 24)));

  let sixth_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &recharged_observation),
    Some(sixth_command)
  );
  let sixth_expected_events = direct
    .step(sixth_command)
    .expect("direct sixth Nuclear Plasma chainfire command");
  let sixth_step = browser
    .submit(sixth_command)
    .expect("browser sixth Nuclear Plasma chainfire command");
  assert_eq!(sixth_step.events, sixth_expected_events);
  assert_eq!(sixth_step.after, direct.observe_player());
  assert_eq!(
    sixth_step.effects,
    effect_timeline_for_observations(
      &sixth_step.before,
      &sixth_step.after,
      &sixth_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&sixth_step.after)
  );
  assert_eq!(
    sixth_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    9
  );
  let nuclear_plasma = sixth_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Nuclear Plasma Rifle");
  assert_eq!(nuclear_plasma.chainfire_level, 6);
  assert_eq!(nuclear_plasma.clip, Some((0, 24)));
  assert_eq!(
    BrowserSession::command_for_key("C", &sixth_step.after),
    None
  );
  expected_events.extend(sixth_expected_events);

  for _ in 0..57 {
    let wait_expected_events = direct
      .step(Command::Wait)
      .expect("direct Nuclear Plasma seventh-level recharge wait");
    let wait_step = browser
      .submit(Command::Wait)
      .expect("browser Nuclear Plasma seventh-level recharge wait");
    assert_eq!(wait_step.events, wait_expected_events);
    assert_eq!(wait_step.after, direct.observe_player());
    assert_eq!(
      wait_step.effects,
      effect_timeline_for_observations(&wait_step.before, &wait_step.after, &wait_expected_events,)
    );
    assert_eq!(
      browser.scene(),
      RenderScene::from_observation(&wait_step.after)
    );
    expected_events.extend(wait_expected_events);
  }
  let recharged_observation = direct.observe_player();
  let recharged = recharged_observation
    .equipped_weapon
    .as_ref()
    .expect("Nuclear Plasma Rifle after seventh-level recharge");
  assert_eq!(recharged.clip, Some((9, 24)));

  let seventh_command = Command::AttackRangedChainfire(target_position);
  assert_eq!(
    BrowserSession::command_for_key("C", &recharged_observation),
    Some(seventh_command)
  );
  let seventh_expected_events = direct
    .step(seventh_command)
    .expect("direct seventh Nuclear Plasma chainfire command");
  let seventh_step = browser
    .submit(seventh_command)
    .expect("browser seventh Nuclear Plasma chainfire command");
  assert_eq!(seventh_step.events, seventh_expected_events);
  assert_eq!(seventh_step.after, direct.observe_player());
  assert_eq!(
    seventh_step.effects,
    effect_timeline_for_observations(
      &seventh_step.before,
      &seventh_step.after,
      &seventh_expected_events,
    )
  );
  assert_eq!(
    browser.scene(),
    RenderScene::from_observation(&seventh_step.after)
  );
  assert_eq!(
    seventh_expected_events
      .iter()
      .filter(|event| matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id,
          target_id: event_target,
          is_ranged: true,
          ..
        } if *attacker_id == direct.world().player_id().unwrap() && *event_target == target_id
      ))
      .count(),
    9
  );
  let nuclear_plasma = seventh_step
    .after
    .equipped_weapon
    .as_ref()
    .expect("Nuclear Plasma Rifle");
  assert_eq!(nuclear_plasma.chainfire_level, 7);
  assert_eq!(nuclear_plasma.clip, Some((0, 24)));
  assert_eq!(
    BrowserSession::command_for_key("C", &seventh_step.after),
    None
  );
  expected_events.extend(seventh_expected_events);

  let mut command_replay = setup_replay;
  command_replay.record_command(command);
  command_replay.record_command(second_command);
  command_replay.record_command(third_command);
  for _ in 0..47 {
    command_replay.record_command(Command::Wait);
  }
  command_replay.record_command(fourth_command);
  for _ in 0..57 {
    command_replay.record_command(Command::Wait);
  }
  command_replay.record_command(fifth_command);
  for _ in 0..57 {
    command_replay.record_command(Command::Wait);
  }
  command_replay.record_command(sixth_command);
  for _ in 0..57 {
    command_replay.record_command(Command::Wait);
  }
  command_replay.record_command(seventh_command);
  let (replayed, replay_events) =
    drl_core::ReplayEngine::run(&command_replay).expect("vertical command replay");
  assert_eq!(replay_events, expected_events);
  assert_eq!(replayed, direct);
  assert!(drl_core::ReplayEngine::verify_determinism(&command_replay).expect("replay determinism"));
}
