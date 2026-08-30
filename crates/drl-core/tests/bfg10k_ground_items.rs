use drl_core::bfg10k::{radius_two_blast_positions, roll_explosion_damage};
use drl_core::game::Game;
use drl_core::item::Item;
use drl_protocol::{Command, DamageSource, DamageType, EquipmentSlot, GameEvent, ItemId, Position};

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

#[test]
fn bfg10k_destroys_lowest_id_loose_ammo_only_above_threshold() {
  let seed = 27_500;
  let center = Position::new(6, 4);
  let mut game = equipped_bfg10k(seed);
  game
    .world_mut()
    .spawn_monster(center, "Lethal Target", 1, 0, (0, 0))
    .unwrap();

  let mut expected_rng = game.rng().clone();
  expected_rng.gen_range(6..25);
  let blast_positions = radius_two_blast_positions(game.world().map(), center);
  let rolls: Vec<_> = blast_positions
    .iter()
    .map(|_| roll_explosion_damage(&mut expected_rng))
    .collect();
  let (hot_index, hot_damage) = rolls
    .iter()
    .enumerate()
    .skip(1)
    .find(|(_, damage)| **damage > 10)
    .map(|(index, damage)| (index, *damage))
    .expect("fixed seed should include a non-center destructive blast cell");
  let empty_hot_index = rolls
    .iter()
    .enumerate()
    .skip(1)
    .find(|(index, damage)| {
      *index != hot_index
        && **damage > 10
        && blast_positions[*index].distance_chebyshev(blast_positions[hot_index]) > 1
    })
    .map(|(index, _)| index)
    .expect("fixed seed should include a second non-center destructive blast cell");
  let cold_index = rolls
    .iter()
    .enumerate()
    .find(|(index, damage)| *index != hot_index && *index != empty_hot_index && **damage <= 10)
    .map(|(index, _)| index)
    .expect("fixed seed should include a non-destructive blast cell");
  let hot_position = blast_positions[hot_index];
  let empty_hot_position = blast_positions[empty_hot_index];
  let cold_position = blast_positions[cold_index];

  let durable_id = game
    .world_mut()
    .spawn_monster(hot_position, "Durable Target", 500, 0, (0, 0))
    .unwrap();
  let low_ammo_id = ItemId::new(100);
  let high_ammo_id = ItemId::new(101);
  let ammo_pack_id = ItemId::new(102);
  let weapon_id = ItemId::new(103);
  let cold_ammo_id = ItemId::new(104);
  let cold_pack_id = ItemId::new(105);
  let empty_ammo_id = ItemId::new(106);
  let empty_pack_id = ItemId::new(107);
  game
    .world_mut()
    .spawn_ground_item(hot_position, Item::ammo_cells(high_ammo_id, 4))
    .unwrap();
  game
    .world_mut()
    .spawn_ground_item(hot_position, Item::shotgun(weapon_id))
    .unwrap();
  game
    .world_mut()
    .spawn_ground_item(hot_position, Item::ammo_pack_cells(ammo_pack_id))
    .unwrap();
  game
    .world_mut()
    .spawn_ground_item(hot_position, Item::ammo_cells(low_ammo_id, 3))
    .unwrap();
  game
    .world_mut()
    .spawn_ground_item(empty_hot_position, Item::ammo_cells(empty_ammo_id, 2))
    .unwrap();
  game
    .world_mut()
    .spawn_ground_item(empty_hot_position, Item::ammo_pack_cells(empty_pack_id))
    .unwrap();
  game
    .world_mut()
    .spawn_ground_item(cold_position, Item::ammo_cells(cold_ammo_id, 2))
    .unwrap();
  game
    .world_mut()
    .spawn_ground_item(cold_position, Item::ammo_pack_cells(cold_pack_id))
    .unwrap();

  let events = game
    .step(Command::AttackRanged(center))
    .expect("BFG 10K direct fire should resolve");
  let destroyed: Vec<_> = events
    .iter()
    .filter_map(|event| match event {
      GameEvent::GroundItemDestroyed { item_id, position } => Some((*item_id, *position)),
      _ => None,
    })
    .collect();
  assert_eq!(
    destroyed,
    vec![
      (low_ammo_id, hot_position),
      (empty_ammo_id, empty_hot_position),
    ]
  );
  assert!(hot_damage > 10);

  let durable_damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::DamageApplied {
          target_id,
          source: DamageSource::Environment,
          damage_type: Some(DamageType::Plasma),
          ..
        } if *target_id == durable_id
      )
    })
    .expect("durable actor should receive the destructive cell's plasma damage");
  let destroyed_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::GroundItemDestroyed { item_id, .. } if *item_id == low_ammo_id))
    .unwrap();
  assert!(durable_damage_index < destroyed_index);

  assert!(!game.world().ground_items().contains_key(&low_ammo_id));
  assert!(game.world().ground_items().contains_key(&high_ammo_id));
  assert!(game.world().ground_items().contains_key(&ammo_pack_id));
  assert!(game.world().ground_items().contains_key(&weapon_id));
  assert!(game.world().ground_items().contains_key(&cold_ammo_id));
  assert!(game.world().ground_items().contains_key(&cold_pack_id));
  assert!(!game.world().ground_items().contains_key(&empty_ammo_id));
  assert!(game.world().ground_items().contains_key(&empty_pack_id));
  assert_eq!(game.rng(), &expected_rng);
}
