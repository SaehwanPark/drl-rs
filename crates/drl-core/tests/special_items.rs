//! Integration tests for special-use consumable items (Phase Device teleportation).

use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::replay::ReplayEngine;
use drl_protocol::{
  Command, Direction, GameEvent, ItemSpawnKind, ItemSpawnSpec, Position, ReplayLog,
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
