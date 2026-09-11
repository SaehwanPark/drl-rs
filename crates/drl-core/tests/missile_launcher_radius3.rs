//! Focused integration coverage for Missile Launcher's bounded radius-three
//! Fire fanout.
//!
//! The direct clip/reload profile is covered by
//! `missile_launcher_direct_fire.rs`. This fixture pins the follow-on splash
//! contract: deterministic 6d6 rolls in center-first radius-three order,
//! strict ground-item thresholding, schedule/event ordering, death-drop
//! preflight atomicity, replay semantics, and scenario-runner parity.

use drl_core::game::Game;
use drl_core::grid::Tile;
use drl_core::item::Item;
use drl_core::missile_launcher::{
  MISSILE_LAUNCHER_EXPLOSION_DAMAGE_DICE, MISSILE_LAUNCHER_EXPLOSION_DAMAGE_DIE_SIDES,
  MISSILE_LAUNCHER_EXPLOSION_DELAY, MISSILE_LAUNCHER_EXPLOSION_KNOCKBACK,
  MISSILE_LAUNCHER_EXPLOSION_RADIUS, MISSILE_LAUNCHER_GROUND_ITEM_DESTRUCTION_THRESHOLD,
  apply_distance_falloff, radius_three_blast_positions, roll_explosion_damage,
  should_destroy_ground_item,
};
use drl_core::replay::ReplayEngine;
use drl_core::rng::GameRng;
use drl_core::scenario::{Scenario, ScenarioRunner};
use drl_protocol::{
  AttackOutcome, CURRENT_GAMEPLAY_SEMANTICS_VERSION, Command, CommandError, DamageSource,
  DamageType, EquipmentSlot, GameEvent, ItemSpawnKind, MonsterSpawnSpec, PlayerSpawnConfig,
  Position, ReplayLog,
};

