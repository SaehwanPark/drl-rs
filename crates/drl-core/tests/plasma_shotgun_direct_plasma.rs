use drl_core::ReplayEngine;
use drl_core::game::Game;
use drl_core::grid::Tile;
use drl_core::item::Item;
use drl_core::resistance::apply_damage_resistance;
use drl_protocol::{
  AttackOutcome, Command, CommandError, DamageSource, DamageType, EquipmentSlot, GameEvent,
  ItemSpawnKind, MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_plasma_shotgun(seed: u64) -> Game {
  let mut game = Game::new(seed, 16, 12, Position::new(2, 6)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::plasma_shotgun(weapon_id))
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
    .expect("fixed seed should produce a successful Plasma Shotgun hit");
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
    .expect("successful Plasma Shotgun hit should emit typed Plasma damage");
  (raw, applied)
}

#[test]
fn plasma_shotgun_direct_shot_is_typed_plasma_and_blue_armor_mitigates() {
  let seed = 46_005;
  let target_position = Position::new(7, 6);
  let mut plain = equipped_plasma_shotgun(seed);
  let mut armored = equipped_plasma_shotgun(seed);
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
    .expect("unarmored Plasma Shotgun fire should resolve");
  let armored_events = armored
    .step(Command::AttackRanged(target_position))
    .expect("Blue Armor Plasma Shotgun fire should resolve");
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
  assert!(armored_damage < plain_damage);
  assert_eq!(
    plain
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    27
  );
  assert_eq!(plain.rng(), armored.rng());
}

#[test]
fn plasma_shotgun_direct_replay_is_deterministic_and_rejects_stale_semantics() {
  let target_position = Position::new(6, 6);
  let mut replay =
    ReplayLog::new(46_006, 12, 12, Position::new(2, 6)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::PlasmaShotgun),
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
  assert_eq!(
    first_game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .current_clip,
    27
  );

  let mut stale = replay;
  stale.metadata.gameplay_semantics_version = 144;
  let error = ReplayEngine::validate(&stale).expect_err("stale Plasma Shotgun replay must reject");
  assert!(error.contains("unsupported gameplay semantics version"));
}

#[test]
fn plasma_shotgun_direct_rejections_preserve_exact_game_state() {
  let target_position = Position::new(7, 6);
  let mut invalid_target = equipped_plasma_shotgun(46_007);
  let before_invalid_target = invalid_target.clone();
  assert_eq!(
    invalid_target.step(Command::AttackRanged(target_position)),
    Err(CommandError::InvalidTarget(target_position))
  );
  assert_eq!(invalid_target, before_invalid_target);

  let mut under_supplied = equipped_plasma_shotgun(46_008);
  let target_id = configure_direct_target(&mut under_supplied, target_position);
  let player_id = under_supplied.world().player_id().unwrap();
  under_supplied
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 2;
  let before_under_supplied = under_supplied.clone();
  assert_eq!(
    under_supplied.step(Command::AttackRanged(target_position)),
    Err(CommandError::NoAmmoInClip)
  );
  assert_eq!(under_supplied, before_under_supplied);
  assert!(
    under_supplied
      .world()
      .get_actor(target_id)
      .unwrap()
      .is_alive()
  );

  let mut blocked = equipped_plasma_shotgun(46_009);
  configure_direct_target(&mut blocked, target_position);
  blocked
    .world_mut()
    .map_mut()
    .set_tile(Position::new(4, 6), Tile::Wall);
  let before_blocked = blocked.clone();
  assert_eq!(
    blocked.step(Command::AttackRanged(target_position)),
    Err(CommandError::LineOfSightBlocked(target_position))
  );
  assert_eq!(blocked, before_blocked);
}
