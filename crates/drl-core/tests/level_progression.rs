//! Integration tests for procedural level generation, stairs descent, and multi-level progression.

use drl_core::generator::{LevelGenerator, LevelGeneratorConfig};
use drl_core::grid::Tile;
use drl_core::item::Item;
use drl_core::{Game, GameRng, ReplayEngine};
use drl_protocol::{
  Command, CommandError, Direction, EquipmentSlot, GameEvent, LevelId, MonsterSpawnSpec, Position,
  ReplayLog,
};

#[test]
fn test_procedural_generator_connectivity_across_diverse_seeds() {
  let config = LevelGeneratorConfig {
    width: 40,
    height: 25,
    max_rooms: 8,
    min_room_size: 4,
    max_room_size: 8,
    max_monsters_per_room: 2,
    max_items_per_room: 2,
  };

  for seed in [1, 42, 100, 777, 9999, 12345678] {
    let mut rng = GameRng::from_seed(seed);
    let mut item_counter = 0;
    let level = LevelGenerator::generate(&config, &mut rng, &mut item_counter);

    assert!(
      level.map.is_walkable(level.player_spawn),
      "Player spawn must be walkable for seed {seed}"
    );
    assert_eq!(
      level.map.get_tile(level.stairs_position),
      Some(Tile::StairsDown),
      "StairsDown must be placed at stairs_position for seed {seed}"
    );
    assert!(
      LevelGenerator::verify_connectivity(&level.map, level.player_spawn, level.stairs_position),
      "Path must exist between player spawn and exit stairs for seed {seed}"
    );
    assert!(
      !level.rooms.is_empty(),
      "Level must have generated rooms for seed {seed}"
    );
  }
}

#[test]
fn test_stairs_descent_requires_stairs_tile() {
  let mut game = Game::new(1234, 15, 15, Position::new(5, 5)).unwrap();

  // Player at (5, 5) which is Floor
  let err = game.step(Command::Descend).unwrap_err();
  assert_eq!(err, CommandError::NotOnStairs(Position::new(5, 5)));
  assert_eq!(game.world().level_id(), LevelId::new(1));

  // Place stairs at (5, 6), move onto it, then descend
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(5, 6), Tile::StairsDown);

  let move_events = game.step(Command::Move(Direction::South)).unwrap();
  assert!(
    move_events
      .iter()
      .any(|e| matches!(e, GameEvent::EntityMoved { .. }))
  );
  assert_eq!(
    game.world().player().unwrap().position(),
    Position::new(5, 6)
  );

  let descend_events = game.step(Command::Descend).unwrap();
  assert!(descend_events.iter().any(|e| matches!(
    e,
    GameEvent::LevelTransitioned {
      from_level,
      to_level,
    } if *from_level == LevelId::new(1) && *to_level == LevelId::new(2)
  )));

  assert_eq!(game.world().level_id(), LevelId::new(2));
}

#[test]
fn test_player_state_persistence_across_level_transition() {
  let mut game = Game::new(4242, 20, 20, Position::new(2, 2)).unwrap();

  // Modify player state: take damage, equip armor, add ammo
  let p_id = game.world().player_id().unwrap();
  let armor_id = game.world_mut().allocate_item_id();
  let armor = Item::green_armor(armor_id);
  let extra_ammo_id = game.world_mut().allocate_item_id();
  let extra_ammo = Item::ammo_shells(extra_ammo_id, 16);

  {
    let player = game.world_mut().get_actor_mut(p_id).unwrap();
    player.hp_mut().take_damage(15);
    let _ = player.equipment_mut().equip(EquipmentSlot::Armor, armor);
    let _ = player.inventory_mut().add_item(extra_ammo);
  }

  assert_eq!(game.world().player().unwrap().hp().current, 35);
  assert!(game.world().player().unwrap().equipment().armor().is_some());
  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .armor()
      .unwrap()
      .name(),
    "Green Armor"
  );

  // Set stairs under player and descend
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(2, 2), Tile::StairsDown);

  let events = game.step(Command::Descend).unwrap();
  assert!(events.iter().any(|e| matches!(
    e,
    GameEvent::LevelTransitioned {
      from_level,
      to_level,
    } if *from_level == LevelId::new(1) && *to_level == LevelId::new(2)
  )));

  // Verify player on Level 2 has preserved stats and items
  assert_eq!(game.world().level_id(), LevelId::new(2));
  let p2 = game.world().player().unwrap();
  assert_eq!(p2.hp().max, 50);
  assert!(p2.hp().current <= 35 && p2.hp().current >= 30);
  assert!(p2.equipment().armor().is_some());
  assert_eq!(p2.equipment().armor().unwrap().name(), "Green Armor");
  assert!(p2.equipment().weapon().is_some());
  assert_eq!(p2.equipment().weapon().unwrap().name(), "Pistol");
  assert!(p2.inventory().has_ammo(drl_protocol::AmmoType::Shells, 16));
}

#[test]
fn test_multi_level_run_replay_determinism() {
  let mut replay = ReplayLog::new(8888, 15, 15, Position::new(2, 2));
  replay.record_stairs(Position::new(2, 2));
  replay.record_monster(MonsterSpawnSpec::new(
    Position::new(8, 2),
    "Former Human",
    15,
    100,
    (2, 4),
  ));

  // 1. Move East, Move West back to (2, 2)
  replay.record_command(Command::Move(Direction::East));
  replay.record_command(Command::Move(Direction::West));
  // 2. Descend stairs at (2, 2) to Level 2
  replay.record_command(Command::Descend);
  // 3. Move on Level 2
  replay.record_command(Command::Move(Direction::East));
  replay.record_command(Command::Wait);

  // Run replay determinism check across both levels
  let is_det = ReplayEngine::verify_determinism(&replay).unwrap();
  assert!(is_det);
}
