//! Deterministic MCP tool input schemas.

use crate::json::JsonValue;
use crate::replay_json::MAX_REPLAY_DIMENSION;
use std::collections::BTreeMap;

const JSON_SAFE_INTEGER_MAX: f64 = 9_007_199_254_740_992.0;
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
  "invoke",
  "alt_reload",
  "pickup",
  "use",
  "equip",
  "unequip",
  "drop",
  "descend",
];
const MOVE_ACTIONS: &[&str] = &["move", "attack_melee", "melee"];
const RANGED_ACTIONS: &[&str] = &["attack_ranged", "fire", "shoot"];
const ITEM_ACTIONS: &[&str] = &["use", "equip", "drop", "invoke"];
const ALT_RELOAD_ACTIONS: &[&str] = &["alt_reload"];
const UNEQUIP_ACTIONS: &[&str] = &["unequip"];
const NO_ARGUMENT_ACTIONS: &[&str] = &["wait", "pickup", "reload", "descend"];
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

/// Schema for `game_start` arguments.
#[must_use]
pub fn game_start_schema() -> JsonValue {
  create_object_schema(&[
    SchemaField::new(
      "seed",
      "integer",
      "RNG seed for procedural generation",
      false,
    )
    .with_range(0.0, JSON_SAFE_INTEGER_MAX),
    SchemaField::new(
      "max_turns",
      "integer",
      "Maximum turn limit before episode cutoff",
      false,
    )
    .with_range(0.0, JSON_SAFE_INTEGER_MAX),
    SchemaField::new(
      "width",
      "integer",
      "Map width in tiles (default: 40)",
      false,
    )
    .with_range(3.0, f64::from(MAX_REPLAY_DIMENSION)),
    SchemaField::new(
      "height",
      "integer",
      "Map height in tiles (default: 20)",
      false,
    )
    .with_range(3.0, f64::from(MAX_REPLAY_DIMENSION)),
  ])
}

/// Schema for `game_load_scenario` arguments.
#[must_use]
pub fn game_load_scenario_schema() -> JsonValue {
  create_object_schema(&[
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
  ])
}

/// Schema for transactional canonical V1 replay restoration.
#[must_use]
pub fn game_load_replay_schema() -> JsonValue {
  let mut schema = BTreeMap::new();
  schema.insert("type".to_string(), JsonValue::from("object"));
  schema.insert(
    "description".to_string(),
    JsonValue::from(
      "Replace the session transactionally with a canonical drl-rust-replay-v1 envelope",
    ),
  );
  let mut replay = BTreeMap::new();
  replay.insert("type".to_string(), JsonValue::from("object"));
  replay.insert(
    "description".to_string(),
    JsonValue::from("Canonical drl-rust-replay-v1 schema_version 1 envelope"),
  );
  schema.insert(
    "properties".to_string(),
    JsonValue::Object(BTreeMap::from([(
      "replay".to_string(),
      JsonValue::Object(replay),
    )])),
  );
  schema.insert(
    "required".to_string(),
    JsonValue::Array(vec![JsonValue::from("replay")]),
  );
  JsonValue::Object(schema)
}

/// Schema for optional supplied replay verification.
#[must_use]
pub fn game_verify_replay_schema() -> JsonValue {
  let JsonValue::Object(mut schema) = create_object_schema(&[]) else {
    unreachable!("object schema builder must return an object");
  };
  let mut replay = BTreeMap::new();
  replay.insert("type".to_string(), JsonValue::from("object"));
  replay.insert(
    "description".to_string(),
    JsonValue::from("Canonical drl-rust-replay-v1 schema_version 1 envelope to verify read-only"),
  );
  schema.insert(
    "properties".to_string(),
    JsonValue::Object(BTreeMap::from([(
      "replay".to_string(),
      JsonValue::Object(replay),
    )])),
  );
  JsonValue::Object(schema)
}

