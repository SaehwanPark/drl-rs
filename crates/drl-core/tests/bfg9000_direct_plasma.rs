use drl_core::ReplayEngine;
use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::resistance::apply_damage_resistance;
use drl_protocol::{
  AttackOutcome, Command, DamageSource, DamageType, EquipmentSlot, GameEvent, ItemSpawnKind,
  MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_bfg9000(seed: u64) -> Game {
  let mut game = Game::new(seed, 24, 24, Position::new(3, 12)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::bfg9000(weapon_id))
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
    .expect("Standard BFG 9000 direct hit should resolve");
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
    .expect("Standard BFG 9000 direct damage should be typed Plasma");
  (raw, applied)
}

#[test]
fn bfg9000_direct_hit_is_typed_plasma_and_blue_armor_mitigates() {
  let seed = 41_003;
  let target_position = Position::new(7, 12);
  let mut plain = equipped_bfg9000(seed);
  let mut armored = equipped_bfg9000(seed);
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
    .expect("unarmored Standard BFG 9000 fire should resolve");
  let armored_events = armored
    .step(Command::AttackRanged(target_position))
    .expect("Blue Armor Standard BFG 9000 fire should resolve");
  let (raw_damage, plain_damage) = direct_attack_damage(&plain_events, plain_target_id);
  let (armored_raw_damage, armored_damage) =
    direct_attack_damage(&armored_events, armored_target_id);

  assert_eq!(armored_raw_damage, raw_damage);
  assert_eq!(plain_damage, raw_damage);
  assert_eq!(
    armored_damage,
    apply_damage_resistance(raw_damage, 20)
      .saturating_sub(2)
      .max(1)
  );
  assert!(armored_damage < plain_damage);
  assert_eq!(plain.rng(), armored.rng());
}

#[test]
fn bfg9000_direct_plasma_replays_deterministically() {
  let seed = 41_004;
  let player_position = Position::new(2, 6);
  let target_position = Position::new(6, 6);
  let mut replay =
    ReplayLog::new(seed, 12, 12, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Bfg9000),
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
        damage_type: Some(DamageType::Plasma),
        ..
      }
    )
  }));

  let mut stale = replay;
  stale.metadata.gameplay_semantics_version = 135;
  let error = ReplayEngine::validate(&stale).expect_err("stale direct-Plasma replay must reject");
  assert!(error.contains("unsupported gameplay semantics version"));
}
