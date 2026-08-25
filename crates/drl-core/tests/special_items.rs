//! Integration tests for special-use consumable items (Phase Device teleportation).

use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::replay::ReplayEngine;
use drl_protocol::{
  Command, Direction, EquipmentSlot, GameEvent, ItemSpawnKind, ItemSpawnSpec, PlayerSpawnConfig,
  Position, ReplayLog,
};

#[test]
fn test_phase_device_use_teleports_player_and_updates_visibility() {
  let mut game = Game::new(9999, 20, 20, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();

  // Add Phase Device to player inventory
  let device_id = game.world_mut().allocate_item_id();
  let device = Item::phase_device(device_id);
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .inventory_mut()
    .add_item(device)
    .unwrap();

  let initial_pos = game.world().player().unwrap().position();
  assert_eq!(initial_pos, Position::new(2, 2));

  // Use Phase Device
  let events = game.step(Command::Use(device_id)).unwrap();

  // Verify PlayerTeleported event was emitted
  let teleport_event = events.iter().find_map(|e| match e {
    GameEvent::PlayerTeleported { from, to } => Some((*from, *to)),
    _ => None,
  });

  assert!(
    teleport_event.is_some(),
    "PlayerTeleported event must be emitted"
  );
  let (from, to) = teleport_event.unwrap();
  assert_eq!(from, initial_pos);
  assert_ne!(to, from);

  // Verify player position was updated and is within walkable map bounds
  let current_pos = game.world().player().unwrap().position();
  assert_eq!(current_pos, to);
  assert!(game.world().map().is_in_bounds(current_pos));
  assert!(game.world().map().is_walkable(current_pos));

  // Verify item was consumed from inventory
  assert!(
    game
      .world()
      .player()
      .unwrap()
      .inventory()
      .get_item(device_id)
      .is_none(),
    "Phase device should be consumed on use"
  );

  // Verify player exploration memory includes new position
  assert!(
    game.world().is_explored(current_pos),
    "New position must be explored in fog of war"
  );
}

#[test]
fn test_phase_device_pickup_and_replay_determinism() {
  let mut replay = ReplayLog::new(5555, 15, 15, Position::new(2, 2));
  replay.record_item(ItemSpawnSpec::new(
    Position::new(3, 2),
    ItemSpawnKind::PhaseDevice,
  ));

  // 1. Move East onto Phase Device
  replay.record_command(Command::Move(Direction::East));
  // 2. Pick up Phase Device
  replay.record_command(Command::Pickup);
  // 3. Move North
  replay.record_command(Command::Move(Direction::North));
  // 4. Wait
  replay.record_command(Command::Wait);

  let is_det = ReplayEngine::verify_determinism(&replay).unwrap();
  assert!(
    is_det,
    "Replay with Phase Device pickup must be deterministic"
  );
}

#[test]
fn medical_powerarmor_repairs_on_the_thirtieth_accepted_command() {
  let mut game = Game::new_arena(777, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let armor_id = game.world_mut().allocate_item_id();
  let armor = Item::medical_powerarmor(armor_id);
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player.hp_mut().take_damage(30);
  player
    .equipment_mut()
    .equip(EquipmentSlot::Armor, armor)
    .unwrap();

  for _ in 0..29 {
    let events = game.step(Command::Wait).unwrap();
    assert!(
      !events
        .iter()
        .any(|event| matches!(event, GameEvent::MedicalPowerarmorRepaired { .. }))
    );
  }
  assert_eq!(game.world().player().unwrap().hp().current, 20);
  assert_eq!(game.world().player().unwrap().medical_repair_timer(), 29);

  let events = game.step(Command::Wait).unwrap();
  let repair_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        GameEvent::MedicalPowerarmorRepaired {
          entity_id,
          item_id,
          healed: 1,
          remaining_hp: 21,
          durability_remaining: 99,
          timer: 20,
        } if *entity_id == player_id && *item_id == armor_id
      )
    })
    .expect("repair event must be emitted");
  assert_eq!(repair_index, 2);
  assert!(matches!(events[0], GameEvent::TurnStarted { .. }));
  assert!(matches!(events[1], GameEvent::EntityWaited { .. }));
  assert!(matches!(events[3], GameEvent::ActionCostPaid { .. }));
  assert!(matches!(events[4], GameEvent::TurnEnded { .. }));
  assert_eq!(game.world().player().unwrap().hp().current, 21);
  assert_eq!(game.world().player().unwrap().medical_repair_timer(), 20);
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .armor()
      .unwrap()
      .armor_properties()
      .unwrap()
      .durability,
    99
  );
}

#[test]
fn medical_powerarmor_replay_events_are_deterministic() {
  let mut replay =
    ReplayLog::new(778, 12, 12, Position::new(1, 1)).with_player_config(PlayerSpawnConfig {
      hp: 20,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::Pistol),
      equipped_armor: Some(ItemSpawnKind::MedicalPowerarmor),
    });
  for _ in 0..30 {
    replay.record_command(Command::Wait);
  }

  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
  let (_game, events) = ReplayEngine::run(&replay).unwrap();
  let repair_index = events
    .iter()
    .position(|event| matches!(event, GameEvent::MedicalPowerarmorRepaired { .. }))
    .expect("replay must include the accepted-turn repair event");
  assert!(repair_index >= 2);
  assert!(matches!(
    events[repair_index - 2],
    GameEvent::TurnStarted { .. }
  ));
  assert!(matches!(
    events[repair_index - 1],
    GameEvent::EntityWaited { .. }
  ));
  assert!(matches!(
    events[repair_index + 1],
    GameEvent::ActionCostPaid { .. }
  ));
  assert!(matches!(
    events[repair_index + 2],
    GameEvent::TurnEnded { .. }
  ));
  assert!(matches!(
    events[repair_index],
    GameEvent::MedicalPowerarmorRepaired {
      healed: 1,
      remaining_hp: 21,
      durability_remaining: 99,
      timer: 20,
      ..
    }
  ));
}

#[test]
fn rejected_commands_roll_back_medical_repair_state() {
  let mut game = Game::new(779, 5, 5, Position::new(1, 1)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let armor_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player.hp_mut().take_damage(30);
  player
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::medical_powerarmor(armor_id))
    .unwrap();
  for _ in 0..7 {
    game.step(Command::Wait).unwrap();
  }
  let before = game.clone();

  assert!(game.step(Command::Move(Direction::North)).is_err());
  assert_eq!(game, before);
}

#[test]
fn medical_powerarmor_timer_moves_with_the_equipped_item() {
  let mut game = Game::new_arena(780, 12, 12).unwrap();
  let player_id = game.world().player_id().unwrap();
  let armor_id = game.world_mut().allocate_item_id();
  let player = game.world_mut().get_actor_mut(player_id).unwrap();
  player.hp_mut().take_damage(30);
  player
    .equipment_mut()
    .equip(EquipmentSlot::Armor, Item::medical_powerarmor(armor_id))
    .unwrap();
  for _ in 0..7 {
    game.step(Command::Wait).unwrap();
  }
  assert_eq!(game.world().player().unwrap().medical_repair_timer(), 7);

  game.step(Command::Unequip(EquipmentSlot::Armor)).unwrap();
  assert_eq!(game.world().player().unwrap().medical_repair_timer(), 0);
  game.step(Command::Equip(armor_id)).unwrap();
  assert_eq!(game.world().player().unwrap().medical_repair_timer(), 8);
}
