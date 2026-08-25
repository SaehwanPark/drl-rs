//! Rejected command invariants for the deterministic simulation kernel.

use drl_core::{Game, Item, Tile};
use drl_protocol::{Command, CommandError, Direction, ItemCategory, ItemSpawnKind, Position};

fn assert_rejected_command_is_atomic(
  game: &mut Game,
  command: Command,
  expected_error: CommandError,
) {
  let before = game.clone();
  let error = game.step(command).expect_err("command should be rejected");

  assert_eq!(error, expected_error);
  assert_eq!(
    game, &before,
    "rejected commands must not mutate Game state"
  );
}

#[test]
fn representative_rejection_matrix_preserves_exact_game_and_rng() {
  let mut game = Game::new(3_200, 5, 5, Position::new(1, 1)).unwrap();
  let missing_item_id = drl_protocol::ItemId::new(u64::MAX);
  let cases = [
    (
      Command::Move(Direction::North),
      CommandError::BlockedByTerrain(Position::new(1, 0)),
    ),
    (
      Command::AttackMelee(Direction::None),
      CommandError::InvalidDirection(Direction::None),
    ),
    (
      Command::AttackRanged(Position::new(-1, 1)),
      CommandError::OutOfBounds(Position::new(-1, 1)),
    ),
    (
      Command::Pickup,
      CommandError::NoItemAtPosition(Position::new(1, 1)),
    ),
    (
      Command::Drop(missing_item_id),
      CommandError::ItemNotFound(missing_item_id),
    ),
    (
      Command::Equip(missing_item_id),
      CommandError::ItemNotFound(missing_item_id),
    ),
    (
      Command::Unequip(drl_protocol::EquipmentSlot::Armor),
      CommandError::SlotEmpty(drl_protocol::EquipmentSlot::Armor),
    ),
    (
      Command::Use(missing_item_id),
      CommandError::ItemNotFound(missing_item_id),
    ),
    (
      Command::Invoke(missing_item_id),
      CommandError::CannotInvoke(missing_item_id),
    ),
    (
      Command::AltReload {
        item_id: missing_item_id,
        confirmed: true,
      },
      CommandError::CannotAltReload(missing_item_id),
    ),
    (Command::Reload, CommandError::ClipAlreadyFull),
    (
      Command::Descend,
      CommandError::NotOnStairs(Position::new(1, 1)),
    ),
  ];

  for (command, expected_error) in cases {
    assert_rejected_command_is_atomic(&mut game, command, expected_error);
  }
}

#[test]
fn ranged_attack_out_of_range_preserves_ammo_and_rng() {
  let mut game = Game::new(1, 20, 20, Position::new(2, 2)).unwrap();
  let monster_position = Position::new(12, 2);
  game
    .world_mut()
    .spawn_monster(monster_position, "Demon", 100, 0, (2, 4))
    .unwrap();

  assert_rejected_command_is_atomic(
    &mut game,
    Command::AttackRanged(monster_position),
    CommandError::TargetOutOfRange(monster_position),
  );
}

#[test]
fn ranged_attack_blocked_by_wall_preserves_ammo_and_rng() {
  let mut game = Game::new(2, 20, 20, Position::new(2, 2)).unwrap();
  let monster_position = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(monster_position, "Demon", 100, 0, (2, 4))
    .unwrap();
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(3, 2), Tile::Wall);

  assert_rejected_command_is_atomic(
    &mut game,
    Command::AttackRanged(monster_position),
    CommandError::LineOfSightBlocked(monster_position),
  );
}

#[test]
fn rejected_command_restores_turn_and_rng_state() {
  let mut game = Game::new(2_024, 10, 10, Position::new(2, 2)).unwrap();
  let before_turn = game.turn();
  let before_rng = game.rng().clone();

  assert_eq!(
    game.step(Command::AttackRanged(Position::new(99, 99))),
    Err(CommandError::OutOfBounds(Position::new(99, 99)))
  );
  assert_eq!(game.turn(), before_turn);
  assert_eq!(game.rng(), &before_rng);
}

#[test]
fn equipping_non_equippable_item_preserves_inventory() {
  let mut game = Game::new(3, 10, 10, Position::new(2, 2)).unwrap();
  let medpack_id = game
    .world()
    .player()
    .unwrap()
    .inventory()
    .items()
    .values()
    .find(|item| item.consumable_properties().is_some())
    .unwrap()
    .id();

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Equip(medpack_id),
    CommandError::CannotEquip(medpack_id),
  );
}