fn equipped_missile_launcher(seed: u64) -> Game {
  let mut game = Game::new(seed, 20, 20, Position::new(4, 10)).unwrap();
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

fn configure_accuracy(game: &mut Game, accuracy: i32) {
  game
    .world_mut()
    .player_mut()
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .accuracy = accuracy;
}

fn center_splash_roll(seed: u64) -> Option<u32> {
  let mut rng = GameRng::from_seed(seed);
  // At distance three, accuracy 100 is clamped to an effective 95% hit
  // chance. The direct 6d6 roll follows the hit sample.
  if rng.gen_range(0..100) >= 95 {
    return None;
  }
  let _direct_damage = rng.gen_range(6..37);
  Some(roll_explosion_damage(&mut rng))
}

fn seed_with_center_roll(expected: u32) -> u64 {
  (0..100_000)
    .find(|seed| center_splash_roll(*seed) == Some(expected))
    .expect("the deterministic seed search must find a matching 6d6 roll")
}

fn seed_with_hit() -> u64 {
  (0..100_000)
    .find(|seed| {
      let mut rng = GameRng::from_seed(*seed);
      rng.gen_range(0..100) < 95
    })
    .expect("the deterministic seed search must find a hit")
}

fn seed_with_miss() -> u64 {
  (0..100_000)
    .find(|seed| {
      let mut rng = GameRng::from_seed(*seed);
      rng.gen_range(0..100) >= 95
    })
    .expect("the deterministic seed search must find a miss")
}

#[test]
fn missile_launcher_hit_schedules_and_fans_out_radius_three_in_order() {
  assert_eq!(MISSILE_LAUNCHER_EXPLOSION_DAMAGE_DICE, 6);
  assert_eq!(MISSILE_LAUNCHER_EXPLOSION_DAMAGE_DIE_SIDES, 6);
  assert_eq!(MISSILE_LAUNCHER_EXPLOSION_DELAY, 40);
  assert_eq!(MISSILE_LAUNCHER_EXPLOSION_RADIUS, 3);
  assert_eq!(MISSILE_LAUNCHER_EXPLOSION_KNOCKBACK, 8);

  let seed = seed_with_hit();
  let player_position = Position::new(4, 10);
  let center = Position::new(7, 10);
  let north = Position::new(7, 7);
  let outside = Position::new(11, 10);
  let mut game = equipped_missile_launcher(seed);
  configure_accuracy(&mut game, 100);
  let center_id = game
    .world_mut()
    .spawn_monster(center, "Center Target", 10_000, 0, (0, 0))
    .unwrap();
  let north_id = game
    .world_mut()
    .spawn_monster(north, "North Target", 10_000, 0, (0, 0))
    .unwrap();
  let outside_id = game
    .world_mut()
    .spawn_monster(outside, "Outside Target", 10_000, 0, (0, 0))
    .unwrap();

  let blast_positions = radius_three_blast_positions(game.world().map(), center);
  assert_eq!(blast_positions.len(), 49);
  assert_eq!(blast_positions[0], center);
  assert!(blast_positions.contains(&north));
  assert!(blast_positions.contains(&player_position));
  assert!(!blast_positions.contains(&outside));

  let mut expected_rng = game.rng().clone();
  let expected_hit_sample = expected_rng.gen_range(0..100);
  assert!(expected_hit_sample < 95, "fixture seed must hit the target");
  let expected_direct_damage = expected_rng.gen_range(6..37);
  let mut expected_center_splash = None;
  let mut expected_north_splash = None;
  let mut expected_player_splash = None;
  for position in blast_positions {
    let roll = roll_explosion_damage(&mut expected_rng);
    let damage = apply_distance_falloff(roll, center.distance_chebyshev(position));
    if position == center {
      expected_center_splash = Some(damage);
    } else if position == north {
      expected_north_splash = Some(damage);
    } else if position == player_position {
      expected_player_splash = Some(damage);
    }
  }

  let events = game
    .step(Command::AttackRanged(center))
    .expect("Missile Launcher direct fire should resolve");

  assert_eq!(game.rng(), &expected_rng);
  assert_eq!(
    game.world().get_actor(center_id).unwrap().hp().current,
    10_000 - expected_direct_damage - expected_center_splash.unwrap()
  );
  assert_eq!(
    game.world().get_actor(north_id).unwrap().hp().current,
    10_000 - expected_north_splash.unwrap()
  );
  assert_eq!(
    game.world().get_actor(outside_id).unwrap().hp().current,
    10_000
  );
  assert_eq!(
    game.world().player().unwrap().hp().current,
    50 - expected_player_splash.unwrap(),
    "the bounded Missile Launcher splash is not source-self-safe"
  );

  let attack_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          target_id,
          outcome: AttackOutcome::Hit { damage, .. },
          is_ranged: true,
          ..
        } if *target_id == center_id && *damage == expected_direct_damage
      )
    })
    .unwrap();
  let direct_damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id,
          amount,
          source: DamageSource::Actor(_),
          damage_type: Some(DamageType::Fire),
          ..
        } if *target_id == center_id && *amount == expected_direct_damage
      )
    })
    .unwrap();
  let schedule_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::MissileLauncherExplosionScheduled {
          target_id,
          delay,
          radius,
          knockback,
          ..
        } if *target_id == center_id
          && *delay == MISSILE_LAUNCHER_EXPLOSION_DELAY
          && *radius == MISSILE_LAUNCHER_EXPLOSION_RADIUS
          && *knockback == MISSILE_LAUNCHER_EXPLOSION_KNOCKBACK
      )
    })
    .unwrap();
  let splash_damage_indices: Vec<_> = events
    .iter()
    .enumerate()
    .filter_map(|(index, event)| {
      matches!(
        event,
        GameEvent::DamageApplied {
          source: DamageSource::Environment,
          damage_type: Some(DamageType::Fire),
          ..
        }
      )
      .then_some(index)
    })
    .collect();

  assert_eq!(attack_index + 1, direct_damage_index);
  assert_eq!(direct_damage_index + 1, schedule_index);
  assert_eq!(splash_damage_indices.len(), 3);
  assert!(
    splash_damage_indices
      .iter()
      .all(|index| *index > schedule_index)
  );
}

