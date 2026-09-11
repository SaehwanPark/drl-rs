use drl_core::ReplayEngine;
use drl_core::game::Game;
use drl_core::grid::Tile;
use drl_core::item::Item;
use drl_core::resistance::apply_damage_resistance;
use drl_protocol::{
  AttackOutcome, Command, CommandError, DamageSource, DamageType, EquipmentSlot, GameEvent,
  ItemSpawnKind, MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_missile_launcher(seed: u64) -> Game {
  let mut game = Game::new(seed, 16, 12, Position::new(2, 6)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::missile_launcher(weapon_id))
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
    .expect("fixed seed should produce a successful Missile Launcher hit");
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
    .expect("successful Missile Launcher hit must emit typed Fire damage");
  (raw, applied)
}

#[test]
fn missile_launcher_direct_shot_is_typed_fire_and_red_armor_mitigates() {
  let seed = 46_100;
  let target_position = Position::new(7, 6);
  let mut plain = equipped_missile_launcher(seed);
  let mut red_armored = equipped_missile_launcher(seed);
  let mut blue_armored = equipped_missile_launcher(seed);

  let plain_target_id = configure_direct_target(&mut plain, target_position);
  let red_target_id = configure_direct_target(&mut red_armored, target_position);
  let blue_target_id = configure_direct_target(&mut blue_armored, target_position);

  let red_armor_id = red_armored.world_mut().allocate_item_id();
  red_armored
    .world_mut()
    .get_actor_mut(red_target_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::red_armor(red_armor_id))
    .unwrap();

  let blue_armor_id = blue_armored.world_mut().allocate_item_id();
  blue_armored
    .world_mut()
    .get_actor_mut(blue_target_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::blue_armor(blue_armor_id))
    .unwrap();

  let plain_events = plain
    .step(Command::AttackRanged(target_position))
    .expect("plain shot");
  let red_events = red_armored
    .step(Command::AttackRanged(target_position))
    .expect("red armored shot");
  let blue_events = blue_armored
    .step(Command::AttackRanged(target_position))
    .expect("blue armored shot");

  let (plain_raw, plain_applied) = direct_hit(&plain_events, plain_target_id);
  let (red_raw, red_applied) = direct_hit(&red_events, red_target_id);
  let (blue_raw, blue_applied) = direct_hit(&blue_events, blue_target_id);

  assert_eq!(
    plain_raw, red_raw,
    "same seed must produce identical raw damage rolls"
  );
  assert_eq!(
    plain_raw, blue_raw,
    "same seed must produce identical raw damage rolls"
  );
  assert_eq!(
    plain_applied,
    plain_raw.max(1),
    "unarmored target takes raw damage"
  );

  // Red Armor: 25% Fire resistance before 4 flat protection.
  let expected_red = apply_damage_resistance(plain_raw, 25)
    .saturating_sub(4)
    .max(1);
  assert_eq!(
    red_applied, expected_red,
    "Red Armor must apply 25% Fire resistance before 4 flat protection"
  );

  // Blue Armor: 0% Fire resistance, 2 flat protection.
  let expected_blue = plain_raw.saturating_sub(2).max(1);
  assert_eq!(
    blue_applied, expected_blue,
    "Blue Armor has 0% Fire resistance and must apply only flat protection"
  );

  // Armored targets take less damage than unarmored target.
  assert!(red_applied < plain_applied);

  // Clips decrement by 1 (4 -> 3).
  for game in [&plain, &red_armored, &blue_armored] {
    let weapon = game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap();
    assert_eq!(weapon.current_clip, 3);
  }

  // Final RNG state matches between unarmored and armored runs.
  assert_eq!(plain.rng().state(), red_armored.rng().state());
  assert_eq!(plain.rng().state(), blue_armored.rng().state());
}

#[test]
fn missile_launcher_multi_round_clip_depletion_and_rejection_atomicity() {
  let seed = 46_101;
  let target_position = Position::new(6, 6);
  let mut game = equipped_missile_launcher(seed);
  // Keep the repeated-shot target pinned in place while still exercising the
  // real radius-three splash/knockback path.
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(7, 6), Tile::Wall);
  let target_id = configure_direct_target(&mut game, target_position);

  // Initial clip is 4.
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
      .current_clip,
    4
  );

  // Fire 4 times to empty the clip: 4 -> 3 -> 2 -> 1 -> 0.
  for expected_remaining in [3, 2, 1, 0] {
    let events = game
      .step(Command::AttackRanged(target_position))
      .expect("shot with ammo in clip");
    let (_, applied) = direct_hit(&events, target_id);
    assert!(applied > 0);
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
        .current_clip,
      expected_remaining
    );
  }

  // 5th shot must reject atomically because clip is empty.
  let before_empty = game.clone();
  let err = game
    .step(Command::AttackRanged(target_position))
    .unwrap_err();
  assert_eq!(err, CommandError::NoAmmoInClip);
  assert_eq!(
    game, before_empty,
    "empty clip rejection must preserve exact Game state"
  );
}

