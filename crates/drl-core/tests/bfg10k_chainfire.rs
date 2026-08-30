use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::replay::ReplayEngine;
use drl_protocol::{
  AttackOutcome, Command, CommandError, EquipmentSlot, GameEvent, ItemSpawnKind, MonsterSpawnSpec,
  PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_bfg10k(seed: u64) -> Game {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::bfg10k(weapon_id))
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

fn bfg10k_schedules(events: &[GameEvent], target_id: drl_protocol::EntityId) -> usize {
  events
    .iter()
    .filter(|event| {
      matches!(
        event,
        GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      )
    })
    .count()
}

#[test]
fn bfg10k_first_chainfire_emits_four_exact_hits_and_schedules_explosions() {
  let mut game = equipped_bfg10k(2_700);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");

  assert_eq!(ranged_events(&events, player_id), 4);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          target_id: event_target,
          outcome: AttackOutcome::Hit { .. },
          is_ranged: true,
          ..
        } if *event_target == target_id
      ))
      .count(),
    4,
    "BFG 10K exact-hit chainfire should resolve four hits"
  );
  assert_eq!(bfg10k_schedules(&events, target_id), 4);

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 30);
  assert_eq!(props.chainfire_level, 1);
}

#[test]
fn bfg10k_second_chainfire_emits_five_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_708);
  let initial_target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(initial_target, "Static Target", 10_000, 100, (1, 7))
    .unwrap();

  game
    .step(Command::AttackRangedChainfire(initial_target))
    .expect("first BFG 10K chainfire burst should be accepted");
  let second_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the first burst")
    .position();
  let player_id = game.world().player_id().unwrap();
  let events = game
    .step(Command::AttackRangedChainfire(second_target))
    .expect("second BFG 10K chainfire burst should be accepted");

  assert_eq!(ranged_events(&events, player_id), 5);
  assert_eq!(bfg10k_schedules(&events, target_id), 5);
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 5);
  assert_eq!(props.chainfire_level, 2);
}

#[test]
fn bfg10k_third_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_709);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  let second_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the first burst")
    .position();
  game
    .step(Command::AttackRangedChainfire(second_target))
    .expect("second BFG 10K chainfire burst should be accepted");
  let third_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the second burst")
    .position();
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
    .current_clip = 35;

  let events = game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("third BFG 10K chainfire burst should be accepted");

  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 0);
  assert_eq!(props.chainfire_level, 3);
}

#[test]
fn bfg10k_fourth_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_711);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourth BFG 10K chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 4);
  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
}

#[test]
fn bfg10k_chainfire_keeps_four_outcomes_after_lethal_target() {
  let mut game = equipped_bfg10k(2_701);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Fragile Target", 1, 100, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("BFG 10K chainfire against a visible target should be accepted");

  assert_eq!(ranged_events(&events, player_id), 4);
  assert_eq!(bfg10k_schedules(&events, target_id), 1);
  let death_index = events
    .iter()
    .position(
      |event| matches!(event, GameEvent::ActorDied { entity_id, .. } if *entity_id == target_id),
    )
    .expect("the exact-hit first projectile should kill the fragile target");
  assert!(!events[death_index + 1..].iter().any(|event| {
    matches!(
      event,
      GameEvent::DamageApplied {
        target_id: event_target,
        ..
      } if *event_target == target_id
    )
  }));
}

#[test]
fn bfg10k_chainfire_below_twenty_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_702);
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
    .current_clip = 19;
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
fn bfg10k_ordinary_fire_resets_chainfire_warmup() {
  let mut game = equipped_bfg10k(2_703);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 0, (1, 7))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
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
    .current_clip = 50;
  game
    .step(Command::AttackRanged(
      game.world().get_actor(target_id).unwrap().position(),
    ))
    .expect("ordinary fire after BFG 10K chainfire");

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
fn bfg10k_fifth_chainfire_level_is_rejected_without_mutation() {
  let mut game = equipped_bfg10k(2_704);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  let second_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the first burst")
    .position();
  game
    .step(Command::AttackRangedChainfire(second_target))
    .expect("second BFG 10K chainfire burst");
  let third_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the second burst")
    .position();
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
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("third BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("fourth BFG 10K chainfire burst");
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(
        game.world().get_actor(target_id).unwrap().position(),
      ))
      .unwrap_err(),
    CommandError::InvalidCommand("higher BFG 10K chainfire levels are deferred".to_string())
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_third_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_710);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  let second_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the first burst")
    .position();
  game
    .step(Command::AttackRangedChainfire(second_target))
    .expect("second BFG 10K chainfire burst");
  let third_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the second burst")
    .position();
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
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(third_target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_fourth_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_712);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst");
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
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(
        game.world().get_actor(target_id).unwrap().position(),
      ))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_705, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(45)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) = ReplayEngine::run(&replay).expect("BFG 10K chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 15);
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
    16
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    16
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_fourth_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_713, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(150)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) =
    ReplayEngine::run(&replay).expect("BFG 10K fourth chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
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
    23
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    23
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}
