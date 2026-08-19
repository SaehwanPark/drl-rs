//! Integration tests for inventory, equipment, ground items, weapon reloading, and consumables.

use drl_core::Game;
use drl_core::item::Item;
use drl_protocol::{Command, CommandError, EquipmentSlot, GameEvent, ItemCategory, Position};

#[test]
fn test_inventory_pickup_drop_and_observation_views() {
  let mut game = Game::new(100, 20, 20, Position::new(5, 5)).unwrap();

  // Spawn a shotgun and shells on the ground at (6, 5)
  let shotgun_id = game.world_mut().allocate_item_id();
  let shotgun = Item::shotgun(shotgun_id);
  game
    .world_mut()
    .spawn_ground_item(Position::new(6, 5), shotgun)
    .unwrap();

  let shells_id = game.world_mut().allocate_item_id();
  let shells = Item::ammo_shells(shells_id, 16);
  game
    .world_mut()
    .spawn_ground_item(Position::new(6, 5), shells)
    .unwrap();

  // Step east onto (6, 5)
  game
    .step(Command::Move(drl_protocol::Direction::East))
    .unwrap();
  assert_eq!(
    game.world().player().unwrap().position(),
    Position::new(6, 5)
  );

  // Pick up first item (Shotgun)
  let pickup_events = game.step(Command::Pickup).unwrap();
  assert!(pickup_events.iter().any(|e| matches!(
    e,
    GameEvent::ItemPickedUp {
      item_name,
      ..
    } if item_name == "Shotgun"
  )));

  // Pick up second item (Shells)
  let pickup_events2 = game.step(Command::Pickup).unwrap();
  assert!(pickup_events2.iter().any(|e| matches!(
    e,
    GameEvent::ItemPickedUp {
      item_name,
      ..
    } if item_name == "Shotgun Shells"
  )));

  // Check player observation reflects new inventory
  let obs = game.observe_player();
  let names: Vec<String> = obs.inventory.iter().map(|i| i.name.clone()).collect();
  assert!(names.contains(&"Shotgun".to_string()));
  assert!(names.contains(&"Shotgun Shells".to_string()));

  // Drop Shotgun back to the ground
  let drop_events = game.step(Command::Drop(shotgun_id)).unwrap();
  assert!(drop_events.iter().any(|e| matches!(
    e,
    GameEvent::ItemDropped {
      item_id,
      position,
      ..
    } if *item_id == shotgun_id && *position == Position::new(6, 5)
  )));

  // Check observation shows ground item at (6, 5)
  let obs2 = game.observe_player();
  assert!(
    obs2
      .ground_items
      .iter()
      .any(|gi| gi.position == Position::new(6, 5) && gi.item.name == "Shotgun")
  );
}

#[test]
fn test_equipment_weapon_swap_and_armor_protection() {
  let mut game = Game::new(200, 20, 20, Position::new(5, 5)).unwrap();

  // Add shotgun to inventory
  let shotgun_id = game.world_mut().allocate_item_id();
  let shotgun = Item::shotgun(shotgun_id);
  game
    .world_mut()
    .player_mut()
    .unwrap()
    .inventory_mut()
    .add_item(shotgun)
    .unwrap();

  // Equip shotgun
  let equip_events = game.step(Command::Equip(shotgun_id)).unwrap();
  assert!(equip_events.iter().any(|e| matches!(
    e,
    GameEvent::ItemEquipped {
      item_id,
      slot: EquipmentSlot::Weapon,
      ..
    } if *item_id == shotgun_id
  )));

  // Player's equipped weapon should now be Shotgun
  let player = game.world().player().unwrap();
  assert_eq!(player.equipment().weapon().unwrap().name(), "Shotgun");

  // Previous Pistol should now be back in inventory
  let pistol_in_inv = player
    .inventory()
    .items()
    .values()
    .any(|i| i.name() == "Pistol");
  assert!(pistol_in_inv);

  // Add and equip Green Armor
  let armor_id = game.world_mut().allocate_item_id();
  let armor = Item::green_armor(armor_id);
  game
    .world_mut()
    .player_mut()
    .unwrap()
    .inventory_mut()
    .add_item(armor)
    .unwrap();

  game.step(Command::Equip(armor_id)).unwrap();
  assert_eq!(game.world().player().unwrap().armor_protection(), 5);

  // Unequip Armor
  let unequip_events = game.step(Command::Unequip(EquipmentSlot::Armor)).unwrap();
  assert!(unequip_events.iter().any(|e| matches!(
    e,
    GameEvent::ItemUnequipped {
      slot: EquipmentSlot::Armor,
      ..
    }
  )));
  assert_eq!(game.world().player().unwrap().armor_protection(), 0);
}

