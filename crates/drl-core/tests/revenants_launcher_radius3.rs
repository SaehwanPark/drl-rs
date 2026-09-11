//! Focused integration coverage for Revenant's Launcher radius-3 Fire fanout.
//!
//! The direct exact-hit profile is covered by `revenants_launcher_direct_fire.rs`.
//! This fixture pins the follow-on splash contract: deterministic 7d6 rolls in
//! center-first radius-3 order, strict ground-item thresholding, schedule/event
//! ordering, death-drop preflight atomicity, and replay semantics.

use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::replay::ReplayEngine;
use drl_core::revenants_launcher::{
  REVENANTS_LAUNCHER_EXPLOSION_DAMAGE_DICE, REVENANTS_LAUNCHER_EXPLOSION_DAMAGE_DIE_SIDES,
  REVENANTS_LAUNCHER_EXPLOSION_DELAY, REVENANTS_LAUNCHER_EXPLOSION_KNOCKBACK,
  REVENANTS_LAUNCHER_EXPLOSION_RADIUS, REVENANTS_LAUNCHER_GROUND_ITEM_DESTRUCTION_THRESHOLD,
  apply_distance_falloff, radius_three_blast_positions, roll_explosion_damage,
  should_destroy_ground_item,
};
use drl_core::rng::GameRng;
use drl_core::scenario::{Scenario, ScenarioRunner};
use drl_protocol::{
  AttackOutcome, CURRENT_GAMEPLAY_SEMANTICS_VERSION, Command, CommandError, DamageSource,
  DamageType, EquipmentSlot, GameEvent, ItemSpawnKind, MonsterSpawnSpec, PlayerSpawnConfig,
  Position, ReplayLog,
};

