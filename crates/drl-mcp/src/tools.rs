//! MCP Tool definitions and execution dispatch for DRL-Rust.

use crate::json::JsonValue;
use crate::protocol::{JsonRpcError, ToolDefinition, error_codes};
use crate::session::{
  McpSession, compute_legal_actions, episode_metrics_to_json, json_to_command,
  omniscient_observation_to_json, player_observation_to_json,
};
use drl_core::ReplayEngine;
use drl_protocol::ReplayLog;
use std::collections::BTreeMap;

const JSON_SAFE_INTEGER_MAX: f64 = 9_007_199_254_740_992.0;
const U32_MAX: f64 = 4_294_967_295.0;
const I32_MIN: f64 = -2_147_483_648.0;
const I32_MAX: f64 = 2_147_483_647.0;

const ACTION_ALIASES: &[&str] = &[
  "move",
  "wait",
  "attack_melee",
  "melee",
  "attack_ranged",
  "fire",
  "shoot",
  "reload",
  "pickup",
  "use",
  "equip",
  "unequip",
  "drop",
  "descend",
];
const DIRECTION_ALIASES: &[&str] = &[
  "north",
  "n",
  "up",
  "k",
  "south",
  "s",
  "down",
  "j",
  "east",
  "e",
  "right",
  "l",
  "west",
  "w",
  "left",
  "h",
  "northeast",
  "ne",
  "u",
  "northwest",
  "nw",
  "y",
  "southeast",
  "se",
  "n_key",
  "b",
  "southwest",
  "sw",
  "m",
  "none",
  "wait",
  ".",
];
const SLOT_ALIASES: &[&str] = &["weapon", "armor", "Weapon", "Armor"];

#[derive(Debug, Clone, Copy)]
struct SchemaField {
  name: &'static str,
  type_name: &'static str,
  description: &'static str,
  required: bool,
  enum_values: Option<&'static [&'static str]>,
  minimum: Option<f64>,
  maximum: Option<f64>,
}

impl SchemaField {
  const fn new(
    name: &'static str,
    type_name: &'static str,
    description: &'static str,
    required: bool,
  ) -> Self {
    Self {
      name,
      type_name,
      description,
      required,
      enum_values: None,
      minimum: None,
      maximum: None,
    }
  }

  const fn with_enum(mut self, values: &'static [&'static str]) -> Self {
    self.enum_values = Some(values);
    self
  }

  const fn with_range(mut self, minimum: f64, maximum: f64) -> Self {
    self.minimum = Some(minimum);
    self.maximum = Some(maximum);
    self
  }
}

