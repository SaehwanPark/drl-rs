use drl_core::game::Game;
use drl_core::grid::Tile;
use drl_core::item::Item;
use drl_core::resistance::apply_damage_resistance;
use drl_core::rocket_launcher::{
  ROCKET_LAUNCHER_EXPLOSION_DELAY, ROCKET_LAUNCHER_EXPLOSION_KNOCKBACK,
  ROCKET_LAUNCHER_EXPLOSION_RADIUS, apply_distance_falloff, radius_four_blast_positions,
  roll_explosion_damage,
};
use drl_protocol::{
  AttackOutcome, Command, CommandError, DamageSource, DamageType, EquipmentSlot, GameEvent,
  ItemSpawnKind, Position,
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

#[test]
fn rocket_launcher_hit_schedules_and_fans_out_with_falloff() {
  let seed = 40_000;
  let player_position = Position::new(3, 10);
  let center = Position::new(7, 10);
  let north = Position::new(7, 7);
  let outside = Position::new(12, 10);
  let mut game = equipped_rocket_launcher(seed);
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(8, 10), Tile::Wall);
  let center_id = game
    .world_mut()
    .spawn_monster(center, "Center Target", 500, 0, (0, 0))
    .unwrap();
  let north_id = game
    .world_mut()
    .spawn_monster(north, "North Target", 500, 0, (0, 0))
    .unwrap();
  let outside_id = game
    .world_mut()
    .spawn_monster(outside, "Outside Target", 500, 0, (0, 0))
    .unwrap();
  let ammo_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .spawn_ground_item(Position::new(7, 9), Item::ammo_rockets(ammo_id, 2))
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

  let map = game.world().map().clone();
  let blast_positions = radius_four_blast_positions(&map, center);
  let mut expected_rng = game.rng().clone();
  let expected_direct_hit = expected_rng.gen_range(0..100);
  assert!(expected_direct_hit < 94, "fixture seed must hit the target");
  let expected_direct_damage = expected_rng.gen_range(6..37);
  let mut expected_center_splash = None;
  let mut expected_north_splash = None;
  let mut expected_player_splash = None;
  for position in blast_positions {
    let roll = roll_explosion_damage(&mut expected_rng);
    if position == center {
      expected_center_splash = Some(apply_distance_falloff(roll, 0));
    } else if position == north {
      expected_north_splash = Some(apply_distance_falloff(roll, 3));
    } else if position == player_position {
      expected_player_splash = Some(apply_distance_falloff(roll, 4));
    }
  }

  let events = game
    .step(Command::AttackRanged(center))
    .expect("Rocket Launcher direct fire should resolve");

  assert_eq!(game.rng(), &expected_rng);
  assert_eq!(
    game.world().get_actor(center_id).unwrap().hp().current,
    500 - expected_direct_damage - expected_center_splash.unwrap()
  );
  assert_eq!(
    game.world().get_actor(north_id).unwrap().hp().current,
    500 - expected_north_splash.unwrap()
  );
  assert_eq!(
    game.world().get_actor(outside_id).unwrap().hp().current,
    500
  );
  assert_eq!(
    game.world().player().unwrap().hp().current,
    50 - expected_player_splash.unwrap()
  );
  assert_eq!(
    game.world().ground_items().get(&ammo_id).unwrap().1.count(),
    2,
    "the bounded actor-only resolver must not destroy ground items"
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
          damage_type: None,
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
        GameEvent::RocketLauncherExplosionScheduled {
          entity_id: _,
          target_id,
          delay,
          radius,
          knockback,
        } if *target_id == center_id
          && *delay == ROCKET_LAUNCHER_EXPLOSION_DELAY
          && *radius == ROCKET_LAUNCHER_EXPLOSION_RADIUS
          && *knockback == ROCKET_LAUNCHER_EXPLOSION_KNOCKBACK
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
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::ActorKnockedBack { entity_id, .. } if *entity_id == north_id
    )
  }));
}

#[test]
fn rocket_launcher_splash_death_drop_preflight_is_atomic() {
  let center = Position::new(7, 10);
  let victim = Position::new(7, 9);
  let mut game = equipped_rocket_launcher(40_001);
  game
    .world_mut()
    .spawn_monster(center, "Center Target", 500, 0, (0, 0))
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
fn red_armor_mitigates_same_seed_rocket_fire_splash() {
  let seed = 40_002;
  let center = Position::new(7, 10);
  let target_position = Position::new(7, 9);
  let mut plain = equipped_rocket_launcher(seed);
  let mut armored = equipped_rocket_launcher(seed);

  let plain_center_id = plain
    .world_mut()
    .spawn_monster(center, "Center Target", 500, 0, (0, 0))
    .unwrap();
  let plain_target_id = plain
    .world_mut()
    .spawn_monster(target_position, "Plain Target", 500, 0, (0, 0))
    .unwrap();
  let armored_center_id = armored
    .world_mut()
    .spawn_monster(center, "Center Target", 500, 0, (0, 0))
    .unwrap();
  let armored_target_id = armored
    .world_mut()
    .spawn_monster(target_position, "Armored Target", 500, 0, (0, 0))
    .unwrap();
  let armor_id = armored.world_mut().allocate_item_id();
  armored
    .world_mut()
    .get_actor_mut(armored_target_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::red_armor(armor_id))
    .unwrap();
  for game in [&mut plain, &mut armored] {
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
  }

  let plain_events = plain
    .step(Command::AttackRanged(center))
    .expect("plain Rocket Launcher fire should resolve");
  let armored_events = armored
    .step(Command::AttackRanged(center))
    .expect("Red Armor Rocket Launcher fire should resolve");
  let plain_damage = plain_events
    .iter()
    .find_map(|event| match event {
      GameEvent::DamageApplied {
        target_id,
        amount,
        source: DamageSource::Environment,
        damage_type: Some(DamageType::Fire),
        ..
      } if *target_id == plain_target_id => Some(*amount),
      _ => None,
    })
    .expect("plain target should receive typed Fire splash");
  let armored_damage = armored_events
    .iter()
    .find_map(|event| match event {
      GameEvent::DamageApplied {
        target_id,
        amount,
        source: DamageSource::Environment,
        damage_type: Some(DamageType::Fire),
        ..
      } if *target_id == armored_target_id => Some(*amount),
      _ => None,
    })
    .expect("Red Armor target should receive typed Fire splash");

  assert_eq!(plain_center_id, armored_center_id);
  assert_eq!(plain_target_id, armored_target_id);
  assert_eq!(
    armored_damage,
    apply_damage_resistance(plain_damage, 25)
      .saturating_sub(4)
      .max(1)
  );
  assert!(armored_damage < plain_damage);
}
