//! Focused integration coverage for Mega Buster exact dice and radius-one
//! Fire/Acid fanout.
//!
//! These fixtures keep the current Rust convention explicit: Fire and Acid
//! schedule a delay-40 explosion and resolve center-first radius-one
//! environment damage immediately, including after a direct miss. The
//! delayed queue, legacy traversal/accuracy timing, and same-volley mutation
//! remain outside this bounded slice.

use drl_core::anti_freak::radius_one_blast_positions;
use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::mega_buster::{
  MEGA_BUSTER_ACID_PROFILE, MEGA_BUSTER_FIRE_PROFILE, MegaBusterMorphProfile,
};
use drl_core::replay::ReplayEngine;
use drl_core::rng::GameRng;
use drl_protocol::{
  AttackOutcome, Command, CommandError, DamageSource, DamageType, EquipmentSlot, GameEvent,
  ItemSpawnKind, MegaBusterMorphMode, MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

const EXPLOSION_DELAY: u32 = 40;
const EXPLOSION_RADIUS: u32 = 1;
const EXPLOSION_KNOCKBACK: u32 = 8;

fn equipped_mega_buster(seed: u64) -> Game {
  let mut game = Game::new(seed, 16, 12, Position::new(2, 6)).expect("arena");
  let player_id = game.world().player_id().expect("player");
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .expect("player actor")
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::mega_buster(weapon_id))
    .expect("equip Mega Buster");
  game
}

fn set_mega_profile(game: &mut Game, mode: MegaBusterMorphMode) {
  game
    .world_mut()
    .player_mut()
    .expect("player")
    .equipment_mut()
    .weapon_mut()
    .expect("Mega Buster")
    .apply_mega_buster_morph(mode)
    .expect("initial Bullet profile should change");
}

fn set_accuracy(game: &mut Game, accuracy: i32) {
  game
    .world_mut()
    .player_mut()
    .expect("player")
    .equipment_mut()
    .weapon_mut()
    .expect("Mega Buster")
    .weapon_properties_mut()
    .expect("Mega Buster properties")
    .accuracy = accuracy;
}

fn spawn_target(game: &mut Game, position: Position, hp: u32) -> drl_protocol::EntityId {
  game
    .world_mut()
    .spawn_monster(position, "Mega Buster Target", hp, 0, (0, 0))
    .expect("target");
  game
    .world()
    .living_actor_at(position)
    .expect("spawned target")
    .id()
}

fn roll_splash_cells(profile: MegaBusterMorphProfile, rng: &mut GameRng) -> [u32; 9] {
  std::array::from_fn(|_| profile.roll_damage(rng))
}

fn seed_with_three_misses() -> u64 {
  (0..100_000)
    .find(|seed| {
      let mut rng = GameRng::from_seed(*seed);
      (0..3).all(|_| {
        let missed = rng.gen_range(0..100) >= 5;
        let _ = roll_splash_cells(MEGA_BUSTER_FIRE_PROFILE, &mut rng);
        missed
      })
    })
    .expect("deterministic three-miss seed")
}

#[test]
fn fire_hit_uses_exact_dice_and_center_first_typed_fanout() {
  let center = Position::new(7, 6);
  let north = Position::new(7, 5);
  let profile = MEGA_BUSTER_FIRE_PROFILE;
  let mut game = equipped_mega_buster(0);
  set_mega_profile(&mut game, MegaBusterMorphMode::Fire);
  set_accuracy(&mut game, 100);
  // A lethal first projectile makes ordinary Mega Buster fire stop after one
  // shot, isolating exact direct-plus-splash RNG from the three-shot contract.
  let center_id = spawn_target(&mut game, center, 1);
  let north_id = spawn_target(&mut game, north, 1_000);
  let weapon_id = game
    .world()
    .player()
    .expect("player")
    .equipment()
    .weapon()
    .expect("Mega Buster")
    .id();

  let mut expected_rng = game.rng().clone();
  assert!(expected_rng.gen_range(0..100) < 95, "fixture must hit");
  let expected_direct_damage = profile.roll_damage(&mut expected_rng);
  let expected_splash = roll_splash_cells(profile, &mut expected_rng);

  let events = game
    .step(Command::AttackRanged(center))
    .expect("Fire Mega Buster shot");

  assert_eq!(game.rng(), &expected_rng);
  assert_eq!(game.world().get_actor(center_id).unwrap().hp().current, 0);
  assert_eq!(
    game.world().get_actor(north_id).unwrap().hp().current,
    1_000 - expected_splash[1]
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
    .expect("direct Fire attack");
  let direct_damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id,
          amount: 1,
          source: DamageSource::Actor(_),
          damage_type: Some(DamageType::Fire),
          ..
        } if *target_id == center_id
      )
    })
    .expect("typed direct Fire damage");
  let schedule_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::MegaBusterExplosionScheduled {
          item_id,
          target_id,
          delay,
          radius,
          knockback,
          damage_type: DamageType::Fire,
          ..
        } if *item_id == weapon_id
          && *target_id == center_id
          && *delay == EXPLOSION_DELAY
          && *radius == EXPLOSION_RADIUS
          && *knockback == EXPLOSION_KNOCKBACK
      )
    })
    .expect("typed Fire explosion schedule");
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
  // The direct target is already dead before fanout; only the living north
  // neighbor receives a typed environmental DamageApplied event.
  assert_eq!(splash_damage_indices.len(), 1);
  assert!(splash_damage_indices[0] > schedule_index);
}