/// Returns the complete registry of MCP tools exposed by DRL-Rust.
#[must_use]
pub fn get_all_tool_definitions() -> Vec<ToolDefinition> {
  vec![
    ToolDefinition {
      name: "game_start".to_string(),
      description: "Start a new seeded procedural dungeon game session.".to_string(),
      input_schema: create_object_schema_with_fields(&[
        SchemaField::new("seed", "integer", "RNG seed for procedural generation", false)
          .with_range(0.0, JSON_SAFE_INTEGER_MAX),
        SchemaField::new(
          "max_turns",
          "integer",
          "Maximum turn limit before episode cutoff",
          false,
        )
        .with_range(0.0, JSON_SAFE_INTEGER_MAX),
        SchemaField::new("width", "integer", "Map width in tiles (default: 40)", false)
          .with_range(0.0, U32_MAX),
        SchemaField::new("height", "integer", "Map height in tiles (default: 20)", false)
          .with_range(0.0, U32_MAX),
      ]),
    },
    ToolDefinition {
      name: "game_load_scenario".to_string(),
      description: "Load an approved ASCII scenario fixture into the game session.".to_string(),
      input_schema: create_object_schema_with_fields(&[
        SchemaField::new(
          "ascii_map",
          "string",
          "ASCII representation of the dungeon map and actors",
          true,
        ),
        SchemaField::new(
          "max_turns",
          "integer",
          "Maximum turn limit before episode cutoff",
          false,
        )
        .with_range(0.0, JSON_SAFE_INTEGER_MAX),
      ]),
    },
    ToolDefinition {
      name: "game_get_observation".to_string(),
      description: "Retrieve the latest player-visible observation (grid, visible actors, inventory, equipment).".to_string(),
      input_schema: create_object_schema_with_fields(&[]),
    },
    ToolDefinition {
      name: "game_list_actions".to_string(),
      description: "List all currently legal semantic player actions available in this turn.".to_string(),
      input_schema: create_object_schema_with_fields(&[]),
    },
    ToolDefinition {
      name: "game_step_action".to_string(),
      description: "Execute a semantic player action (move, wait, fire, reload, pickup, use, equip, unequip, drop, descend).".to_string(),
      input_schema: create_object_schema_with_fields(&[
        SchemaField::new(
          "action",
          "string",
          "Canonical action category spelling; runtime also accepts case variants",
          true,
        )
        .with_enum(ACTION_ALIASES),
        SchemaField::new(
          "command",
          "string",
          "Runtime compatibility alias for action; canonical schemas require action",
          false,
        )
        .with_enum(ACTION_ALIASES),
        SchemaField::new(
          "direction",
          "string",
          "Cardinal/diagonal direction alias for move or attack_melee",
          false,
        )
        .with_enum(DIRECTION_ALIASES),
        SchemaField::new("target_x", "integer", "Target X coordinate (for fire)", false)
          .with_range(I32_MIN, I32_MAX),
        SchemaField::new("target_y", "integer", "Target Y coordinate (for fire)", false)
          .with_range(I32_MIN, I32_MAX),
        SchemaField::new("x", "integer", "Runtime alias for target_x", false)
          .with_range(I32_MIN, I32_MAX),
        SchemaField::new("y", "integer", "Runtime alias for target_y", false)
          .with_range(I32_MIN, I32_MAX),
        SchemaField::new(
          "item_id",
          "integer",
          "Item entity ID (for use, equip, drop)",
          false,
        )
        .with_range(0.0, JSON_SAFE_INTEGER_MAX),
        SchemaField::new(
          "slot",
          "string",
          "Equipment slot name (for unequip; accepted by equip for compatibility)",
          false,
        )
        .with_enum(SLOT_ALIASES),
      ]),
    },
    ToolDefinition {
      name: "game_reset".to_string(),
      description: "Reset the current game session back to its initial state.".to_string(),
      input_schema: create_object_schema_with_fields(&[]),
    },
    ToolDefinition {
      name: "game_get_metrics".to_string(),
      description: "Retrieve cumulative episode metrics (damage dealt/taken, kills, turns survived, outcome).".to_string(),
      input_schema: create_object_schema_with_fields(&[]),
    },
    ToolDefinition {
      name: "game_save_replay".to_string(),
      description: "Export the current session's deterministic replay log as JSON.".to_string(),
      input_schema: create_object_schema_with_fields(&[]),
    },
    ToolDefinition {
      name: "game_verify_replay".to_string(),
      description: "Verify the current session's replay is deterministic without changing game state.".to_string(),
      input_schema: create_object_schema_with_fields(&[]),
    },
    ToolDefinition {
      name: "game_get_dev_state".to_string(),
      description: "Developer-only omniscient world state inspection. Fails if dev_mode is not enabled.".to_string(),
      input_schema: create_object_schema_with_fields(&[]),
    },
  ]
}

