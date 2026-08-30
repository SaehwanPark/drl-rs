use drl_core::bfg10k::{radius_two_blast_positions, roll_explosion_damage};
use drl_core::game::Game;
use drl_core::item::Item;
use drl_protocol::{
  Command, DamageSource, DamageType, EquipmentSlot, GameEvent, ItemSpawnKind, Position,
};

fn equipped_bfg10k(seed: u64) -> Game {
  let mut game = Game::new(seed, 14, 10, Position::new(2, 4)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::bfg10k(weapon_id))
    .unwrap();
  game
}

fn environment_damage_for(events: &[GameEvent], target_id: drl_protocol::EntityId) -> Vec<u32> {
  events
    .iter()
    .filter_map(|event| match event {
      GameEvent::DamageApplied {
        target_id: event_target,
        amount,
        source: DamageSource::Environment,
        damage_type: Some(DamageType::Plasma),
        ..
      } if *event_target == target_id => Some(*amount),
      _ => None,
    })
    .collect()
}

#[test]
fn bfg10k_radius_two_splash_hits_each_actor_once_in_stable_order() {
  let mut game = equipped_bfg10k(27_100);
  let center = Position::new(6, 4);
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(7, 4), drl_core::grid::Tile::Wall);
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(7, 6), drl_core::grid::Tile::Wall);
  let target_id = game
    .world_mut()
    .spawn_monster(center, "Center Target", 500, 0, (1, 7))
    .unwrap();
  let neighbor_id = game
    .world_mut()
    .spawn_monster(Position::new(6, 5), "Neighbor Target", 500, 0, (1, 7))
    .unwrap();

  let events = game
    .step(Command::AttackRanged(center))
    .expect("BFG 10K direct fire should resolve");

  let schedule_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
    .expect("direct hit should schedule the BFG 10K explosion");
  let first_splash_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          source: DamageSource::Environment,
          damage_type: Some(DamageType::Plasma),
          ..
        }
      )
    })
    .expect("BFG 10K splash should apply plasma damage");
  assert!(schedule_index < first_splash_index);

  let center_damage = environment_damage_for(&events, target_id);
  let neighbor_damage = environment_damage_for(&events, neighbor_id);
  assert_eq!(center_damage.len(), 5);
  assert_eq!(neighbor_damage.len(), 5);
  assert!(center_damage[0] >= 6 && center_damage[0] <= 24);
  assert!(neighbor_damage[0] >= 6 && neighbor_damage[0] <= 24);

  let plasma_damage_events = events
    .iter()
    .filter(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          source: DamageSource::Environment,
          damage_type: Some(DamageType::Plasma),
          ..
        }
      )
    })
    .count();
  assert_eq!(plasma_damage_events, 10);
}

#[test]
fn bfg10k_splash_consumes_one_six_d_four_roll_per_clear_blast_cell() {
  let seed = 27_101;
  let mut game = equipped_bfg10k(seed);
  let center = Position::new(6, 4);
  game
    .world_mut()
    .spawn_monster(center, "Static Target", 500, 0, (1, 7))
    .unwrap();

  let mut expected_rng = game.rng().clone();
  for _ in 0..5 {
    let _direct_damage = expected_rng.gen_range(6..25);
    for _ in radius_two_blast_positions(game.world().map(), center) {
      let _ = roll_explosion_damage(&mut expected_rng);
    }
  }

  game
    .step(Command::AttackRanged(center))
    .expect("BFG 10K direct fire should resolve");
  assert_eq!(game.rng(), &expected_rng);
}

#[test]
fn bfg10k_splash_knockback_precedes_lethal_death_drop() {
  let center = Position::new(6, 4);
  let mut selected = None;
  for seed in 27_102..27_200 {
    let mut game = equipped_bfg10k(seed);
    game
      .world_mut()
      .spawn_monster(center, "Durable Target", 500, 0, (1, 7))
      .unwrap();
    let victim_id = game
      .world_mut()
      .spawn_monster(Position::new(6, 5), "Fragile Target", 1, 0, (1, 7))
      .unwrap();
    game
      .world_mut()
      .get_actor_mut(victim_id)
      .unwrap()
      .set_death_drop(Some(ItemSpawnKind::AmmoCells(2)));
    let events = game
      .step(Command::AttackRanged(center))
      .expect("BFG 10K direct fire should resolve");
    if events.iter().any(|event| {
      matches!(event, GameEvent::ActorKnockedBack { entity_id, .. } if *entity_id == victim_id)
    }) {
      selected = Some((game, events, victim_id));
      break;
    }
  }
  let (game, events, victim_id) = selected.expect("a fixed seed should exercise splash knockback");
  let knockback_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::ActorKnockedBack { entity_id, .. } if *entity_id == victim_id))
    .expect("lethal splash victim should be displaced when the roll reaches 16");
  let damage_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::DamageApplied { target_id, source: DamageSource::Environment, .. } if *target_id == victim_id))
    .expect("lethal splash victim should receive environmental damage");
  let death_index = events
    .iter()
    .position(
      |event| matches!(event, GameEvent::ActorDied { entity_id, .. } if *entity_id == victim_id),
    )
    .expect("lethal splash victim should die");
  let drop_index = events
    .iter()
    .position(
      |event| matches!(event, GameEvent::ItemDropped { entity_id, .. } if *entity_id == victim_id),
    )
    .expect("lethal splash victim should drop its configured item");
  assert!(knockback_index < damage_index);
  assert!(damage_index < death_index);
  assert!(death_index < drop_index);
  assert!(!game.world().get_actor(victim_id).unwrap().is_alive());
  assert_eq!(game.world().ground_items().len(), 1);
}
