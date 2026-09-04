//! MCP JSON boundary coverage for the Null Pointer SPLASMA armor policy.

use drl_core::ReplayEngine;
use drl_mcp::JsonValue;
use drl_mcp::McpServer;
use drl_mcp::replay_json;
use drl_protocol::{
  Command, DamageType, ItemSpawnKind, MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

fn ready_server() -> McpServer {
  let mut server = McpServer::new();
  let _ = server.handle_request(
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"drl-null-pointer-test","version":"1"}}}"#,
  );
  let _ = server.handle_request(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
  server
}

#[test]
fn null_pointer_splasma_mcp_json_boundary_matches_direct_core() {
  let player_position = Position::new(5, 5);
  let target_position = Position::new(6, 5);
  let player_config = PlayerSpawnConfig {
    hp: 500,
    max_hp: 500,
    speed: 100,
    initial_items: Vec::new(),
    equipped_weapon: Some(ItemSpawnKind::NullPointer),
    equipped_armor: Some(ItemSpawnKind::BlueArmor),
    equipped_armor_durability: None,
  };
  let mut setup = ReplayLog::new(25, 12, 12, player_position).with_player_config(player_config);
  setup.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Blast Target",
    500,
    0,
    (0, 0),
  ));

  let (initial, _) = ReplayEngine::run(&setup).expect("direct Null Pointer setup replay");
  let player_id = initial.world().player_id().expect("player identity");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("Null Pointer target")
    .id();
  let command = Command::AttackRanged(target_position);
  let mut direct = initial;
  let expected_events = direct.step(command).expect("direct Null Pointer shot");

  let mut server = ready_server();
  let setup_json = replay_json::to_json_value(&setup).to_compact_string();
  let load_request = format!(
    r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"game_load_replay","arguments":{{"replay":{setup_json}}}}}}}"#
  );
  let load_response = JsonValue::parse(&server.handle_request(&load_request)).unwrap();
  assert_eq!(
    load_response
      .get("result")
      .and_then(|result| result.get("data"))
      .and_then(|data| data.get("status"))
      .and_then(JsonValue::as_str),
    Some("ReplayLoaded")
  );

  let step_response = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"game_step_action","arguments":{"action":"attack_ranged","target_x":6,"target_y":5}}}"#,
  ))
  .unwrap();
  let step_data = step_response
    .get("result")
    .and_then(|result| result.get("data"))
    .expect("MCP step data");
  let events = step_data
    .get("events")
    .and_then(JsonValue::as_array)
    .expect("MCP event array");
  assert!(events.iter().any(|event| {
    event.get("type").and_then(JsonValue::as_str) == Some("DamageApplied")
      && event.get("target_id").and_then(JsonValue::as_u64) == Some(target_id.as_u64())
      && event.get("amount").and_then(JsonValue::as_u64) == Some(10)
      && event.get("damage_type").and_then(JsonValue::as_str) == Some("Plasma")
  }));
  assert!(events.iter().any(|event| {
    event.get("type").and_then(JsonValue::as_str) == Some("DamageApplied")
      && event.get("target_id").and_then(JsonValue::as_u64) == Some(player_id.as_u64())
      && event.get("amount").and_then(JsonValue::as_u64) == Some(8)
      && event.get("damage_type").and_then(JsonValue::as_str) == Some("Plasma")
  }));
  assert_eq!(
    step_data.get("observation"),
    Some(&drl_mcp::session::player_observation_to_json(
      &direct.observe_player(),
    ))
  );

  let save_response = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"game_save_replay","arguments":{}}}"#,
  ))
  .unwrap();
  let saved_replay_value = save_response
    .get("result")
    .and_then(|result| result.get("data"))
    .expect("MCP saved replay");
  let saved_replay = replay_json::from_json_value(saved_replay_value).expect("decode MCP replay");
  let (replayed, replay_events) = ReplayEngine::run(&saved_replay).expect("MCP replay execution");
  assert_eq!(replayed, direct);
  assert_eq!(replay_events, expected_events);
  assert!(ReplayEngine::verify_determinism(&saved_replay).expect("MCP replay determinism"));

  let verify_response = JsonValue::parse(&server.handle_request(
    r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"game_verify_replay","arguments":{}}}"#,
  ))
  .unwrap();
  assert_eq!(
    verify_response
      .get("result")
      .and_then(|result| result.get("data"))
      .and_then(|data| data.get("deterministic"))
      .and_then(JsonValue::as_bool),
    Some(true)
  );

  assert!(expected_events.iter().any(|event| {
    matches!(
      event,
      drl_protocol::GameEvent::DamageApplied {
        target_id: resolved_target,
        amount: 8,
        damage_type: Some(DamageType::Plasma),
        ..
      } if *resolved_target == player_id
    )
  }));
}