/// Executes a tool call against the given session.
pub fn execute_tool(
  session: &mut McpSession,
  name: &str,
  arguments: &JsonValue,
) -> Result<JsonValue, JsonRpcError> {
  match name {
    "game_start" => {
      let seed = optional_u64_argument(arguments, "seed")?.unwrap_or(1);
      let max_turns = optional_u64_argument(arguments, "max_turns")?;
      let width = optional_u32_argument(arguments, "width")?;
      let height = optional_u32_argument(arguments, "height")?;

      let obs = session
        .start_game(seed, max_turns, width, height)
        .map_err(|e| JsonRpcError::new(error_codes::INVALID_ACTION, e))?;

      let legal_actions = compute_legal_actions(&obs);
      let mut res = BTreeMap::new();
      res.insert("status".to_string(), JsonValue::from("GameStarted"));
      res.insert("seed".to_string(), JsonValue::from(seed));
      res.insert("observation".to_string(), player_observation_to_json(&obs));
      res.insert(
        "legal_actions".to_string(),
        JsonValue::Array(
          legal_actions
            .iter()
            .map(crate::session::LegalAction::to_json_value)
            .collect(),
        ),
      );
      Ok(wrap_mcp_tool_result(JsonValue::Object(res)))
    }

    "game_load_scenario" => {
      let ascii = arguments
        .get("ascii_map")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
          JsonRpcError::new(error_codes::INVALID_PARAMS, "Missing 'ascii_map' parameter")
        })?;
      let max_turns = optional_u64_argument(arguments, "max_turns")?;

      let obs = session
        .load_scenario(ascii, max_turns)
        .map_err(|e| JsonRpcError::new(error_codes::INVALID_ACTION, e))?;

      let legal_actions = compute_legal_actions(&obs);
      let mut res = BTreeMap::new();
      res.insert("status".to_string(), JsonValue::from("ScenarioLoaded"));
      res.insert("observation".to_string(), player_observation_to_json(&obs));
      res.insert(
        "legal_actions".to_string(),
        JsonValue::Array(
          legal_actions
            .iter()
            .map(crate::session::LegalAction::to_json_value)
            .collect(),
        ),
      );
      Ok(wrap_mcp_tool_result(JsonValue::Object(res)))
    }

    "game_get_observation" => {
      let obs = session
        .get_observation()
        .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?;
      let legal_actions = compute_legal_actions(&obs);

      let mut res = BTreeMap::new();
      res.insert("observation".to_string(), player_observation_to_json(&obs));
      res.insert(
        "legal_actions".to_string(),
        JsonValue::Array(
          legal_actions
            .iter()
            .map(crate::session::LegalAction::to_json_value)
            .collect(),
        ),
      );
      Ok(wrap_mcp_tool_result(JsonValue::Object(res)))
    }

    "game_list_actions" => {
      let obs = session
        .get_observation()
        .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?;
      let legal_actions = compute_legal_actions(&obs);

      let mut res = BTreeMap::new();
      res.insert(
        "legal_actions".to_string(),
        JsonValue::Array(
          legal_actions
            .iter()
            .map(crate::session::LegalAction::to_json_value)
            .collect(),
        ),
      );
      Ok(wrap_mcp_tool_result(JsonValue::Object(res)))
    }

    "game_step_action" => {
      let cmd = json_to_command(arguments)
        .map_err(|e| JsonRpcError::new(error_codes::INVALID_PARAMS, e))?;

      let (events, obs, outcome) = session
        .step(cmd)
        .map_err(|e| JsonRpcError::new(error_codes::INVALID_ACTION, e))?;

      let legal_actions = compute_legal_actions(&obs);

      let mut res = BTreeMap::new();
      res.insert(
        "events".to_string(),
        JsonValue::Array(
          events
            .iter()
            .map(crate::session::game_event_to_json)
            .collect(),
        ),
      );
      res.insert("observation".to_string(), player_observation_to_json(&obs));
      res.insert(
        "legal_actions".to_string(),
        JsonValue::Array(
          legal_actions
            .iter()
            .map(crate::session::LegalAction::to_json_value)
            .collect(),
        ),
      );
      if let Some(ref o) = outcome {
        res.insert("outcome".to_string(), JsonValue::from(format!("{o:?}")));
        res.insert("game_over".to_string(), JsonValue::Bool(true));
      } else {
        res.insert("game_over".to_string(), JsonValue::Bool(false));
      }
      Ok(wrap_mcp_tool_result(JsonValue::Object(res)))
    }

    "game_reset" => {
      let obs = session
        .reset()
        .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?;
      let legal_actions = compute_legal_actions(&obs);

      let mut res = BTreeMap::new();
      res.insert("status".to_string(), JsonValue::from("SessionReset"));
      res.insert("observation".to_string(), player_observation_to_json(&obs));
      res.insert(
        "legal_actions".to_string(),
        JsonValue::Array(
          legal_actions
            .iter()
            .map(crate::session::LegalAction::to_json_value)
            .collect(),
        ),
      );
      Ok(wrap_mcp_tool_result(JsonValue::Object(res)))
    }

    "game_get_metrics" => {
      let metrics = session.get_metrics();
      let res = episode_metrics_to_json(metrics);
      Ok(wrap_mcp_tool_result(res))
    }

    "game_save_replay" => {
      let replay = session
        .export_replay()
        .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?;
      let res = replay_to_json_value(replay);
      Ok(wrap_mcp_tool_result(res))
    }

    "game_verify_replay" => {
      let replay = session
        .export_replay()
        .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?;
      let deterministic = ReplayEngine::verify_determinism(replay).map_err(|e| {
        JsonRpcError::new(
          error_codes::INTERNAL_ERROR,
          format!("Replay verification failed: {e}"),
        )
      })?;
      let mut res = BTreeMap::new();
      res.insert("deterministic".to_string(), JsonValue::Bool(deterministic));
      res.insert(
        "command_count".to_string(),
        JsonValue::from(replay.commands.len() as u64),
      );
      res.insert(
        "version".to_string(),
        JsonValue::from(replay.version as u32),
      );
      Ok(wrap_mcp_tool_result(JsonValue::Object(res)))
    }

    "game_get_dev_state" => {
      let obs = session
        .get_dev_state()
        .map_err(|e| JsonRpcError::new(error_codes::PERMISSION_DENIED, e))?;
      let res = omniscient_observation_to_json(&obs);
      Ok(wrap_mcp_tool_result(res))
    }

    other => Err(JsonRpcError::new(
      error_codes::METHOD_NOT_FOUND,
      format!("Unknown tool: '{other}'"),
    )),
  }
}

fn wrap_mcp_tool_result(content_json: JsonValue) -> JsonValue {
  let text_str = content_json.to_compact_string();
  let mut text_item = BTreeMap::new();
  text_item.insert("type".to_string(), JsonValue::from("text"));
  text_item.insert("text".to_string(), JsonValue::String(text_str));

  let mut map = BTreeMap::new();
  map.insert(
    "content".to_string(),
    JsonValue::Array(vec![JsonValue::Object(text_item)]),
  );
  map.insert("isError".to_string(), JsonValue::Bool(false));
  map.insert("data".to_string(), content_json);
  JsonValue::Object(map)
}

