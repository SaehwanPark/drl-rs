use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::replay::ReplayEngine;
use drl_protocol::{
  Command, CommandError, EquipmentSlot, GameEvent, ItemSpawnKind, MonsterSpawnSpec,
  PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_nuclear_plasma_rifle(seed: u64) -> Game {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::nuclear_plasma_rifle(weapon_id))
    .unwrap();
  game
}

fn ranged_events(events: &[GameEvent], player_id: drl_protocol::EntityId) -> usize {
  events
    .iter()
    .filter(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          is_ranged: true,
          ..
        } if *attacker_id == player_id
      )
    })
    .count()
}

#[test]
fn nuclear_plasma_first_chainfire_emits_four_projectiles_and_advances_state() {
  let mut game = equipped_nuclear_plasma_rifle(2_560);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Nuclear Plasma chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 20);
  assert_eq!(weapon.weapon_properties().unwrap().chainfire_level, 1);
  assert_eq!(ranged_events(&events, player_id), 4);
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::AttackResolved {
      attacker_id,
      target_id: event_target,
      is_ranged: true,
      ..
    } if *attacker_id == player_id && *event_target == target_id
  )));
}

#[test]
fn nuclear_plasma_second_chainfire_emits_six_projectiles_and_advances_state() {
  let mut game = equipped_nuclear_plasma_rifle(2_560);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Nuclear Plasma chainfire burst should be accepted");
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("second Nuclear Plasma chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 14);
  assert_eq!(weapon.weapon_properties().unwrap().chainfire_level, 2);
  assert_eq!(ranged_events(&events, player_id), 6);
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::AttackResolved {
      attacker_id,
      target_id: event_target,
      is_ranged: true,
      ..
    } if *attacker_id == player_id && *event_target == target_id
  )));
}

#[test]
fn nuclear_plasma_third_chainfire_emits_nine_projectiles_and_advances_state() {
  let mut game = equipped_nuclear_plasma_rifle(2_646);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Nuclear Plasma chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second Nuclear Plasma chainfire burst should be accepted");
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("third Nuclear Plasma chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 5);
  assert_eq!(weapon.weapon_properties().unwrap().chainfire_level, 3);
  assert_eq!(ranged_events(&events, player_id), 9);
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::AttackResolved {
      attacker_id,
      target_id: event_target,
      is_ranged: true,
      ..
    } if *attacker_id == player_id && *event_target == target_id
  )));
}

#[test]
fn nuclear_plasma_fourth_chainfire_emits_nine_projectiles_and_advances_state() {
  let mut game = equipped_nuclear_plasma_rifle(2_648);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Nuclear Plasma chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second Nuclear Plasma chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third Nuclear Plasma chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 24;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourth Nuclear Plasma chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 4);
  assert_eq!(ranged_events(&events, player_id), 9);
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::AttackResolved {
      attacker_id,
      target_id: event_target,
      is_ranged: true,
      ..
    } if *attacker_id == player_id && *event_target == target_id
  )));
}

#[test]
fn nuclear_plasma_fifth_chainfire_emits_nine_projectiles_and_advances_state() {
  let mut game = equipped_nuclear_plasma_rifle(2_653);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Nuclear Plasma chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second Nuclear Plasma chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third Nuclear Plasma chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 24;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourth Nuclear Plasma chainfire burst should be accepted");
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("fifth Nuclear Plasma chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 6);
  assert_eq!(props.chainfire_level, 5);
  assert_eq!(ranged_events(&events, player_id), 9);
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::AttackResolved {
      attacker_id,
      target_id: event_target,
      is_ranged: true,
      ..
    } if *attacker_id == player_id && *event_target == target_id
  )));
}

