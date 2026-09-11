//! Direct Fire classification coverage for Revenant's Launcher.
//!
//! Rejection-path atomicity (out-of-range, blocked LOS, no-target, empty
//! clip) is covered by `special_items.rs`; this file covers the typed Fire
//! damage classification, the one-rocket clip with ordinary reload, and
//! replay determinism for the new semantics.

use drl_core::ReplayEngine;
use drl_core::game::Game;
use drl_core::grid::Tile;
use drl_core::item::Item;
use drl_core::resistance::apply_damage_resistance;
use drl_protocol::{
  AttackOutcome, Command, DamageSource, DamageType, EquipmentSlot, GameEvent, ItemSpawnKind,
  MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_revenants_launcher(seed: u64) -> Game {
  let mut game = Game::new(seed, 16, 12, Position::new(2, 6)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::revenants_launcher(weapon_id))
    .unwrap();
  game
}

fn configure_direct_target(game: &mut Game, target_position: Position) -> drl_protocol::EntityId {
  game
    .world_mut()
    .spawn_monster(target_position, "Direct Target", 10_000, 0, (0, 0))
    .unwrap()
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
    .expect("exact-hit Revenant's Launcher must always land a successful hit");
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
    .expect("successful Revenant's Launcher hit must emit typed Fire damage");
  (raw, applied)
}

#[test]
fn revenants_launcher_direct_hit_is_typed_fire_and_armor_resists() {
  let seed = 46_201;
  let target_position = Position::new(7, 6);
  let mut plain = equipped_revenants_launcher(seed);
  let mut red_armored = equipped_revenants_launcher(seed);
  let mut blue_armored = equipped_revenants_launcher(seed);

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

  // One-rocket clip decrements by 1 (1 -> 0).
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
    assert_eq!(weapon.current_clip, 0);
  }

  // Final RNG state matches between unarmored and armored runs.
  assert_eq!(plain.rng().state(), red_armored.rng().state());
  assert_eq!(plain.rng().state(), blue_armored.rng().state());
}

#[test]
fn revenants_launcher_single_shot_clip_and_reload_preserve_fire() {
  let seed = 46_202;
  // Keep the follow-up target against the arena's east wall so the new
  // radius-3 center knockback cannot move it between the two shots.
  let target_position = Position::new(10, 6);
  let mut game = equipped_revenants_launcher(seed);
  let target_id = configure_direct_target(&mut game, target_position);
  // Bound center knockback for the two-shot reload fixture without changing
  // the target's visibility or direct Fire assertions.
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(11, 6), Tile::Wall);

  // Initial clip is 1.
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

  // Single shot empties the one-rocket clip and deals typed Fire damage.
  let shot_events = game
    .step(Command::AttackRanged(target_position))
    .expect("first shot");
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

  // Ordinary reload loads 1 rocket into the one-rocket clip.
  let reload_events = game.step(Command::Reload).expect("ordinary reload");
  assert!(reload_events.iter().any(|e| matches!(
    e,
    GameEvent::WeaponReloaded {
      ammo_loaded: 1,
      current_clip: 1,
      max_clip: 1,
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

  // Subsequent attack succeeds and deals typed Fire damage again.
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
fn revenants_launcher_direct_fire_replay_is_deterministic() {
  let player_position = Position::new(2, 2);
  // The east-wall target remains addressable after each radius-3 center
  // splash, allowing this legacy direct-fire replay fixture to retain its
  // repeated command sequence.
  let target_position = Position::new(10, 2);
  let player_config = PlayerSpawnConfig {
    hp: 100,
    max_hp: 100,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoRockets(2)],
    equipped_weapon: Some(ItemSpawnKind::RevenantsLauncher),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let mut replay =
    ReplayLog::new(46_203, 12, 12, player_position).with_player_config(player_config);
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
    Command::Reload,
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

  // Stale semantics 147 is rejected.
  let mut stale_replay = replay;
  stale_replay.metadata.gameplay_semantics_version = 147;
  let err = ReplayEngine::validate(&stale_replay).unwrap_err();
  assert!(err.contains("unsupported gameplay semantics version"));
}