/// Schema for `game_step_action` arguments, including action-specific needs.
#[must_use]
pub fn game_step_action_schema() -> JsonValue {
  let JsonValue::Object(mut schema) = create_object_schema(&[
    SchemaField::new(
      "action",
      "string",
      "Canonical action category spelling; runtime also accepts case variants",
      false,
    )
    .with_enum(ACTION_ALIASES),
    SchemaField::new(
      "command",
      "string",
      "Runtime compatibility alias for action",
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
    SchemaField::new(
      "target_x",
      "integer",
      "Target X coordinate (for fire)",
      false,
    )
    .with_range(I32_MIN, I32_MAX),
    SchemaField::new(
      "target_y",
      "integer",
      "Target Y coordinate (for fire)",
      false,
    )
    .with_range(I32_MIN, I32_MAX),
    SchemaField::new("x", "integer", "Runtime alias for target_x", false)
      .with_range(I32_MIN, I32_MAX),
    SchemaField::new("y", "integer", "Runtime alias for target_y", false)
      .with_range(I32_MIN, I32_MAX),
    SchemaField::new(
      "item_id",
      "integer",
      "Item entity ID (for use, equip, drop, invoke, alt_reload)",
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
    SchemaField::new(
      "confirmed",
      "boolean",
      "Explicit confirmation for the Trigun alternate reload",
      false,
    ),
  ]) else {
    unreachable!("object schema builder must return an object");
  };

  schema.insert(
    "anyOf".to_string(),
    JsonValue::Array(vec![
      required_fields(&["action"]),
      required_fields(&["command"]),
    ]),
  );

  let conditions = [
    action_condition(MOVE_ACTIONS, required_fields(&["direction"])),
    action_condition(
      RANGED_ACTIONS,
      any_required_fields(&[
        ["target_x", "target_y"],
        ["target_x", "y"],
        ["x", "target_y"],
        ["x", "y"],
      ]),
    ),
    action_condition(ITEM_ACTIONS, required_fields(&["item_id"])),
    action_condition(
      ALT_RELOAD_ACTIONS,
      required_fields(&["item_id", "confirmed"]),
    ),
    action_condition(UNEQUIP_ACTIONS, required_fields(&["slot"])),
    action_condition(NO_ARGUMENT_ACTIONS, required_fields(&[])),
  ];
  schema.insert(
    "allOf".to_string(),
    JsonValue::Array(conditions.into_iter().map(JsonValue::Object).collect()),
  );
  JsonValue::Object(schema)
}

/// Schema for tools with no arguments.
#[must_use]
pub fn empty_object_schema() -> JsonValue {
  create_object_schema(&[])
}

fn action_condition(actions: &[&str], then_schema: JsonValue) -> BTreeMap<String, JsonValue> {
  let mut condition = BTreeMap::new();
  let mut any_action = Vec::new();
  for field in ["action", "command"] {
    let mut properties = BTreeMap::new();
    let mut action_schema = BTreeMap::new();
    action_schema.insert(
      "enum".to_string(),
      JsonValue::Array(
        actions
          .iter()
          .map(|value| JsonValue::from(*value))
          .collect(),
      ),
    );
    properties.insert(field.to_string(), JsonValue::Object(action_schema));
    let mut branch = BTreeMap::new();
    branch.insert(
      "required".to_string(),
      JsonValue::Array(vec![JsonValue::from(field)]),
    );
    branch.insert("properties".to_string(), JsonValue::Object(properties));
    if field == "command" {
      let mut not_action = BTreeMap::new();
      not_action.insert(
        "required".to_string(),
        JsonValue::Array(vec![JsonValue::from("action")]),
      );
      branch.insert("not".to_string(), JsonValue::Object(not_action));
    }
    any_action.push(JsonValue::Object(branch));
  }
  let mut if_schema = BTreeMap::new();
  if_schema.insert("anyOf".to_string(), JsonValue::Array(any_action));
  condition.insert("if".to_string(), JsonValue::Object(if_schema));
  condition.insert("then".to_string(), then_schema);
  condition
}

fn required_fields(fields: &[&str]) -> JsonValue {
  let mut schema = BTreeMap::new();
  schema.insert(
    "required".to_string(),
    JsonValue::Array(fields.iter().map(|field| JsonValue::from(*field)).collect()),
  );
  JsonValue::Object(schema)
}

fn any_required_fields<const N: usize>(alternatives: &[[&str; N]]) -> JsonValue {
  let mut schema = BTreeMap::new();
  schema.insert(
    "anyOf".to_string(),
    JsonValue::Array(
      alternatives
        .iter()
        .map(|fields| required_fields(fields))
        .collect(),
    ),
  );
  JsonValue::Object(schema)
}

fn create_object_schema(fields: &[SchemaField]) -> JsonValue {
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
