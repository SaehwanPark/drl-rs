//! MCP JSON boundary coverage for Missile Launcher direct Fire damage.

use drl_core::ReplayEngine;
use drl_mcp::JsonValue;
use drl_mcp::McpServer;
use drl_mcp::replay_json;
use drl_protocol::{
  Command, DamageSource, DamageType, ItemSpawnKind, MonsterSpawnSpec, PlayerSpawnConfig, Position,
  ReplayLog,
};

fn ready_server() -> McpServer {
  let mut server = McpServer::new();
  let _ = server.handle_request(
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"drl-missile-launcher-test","version":"1"}}}"#,
  );
  let _ = server.handle_request(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
  server
}

#[test]
fn missile_launcher_mcp_json_matches_direct_core_typed_damage() {
  let player_position = Position::new(5, 5);
  let target_position = Position::new(6, 5);
  let mut setup =
    ReplayLog::new(46_105, 12, 12, player_position).with_player_config(PlayerSpawnConfig {
      hp: 500,
      max_hp: 500,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(ItemSpawnKind::MissileLauncher),
      equipped_armor: Some(ItemSpawnKind::RedArmor),
      equipped_armor_durability: None,
    });
  setup.record_monster(MonsterSpawnSpec::new(
    target_position,
    "Direct Target",
    10_000,
    0,
    (0, 0),
  ));

  let (initial, _) = ReplayEngine::run(&setup).expect("direct Missile Launcher setup replay");
  let target_id = initial
    .world()
    .actors()
    .values()
    .find(|actor| !actor.is_player())
    .expect("Missile Launcher target")
    .id();
  let command = Command::AttackRanged(target_position);
  let mut direct = initial;
  let expected_events = direct.step(command).expect("direct Missile Launcher shot");
  let expected_damage = expected_events
    .iter()
    .filter_map(|event| match event {
      drl_protocol::GameEvent::DamageApplied {
        target_id: event_target,
        amount,
        source: DamageSource::Actor(_),
        damage_type: Some(DamageType::Fire),
        ..
      } if *event_target == target_id => Some(*amount),
      _ => None,
    })
    .collect::<Vec<_>>();
  assert!(!expected_damage.is_empty());

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
  let mcp_damage = events
    .iter()
    .filter(|event| {
      event.get("type").and_then(JsonValue::as_str) == Some("DamageApplied")
        && event.get("target_id").and_then(JsonValue::as_u64) == Some(target_id.as_u64())
        && event.get("damage_type").and_then(JsonValue::as_str) == Some("Fire")
    })
    .map(|event| event.get("amount").and_then(JsonValue::as_u64).unwrap() as u32)
    .collect::<Vec<_>>();
  assert_eq!(mcp_damage, expected_damage);
  assert_eq!(
    step_data.get("observation"),
    Some(&drl_mcp::session::player_observation_to_json(
      &direct.observe_player(),
    ))
  );
}