#[test]
fn pickup_with_full_backpack_preserves_partial_ammo_merge() {
  let mut game = Game::new(4, 10, 10, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();

  let extra_ammo_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::ammo_9mm(extra_ammo_id, 65))
    .unwrap();

  for _ in 0..8 {
    let item_id = game.world_mut().allocate_item_id();
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .inventory_mut()
      .add_item(Item::small_medpack(item_id))
      .unwrap();
  }

  let ground_ammo_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .spawn_ground_item(Position::new(2, 2), Item::ammo_9mm(ground_ammo_id, 10))
    .unwrap();

  assert_rejected_command_is_atomic(&mut game, Command::Pickup, CommandError::InventoryFull);
}

#[test]
fn drop_with_out_of_bounds_position_preserves_inventory() {
  let mut game = Game::new(5, 10, 10, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let item_id = game
    .world()
    .get_actor(player_id)
    .unwrap()
    .inventory()
    .items()
    .keys()
    .next()
    .copied()
    .unwrap();

  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .set_position(Position::new(-1, 2));

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Drop(item_id),
    CommandError::OutOfBounds(Position::new(-1, 2)),
  );
}

#[test]
fn unequip_empty_slot_preserves_game_state() {
  let mut game = Game::new(6, 10, 10, Position::new(2, 2)).unwrap();

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Unequip(drl_protocol::EquipmentSlot::Armor),
    CommandError::SlotEmpty(drl_protocol::EquipmentSlot::Armor),
  );
}

#[test]
fn unequip_with_full_backpack_preserves_equipment() {
  let mut game = Game::new(7, 10, 10, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let armor_id = game.world_mut().allocate_item_id();

  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::green_armor(armor_id))
    .unwrap();
  game.step(Command::Equip(armor_id)).unwrap();

  for _ in 0..8 {
    let item_id = game.world_mut().allocate_item_id();
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .inventory_mut()
      .add_item(Item::small_medpack(item_id))
      .unwrap();
  }

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Unequip(drl_protocol::EquipmentSlot::Armor),
    CommandError::InventoryFull,
  );
}

#[test]
fn use_non_consumable_item_preserves_game_state() {
  let mut game = Game::new(8, 10, 10, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let armor_id = game.world_mut().allocate_item_id();

  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::green_armor(armor_id))
    .unwrap();

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Use(armor_id),
    CommandError::CannotUse(armor_id),
  );
}

#[test]
fn use_missing_item_preserves_game_state() {
  let mut game = Game::new(9, 10, 10, Position::new(2, 2)).unwrap();
  let missing_item_id = drl_protocol::ItemId::new(u64::MAX);

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Use(missing_item_id),
    CommandError::ItemNotFound(missing_item_id),
  );
}

#[test]
fn reload_without_ranged_weapon_preserves_game_state() {
  let mut game = Game::new(10, 10, 10, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let knife_id = game.world_mut().allocate_item_id();

  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::combat_knife(knife_id))
    .unwrap();
  game.step(Command::Equip(knife_id)).unwrap();

  assert_rejected_command_is_atomic(&mut game, Command::Reload, CommandError::NoEquippedWeapon);
}

#[test]
fn reload_with_full_clip_preserves_game_state() {
  let mut game = Game::new(11, 10, 10, Position::new(2, 2)).unwrap();

  assert_rejected_command_is_atomic(&mut game, Command::Reload, CommandError::ClipAlreadyFull);
}

#[test]
fn reload_without_matching_ammo_preserves_game_state() {
  let mut game = Game::new(12, 10, 10, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 0;

  let ammo_ids: Vec<_> = game
    .world()
    .get_actor(player_id)
    .unwrap()
    .inventory()
    .items()
    .values()
    .filter(|item| item.category() == ItemCategory::Ammo)
    .map(Item::id)
    .collect();
  for item_id in ammo_ids {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .inventory_mut()
      .remove_item(item_id)
      .unwrap();
  }

  assert_rejected_command_is_atomic(&mut game, Command::Reload, CommandError::NoMatchingAmmo);
}

#[test]
fn descend_off_stairs_preserves_game_state() {
  let mut game = Game::new(13, 10, 10, Position::new(2, 2)).unwrap();

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Descend,
    CommandError::NotOnStairs(Position::new(2, 2)),
  );
}

