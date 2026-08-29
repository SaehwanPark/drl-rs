//! MCP Tool definitions and execution dispatch for DRL-Rust.

use crate::json::JsonValue;
use crate::protocol::{JsonRpcError, ToolDefinition, error_codes};
use crate::replay_json;
use crate::session::{
  McpSession, episode_metrics_to_json, json_to_command, omniscient_observation_to_json,
  player_observation_to_json,
};
use crate::tools_schema::{
  empty_object_schema, game_load_replay_schema, game_load_scenario_schema, game_start_schema,
  game_step_action_schema, game_verify_replay_schema,
};
use drl_core::ReplayEngine;
use std::collections::BTreeMap;

/// Returns the complete registry of MCP tools exposed by DRL-Rust.
#[must_use]
pub fn get_all_tool_definitions() -> Vec<ToolDefinition> {
  vec![
    ToolDefinition {
      name: "game_start".to_string(),
      description: "Start a new seeded procedural dungeon game session.".to_string(),
      input_schema: game_start_schema(),
    },
    ToolDefinition {
      name: "game_load_scenario".to_string(),
      description: "Load an approved ASCII scenario fixture into the game session.".to_string(),
      input_schema: game_load_scenario_schema(),
    },
    ToolDefinition {
      name: "game_load_replay".to_string(),
      description: "Restore a canonical V2 replay into the session transactionally.".to_string(),
      input_schema: game_load_replay_schema(),
    },
    ToolDefinition {
      name: "game_get_observation".to_string(),
      description: "Retrieve the latest player-visible observation (grid, visible actors, inventory, equipment).".to_string(),
      input_schema: empty_object_schema(),
    },
    ToolDefinition {
      name: "game_list_actions".to_string(),
      description: "List all currently legal semantic player actions available in this turn.".to_string(),
      input_schema: empty_object_schema(),
    },
    ToolDefinition {
      name: "game_step_action".to_string(),
      description: "Execute a semantic player action (move, wait, fire, aimed_fire, reload, invoke, alt_reload, pickup, use, equip, unequip, drop, descend).".to_string(),
      input_schema: game_step_action_schema(),
    },
    ToolDefinition {
      name: "game_reset".to_string(),
      description: "Reset the current game session back to its initial state.".to_string(),
      input_schema: empty_object_schema(),
    },
    ToolDefinition {
      name: "game_get_metrics".to_string(),
      description: "Retrieve cumulative episode metrics (damage dealt/taken, kills, turns survived, outcome).".to_string(),
      input_schema: empty_object_schema(),
    },
    ToolDefinition {
      name: "game_save_replay".to_string(),
      description: "Export the current session's deterministic replay log as JSON.".to_string(),
      input_schema: empty_object_schema(),
    },
    ToolDefinition {
      name: "game_verify_replay".to_string(),
      description: "Verify the current session's replay, or a supplied canonical V2 replay, without changing game state.".to_string(),
      input_schema: game_verify_replay_schema(),
    },
    ToolDefinition {
      name: "game_get_dev_state".to_string(),
      description: "Developer-only omniscient world state inspection. Fails if dev_mode is not enabled.".to_string(),
      input_schema: empty_object_schema(),
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

      let legal_actions = session
        .legal_actions()
        .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?;
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

      let legal_actions = session
        .legal_actions()
        .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?;
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

    "game_load_replay" => {
      let replay = arguments
        .get("replay")
        .ok_or_else(|| JsonRpcError::new(error_codes::INVALID_PARAMS, "Missing 'replay' parameter"))
        .and_then(|value| {
          replay_json::from_json_value(value)
            .map_err(|e| JsonRpcError::new(error_codes::INVALID_PARAMS, e))
        })?;
      let obs = session
        .load_replay(replay)
        .map_err(|e| JsonRpcError::new(error_codes::INVALID_ACTION, e))?;
      let legal_actions = session
        .legal_actions()
        .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?;
      let mut res = BTreeMap::new();
      res.insert("status".to_string(), JsonValue::from("ReplayLoaded"));
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
      res.insert(
        "metrics".to_string(),
        episode_metrics_to_json(session.get_metrics()),
      );
      Ok(wrap_mcp_tool_result(JsonValue::Object(res)))
    }

    "game_get_observation" => {
      let obs = session
        .get_observation()
        .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?;
      let legal_actions = session
        .legal_actions()
        .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?;

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
      let legal_actions = session
        .legal_actions()
        .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?;

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

      let legal_actions = session
        .legal_actions()
        .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?;

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
      let legal_actions = session
        .legal_actions()
        .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?;

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
      let res = replay_json::to_json_value(replay);
      Ok(wrap_mcp_tool_result(res))
    }

    "game_verify_replay" => {
      let replay = if let Some(value) = arguments.get("replay") {
        replay_json::from_json_value(value)
          .map_err(|e| JsonRpcError::new(error_codes::INVALID_PARAMS, e))?
      } else {
        session
          .export_replay()
          .map_err(|e| JsonRpcError::new(error_codes::SESSION_NOT_ACTIVE, e))?
          .clone()
      };
      let deterministic = ReplayEngine::verify_determinism(&replay).map_err(|e| {
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

/// Wraps a recognized tool's runtime failure in the MCP tool-result envelope.
/// The numeric code and message remain available under `data` for agents that
/// need deterministic machine-readable diagnostics.
pub(crate) fn tool_error_result(error: &JsonRpcError) -> JsonValue {
  let mut data = BTreeMap::new();
  data.insert("code".to_string(), JsonValue::from(error.code));
  data.insert(
    "message".to_string(),
    JsonValue::String(error.message.clone()),
  );
  if let Some(details) = &error.data {
    data.insert("details".to_string(), details.clone());
  }

  let mut text_item = BTreeMap::new();
  text_item.insert("type".to_string(), JsonValue::from("text"));
  text_item.insert("text".to_string(), JsonValue::String(error.message.clone()));

  let mut result = BTreeMap::new();
  result.insert(
    "content".to_string(),
    JsonValue::Array(vec![JsonValue::Object(text_item)]),
  );
  result.insert("isError".to_string(), JsonValue::Bool(true));
  result.insert("data".to_string(), JsonValue::Object(data));
  JsonValue::Object(result)
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