#[test]
fn missile_launcher_miss_still_schedules_and_fans_out() {
  let seed = seed_with_miss();
  let center = Position::new(7, 10);
  let mut game = equipped_missile_launcher(seed);
  configure_accuracy(&mut game, 100);
  let center_id = game
    .world_mut()
    .spawn_monster(center, "Missed Center", 10_000, 0, (0, 0))
    .unwrap();

  let mut expected_rng = game.rng().clone();
  assert!(expected_rng.gen_range(0..100) >= 95);
  let blast_positions = radius_three_blast_positions(game.world().map(), center);
  let mut expected_center_splash = None;
  for position in blast_positions {
    let roll = roll_explosion_damage(&mut expected_rng);
    if position == center {
      expected_center_splash = Some(roll);
    }
  }

  let events = game
    .step(Command::AttackRanged(center))
    .expect("a missed Missile Launcher shot still resolves its explosion");

  assert_eq!(game.rng(), &expected_rng);
  assert_eq!(
    game.world().get_actor(center_id).unwrap().hp().current,
    10_000 - expected_center_splash.unwrap()
  );
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::AttackResolved {
      target_id,
      outcome: AttackOutcome::Miss,
      is_ranged: true,
      ..
    } if *target_id == center_id
  )));
  assert!(!events.iter().any(|event| matches!(
    event,
    GameEvent::DamageApplied {
      source: DamageSource::Actor(_),
      ..
    }
  )));
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::MissileLauncherExplosionScheduled {
      target_id,
      delay: MISSILE_LAUNCHER_EXPLOSION_DELAY,
      radius: MISSILE_LAUNCHER_EXPLOSION_RADIUS,
      knockback: MISSILE_LAUNCHER_EXPLOSION_KNOCKBACK,
      ..
    } if *target_id == center_id
  )));
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::DamageApplied {
      target_id,
      source: DamageSource::Environment,
      damage_type: Some(DamageType::Fire),
      ..
    } if *target_id == center_id
  )));
}

#[test]
fn missile_launcher_uses_strict_ground_item_threshold_after_falloff() {
  assert_eq!(MISSILE_LAUNCHER_GROUND_ITEM_DESTRUCTION_THRESHOLD, 10);

  let low_seed = seed_with_center_roll(10);
  let high_seed = seed_with_center_roll(11);
  assert!(!should_destroy_ground_item(
    center_splash_roll(low_seed).unwrap()
  ));
  assert!(should_destroy_ground_item(
    center_splash_roll(high_seed).unwrap()
  ));

  let center = Position::new(7, 10);
  let mut low = equipped_missile_launcher(low_seed);
  configure_accuracy(&mut low, 100);
  low
    .world_mut()
    .spawn_monster(center, "Low Roll Target", 10_000, 0, (0, 0))
    .unwrap();
  let low_item_id = low.world_mut().allocate_item_id();
  low
    .world_mut()
    .spawn_ground_item(center, Item::small_medpack(low_item_id))
    .unwrap();
  let low_events = low.step(Command::AttackRanged(center)).unwrap();
  assert_eq!(low.world().ground_items_at(center).len(), 1);
  assert!(!low_events.iter().any(|event| matches!(
    event,
    GameEvent::GroundItemDestroyed { position, .. } if *position == center
  )));

  let mut high = equipped_missile_launcher(high_seed);
  configure_accuracy(&mut high, 100);
  high
    .world_mut()
    .spawn_monster(center, "High Roll Target", 10_000, 0, (0, 0))
    .unwrap();
  let high_item_id = high.world_mut().allocate_item_id();
  high
    .world_mut()
    .spawn_ground_item(center, Item::small_medpack(high_item_id))
    .unwrap();
  let high_events = high.step(Command::AttackRanged(center)).unwrap();
  assert!(high.world().ground_items_at(center).is_empty());
  assert!(high_events.iter().any(|event| matches!(
    event,
    GameEvent::GroundItemDestroyed { item_id, position }
      if *item_id == high_item_id && *position == center
  )));
}

#[test]
fn missile_launcher_splash_death_drop_preflight_is_atomic() {
  let center = Position::new(7, 10);
  let victim = Position::new(7, 9);
  let mut game = equipped_missile_launcher(seed_with_hit());
  game
    .world_mut()
    .spawn_monster(center, "Center Target", 10_000, 0, (0, 0))
    .unwrap();
  let victim_id = game
    .world_mut()
    .spawn_monster(victim, "Dropper", 1, 0, (0, 0))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(victim_id)
    .unwrap()
    .set_death_drop(Some(ItemSpawnKind::SmallMedPack));
  game.world_mut().map_mut().set_tile(victim, Tile::Wall);

  let before = game.clone();
  let error = game.step(Command::AttackRanged(center)).unwrap_err();

  assert_eq!(error, CommandError::BlockedByTerrain(victim));
  assert_eq!(game, before);
}

