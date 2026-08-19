//! End-to-end integration tests for Field of View (FOV), Fog-of-War, and Line of Sight (LOS).

use drl_core::{DEFAULT_VISION_RADIUS, Game, Map, Tile, compute_fov, has_line_of_sight};
use drl_protocol::{Command, CommandError, Direction, EntityId, Position};

#[test]
fn test_field_of_view_and_wall_occlusion() {
  let mut map = Map::simple_arena(20, 20);
  let center = Position::new(10, 10);

  // Place a horizontal barrier wall from (8, 12) to (12, 12)
  for x in 8..=12 {
    map.set_tile(Position::new(x, 12), Tile::Wall);
  }

  let fov = compute_fov(&map, center, DEFAULT_VISION_RADIUS);

  // Center is visible
  assert!(fov.contains(&center));

  // Front of wall facing player is visible
  assert!(fov.contains(&Position::new(10, 12)));
  assert!(fov.contains(&Position::new(8, 12)));
  assert!(fov.contains(&Position::new(12, 12)));

  // Directly behind the wall is occluded / in shadow
  assert!(!fov.contains(&Position::new(10, 13)));
  assert!(!fov.contains(&Position::new(10, 14)));

  // Flanks around the barrier are visible
  assert!(fov.contains(&Position::new(6, 13)));
  assert!(fov.contains(&Position::new(14, 13)));
}

#[test]
fn test_line_of_sight_raycasting() {
  let mut map = Map::simple_arena(20, 20);

  let from = Position::new(5, 5);
  let to = Position::new(5, 10);

  // Unobstructed LOS
  assert!(has_line_of_sight(&map, from, to));

  // Add wall obstructing LOS
  map.set_tile(Position::new(5, 7), Tile::Wall);
  assert!(!has_line_of_sight(&map, from, to));

  // Target wall cell itself is visible from `from`
  assert!(has_line_of_sight(&map, from, Position::new(5, 7)));
}

#[test]
fn test_player_observation_filters_hidden_monsters() {
  let mut game = Game::new(42, 20, 20, Position::new(5, 5)).expect("game init failed");

  // Build a dividing wall across x = 10
  for y in 0..20 {
    game
      .world_mut()
      .map_mut()
      .set_tile(Position::new(10, y), Tile::Wall);
  }

  // Monster A on visible side at (7, 5)
  let m_visible = game
    .world_mut()
    .spawn_monster(Position::new(7, 5), "Former Human", 20, 100, (2, 4))
    .expect("spawn m1");

  // Monster B on hidden side behind wall at (15, 5)
  let m_hidden = game
    .world_mut()
    .spawn_monster(Position::new(15, 5), "Arch-Vile", 50, 100, (4, 8))
    .expect("spawn m2");

  let obs = game.observe_player();
  let visible_actor_ids: Vec<EntityId> = obs.visible_actors.iter().map(|a| a.id).collect();

  // Player observation contains visible monster, but hides the occluded monster
  assert!(visible_actor_ids.contains(&m_visible));
  assert!(
    !visible_actor_ids.contains(&m_hidden),
    "hidden monster behind wall must not leak into player observation"
  );

  // Omniscient observation contains both
  let omni = game.observe_omniscient();
  let omni_actor_ids: Vec<EntityId> = omni.actors.iter().map(|a| a.id).collect();
  assert!(omni_actor_ids.contains(&m_visible));
  assert!(omni_actor_ids.contains(&m_hidden));
}

#[test]
fn test_fog_of_war_exploration_memory_expands_with_movement() {
  let mut game = Game::new(100, 30, 30, Position::new(5, 5)).expect("game init failed");

  let initial_explored_count = game.world().explored_tiles().len();
  assert!(
    initial_explored_count > 0,
    "initial spawn should explore starting FOV"
  );

  let initial_obs = game.observe_player();
  assert_eq!(initial_obs.visible_tiles.len(), initial_explored_count);

  // Step South several times to explore new areas
  for _ in 0..5 {
    game
      .step(Command::Move(Direction::South))
      .expect("step South");
  }

  let updated_explored_count = game.world().explored_tiles().len();
  assert!(
    updated_explored_count > initial_explored_count,
    "moving to new areas should expand explored fog-of-war memory"
  );

  let updated_obs = game.observe_player();
  assert_eq!(updated_obs.visible_tiles.len(), updated_explored_count);

  // Confirm previously seen starting tile is still in explored memory
  assert!(game.world().is_explored(Position::new(5, 5)));
}

#[test]
fn test_line_of_fire_blocking_in_ranged_attack() {
  let mut game = Game::new(200, 15, 15, Position::new(2, 5)).expect("game init failed");
  let monster_pos = Position::new(8, 5);

  let _m_id = game
    .world_mut()
    .spawn_monster(monster_pos, "Imp", 20, 100, (2, 4))
    .expect("spawn monster");

  // Place a wall blocking line of fire at (5, 5)
  game
    .world_mut()
    .map_mut()
    .set_tile(Position::new(5, 5), Tile::Wall);

  // Attempt ranged attack through wall
  let err = game.step(Command::AttackRanged(monster_pos)).unwrap_err();
  assert_eq!(err, CommandError::LineOfSightBlocked(monster_pos));
}