#[test]
fn test_ammo_consumption_and_reload_cycle() {
  let mut game = Game::new(300, 20, 20, Position::new(2, 2)).unwrap();
  // Spawn stationary tough monster
  let m_id = game
    .world_mut()
    .spawn_monster(Position::new(5, 2), "Demon", 500, 0, (2, 4))
    .unwrap();

  // Pistol starts with 10/10 ammo. Fire 10 shots at (5, 2)
  for _ in 0..10 {
    let target = game.world().get_actor(m_id).unwrap().position();
    game.step(Command::AttackRanged(target)).unwrap();
  }

  // Next shot fails due to empty clip
  let target = game.world().get_actor(m_id).unwrap().position();
  let err = game.step(Command::AttackRanged(target)).unwrap_err();
  assert_eq!(err, CommandError::NoAmmoInClip);

  // Reload from 30x reserve 9mm ammo in inventory
  let reload_events = game.step(Command::Reload).unwrap();
  assert!(reload_events.iter().any(|e| matches!(
    e,
    GameEvent::WeaponReloaded {
      ammo_loaded: 10,
      current_clip: 10,
      max_clip: 10,
      ..
    }
  )));

  // Reserve ammo in inventory should now be 20
  let ammo_count = game
    .world()
    .player()
    .unwrap()
    .inventory()
    .items()
    .values()
    .find(|i| i.category() == ItemCategory::Ammo)
    .unwrap()
    .count();
  assert_eq!(ammo_count, 20);

  // Firing now succeeds again
  let fire_events = game.step(Command::AttackRanged(target)).unwrap();
  assert!(fire_events.iter().any(|e| matches!(
    e,
    GameEvent::AttackResolved {
      is_ranged: true,
      ..
    }
  )));
}

#[test]
fn test_medpack_use_and_health_restoration() {
  let mut game = Game::new(400, 10, 10, Position::new(3, 3)).unwrap();
  let p_id = game.world().player_id().unwrap();

  // Deduct 25 health
  game
    .world_mut()
    .get_actor_mut(p_id)
    .unwrap()
    .hp_mut()
    .take_damage(25);
  assert_eq!(game.world().player().unwrap().hp().current, 25);

  // Player starts with Small MedPack (+10 HP)
  let med_id = game
    .world()
    .player()
    .unwrap()
    .inventory()
    .find_first_by_category(ItemCategory::MedPack)
    .unwrap();

  let events = game.step(Command::Use(med_id)).unwrap();
  assert!(events.iter().any(|e| matches!(
    e,
    GameEvent::ItemUsed {
      item_name,
      ..
    } if item_name == "Small MedPack"
  )));

  // HP should now be 35
  assert_eq!(game.world().player().unwrap().hp().current, 35);

  // MedPack should no longer be in inventory
  assert!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(med_id)
      .is_none()
  );
}

#[test]
fn test_inventory_capacity_limit_enforced() {
  let mut game = Game::new(500, 10, 10, Position::new(1, 1)).unwrap();
  // Fill up inventory to default capacity (10 items)
  let mut current_len = game.world().player().unwrap().inventory().len();
  while current_len < 10 {
    let med_id = game.world_mut().allocate_item_id();
    let med = Item::large_medpack(med_id);
    game
      .world_mut()
      .player_mut()
      .unwrap()
      .inventory_mut()
      .add_item(med)
      .unwrap();
    current_len += 1;
  }
  assert_eq!(game.world().player().unwrap().inventory().len(), 10);

  // Spawn an 11th item on the ground at player position
  let knife_id = game.world_mut().allocate_item_id();
  let knife = Item::combat_knife(knife_id);
  game
    .world_mut()
    .spawn_ground_item(Position::new(1, 1), knife)
    .unwrap();

  // Attempting pickup should fail with InventoryFull
  let err = game.step(Command::Pickup).unwrap_err();
  assert_eq!(err, CommandError::InventoryFull);

  // Item should remain on the ground
  assert_eq!(game.world().ground_items_at(Position::new(1, 1)).len(), 1);
}