#[test]
fn missile_launcher_single_reload_integration() {
  let seed = 46_102;
  let target_position = Position::new(5, 6);
  let mut game = equipped_missile_launcher(seed);
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(6, 6), Tile::Wall);
  let target_id = configure_direct_target(&mut game, target_position);

  // Deplete all 4 rockets.
  for _ in 0..4 {
    game
      .step(Command::AttackRanged(target_position))
      .expect("depletion shot");
  }

  // Give player 2 loose rockets in inventory.
  let ammo_id = game.world_mut().allocate_item_id();
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::ammo_rockets(ammo_id, 2))
    .unwrap();

  // Ordinary reload loads 1 rocket (IF_SINGLERELOAD policy).
  let reload_events = game.step(Command::Reload).expect("single rocket reload");
  assert!(reload_events.iter().any(|e| matches!(
    e,
    GameEvent::WeaponReloaded {
      ammo_loaded: 1,
      current_clip: 1,
      ..
    }
  )));
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
      .current_clip,
    1
  );
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .total_ammo(drl_protocol::AmmoType::Rocket),
    1
  );

  // Subsequent attack succeeds and deals typed Fire damage.
  let shot_events = game
    .step(Command::AttackRanged(target_position))
    .expect("shot after reload");
  let (_, applied) = direct_hit(&shot_events, target_id);
  assert!(applied > 0);
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
      .current_clip,
    0
  );
}

#[test]
fn missile_launcher_rejection_paths_preserve_exact_game_identity() {
  let seed = 46_103;
  let target_position = Position::new(7, 6);
  let mut game = equipped_missile_launcher(seed);
  let _ = configure_direct_target(&mut game, target_position);

  // 1. Target out of range (> 8).
  let out_of_range_pos = Position::new(14, 6);
  game
    .world_mut()
    .spawn_monster(out_of_range_pos, "Far Target", 100, 0, (0, 0))
    .unwrap();
  let before = game.clone();
  let err = game
    .step(Command::AttackRanged(out_of_range_pos))
    .unwrap_err();
  assert_eq!(err, CommandError::TargetOutOfRange(out_of_range_pos));
  assert_eq!(game, before, "out-of-range rejection must preserve Game");

  // 2. Blocked line-of-sight (place wall between player and target).
  let mut wall_game = game.clone();
  wall_game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(4, 6), Tile::Wall);
  let wall_before = wall_game.clone();
  let err = wall_game
    .step(Command::AttackRanged(target_position))
    .unwrap_err();
  assert_eq!(err, CommandError::LineOfSightBlocked(target_position));
  assert_eq!(
    wall_game, wall_before,
    "blocked LOS rejection must preserve Game"
  );

  // 3. No target at position.
  let empty_pos = Position::new(3, 6);
  let before_empty = game.clone();
  let err = game.step(Command::AttackRanged(empty_pos)).unwrap_err();
  assert_eq!(err, CommandError::InvalidTarget(empty_pos));
  assert_eq!(game, before_empty, "no-target rejection must preserve Game");
}

#[test]
fn missile_launcher_replay_determinism_and_stale_semantics_rejection() {
  let player_position = Position::new(2, 2);
  // The east arena boundary pins the repeated target after splash knockback.
  let target_position = Position::new(10, 2);
  let player_config = PlayerSpawnConfig {
    hp: 100,
    max_hp: 100,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoRockets(4)],
    equipped_weapon: Some(ItemSpawnKind::MissileLauncher),
    equipped_armor: Some(ItemSpawnKind::RedArmor),
    equipped_armor_durability: None,
  };
  let mut replay =
    ReplayLog::new(46_104, 12, 12, player_position).with_player_config(player_config);
  replay.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Replay Target",
    10_000,
    0,
    (0, 0),
  ));

  let (mut live_game, _) = ReplayEngine::run(&replay).expect("setup replay");
  let commands = vec![
    Command::AttackRanged(target_position),
    Command::AttackRanged(target_position),
    Command::Reload,
    Command::AttackRanged(target_position),
  ];

  let mut live_events = Vec::new();
  for cmd in &commands {
    live_events.extend(live_game.step(*cmd).expect("live step"));
    replay.record_command(*cmd);
  }

  // Replay verification reproduces exact state and events.
  let (replayed_game, replay_events) = ReplayEngine::run(&replay).expect("command log replay");
  assert_eq!(
    replayed_game, live_game,
    "replayed game must match live game exactly"
  );
  assert_eq!(
    replay_events, live_events,
    "replayed events must match live events exactly"
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());

  // Stale semantics 149 (before the radius-three fanout) is rejected.
  let mut stale_replay = replay;
  stale_replay.metadata.gameplay_semantics_version = 149;
  let err = ReplayEngine::validate(&stale_replay).unwrap_err();
  assert!(err.contains("unsupported gameplay semantics version"));
}
