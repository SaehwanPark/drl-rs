use drl_core::ReplayEngine;
use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::rocket_launcher::{
  apply_distance_falloff, radius_four_blast_positions, roll_explosion_damage,
  should_destroy_ground_item,
};
use drl_protocol::{
  Command, DamageSource, DamageType, EquipmentSlot, GameEvent, ItemId, ItemSpawnKind,
  ItemSpawnSpec, MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
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

fn rocket_blast_plan(
  game: &Game,
  center: Position,
) -> (Vec<Position>, Vec<u32>, drl_core::rng::GameRng) {
  let mut expected_rng = game.rng().clone();
  let direct_hit = expected_rng.gen_range(0..100);
  assert!(direct_hit < 94, "fixture seed must hit the target");
  expected_rng.gen_range(6..37);
  let positions = radius_four_blast_positions(game.world().map(), center);
  let damages = positions
    .iter()
    .map(|position| {
      let roll = roll_explosion_damage(&mut expected_rng);
      apply_distance_falloff(roll, center.distance_chebyshev(*position))
    })
    .collect();
  (positions, damages, expected_rng)
}

#[test]
fn rocket_launcher_destroys_lowest_id_item_above_post_falloff_threshold() {
  let seed = 40_100;
  let center = Position::new(7, 10);
  let mut game = equipped_rocket_launcher(seed);
  let center_id = game
    .world_mut()
    .spawn_monster(center, "Center Target", 500, 0, (0, 0))
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

  let (blast_positions, blast_damages, expected_rng) = rocket_blast_plan(&game, center);
  let (hot_index, hot_damage) = blast_damages
    .iter()
    .enumerate()
    .skip(1)
    .find(|(_, damage)| should_destroy_ground_item(**damage))
    .map(|(index, damage)| (index, *damage))
    .expect("fixture seed must include a destructive non-center blast cell");
  let (cold_index, cold_damage) = blast_damages
    .iter()
    .enumerate()
    .skip(1)
    .find(|(index, damage)| *index != hot_index && !should_destroy_ground_item(**damage))
    .map(|(index, damage)| (index, *damage))
    .expect("fixture seed must include a non-destructive blast cell");
  let hot_position = blast_positions[hot_index];
  let cold_position = blast_positions[cold_index];
  let hot_actor_id = game
    .world_mut()
    .spawn_monster(hot_position, "Hot Target", 500, 0, (0, 0))
    .unwrap();
  let low_item_id = ItemId::new(100);
  let high_item_id = ItemId::new(101);
  let cold_item_id = ItemId::new(102);
  game
    .world_mut()
    .spawn_ground_item(hot_position, Item::shotgun(low_item_id))
    .unwrap();
  game
    .world_mut()
    .spawn_ground_item(hot_position, Item::small_medpack(high_item_id))
    .unwrap();
  game
    .world_mut()
    .spawn_ground_item(cold_position, Item::small_medpack(cold_item_id))
    .unwrap();

  let events = game
    .step(Command::AttackRanged(center))
    .expect("Rocket Launcher direct fire should resolve");
  let destroyed: Vec<_> = events
    .iter()
    .filter_map(|event| match event {
      GameEvent::GroundItemDestroyed { item_id, position } => Some((*item_id, *position)),
      _ => None,
    })
    .collect();
  assert_eq!(destroyed, vec![(low_item_id, hot_position)]);
  assert_eq!(hot_damage, blast_damages[hot_index]);
  assert!(!should_destroy_ground_item(cold_damage));
  assert!(!game.world().ground_items().contains_key(&low_item_id));
  assert!(game.world().ground_items().contains_key(&high_item_id));
  assert!(game.world().ground_items().contains_key(&cold_item_id));
  assert_eq!(game.rng(), &expected_rng);

  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id,
          source: DamageSource::Environment,
          damage_type: Some(DamageType::Fire),
          ..
        } if *target_id == hot_actor_id
      )
    })
    .expect("destructive blast cell should damage its actor before item cleanup");
  let destroyed_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::GroundItemDestroyed { item_id, position }
          if *item_id == low_item_id && *position == hot_position
      )
    })
    .unwrap();
  assert!(damage_index < destroyed_index);
  assert!(events.iter().any(|event| {
    matches!(
      event,
      GameEvent::DamageApplied {
        target_id,
        source: DamageSource::Environment,
        damage_type: Some(DamageType::Fire),
        ..
      } if *target_id == center_id
    )
  }));
}

#[test]
fn rocket_launcher_ground_item_effect_replays_with_identical_events_and_state() {
  let seed = 25;
  let player_position = Position::new(2, 6);
  let center = Position::new(6, 6);
  let probe = Game::new(seed, 12, 12, player_position).unwrap();
  let (blast_positions, blast_damages, _) = rocket_blast_plan(&probe, center);
  let hot_position = blast_positions
    .iter()
    .zip(blast_damages.iter())
    .skip(1)
    .find_map(|(position, damage)| should_destroy_ground_item(*damage).then_some(*position))
    .expect("fixture seed must include a destructive blast cell");
  let player_config = PlayerSpawnConfig {
    hp: 50,
    max_hp: 50,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::RocketLauncher),
    equipped_armor: None,
    equipped_armor_durability: None,
  };
  let mut setup = ReplayLog::new(seed, 12, 12, player_position).with_player_config(player_config);
  setup.record_monster(MonsterSpawnSpec::new(
    center,
    "Replay Target",
    500,
    0,
    (0, 0),
  ));
  setup.record_item(ItemSpawnSpec::new(hot_position, ItemSpawnKind::Shotgun));
  setup.record_item(ItemSpawnSpec::new(
    hot_position,
    ItemSpawnKind::SmallMedPack,
  ));
  setup.record_command(Command::AttackRanged(center));

  let (direct, expected_events) = ReplayEngine::run(&setup).expect("direct replay should resolve");
  let destroyed_id = expected_events
    .iter()
    .find_map(|event| match event {
      GameEvent::GroundItemDestroyed { item_id, .. } => Some(*item_id),
      _ => None,
    })
    .expect("the replay fixture should destroy one ground item");
  let item_ids: Vec<_> = direct.world().ground_items().keys().copied().collect();
  assert_eq!(item_ids.len(), 1);
  assert_ne!(destroyed_id, item_ids[0]);

  let (replayed, replay_events) = ReplayEngine::run(&setup).expect("replay should be repeatable");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, expected_events);
  assert!(ReplayEngine::verify_determinism(&setup).expect("Rocket replay should be deterministic"));
}
