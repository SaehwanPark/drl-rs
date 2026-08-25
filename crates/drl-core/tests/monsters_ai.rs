//! Integration tests for monster archetypes, tactical AI behaviors, and death loot drops.

use drl_core::game::Game;
use drl_core::grid::Tile;
use drl_core::replay::ReplayEngine;
use drl_protocol::{
  Command, Direction, GameEvent, MonsterKind, MonsterSpawnSpec, Position, ReplayLog,
};

#[test]
fn test_ranged_monster_attacks_player_with_line_of_sight() {
  let mut game = Game::new(12345, 20, 20, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();

  // Spawn a Former Human (pistol zombie) at (6, 2) - distance 4, within range 7
  let monster_id = game
    .world_mut()
    .spawn_monster_kind(Position::new(6, 2), MonsterKind::FormerHuman)
    .unwrap();

  // Player waits in place; monster should execute a ranged attack!
  let events = game.step(Command::Wait).unwrap();

  let ranged_attack_occurred = events.iter().any(|e| {
    matches!(
      e,
      GameEvent::AttackResolved {
        attacker_id,
        target_id,
        is_ranged: true,
        ..
      } if *attacker_id == monster_id && *target_id == player_id
    )
  });

  assert!(
    ranged_attack_occurred,
    "Monster should have resolved a ranged attack against player"
  );
}

#[test]
fn test_blocked_monster_closes_distance_to_player() {
  let mut game = Game::new(12345, 20, 20, Position::new(2, 2)).unwrap();
  let monster_id = game
    .world_mut()
    .spawn_monster_kind(Position::new(6, 2), MonsterKind::FormerHuman)
    .unwrap();

  // Place a wall blocking direct line of fire at (4, 2)
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(4, 2), Tile::Wall);

  // Player waits in place; monster cannot shoot, so it moves towards player
  let events = game.step(Command::Wait).unwrap();

  let monster_moved = events.iter().any(|e| {
    matches!(
      e,
      GameEvent::EntityMoved {
        entity_id,
        from,
        to,
      } if *entity_id == monster_id && *from == Position::new(6, 2) && to.x < 6
    )
  });

  assert!(monster_moved, "Monster should have moved towards player");
}

#[test]
fn test_blocked_diagonal_monster_uses_legacy_cardinal_fallback() {
  let mut game = Game::new(12345, 20, 20, Position::new(4, 4)).unwrap();
  let monster_id = game
    .world_mut()
    .spawn_monster_kind(Position::new(6, 6), MonsterKind::Demon)
    .unwrap();
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(5, 5), Tile::Wall);
  let before_rng = game.rng().clone();

  let events = game.step(Command::Wait).unwrap();

  assert!(events.iter().any(|event| matches!(
    event,
    GameEvent::EntityMoved {
      entity_id,
      from: Position { x: 6, y: 6 },
      to: Position { x: 5, y: 6 },
    } if *entity_id == monster_id
  )));
  assert_eq!(game.rng(), &before_rng);
}

#[test]
fn test_monster_death_spawns_configured_loot_drop() {
  let mut game = Game::new(12345, 20, 20, Position::new(2, 2)).unwrap();

  // Spawn Former Sergeant with shotgun shells drop at (3, 2)
  let sergeant_id = game
    .world_mut()
    .spawn_monster_kind(Position::new(3, 2), MonsterKind::FormerSergeant)
    .unwrap();

  // Set sergeant HP low so 1 bump attack kills it
  if let Some(m) = game.world_mut().get_actor_mut(sergeant_id) {
    m.hp_mut().take_damage(24); // Remaining HP = 1
  }

  // Player steps East into (3, 2) -> bump attack kills sergeant
  let events = game.step(Command::Move(Direction::East)).unwrap();

  let death_event = events.iter().any(|e| {
    matches!(
      e,
      GameEvent::ActorDied {
        entity_id,
        ..
      } if *entity_id == sergeant_id
    )
  });
  assert!(death_event, "Former Sergeant should have died");

  let drop_event = events.iter().any(|e| {
    matches!(
      e,
      GameEvent::ItemDropped {
        entity_id,
        position,
        item_name,
        ..
      } if *entity_id == sergeant_id && *position == Position::new(3, 2) && item_name.contains("Shells")
    )
  });
  assert!(
    drop_event,
    "Former Sergeant should have dropped Shotgun Shells on death"
  );

  // Verify item is now on the ground at (3, 2)
  let items_at_pos = game.world().ground_items_at(Position::new(3, 2));
  assert_eq!(items_at_pos.len(), 1);
  assert_eq!(items_at_pos[0].name(), "Shotgun Shells");
}

#[test]
fn test_demon_high_speed_turn_frequency() {
  let mut game = Game::new(12345, 20, 20, Position::new(2, 2)).unwrap();
  let demon_id = game
    .world_mut()
    .spawn_monster_kind(Position::new(8, 2), MonsterKind::Demon)
    .unwrap();

  // Demon has speed 130 (moves faster than standard speed 100)
  // Let player wait 3 turns; demon should close the 6-tile distance rapidly
  for _ in 0..3 {
    let _ = game.step(Command::Wait).unwrap();
  }

  let demon_pos = game.world().get_actor(demon_id).unwrap().position();
  let dist = Position::new(2, 2).distance_chebyshev(demon_pos);
  assert!(
    dist <= 3,
    "Fast Demon should have rapidly closed distance towards player, current dist: {dist}"
  );
}

#[test]
fn test_multi_archetype_combat_replay_determinism() {
  let mut replay = ReplayLog::new(7777, 20, 20, Position::new(2, 2));
  replay.record_monster(
    MonsterSpawnSpec::new(Position::new(7, 2), "Former Sergeant", 25, 90, (3, 6))
      .with_ranged_combat((8, 14), 5, 60),
  );
  replay.record_monster(MonsterSpawnSpec::new(
    Position::new(10, 2),
    "Demon",
    45,
    130,
    (8, 16),
  ));

  replay.record_command(Command::AttackRanged(Position::new(7, 2)));
  replay.record_command(Command::AttackRanged(Position::new(7, 2)));
  replay.record_command(Command::Move(Direction::West));
  replay.record_command(Command::Wait);
  replay.record_command(Command::Reload);

  let is_det = ReplayEngine::verify_determinism(&replay).unwrap();
  assert!(
    is_det,
    "Multi-archetype combat replay must be bit-exact deterministic"
  );
}