#[test]
fn acid_hit_keeps_typed_damage_through_direct_and_splash_events() {
  let center = Position::new(7, 6);
  let north = Position::new(7, 5);
  let profile = MEGA_BUSTER_ACID_PROFILE;
  let mut game = equipped_mega_buster(0);
  set_mega_profile(&mut game, MegaBusterMorphMode::Acid);
  set_accuracy(&mut game, 100);
  let center_id = spawn_target(&mut game, center, 1);
  let north_id = spawn_target(&mut game, north, 1_000);

  let mut expected_rng = game.rng().clone();
  assert!(expected_rng.gen_range(0..100) < 95, "fixture must hit");
  let expected_direct_damage = profile.roll_damage(&mut expected_rng);
  let expected_splash = roll_splash_cells(profile, &mut expected_rng);
  let events = game
    .step(Command::AttackRanged(center))
    .expect("Acid Mega Buster shot");

  assert_eq!(game.rng(), &expected_rng);
  assert_eq!(game.world().get_actor(center_id).unwrap().hp().current, 0);
  assert_eq!(
    game.world().get_actor(north_id).unwrap().hp().current,
    1_000 - expected_splash[1]
  );
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::AttackResolved {
        target_id,
        outcome: AttackOutcome::Hit { damage, .. },
        is_ranged: true,
        ..
      } if *target_id == center_id && *damage == expected_direct_damage
    )
  }));
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::DamageApplied {
        source: DamageSource::Actor(_),
        target_id,
        amount: 1,
        damage_type: Some(DamageType::Acid),
        ..
      } if *target_id == center_id
    )
  }));
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::MegaBusterExplosionScheduled {
        target_id,
        delay,
        radius,
        knockback,
        damage_type: DamageType::Acid,
        ..
      } if *target_id == center_id
        && *delay == EXPLOSION_DELAY
        && *radius == EXPLOSION_RADIUS
        && *knockback == EXPLOSION_KNOCKBACK
    )
  }));
  assert_eq!(
    events
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::DamageApplied {
            source: DamageSource::Environment,
            damage_type: Some(DamageType::Acid),
            ..
          }
        )
      })
      .count(),
    1
  );
}

#[test]
fn fire_miss_still_schedules_and_fans_out_without_direct_damage() {
  let center = Position::new(7, 6);
  let seed = seed_with_three_misses();
  let mut game = equipped_mega_buster(seed);
  set_mega_profile(&mut game, MegaBusterMorphMode::Fire);
  set_accuracy(&mut game, 0);
  let center_id = spawn_target(&mut game, center, 1_000);
  let mut replay = game.clone();

  let events = game
    .step(Command::AttackRanged(center))
    .expect("missed Fire Mega Buster shot");
  let replay_events = replay
    .step(Command::AttackRanged(center))
    .expect("repeat missed Fire Mega Buster shot");

  assert_eq!(events, replay_events);
  assert_eq!(game, replay);
  assert_eq!(
    events
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::MegaBusterExplosionScheduled {
            delay: EXPLOSION_DELAY,
            radius: EXPLOSION_RADIUS,
            knockback: EXPLOSION_KNOCKBACK,
            damage_type: DamageType::Fire,
            ..
          }
        )
      })
      .count(),
    3
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            target_id,
            outcome: AttackOutcome::Miss,
            is_ranged: true,
            ..
          } if *target_id == center_id
        )
      })
      .count(),
    3
  );
  assert!(!events.iter().any(|event| {
    matches!(
      event,
      GameEvent::DamageApplied {
        target_id,
        source: DamageSource::Actor(_),
        ..
      } if *target_id == center_id
    )
  }));
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::DamageApplied {
        source: DamageSource::Environment,
        damage_type: Some(DamageType::Fire),
        ..
      }
    )
  }));
}