#[test]
fn nuclear_plasma_chainfire_keeps_four_outcomes_after_lethal_target() {
  let mut lethal_case = None;

  for seed in 2_561..2_640 {
    let mut game = equipped_nuclear_plasma_rifle(seed);
    let target = Position::new(5, 2);
    let target_id = game
      .world_mut()
      .spawn_monster(target, "Fragile Target", 1, 100, (1, 7))
      .unwrap();
    let player_id = game.world().player_id().unwrap();
    let events = game
      .step(Command::AttackRangedChainfire(target))
      .expect("chainfire against a visible target should be accepted");

    if events.iter().any(
      |event| matches!(event, GameEvent::ActorDied { entity_id, .. } if *entity_id == target_id),
    ) {
      assert_eq!(ranged_events(&events, player_id), 4);
      let death_index = events
        .iter()
        .position(|event| matches!(event, GameEvent::ActorDied { entity_id, .. } if *entity_id == target_id))
        .unwrap();
      assert!(!events[death_index + 1..].iter().any(|event| {
        matches!(event, GameEvent::DamageApplied { target_id: event_target, .. } if *event_target == target_id)
      }));
      lethal_case = Some(seed);
      break;
    }
  }

  assert!(
    lethal_case.is_some(),
    "fixed seed window should include a lethal first-projectile Nuclear Plasma chainfire"
  );
}

#[test]
fn nuclear_plasma_chainfire_below_four_cell_cost_rejection_is_atomic() {
  let mut game = equipped_nuclear_plasma_rifle(2_641);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 3;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn nuclear_plasma_ordinary_fire_resets_chainfire_warmup() {
  let mut game = equipped_nuclear_plasma_rifle(2_642);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 0, (1, 7))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Nuclear Plasma chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second Nuclear Plasma chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third Nuclear Plasma chainfire burst");
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 24;
  game
    .step(Command::AttackRanged(target))
    .expect("ordinary fire after Nuclear Plasma chainfire");

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
      .chainfire_level,
    0
  );
}

#[test]
fn nuclear_plasma_sixth_chainfire_level_is_rejected_without_mutation() {
  let mut game = equipped_nuclear_plasma_rifle(2_643);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 0, (1, 7))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Nuclear Plasma chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second Nuclear Plasma chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third Nuclear Plasma chainfire burst");
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 24;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourth Nuclear Plasma chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("fifth Nuclear Plasma chainfire burst");
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::InvalidCommand("higher Nuclear Plasma chainfire levels are deferred".to_string())
  );
  assert_eq!(game, before);
}

#[test]
fn nuclear_plasma_second_chainfire_below_six_cell_cost_rejection_is_atomic() {
  let mut game = equipped_nuclear_plasma_rifle(2_645);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Nuclear Plasma chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 5;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn nuclear_plasma_third_chainfire_below_nine_cell_cost_rejection_is_atomic() {
  let mut game = equipped_nuclear_plasma_rifle(2_647);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Nuclear Plasma chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second Nuclear Plasma chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 8;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn nuclear_plasma_fourth_chainfire_below_nine_cell_cost_rejection_is_atomic() {
  let mut game = equipped_nuclear_plasma_rifle(2_650);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Nuclear Plasma chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second Nuclear Plasma chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third Nuclear Plasma chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 8;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn nuclear_plasma_fifth_chainfire_below_nine_cell_cost_rejection_is_atomic() {
  let mut game = equipped_nuclear_plasma_rifle(2_654);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first Nuclear Plasma chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second Nuclear Plasma chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third Nuclear Plasma chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 24;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourth Nuclear Plasma chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 8;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn nuclear_plasma_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_644, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::NuclearPlasmaRifle),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 500, 100, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) =
    ReplayEngine::run(&replay).expect("Nuclear Plasma chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 5);
  assert_eq!(weapon.weapon_properties().unwrap().chainfire_level, 3);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    19
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn nuclear_plasma_fourth_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_651, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(9)],
      equipped_weapon: Some(ItemSpawnKind::NuclearPlasmaRifle),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  for _ in 0..48 {
    replay.record_command(Command::Wait);
  }
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) =
    ReplayEngine::run(&replay).expect("Nuclear Plasma fourth chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 0);
  assert_eq!(props.chainfire_level, 4);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    28
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn nuclear_plasma_fifth_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_655, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::NuclearPlasmaRifle),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(6, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  for _ in 0..47 {
    replay.record_command(Command::Wait);
  }
  replay.record_command(Command::AttackRangedChainfire(target));
  for _ in 0..57 {
    replay.record_command(Command::Wait);
  }
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) =
    ReplayEngine::run(&replay).expect("Nuclear Plasma fifth chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 0);
  assert_eq!(props.chainfire_level, 5);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    37
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}
