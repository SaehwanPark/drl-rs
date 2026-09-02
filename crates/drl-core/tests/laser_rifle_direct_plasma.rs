use drl_core::ReplayEngine;
use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::resistance::apply_damage_resistance;
use drl_protocol::{
  AttackOutcome, Command, DamageSource, DamageType, EquipmentSlot, GameEvent, ItemSpawnKind,
  MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_laser_rifle(seed: u64) -> Game {
  let mut game = Game::new(seed, 24, 24, Position::new(3, 12)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::laser_rifle(weapon_id))
    .unwrap();
  game
}

fn configure_direct_target(game: &mut Game, target_position: Position) -> drl_protocol::EntityId {
  let target_id = game
    .world_mut()
    .spawn_monster(target_position, "Direct Target", 10_000, 0, (0, 0))
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

fn direct_volley_damage(
  events: &[GameEvent],
  target_id: drl_protocol::EntityId,
  expected_attack_count: usize,
) -> (Vec<u32>, Vec<u32>) {
  let raw = events
    .iter()
    .filter_map(|event| match event {
      GameEvent::AttackResolved {
        target_id: event_target,
        outcome: AttackOutcome::Hit { damage, .. },
        is_ranged: true,
        ..
      } if *event_target == target_id => Some(*damage),
      _ => None,
    })
    .collect::<Vec<_>>();
  let applied = events
    .iter()
    .filter_map(|event| match event {
      GameEvent::DamageApplied {
        target_id: event_target,
        amount,
        source: DamageSource::Actor(_),
        damage_type: Some(DamageType::Plasma),
        ..
      } if *event_target == target_id => Some(*amount),
      _ => None,
    })
    .collect::<Vec<_>>();
  let attack_count = events
    .iter()
    .filter(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          target_id: event_target,
          is_ranged: true,
          ..
        } if *event_target == target_id
      )
    })
    .count();
  assert_eq!(attack_count, expected_attack_count);
  assert!(
    !raw.is_empty(),
    "fixed seed should include a successful hit"
  );
  assert_eq!(
    applied.len(),
    raw.len(),
    "each successful Laser Rifle direct hit must emit one typed Plasma event"
  );
  (raw, applied)
}

#[test]
fn laser_rifle_direct_volley_is_typed_plasma_and_blue_armor_mitigates() {
  let seed = 45_003;
  let target_position = Position::new(7, 12);
  let mut plain = equipped_laser_rifle(seed);
  let mut armored = equipped_laser_rifle(seed);
  let plain_target_id = configure_direct_target(&mut plain, target_position);
  let armored_target_id = configure_direct_target(&mut armored, target_position);
  let armor_id = armored.world_mut().allocate_item_id();
  armored
    .world_mut()
    .get_actor_mut(armored_target_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::blue_armor(armor_id))
    .unwrap();

  let plain_events = plain
    .step(Command::AttackRanged(target_position))
    .expect("unarmored Laser Rifle fire should resolve");
  let armored_events = armored
    .step(Command::AttackRanged(target_position))
    .expect("Blue Armor Laser Rifle fire should resolve");
  let (raw_damage, plain_damage) = direct_volley_damage(&plain_events, plain_target_id, 5);
  let (armored_raw_damage, armored_damage) =
    direct_volley_damage(&armored_events, armored_target_id, 5);

  assert_eq!(armored_raw_damage, raw_damage);
  assert_eq!(plain_damage, raw_damage);
  let expected_armored = raw_damage
    .iter()
    .map(|damage| {
      apply_damage_resistance(*damage, 20)
        .saturating_sub(2)
        .max(1)
    })
    .collect::<Vec<_>>();
  assert_eq!(armored_damage, expected_armored);
  assert!(
    armored_damage
      .iter()
      .zip(plain_damage.iter())
      .all(|(armored, plain)| armored <= plain)
  );
  assert!(
    armored_damage
      .iter()
      .zip(plain_damage.iter())
      .any(|(armored, plain)| armored < plain)
  );
  assert_eq!(plain.rng(), armored.rng());
}

#[test]
fn laser_rifle_chainfire_direct_hits_are_typed_plasma_at_first_and_seventh_levels() {
  let target_position = Position::new(7, 12);
  let mut first = equipped_laser_rifle(45_004);
  let first_target_id = configure_direct_target(&mut first, target_position);
  let first_events = first
    .step(Command::AttackRangedChainfire(target_position))
    .expect("first Laser Rifle chainfire burst should resolve");
  let (_, first_applied) = direct_volley_damage(&first_events, first_target_id, 4);
  assert_eq!(first_applied.len(), 4);

  let mut seventh = equipped_laser_rifle(45_005);
  let seventh_target_id = configure_direct_target(&mut seventh, target_position);
  for _ in 0..6 {
    seventh
      .step(Command::AttackRangedChainfire(target_position))
      .expect("Laser Rifle chainfire warm-up burst should resolve");
  }
  let player_id = seventh.world().player_id().unwrap();
  seventh
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 7;
  let seventh_events = seventh
    .step(Command::AttackRangedChainfire(target_position))
    .expect("seventh Laser Rifle chainfire burst should resolve");
  let (_, seventh_applied) = direct_volley_damage(&seventh_events, seventh_target_id, 7);
  assert_eq!(seventh_applied.len(), 7);
  let weapon = seventh
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap();
  assert_eq!(weapon.weapon_properties().unwrap().chainfire_level, 7);
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 0);
}

#[test]
fn laser_rifle_direct_plasma_replays_deterministically() {
  let seed = 45_006;
  let player_position = Position::new(2, 6);
  let target_position = Position::new(6, 6);
  let mut replay =
    ReplayLog::new(seed, 12, 12, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::LaserRifle),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Direct Target",
    10_000,
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
  let target_id = first_game
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("replay target should exist")
    .id();
  let (_, applied) = direct_volley_damage(&first_events, target_id, 5);
  assert!(!applied.is_empty());

  let mut stale = replay;
  stale.metadata.gameplay_semantics_version = 140;
  let error = ReplayEngine::validate(&stale).expect_err("stale direct-Plasma replay must reject");
  assert!(error.contains("unsupported gameplay semantics version"));
}