#[test]
fn move_into_blocked_terrain_preserves_game_state() {
  let mut game = Game::new(14, 5, 5, Position::new(1, 1)).unwrap();

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Move(drl_protocol::Direction::North),
    CommandError::BlockedByTerrain(Position::new(1, 0)),
  );
}

#[test]
fn move_out_of_bounds_preserves_game_state() {
  let mut game = Game::new(15, 5, 5, Position::new(1, 1)).unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .set_position(Position::new(4, 1));

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Move(drl_protocol::Direction::East),
    CommandError::OutOfBounds(Position::new(5, 1)),
  );
}

#[test]
fn melee_invalid_direction_preserves_game_state() {
  let mut game = Game::new(16, 10, 10, Position::new(2, 2)).unwrap();

  assert_rejected_command_is_atomic(
    &mut game,
    Command::AttackMelee(drl_protocol::Direction::None),
    CommandError::InvalidDirection(drl_protocol::Direction::None),
  );
}

#[test]
fn melee_empty_target_preserves_game_state() {
  let mut game = Game::new(17, 10, 10, Position::new(2, 2)).unwrap();

  assert_rejected_command_is_atomic(
    &mut game,
    Command::AttackMelee(drl_protocol::Direction::East),
    CommandError::InvalidTarget(Position::new(3, 2)),
  );
}

#[test]
fn equip_missing_item_preserves_game_state() {
  let mut game = Game::new(18, 10, 10, Position::new(2, 2)).unwrap();
  let missing_item_id = drl_protocol::ItemId::new(u64::MAX);

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Equip(missing_item_id),
    CommandError::ItemNotFound(missing_item_id),
  );
}

#[test]
fn drop_missing_item_preserves_game_state() {
  let mut game = Game::new(19, 10, 10, Position::new(2, 2)).unwrap();
  let missing_item_id = drl_protocol::ItemId::new(u64::MAX);

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Drop(missing_item_id),
    CommandError::ItemNotFound(missing_item_id),
  );
}

#[test]
fn pickup_without_ground_item_preserves_game_state() {
  let mut game = Game::new(20, 10, 10, Position::new(2, 2)).unwrap();

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Pickup,
    CommandError::NoItemAtPosition(Position::new(2, 2)),
  );
}

#[test]
fn pickup_with_out_of_bounds_position_preserves_game_state() {
  let mut game = Game::new(27, 10, 10, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let out_of_bounds_position = Position::new(-1, 2);
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .set_position(out_of_bounds_position);

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Pickup,
    CommandError::OutOfBounds(out_of_bounds_position),
  );
}

#[test]
fn ranged_attack_out_of_bounds_preserves_game_state() {
  let mut game = Game::new(21, 10, 10, Position::new(2, 2)).unwrap();
  let target_position = Position::new(-1, 2);

  assert_rejected_command_is_atomic(
    &mut game,
    Command::AttackRanged(target_position),
    CommandError::OutOfBounds(target_position),
  );
}

#[test]
fn ranged_attack_empty_target_preserves_game_state() {
  let mut game = Game::new(22, 10, 10, Position::new(2, 2)).unwrap();
  let target_position = Position::new(3, 2);

  assert_rejected_command_is_atomic(
    &mut game,
    Command::AttackRanged(target_position),
    CommandError::InvalidTarget(target_position),
  );
}

#[test]
fn ranged_attack_without_weapon_preserves_game_state() {
  let mut game = Game::new(23, 20, 20, Position::new(2, 2)).unwrap();
  let target_position = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target_position, "Demon", 100, 0, (2, 4))
    .unwrap();

  let player_id = game.world().player_id().unwrap();
  let knife_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::combat_knife(knife_id))
    .unwrap();
  game.step(Command::Equip(knife_id)).unwrap();

  assert_rejected_command_is_atomic(
    &mut game,
    Command::AttackRanged(target_position),
    CommandError::NoEquippedWeapon,
  );
}

#[test]
fn ranged_attack_without_ammo_preserves_game_state() {
  let mut game = Game::new(24, 20, 20, Position::new(2, 2)).unwrap();
  let target_position = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target_position, "Demon", 100, 0, (2, 4))
    .unwrap();

  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 0;

  assert_rejected_command_is_atomic(
    &mut game,
    Command::AttackRanged(target_position),
    CommandError::NoAmmoInClip,
  );
}