fn equipped_revenants_launcher(seed: u64) -> Game {
  let mut game = Game::new(seed, 20, 20, Position::new(4, 10)).unwrap();
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

fn center_splash_roll(seed: u64) -> u32 {
  let mut rng = GameRng::from_seed(seed);
  // Revenant's Launcher is exact-hit: the first combat sample is its 7d6
  // direct damage, followed by the center splash's 7d6 roll.
  let _direct_damage = rng.gen_range(7..43);
  roll_explosion_damage(&mut rng)
}

fn seed_with_center_roll(predicate: impl Fn(u32) -> bool) -> u64 {
  (0..100_000)
    .find(|seed| predicate(center_splash_roll(*seed)))
    .expect("the deterministic seed search must find a matching 7d6 roll")
}

#[test]
fn revenants_launcher_schedules_and_fans_out_radius_three_in_order() {
  let seed = 47_301;
  let player_position = Position::new(4, 10);
  let center = Position::new(7, 10);
  let north = Position::new(7, 7);
  let outside = Position::new(11, 10);
  let mut game = equipped_revenants_launcher(seed);
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
  assert!(!blast_positions.contains(&outside));

  let mut expected_rng = game.rng().clone();
  let expected_direct_damage = expected_rng.gen_range(7..43);
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
    .expect("Revenant's Launcher direct fire should resolve");

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
    "source actor is not self-safe for the Revenant's Launcher splash"
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
        GameEvent::RevenantsLauncherExplosionScheduled {
          target_id,
          delay,
          radius,
          knockback,
          ..
        } if *target_id == center_id
          && *delay == REVENANTS_LAUNCHER_EXPLOSION_DELAY
          && *radius == REVENANTS_LAUNCHER_EXPLOSION_RADIUS
          && *knockback == REVENANTS_LAUNCHER_EXPLOSION_KNOCKBACK
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
fn revenants_launcher_uses_strict_ground_item_threshold_after_falloff() {
  assert_eq!(REVENANTS_LAUNCHER_EXPLOSION_DAMAGE_DICE, 7);
  assert_eq!(REVENANTS_LAUNCHER_EXPLOSION_DAMAGE_DIE_SIDES, 6);
  assert_eq!(REVENANTS_LAUNCHER_GROUND_ITEM_DESTRUCTION_THRESHOLD, 10);

  let low_seed = seed_with_center_roll(|roll| roll == 10);
  let high_seed = seed_with_center_roll(|roll| roll == 11);
  assert!(!should_destroy_ground_item(center_splash_roll(low_seed)));
  assert!(should_destroy_ground_item(center_splash_roll(high_seed)));

  let center = Position::new(7, 10);
  let mut low = equipped_revenants_launcher(low_seed);
  low
    .world_mut()
    .spawn_monster(center, "Center Target", 10_000, 0, (0, 0))
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

  let mut high = equipped_revenants_launcher(high_seed);
  high
    .world_mut()
    .spawn_monster(center, "Center Target", 10_000, 0, (0, 0))
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
fn revenants_launcher_splash_death_drop_preflight_is_atomic() {
  let center = Position::new(7, 10);
  let victim = Position::new(7, 9);
  let mut game = equipped_revenants_launcher(47_302);
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
  game
    .world_mut()
    .map_mut()
    .set_tile(victim, drl_core::grid::Tile::Wall);

  let before = game.clone();
  let error = game.step(Command::AttackRanged(center)).unwrap_err();

  assert_eq!(error, CommandError::BlockedByTerrain(victim));
  assert_eq!(game, before);
}

#[test]
fn revenants_launcher_radius_three_replay_is_deterministic_and_versioned() {
  assert_eq!(CURRENT_GAMEPLAY_SEMANTICS_VERSION, 152);

  let center = Position::new(6, 8);
  let player_start = Position::new(2, 8);
  let mut replay =
    ReplayLog::new(47_303, 16, 16, player_start).with_player_config(PlayerSpawnConfig {
      hp: 500,
      max_hp: 500,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoRockets(2)],
      equipped_weapon: Some(ItemSpawnKind::RevenantsLauncher),
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
    Position::new(6, 5),
    "Replay North",
    10_000,
    0,
    (0, 0),
  ));
  replay.record_monster(MonsterSpawnSpec::new(
    Position::new(10, 8),
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
    GameEvent::RevenantsLauncherExplosionScheduled {
      delay: REVENANTS_LAUNCHER_EXPLOSION_DELAY,
      radius: REVENANTS_LAUNCHER_EXPLOSION_RADIUS,
      knockback: REVENANTS_LAUNCHER_EXPLOSION_KNOCKBACK,
      ..
    }
  )));
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());

  let mut stale = replay;
  stale.metadata.gameplay_semantics_version = CURRENT_GAMEPLAY_SEMANTICS_VERSION - 1;
  let error = ReplayEngine::validate(&stale).unwrap_err();
  assert!(error.contains("unsupported gameplay semantics version"));
}

#[test]
fn revenants_launcher_radius_three_runs_through_scenario_runner() {
  let mut scenario = Scenario::from_ascii(
    "Radius Three Revenant",
    "Center-inclusive Revenant's Launcher fanout",
    "############\n#..........#\n#..........#\n#..........#\n#..........#\n#..........#\n#.@........#\n#..........#\n#..........#\n#..........#\n#..........#\n############",
  )
  .unwrap();
  scenario.seed = 47_304;
  scenario.player_config = Some(PlayerSpawnConfig {
    hp: 500,
    max_hp: 500,
    speed: 100,
    initial_items: vec![ItemSpawnKind::AmmoRockets(2)],
    equipped_weapon: Some(ItemSpawnKind::RevenantsLauncher),
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
      .expect("ScenarioRunner should execute Revenant's Launcher fanout");

  assert_eq!(replay.commands, vec![Command::AttackRanged(center)]);
  assert_eq!(replay.metadata.gameplay_semantics_version, 152);
  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::RevenantsLauncherExplosionScheduled {
      delay: REVENANTS_LAUNCHER_EXPLOSION_DELAY,
      radius: REVENANTS_LAUNCHER_EXPLOSION_RADIUS,
      knockback: REVENANTS_LAUNCHER_EXPLOSION_KNOCKBACK,
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
    "scenario runner preserves the center, north, and source-actor splash"
  );
  assert!(game.world().player().unwrap().is_alive());
}
