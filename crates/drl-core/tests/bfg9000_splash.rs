use drl_core::bfg9000::{radius_eight_blast_positions, roll_explosion_damage};
use drl_core::game::Game;
use drl_core::item::Item;
use drl_protocol::{
  Command, DamageSource, DamageType, EquipmentSlot, GameEvent, ItemSpawnKind, Position,
};

fn equipped_bfg9000(seed: u64) -> Game {
  let mut game = Game::new(seed, 24, 24, Position::new(12, 12)).unwrap();
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
fn bfg9000_radius_eight_splash_is_actor_only_and_self_safe() {
  let seed = 30_000;
  let center = Position::new(14, 12);
  let player_position = Position::new(12, 12);
  let mut game = equipped_bfg9000(seed);
  let player_id = game.world().player_id().unwrap();
  let center_id = game
    .world_mut()
    .spawn_monster(center, "Center Target", 500, 0, (0, 0))
    .unwrap();
  let ring_id = game
    .world_mut()
    .spawn_monster(Position::new(16, 12), "Ring Target", 500, 0, (0, 0))
    .unwrap();
  let ground_item_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .spawn_ground_item(Position::new(17, 12), Item::ammo_cells(ground_item_id, 2))
    .unwrap();
  assert_eq!(game.world().player().unwrap().position(), player_position);

  let mut expected_rng = game.rng().clone();
  expected_rng.gen_range(10..61);
  for _ in radius_eight_blast_positions(game.world().map(), center) {
    roll_explosion_damage(&mut expected_rng);
  }

  let events = game
    .step(Command::AttackRanged(center))
    .expect("Standard BFG 9000 direct fire should resolve");

  assert_eq!(
    environment_damage_for(&events, player_id),
    Vec::<u32>::new()
  );
  assert_eq!(environment_damage_for(&events, center_id).len(), 1);
  assert_eq!(environment_damage_for(&events, ring_id).len(), 1);
  assert_eq!(game.rng(), &expected_rng);
  assert!(
    events
      .iter()
      .all(|event| { !matches!(event, GameEvent::GroundItemDestroyed { .. }) })
  );
  assert!(game.world().ground_items().contains_key(&ground_item_id));
}

#[test]
fn bfg9000_splash_preserves_knockback_before_lethal_death_drop() {
  let center = Position::new(14, 12);
  let victim_position = Position::new(14, 13);
  let mut selected = None;
  for seed in 30_001..30_020 {
    let mut game = equipped_bfg9000(seed);
    let center_id = game
      .world_mut()
      .spawn_monster(center, "Durable Target", 500, 0, (0, 0))
      .unwrap();
    let victim_id = game
      .world_mut()
      .spawn_monster(victim_position, "Fragile Target", 1, 0, (0, 0))
      .unwrap();
    game
      .world_mut()
      .get_actor_mut(victim_id)
      .unwrap()
      .set_death_drop(Some(ItemSpawnKind::SmallMedPack));
    let events = game
      .step(Command::AttackRanged(center))
      .expect("Standard BFG 9000 direct fire should resolve");
    if events.iter().any(|event| {
      matches!(event, GameEvent::ActorKnockedBack { entity_id, .. } if *entity_id == victim_id)
    }) {
      selected = Some((game, events, center_id, victim_id));
      break;
    }
  }

  let (game, events, center_id, victim_id) =
    selected.expect("a fixed seed should exercise Standard BFG 9000 knockback");
  let knockback_index = events
    .iter()
    .position(|event| {
      matches!(event, GameEvent::ActorKnockedBack { entity_id, .. } if *entity_id == victim_id)
    })
    .unwrap();
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id,
          source: DamageSource::Environment,
          damage_type: Some(DamageType::Plasma),
          ..
        } if *target_id == victim_id
      )
    })
    .unwrap();
  let death_index = events
    .iter()
    .position(
      |event| matches!(event, GameEvent::ActorDied { entity_id, .. } if *entity_id == victim_id),
    )
    .unwrap();
  let drop_index = events
    .iter()
    .position(
      |event| matches!(event, GameEvent::ItemDropped { entity_id, .. } if *entity_id == victim_id),
    )
    .unwrap();
  assert!(knockback_index < damage_index);
  assert!(damage_index < death_index);
  assert!(death_index < drop_index);
  assert_eq!(environment_damage_for(&events, center_id).len(), 1);
  assert!(!game.world().get_actor(victim_id).unwrap().is_alive());
  assert_eq!(game.world().ground_items().len(), 1);
}
