use drl_core::ReplayEngine;
use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::resistance::apply_damage_resistance;
use drl_protocol::{
  AttackOutcome, Command, DamageSource, DamageType, EquipmentSlot, GameEvent, ItemSpawnKind,
  MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_blaster(seed: u64) -> Game {
  let mut game = Game::new(seed, 16, 12, Position::new(2, 6)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::blaster(weapon_id))
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

fn direct_hit(events: &[GameEvent], target_id: drl_protocol::EntityId) -> (u32, u32) {
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
    .expect("fixed seed should produce a successful Blaster hit");
  let applied = events
    .iter()
    .find_map(|event| match event {
      GameEvent::DamageApplied {
        target_id: event_target,
        amount,
        source: DamageSource::Actor(_),
        damage_type: Some(DamageType::Plasma),
        ..
      } if *event_target == target_id => Some(*amount),
      _ => None,
    })
    .expect("successful Blaster hit should emit typed Plasma damage");
  (raw, applied)
}

#[test]
fn blaster_direct_shot_is_typed_plasma_and_blue_armor_mitigates() {
  let seed = 46_001;
  let target_position = Position::new(7, 6);
  let mut plain = equipped_blaster(seed);
  let mut armored = equipped_blaster(seed);
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
    .expect("unarmored Blaster fire should resolve");
  let armored_events = armored
    .step(Command::AttackRanged(target_position))
    .expect("Blue Armor Blaster fire should resolve");
  let (raw_damage, plain_damage) = direct_hit(&plain_events, plain_target_id);
  let (armored_raw_damage, armored_damage) = direct_hit(&armored_events, armored_target_id);

  assert_eq!(armored_raw_damage, raw_damage);
  assert_eq!(plain_damage, raw_damage);
  assert_eq!(
    armored_damage,
    apply_damage_resistance(raw_damage, 20)
      .saturating_sub(2)
      .max(1)
  );
  assert!(armored_damage <= plain_damage);
  assert_eq!(plain.rng(), armored.rng());
}

#[test]
fn blaster_aimed_direct_shot_is_typed_plasma_and_replay_is_deterministic() {
  let seed = 46_002;
  let player_position = Position::new(2, 6);
  let target_position = Position::new(6, 6);
  let mut replay =
    ReplayLog::new(seed, 12, 12, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Blaster),
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
  replay.record_command(Command::AttackRangedAimed(target_position));

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (first_game, first_events) = ReplayEngine::run(&replay).unwrap();
  let (second_game, second_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(first_game, second_game);
  assert_eq!(first_events, second_events);
  let target_id = first_game
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .unwrap()
    .id();
  let (raw, applied) = direct_hit(&first_events, target_id);
  assert_eq!(raw, applied);
  assert!(first_events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActionCostPaid {
        cost: drl_protocol::ActionCost(2_000),
        ..
      }
    )
  }));

  let mut stale = replay;
  stale.metadata.gameplay_semantics_version = 141;
  let error = ReplayEngine::validate(&stale).expect_err("stale Blaster replay must reject");
  assert!(error.contains("unsupported gameplay semantics version"));
}
