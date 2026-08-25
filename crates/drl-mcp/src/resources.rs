//! MCP Resource definitions and read handlers for DRL-Rust.

use crate::json::JsonValue;
use crate::protocol::{JsonRpcError, ResourceDefinition, error_codes};
use crate::session::{McpSession, episode_metrics_to_json, game_event_to_json};
use std::collections::BTreeMap;

/// Returns all static and dynamic resource definitions available via MCP.
#[must_use]
pub fn get_all_resource_definitions() -> Vec<ResourceDefinition> {
  vec![
    ResourceDefinition {
      uri: "drl://rules/game".to_string(),
      name: "DRL-Rust Game Rules and Domain Guide".to_string(),
      description:
        "Overview of simulation mechanics, energy scheduler, combat, items, and monsters."
          .to_string(),
      mime_type: "text/markdown".to_string(),
    },
    ResourceDefinition {
      uri: "drl://rules/actions".to_string(),
      name: "DRL-Rust Semantic Action Catalog".to_string(),
      description: "Catalog of valid player actions, required parameters, and outcome rules."
        .to_string(),
      mime_type: "text/markdown".to_string(),
    },
    ResourceDefinition {
      uri: "drl://session/metrics".to_string(),
      name: "Current Session Telemetry Metrics".to_string(),
      description: "Live episode metrics (damage, turns survived, enemies slain, current outcome)."
        .to_string(),
      mime_type: "application/json".to_string(),
    },
    ResourceDefinition {
      uri: "drl://session/events".to_string(),
      name: "Recent Game Events Log".to_string(),
      description: "Recent simulation events emitted during the last turn execution.".to_string(),
      mime_type: "application/json".to_string(),
    },
  ]
}

/// Reads a resource by URI from the given session.
pub fn read_resource(session: &McpSession, uri: &str) -> Result<JsonValue, JsonRpcError> {
  match uri {
    "drl://rules/game" => {
      let text = r#"# DRL-Rust Game Rules

## Overview
DRL-Rust is a deterministic, headless, turn-based roguelike simulation of Doom the Roguelike.

## Core Rules:
1. Turn Economy: Energy-based scheduler where standard actions cost 100 energy. Faster actors act more frequently.
2. Combat: Pure deterministic resolution with explicit RNG rolls. Melee attacks trigger on bump or explicit attack. Ranged attacks require line of sight, range limits, and loaded ammunition.
3. Items & Inventory: Backpack inventory with capacity limit. Weapons and armor equip into designated slots. Ranged weapons consume ammunition and require reloading.
4. Levels & Stairs: Procedural or scenario levels. Standing on stairs and submitting `Descend` transitions to the next level while preserving player state.
"#;
      Ok(wrap_resource_content(uri, "text/markdown", text))
    }

    "drl://rules/actions" => {
      let text = r#"# DRL-Rust Semantic Action Catalog

1. `move` (`direction`: North, South, East, West, NorthEast, NorthWest, SouthEast, SouthWest)
   - Step into an adjacent walkable tile or bump-attack an adjacent enemy.
2. `attack_melee` (`direction`: North, South, East, West, NorthEast, NorthWest, SouthEast, SouthWest)
   - Directly attack a visible adjacent enemy without moving.
3. `wait`
   - Wait in place for 1 turn (costs 100 energy).
4. `fire` (`target_x`: int, `target_y`: int)
   - Fire equipped ranged weapon at target grid coordinates.
5. `reload`
   - Reload equipped ranged weapon from inventory ammunition.
6. `invoke` (`item_id`: int)
   - Invoke the equipped Subtle Knife alternate action against visible targets.
7. `pickup`
   - Pick up an item lying on the current ground tile into backpack.
8. `use` (`item_id`: int)
   - Consume or activate an inventory item (e.g. MedPack or Phase Device).
9. `equip` (`item_id`: int, `slot`: "Weapon" | "Armor")
   - Equip an item from backpack into active gear slot.
10. `unequip` (`slot`: "Weapon" | "Armor")
   - Unequip gear back into backpack inventory.
11. `drop` (`item_id`: int)
   - Drop an inventory item onto the current floor tile.
12. `descend`
   - Descend down-stairs to enter the next dungeon depth.
"#;
      Ok(wrap_resource_content(uri, "text/markdown", text))
    }

    "drl://session/metrics" => {
      let metrics = session.get_metrics();
      let json_val = episode_metrics_to_json(metrics);
      Ok(wrap_resource_content(
        uri,
        "application/json",
        &json_val.to_compact_string(),
      ))
    }

    "drl://session/events" => {
      let events = session.recent_events();
      let events_json: Vec<JsonValue> = events.iter().map(game_event_to_json).collect();
      let val = JsonValue::Array(events_json);
      Ok(wrap_resource_content(
        uri,
        "application/json",
        &val.to_compact_string(),
      ))
    }

    other => Err(JsonRpcError::new(
      error_codes::INVALID_PARAMS,
      format!("Resource not found: '{other}'"),
    )),
  }
}

fn wrap_resource_content(uri: &str, mime_type: &str, text: &str) -> JsonValue {
  let mut content = BTreeMap::new();
  content.insert("uri".to_string(), JsonValue::from(uri));
  content.insert("mimeType".to_string(), JsonValue::from(mime_type));
  content.insert("text".to_string(), JsonValue::from(text));

  let mut map = BTreeMap::new();
  map.insert(
    "contents".to_string(),
    JsonValue::Array(vec![JsonValue::Object(content)]),
  );
  JsonValue::Object(map)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_resource_definitions_and_read() {
    let defs = get_all_resource_definitions();
    assert!(defs.len() >= 4);

    let session = McpSession::new();
    let res = read_resource(&session, "drl://rules/game").unwrap();
    let contents = res.get("contents").unwrap().as_array().unwrap();
    assert_eq!(contents.len(), 1);
  }
}