fn optional_u64_argument(arguments: &JsonValue, name: &str) -> Result<Option<u64>, JsonRpcError> {
  match arguments.get(name) {
    None => Ok(None),
    Some(value) => exact_u64(value).map(Some).ok_or_else(|| {
      JsonRpcError::new(
        error_codes::INVALID_PARAMS,
        format!("'{name}' argument must be a non-negative integer"),
      )
    }),
  }
}

fn optional_u32_argument(arguments: &JsonValue, name: &str) -> Result<Option<u32>, JsonRpcError> {
  match arguments.get(name) {
    None => Ok(None),
    Some(value) => exact_u64(value)
      .and_then(|number| u32::try_from(number).ok())
      .map(Some)
      .ok_or_else(|| {
        JsonRpcError::new(
          error_codes::INVALID_PARAMS,
          format!("'{name}' argument must be a non-negative 32-bit integer"),
        )
      }),
  }
}

fn exact_u64(value: &JsonValue) -> Option<u64> {
  match value {
    JsonValue::Number(number)
      if number.is_finite()
        && *number >= 0.0
        && number.fract() == 0.0
        && *number <= 9_007_199_254_740_992.0 =>
    {
      Some(*number as u64)
    }
    _ => None,
  }
}

fn replay_to_json_value(replay: &ReplayLog) -> JsonValue {
  let mut map = BTreeMap::new();
  map.insert(
    "version".to_string(),
    JsonValue::from(replay.version as u32),
  );
  map.insert("seed".to_string(), JsonValue::from(replay.seed));
  map.insert("width".to_string(), JsonValue::from(replay.width));
  map.insert("height".to_string(), JsonValue::from(replay.height));
  map.insert(
    "player_start_x".to_string(),
    JsonValue::from(replay.player_start.x),
  );
  map.insert(
    "player_start_y".to_string(),
    JsonValue::from(replay.player_start.y),
  );

  let mut cmds = Vec::with_capacity(replay.commands.len());
  for c in &replay.commands {
    cmds.push(JsonValue::from(format!("{c:?}")));
  }
  map.insert("commands".to_string(), JsonValue::Array(cmds));

  JsonValue::Object(map)
}

fn create_object_schema_with_fields(fields: &[SchemaField]) -> JsonValue {
  let mut map = BTreeMap::new();
  map.insert("type".to_string(), JsonValue::from("object"));

  let mut props = BTreeMap::new();
  let mut required = Vec::new();

  for field in fields {
    let mut field_map = BTreeMap::new();
    field_map.insert("type".to_string(), JsonValue::from(field.type_name));
    field_map.insert(
      "description".to_string(),
      JsonValue::from(field.description),
    );
    if let Some(values) = field.enum_values {
      field_map.insert(
        "enum".to_string(),
        JsonValue::Array(values.iter().map(|value| JsonValue::from(*value)).collect()),
      );
    }
    if let Some(minimum) = field.minimum {
      field_map.insert("minimum".to_string(), JsonValue::from(minimum));
    }
    if let Some(maximum) = field.maximum {
      field_map.insert("maximum".to_string(), JsonValue::from(maximum));
    }
    props.insert(field.name.to_string(), JsonValue::Object(field_map));
    if field.required {
      required.push(JsonValue::from(field.name));
    }
  }

  map.insert("properties".to_string(), JsonValue::Object(props));
  if !required.is_empty() {
    map.insert("required".to_string(), JsonValue::Array(required));
  }

  JsonValue::Object(map)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_all_tool_definitions() {
    let defs = get_all_tool_definitions();
    assert!(defs.len() >= 8);
    assert!(defs.iter().any(|t| t.name == "game_start"));
    assert!(defs.iter().any(|t| t.name == "game_step_action"));
    assert!(defs.iter().any(|t| t.name == "game_get_observation"));
  }

  #[test]
  fn test_execute_tool_workflow() {
    let mut session = McpSession::new();
    let start_args = JsonValue::parse(r#"{"seed":42,"width":20,"height":15}"#).unwrap();
    let res = execute_tool(&mut session, "game_start", &start_args).unwrap();
    assert!(res.get("data").is_some());

    let obs_args = JsonValue::Object(BTreeMap::new());
    let obs_res = execute_tool(&mut session, "game_get_observation", &obs_args).unwrap();
    assert!(obs_res.get("data").is_some());

    let step_args = JsonValue::parse(r#"{"action":"wait"}"#).unwrap();
    let step_res = execute_tool(&mut session, "game_step_action", &step_args).unwrap();
    assert!(step_res.get("data").is_some());
  }
}
