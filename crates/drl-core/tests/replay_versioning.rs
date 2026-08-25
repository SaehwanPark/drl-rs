//! Integration tests for replay versioning, metadata headers, validation, and error diagnostics.

use drl_core::replay::ReplayEngine;
use drl_protocol::{
  Command, CommandError, Direction, ItemSpawnKind, ItemSpawnSpec, MonsterSpawnSpec, Position,
  ProceduralGenerationConfig, ReplayLog, ReplayMetadata, ReplayVersion, Turn,
};

#[test]
fn test_replay_version_and_metadata_headers() {
  let mut replay = ReplayLog::new(42, 20, 20, Position::new(5, 5));
  assert_eq!(replay.version, ReplayVersion::V1);
  assert_eq!(replay.metadata.version, ReplayVersion::V1);
  assert_eq!(replay.metadata.engine_name, "DRL-Rust");
  assert_eq!(
    replay.metadata.gameplay_semantics_version,
    drl_protocol::CURRENT_GAMEPLAY_SEMANTICS_VERSION
  );
  assert_eq!(
    replay.metadata.generator_semantics_version,
    drl_protocol::CURRENT_GENERATOR_SEMANTICS_VERSION
  );
  assert_eq!(replay.metadata.ruleset_id, drl_protocol::CURRENT_RULESET_ID);

  replay = replay.with_metadata(ReplayMetadata {
    version: ReplayVersion::V1,
    engine_name: "DRL-Rust-TestHarness".to_string(),
    engine_version: "0.1.0".to_string(),
    gameplay_semantics_version: drl_protocol::CURRENT_GAMEPLAY_SEMANTICS_VERSION,
    generator_semantics_version: drl_protocol::CURRENT_GENERATOR_SEMANTICS_VERSION,
    ruleset_id: drl_protocol::CURRENT_RULESET_ID.to_string(),
  });

  assert_eq!(replay.metadata.engine_name, "DRL-Rust-TestHarness");
}

#[test]
fn replay_metadata_compatibility_matrix_is_explicit() {
  let current = ReplayLog::new(42, 10, 10, Position::new(2, 2));
  assert!(ReplayEngine::validate(&current).is_ok());

  let mut stale_gameplay = current.clone();
  stale_gameplay.metadata.gameplay_semantics_version =
    drl_protocol::CURRENT_GAMEPLAY_SEMANTICS_VERSION.saturating_sub(1);
  let error = ReplayEngine::validate(&stale_gameplay).unwrap_err();
  assert!(error.contains("unsupported gameplay semantics version"));

  let mut stale_ruleset = current.clone();
  stale_ruleset.metadata.ruleset_id = "legacy-ruleset".to_string();
  let error = ReplayEngine::validate(&stale_ruleset).unwrap_err();
  assert!(error.contains("unsupported replay ruleset"));

  let mut procedural = current
    .clone()
    .with_procedural_config(ProceduralGenerationConfig {
      max_rooms: 5,
      min_room_size: 4,
      max_room_size: 8,
      max_monsters_per_room: 2,
      max_items_per_room: 2,
    });
  procedural.metadata.generator_semantics_version =
    drl_protocol::CURRENT_GENERATOR_SEMANTICS_VERSION.saturating_sub(1);
  let error = ReplayEngine::validate(&procedural).unwrap_err();
  assert!(error.contains("unsupported generator semantics version"));

  let mut fixed_map_with_stale_generator = current;
  fixed_map_with_stale_generator
    .metadata
    .generator_semantics_version =
    drl_protocol::CURRENT_GENERATOR_SEMANTICS_VERSION.saturating_sub(1);
  assert!(ReplayEngine::validate(&fixed_map_with_stale_generator).is_ok());
}

#[test]
fn test_replay_validation_catches_invalid_bounds() {
  // Test zero dimensions
  let zero_dim = ReplayLog::new(1, 0, 10, Position::new(0, 0));
  assert!(ReplayEngine::validate(&zero_dim).is_err());

  // Test player out of bounds
  let oob_player = ReplayLog::new(1, 10, 10, Position::new(12, 5));
  assert!(ReplayEngine::validate(&oob_player).is_err());

  // Test monster out of bounds
  let mut oob_monster = ReplayLog::new(1, 10, 10, Position::new(2, 2));
  oob_monster.record_monster(MonsterSpawnSpec::new(
    Position::new(15, 2),
    "Imp",
    20,
    100,
    (3, 6),
  ));
  assert!(ReplayEngine::validate(&oob_monster).is_err());

  // Test item out of bounds
  let mut oob_item = ReplayLog::new(1, 10, 10, Position::new(2, 2));
  oob_item.record_item(ItemSpawnSpec::new(
    Position::new(2, 20),
    ItemSpawnKind::Pistol,
  ));
  assert!(ReplayEngine::validate(&oob_item).is_err());

  // Test stairs out of bounds
  let mut oob_stairs = ReplayLog::new(1, 10, 10, Position::new(2, 2));
  oob_stairs.record_stairs(Position::new(10, 10)); // Boundary is 0..10 (max index 9)
  assert!(ReplayEngine::validate(&oob_stairs).is_err());

  // Valid replay passes validation
  let mut valid = ReplayLog::new(1, 10, 10, Position::new(2, 2));
  valid.record_stairs(Position::new(8, 8));
  valid.record_monster(MonsterSpawnSpec::new(
    Position::new(5, 5),
    "Imp",
    20,
    100,
    (3, 6),
  ));
  valid.record_item(ItemSpawnSpec::new(
    Position::new(3, 3),
    ItemSpawnKind::SmallMedPack,
  ));
  assert!(ReplayEngine::validate(&valid).is_ok());
}

#[test]
fn test_replay_diagnostics_precise_error_location() {
  let mut replay = ReplayLog::new(100, 10, 10, Position::new(1, 1));
  // Move East 3 times: from (1,1) -> (2,1) -> (3,1) -> (4,1)
  replay.record_command(Command::Move(Direction::East));
  replay.record_command(Command::Move(Direction::East));
  replay.record_command(Command::Move(Direction::East));
  // Invalid command: attempt to descend without stairs
  replay.record_command(Command::Descend);

  let err = ReplayEngine::run_with_diagnostics(&replay).unwrap_err();
  assert_eq!(err.command_index, 3);
  assert_eq!(err.command, Command::Descend);
  assert_eq!(err.error, CommandError::NotOnStairs(Position::new(4, 1)));

  let err_str = format!("{err}");
  assert!(err_str.contains("command #3"));
  assert!(err_str.contains("Descend"));
  assert!(err_str.contains("no stairs present at current position"));
}

#[test]
fn test_corrupted_replay_diagnostics_bounds_rejection() {
  let replay = ReplayLog::new(100, 5, 5, Position::new(10, 10));
  let err = ReplayEngine::run_with_diagnostics(&replay).unwrap_err();
  assert_eq!(err.command_index, 0);
  assert_eq!(err.turn, Turn::zero());
  assert!(matches!(err.error, CommandError::InvalidCommand(_)));
}