#[test]
fn missile_launcher_radius_three_replay_is_deterministic_and_versioned() {
  assert_eq!(CURRENT_GAMEPLAY_SEMANTICS_VERSION, 150);

  let center = Position::new(5, 5);
  let player_start = Position::new(2, 5);
  let mut replay =
    ReplayLog::new(47_305, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 500,
      max_hp: 500,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoRockets(2)],
      equipped_weapon: Some(ItemSpawnKind::MissileLauncher),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    center,
    "Replay Center",
    10_000,
    0,
    (0, 0),
  ));
  replay.record_monster(MonsterSpawnSpec::new(
    Position::new(5, 2),
    "Replay North",
    10_000,
    0,
    (0, 0),
  ));
  replay.record_monster(MonsterSpawnSpec::new(
    Position::new(9, 5),
    "Replay Outside",
    10_000,
    0,
    (0, 0),
  ));
  replay.record_command(Command::AttackRanged(center));

  let (first_game, first_events) = ReplayEngine::run(&replay).unwrap();
  let (second_game, second_events) = ReplayEngine::run(&replay).unwrap();
  assert_eq!(first_game, second_game);
  assert_eq!(first_events, second_events);
  assert!(first_events.iter().any(|event| matches!(
    event,
    GameEvent::MissileLauncherExplosionScheduled {
      delay: MISSILE_LAUNCHER_EXPLOSION_DELAY,
      radius: MISSILE_LAUNCHER_EXPLOSION_RADIUS,
      knockback: MISSILE_LAUNCHER_EXPLOSION_KNOCKBACK,
      ..
    }
  )));
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());

  let mut stale = replay;
  stale.metadata.gameplay_semantics_version = 149;
  let error = ReplayEngine::validate(&stale).unwrap_err();
  assert!(error.contains("unsupported gameplay semantics version"));
}

#[test]
fn missile_launcher_radius_three_runs_through_scenario_runner() {
  let mut scenario = Scenario::from_ascii(
    "Radius Three Missile",
    "Center-inclusive Missile Launcher fanout",
    "############\n#..........#\n#..........#\n#..........#\n#..........#\n#..........#\n#.@........#\n#..........#\n#..........#\n#..........#\n#..........#\n############",
  )
  .unwrap();
  scenario.seed = 47_306;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 500,
    max_hp: 500,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoRockets(2)],
    equipped_weapon: Some(ItemSpawnKind::MissileLauncher),
    equipped_armor: None,
    equipped_armor_durability: None,
  });
  let center = Position::new(5, 6);
  scenario.monsters.push(MonsterSpawnSpec::new(
    center,
    "Scenario Center",
    10_000,
    0,
    (0, 0),
  ));
  scenario.monsters.push(MonsterSpawnSpec::new(
    Position::new(5, 3),
    "Scenario North",
    10_000,
    0,
    (0, 0),
  ));
  scenario.monsters.push(MonsterSpawnSpec::new(
    Position::new(9, 6),
    "Scenario Outside",
    10_000,
    0,
    (0, 0),
  ));

  let (game, events, _metrics, replay) =
    ScenarioRunner::run_commands(&scenario, &[Command::AttackRanged(center)])
      .expect("ScenarioRunner should execute Missile Launcher's fanout");

  assert_eq!(replay.commands, vec![Command::AttackRanged(center)]);
  assert_eq!(replay.metadata.gameplay_semantics_version, 150);
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::MissileLauncherExplosionScheduled {
      delay: MISSILE_LAUNCHER_EXPLOSION_DELAY,
      radius: MISSILE_LAUNCHER_EXPLOSION_RADIUS,
      knockback: MISSILE_LAUNCHER_EXPLOSION_KNOCKBACK,
      ..
    }
  )));
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::DamageApplied {
          source: DamageSource::Environment,
          damage_type: Some(DamageType::Fire),
          ..
        }
      ))
      .count(),
    3,
    "scenario runner preserves center, north, and source-actor splash"
  );
  assert!(game.world().player().unwrap().is_alive());
}
