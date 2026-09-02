use drl_core::ReplayEngine;
use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::resistance::apply_damage_resistance;
use drl_protocol::{
  AttackOutcome, Command, DamageSource, DamageType, EquipmentSlot, GameEvent, ItemSpawnKind,
  MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_rocket_launcher(seed: u64) -> Game {
  let mut game = Game::new(seed, 20, 20, Position::new(3, 10)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::rocket_launcher(weapon_id))
    .unwrap();
  game
}

fn configure_direct_target(game: &mut Game, target_position: Position) -> drl_protocol::EntityId {
  let target_id = game
    .world_mut()
    .spawn_monster(target_position, "Direct Target", 500, 0, (0, 0))
    .unwrap();
  game
    .world_mut()
    .player_mut()
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .accuracy = 100;
  target_id
}

fn direct_attack_damage(events: &[GameEvent], target_id: drl_protocol::EntityId) -> (u32, u32) {
  let raw = events
    .iter()
    .find_map(|event| match event {
      GameEvent::AttackResolved {
        target_id: event_target,
        outcome: AttackOutcome::Hit { damage, .. },
        is_ranged: true,
        ..
      } if *event_target == target_id => Some(*damage),
      _ => None,
    })
    .expect("Rocket Launcher direct hit should resolve");
  let applied = events
    .iter()
    .find_map(|event| match event {
      GameEvent::DamageApplied {
        target_id: event_target,
        amount,
        source: DamageSource::Actor(_),
        damage_type: Some(DamageType::Fire),
        ..
      } if *event_target == target_id => Some(*amount),
      _ => None,
    })
    .expect("Rocket Launcher direct damage should be typed Fire");
  (raw, applied)
}

#[test]
fn rocket_launcher_direct_hit_is_typed_fire_and_red_armor_mitigates() {
  let seed = 40_003;
  let target_position = Position::new(7, 10);
  let mut plain = equipped_rocket_launcher(seed);
  let mut armored = equipped_rocket_launcher(seed);
  let plain_target_id = configure_direct_target(&mut plain, target_position);
  let armored_target_id = configure_direct_target(&mut armored, target_position);
  let armor_id = armored.world_mut().allocate_item_id();
  armored
    .world_mut()
    .get_actor_mut(armored_target_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::red_armor(armor_id))
    .unwrap();

  let plain_events = plain
    .step(Command::AttackRanged(target_position))
    .expect("unarmored Rocket Launcher fire should resolve");
  let armored_events = armored
    .step(Command::AttackRanged(target_position))
    .expect("Red Armor Rocket Launcher fire should resolve");
  let (raw_damage, plain_damage) = direct_attack_damage(&plain_events, plain_target_id);
  let (armored_raw_damage, armored_damage) =
    direct_attack_damage(&armored_events, armored_target_id);

  assert_eq!(armored_raw_damage, raw_damage);
  assert_eq!(plain_damage, raw_damage);
  assert_eq!(
    armored_damage,
    apply_damage_resistance(raw_damage, 25)
      .saturating_sub(4)
      .max(1)
  );
  assert!(armored_damage < plain_damage);
  assert_eq!(plain.rng(), armored.rng());
}

#[test]
fn rocket_launcher_direct_fire_replays_typed_event_deterministically() {
  let seed = 40_004;
  let player_position = Position::new(2, 6);
  let target_position = Position::new(6, 6);
  let mut replay =
    ReplayLog::new(seed, 12, 12, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::RocketLauncher),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Direct Target",
    500,
    0,
    (0, 0),
  ));
  replay.record_command(Command::AttackRanged(target_position));

  let (first_game, first_events) = ReplayEngine::run(&replay).expect("direct replay should run");
  let (second_game, second_events) =
    ReplayEngine::run(&replay).expect("direct replay should repeat");
  assert_eq!(first_game, second_game);
  assert_eq!(first_events, second_events);
  assert!(ReplayEngine::verify_determinism(&replay).expect("replay should be deterministic"));
  assert!(first_events.iter().any(|event| {
    matches!(
      event,
      GameEvent::DamageApplied {
        source: DamageSource::Actor(_),
        damage_type: Some(DamageType::Fire),
        ..
      }
    )
  }));

  let mut stale = replay;
  stale.metadata.gameplay_semantics_version = 134;
  let error = ReplayEngine::validate(&stale).expect_err("stale direct-fire replay must reject");
  assert!(error.contains("unsupported gameplay semantics version"));
}