#[test]
fn radius_one_death_drop_preflight_rejects_before_clip_or_rng_mutation() {
  let center = Position::new(7, 6);
  let north = Position::new(7, 5);
  let mut game = equipped_mega_buster(12_301);
  set_mega_profile(&mut game, MegaBusterMorphMode::Fire);
  set_accuracy(&mut game, 100);
  let _center_id = spawn_target(&mut game, center, 1_000);
  let dropper_id = spawn_target(&mut game, north, 1);
  game
    .world_mut()
    .get_actor_mut(dropper_id)
    .expect("dropper")
    .set_death_drop(Some(ItemSpawnKind::SmallMedPack));
  game
    .world_mut()
    .map_mut()
    .set_tile(north, drl_core::grid::Tile::Wall);

  let before = game.clone();
  let error = game
    .step(Command::AttackRanged(center))
    .expect_err("blocked drop");
  assert_eq!(error, CommandError::BlockedByTerrain(north));
  assert_eq!(game, before);
}

fn fire_replay(seed: u64) -> ReplayLog {
  let mut replay =
    ReplayLog::new(seed, 16, 12, Position::new(2, 6)).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::MegaBuster),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(
    MonsterSpawnSpec::new(Position::new(3, 6), "Fire Profile Source", 1, 0, (0, 0))
      .with_equipped_weapon(Some(ItemSpawnKind::RocketLauncher)),
  );
  replay.record_monster(MonsterSpawnSpec::new(
    Position::new(7, 6),
    "Fire Fanout Target",
    1_000,
    0,
    (0, 0),
  ));
  replay.record_command(Command::AttackRanged(Position::new(3, 6)));
  replay.record_command(Command::AttackRanged(Position::new(7, 6)));
  replay
}

#[test]
fn replay_morph_then_fire_fanout_is_repeatable_with_target_topology() {
  let seed = (0..10_000)
    .find(|seed| {
      let replay = fire_replay(*seed);
      ReplayEngine::run(&replay).is_ok_and(|(_, events)| {
        events.iter().any(|event| {
          matches!(
            event,
            GameEvent::MegaBusterMorphed {
              current: MegaBusterMorphMode::Fire,
              ..
            }
          )
        }) && events.iter().any(|event| {
          matches!(
            event,
            GameEvent::MegaBusterExplosionScheduled {
              damage_type: DamageType::Fire,
              ..
            }
          )
        })
      })
    })
    .expect("deterministic replay seed with Fire morph and fanout");
  let replay = fire_replay(seed);
  let (game, events) = ReplayEngine::run(&replay).expect("replay Mega Buster fanout");
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::MegaBusterMorphed {
        current: MegaBusterMorphMode::Fire,
        ..
      }
    )
  }));
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::MegaBusterExplosionScheduled {
        damage_type: DamageType::Fire,
        delay: EXPLOSION_DELAY,
        radius: EXPLOSION_RADIUS,
        knockback: EXPLOSION_KNOCKBACK,
        ..
      }
    )
  }));
  assert_eq!(
    game
      .world()
      .player()
      .expect("player")
      .equipment()
      .weapon()
      .expect("Mega Buster")
      .mega_buster_morph(),
    Some(MegaBusterMorphMode::Fire)
  );
  assert!(ReplayEngine::verify_determinism(&replay).expect("replay determinism"));

  let mut stale = replay.clone();
  stale.metadata.gameplay_semantics_version = 151;
  assert!(ReplayEngine::run(&stale).is_err());
}

#[test]
fn radius_one_geometry_is_center_then_clockwise_neighbors() {
  let map = drl_core::grid::Map::simple_arena(5, 5);
  assert_eq!(
    radius_one_blast_positions(&map, Position::new(2, 2)),
    vec![
      Position::new(2, 2),
      Position::new(2, 1),
      Position::new(3, 1),
      Position::new(3, 2),
      Position::new(3, 3),
      Position::new(2, 3),
      Position::new(1, 3),
      Position::new(1, 2),
      Position::new(1, 1),
    ]
  );
}