#[test]
fn phase_device_without_destination_preserves_game_state() {
  let mut game = Game::new(25, 3, 3, Position::new(1, 1)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let phase_device_id = game.world_mut().allocate_item_id();

  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(Item::phase_device(phase_device_id))
    .unwrap();

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Use(phase_device_id),
    CommandError::InvalidCommand("no valid teleport destination available".to_string()),
  );
}

#[test]
fn invoke_rejection_preserves_game_state() {
  let mut game = Game::new(28, 10, 10, Position::new(2, 2)).unwrap();
  let missing_item_id = drl_protocol::ItemId::new(u64::MAX);

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Invoke(missing_item_id),
    CommandError::CannotInvoke(missing_item_id),
  );
}

#[test]
fn alternate_reload_rejection_preserves_game_state() {
  let mut game = Game::new(29, 10, 10, Position::new(2, 2)).unwrap();
  let missing_item_id = drl_protocol::ItemId::new(u64::MAX);

  assert_rejected_command_is_atomic(
    &mut game,
    Command::AltReload {
      item_id: missing_item_id,
      confirmed: true,
    },
    CommandError::CannotAltReload(missing_item_id),
  );
}

#[test]
fn subtle_knife_late_death_drop_rejection_is_atomic() {
  let mut game = Game::new_arena(30, 20, 20).unwrap();
  let player_id = game.world().player_id().unwrap();
  let knife_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(
      drl_protocol::EquipmentSlot::Weapon,
      Item::subtle_knife(knife_id),
    )
    .unwrap();

  let target_position = Position::new(11, 10);
  let target_id = game
    .world_mut()
    .spawn_monster(target_position, "Dropper", 1, 0, (1, 1))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(target_id)
    .unwrap()
    .set_death_drop(Some(ItemSpawnKind::SmallMedPack));
  game
    .world_mut()
    .map_mut()
    .set_tile(target_position, Tile::Wall);

  assert_rejected_command_is_atomic(
    &mut game,
    Command::Invoke(knife_id),
    CommandError::BlockedByTerrain(target_position),
  );
  assert!(game.world().ground_items().is_empty());
}

#[test]
fn melee_late_death_drop_rejection_is_atomic() {
  let mut game = Game::new_arena(31, 20, 20).unwrap();
  let target_position = Position::new(11, 10);
  let target_id = game
    .world_mut()
    .spawn_monster(target_position, "Dropper", 1, 0, (1, 1))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(target_id)
    .unwrap()
    .set_death_drop(Some(ItemSpawnKind::SmallMedPack));
  game
    .world_mut()
    .map_mut()
    .set_tile(target_position, Tile::Wall);

  assert_rejected_command_is_atomic(
    &mut game,
    Command::AttackMelee(drl_protocol::Direction::East),
    CommandError::BlockedByTerrain(target_position),
  );
  assert!(game.world().ground_items().is_empty());
}

#[test]
fn ranged_late_death_drop_rejection_restores_clip_and_rng() {
  let mut game = Game::new_arena(3, 20, 20).unwrap();
  let target_position = Position::new(11, 10);
  let target_id = game
    .world_mut()
    .spawn_monster(target_position, "Dropper", 1, 0, (1, 1))
    .unwrap();
  game
    .world_mut()
    .get_actor_mut(target_id)
    .unwrap()
    .set_death_drop(Some(ItemSpawnKind::SmallMedPack));
  game
    .world_mut()
    .map_mut()
    .set_tile(target_position, Tile::Wall);

  assert_rejected_command_is_atomic(
    &mut game,
    Command::AttackRanged(target_position),
    CommandError::BlockedByTerrain(target_position),
  );
  assert!(game.world().ground_items().is_empty());
}

#[test]
fn command_after_game_over_preserves_game_state() {
  let mut game = Game::new(26, 10, 10, Position::new(2, 2)).unwrap();
  game
    .world_mut()
    .spawn_monster(Position::new(3, 2), "Demon", 100, 1000, (1000, 1000))
    .unwrap();

  for _ in 0..10 {
    if game.is_game_over() {
      break;
    }
    game.step(Command::Wait).unwrap();
  }

  assert!(
    game.is_game_over(),
    "the adjacent high-damage monster must kill the player"
  );
  assert_rejected_command_is_atomic(
    &mut game,
    Command::Wait,
    CommandError::InvalidCommand("game is over".to_string()),
  );
}
