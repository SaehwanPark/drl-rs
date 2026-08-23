//! Stable, complete JSON projection for the in-memory V1 replay log.

use crate::json::JsonValue;
use drl_protocol::{
  Command, Direction, ItemSpawnKind, MonsterSpawnSpec, PlayerSpawnConfig,
  ProceduralGenerationConfig, ReplayLog, ReplayMetadata, TileKind,
};
use std::collections::BTreeMap;

pub use crate::replay_json_decode::from_json_value;

const FORMAT: &str = "drl-rust-replay-v1";
const SCHEMA_VERSION: u32 = 1;
pub(crate) const MAX_REPLAY_DIMENSION: u32 = 512;
pub(crate) const MAX_PROCEDURAL_ROOMS: u32 = 64;
pub(crate) const MAX_ROOM_SIZE: u32 = 64;
pub(crate) const MAX_CONTENT_PER_ROOM: u32 = 64;

/// Converts every field of a V1 replay log to a deterministic JSON envelope.
///
/// Objects use `BTreeMap` ordering and the projection contains no timestamps,
/// generated identifiers, or other process-local values. This is an export
/// contract only; replay import and validation are intentionally separate.
#[must_use]
pub fn to_json_value(replay: &ReplayLog) -> JsonValue {
  let mut map = BTreeMap::new();
  map.insert("format".to_string(), JsonValue::from(FORMAT));
  map.insert(
    "schema_version".to_string(),
    JsonValue::from(SCHEMA_VERSION),
  );
  map.insert(
    "version".to_string(),
    JsonValue::from(replay.version as u32),
  );
  map.insert("metadata".to_string(), metadata_to_json(&replay.metadata));
  map.insert(
    "player_config".to_string(),
    replay
      .player_config
      .as_ref()
      .map_or(JsonValue::Null, player_config_to_json),
  );
  map.insert(
    "procedural_config".to_string(),
    replay
      .procedural_config
      .as_ref()
      .map_or(JsonValue::Null, procedural_config_to_json),
  );
  map.insert("seed".to_string(), uint64_to_json(replay.seed));
  map.insert("width".to_string(), JsonValue::from(replay.width));
  map.insert("height".to_string(), JsonValue::from(replay.height));
  map.insert(
    "player_start".to_string(),
    position_to_json(replay.player_start),
  );
  map.insert(
    "initial_stairs".to_string(),
    replay
      .initial_stairs
      .map_or(JsonValue::Null, position_to_json),
  );
  map.insert(
    "initial_monsters".to_string(),
    JsonValue::Array(
      replay
        .initial_monsters
        .iter()
        .map(monster_to_json)
        .collect(),
    ),
  );
  map.insert(
    "initial_items".to_string(),
    JsonValue::Array(replay.initial_items.iter().map(item_spec_to_json).collect()),
  );
  map.insert(
    "custom_tiles".to_string(),
    JsonValue::Array(
      replay
        .custom_tiles
        .iter()
        .map(|(position, kind)| {
          object([
            ("position", position_to_json(*position)),
            ("kind", tile_kind_to_json(*kind)),
          ])
        })
        .collect(),
    ),
  );
  map.insert(
    "commands".to_string(),
    JsonValue::Array(replay.commands.iter().map(command_to_json).collect()),
  );
  JsonValue::Object(map)
}

fn metadata_to_json(metadata: &ReplayMetadata) -> JsonValue {
  object([
    ("version", JsonValue::from(metadata.version as u32)),
    (
      "engine_name",
      JsonValue::from(metadata.engine_name.as_str()),
    ),
    (
      "engine_version",
      JsonValue::from(metadata.engine_version.as_str()),
    ),
  ])
}

fn player_config_to_json(config: &PlayerSpawnConfig) -> JsonValue {
  object([
    ("hp", JsonValue::from(config.hp)),
    ("max_hp", JsonValue::from(config.max_hp)),
    ("speed", JsonValue::from(config.speed)),
    (
      "initial_items",
      JsonValue::Array(
        config
          .initial_items
          .iter()
          .map(|kind| item_kind_to_json(*kind))
          .collect(),
      ),
    ),
    (
      "equipped_weapon",
      config
        .equipped_weapon
        .map_or(JsonValue::Null, item_kind_to_json),
    ),
    (
      "equipped_armor",
      config
        .equipped_armor
        .map_or(JsonValue::Null, item_kind_to_json),
    ),
  ])
}

fn procedural_config_to_json(config: &ProceduralGenerationConfig) -> JsonValue {
  object([
    ("max_rooms", JsonValue::from(config.max_rooms)),
    ("min_room_size", JsonValue::from(config.min_room_size)),
    ("max_room_size", JsonValue::from(config.max_room_size)),
    (
      "max_monsters_per_room",
      JsonValue::from(config.max_monsters_per_room),
    ),
    (
      "max_items_per_room",
      JsonValue::from(config.max_items_per_room),
    ),
  ])
}

fn monster_to_json(monster: &MonsterSpawnSpec) -> JsonValue {
  object([
    ("position", position_to_json(monster.position)),
    ("name", JsonValue::from(monster.name.as_str())),
    ("hp", JsonValue::from(monster.hp)),
    ("speed", JsonValue::from(monster.speed)),
    ("melee_damage", damage_to_json(monster.melee_damage)),
    (
      "ranged_damage",
      monster
        .ranged_damage
        .map_or(JsonValue::Null, damage_to_json),
    ),
    ("ranged_range", JsonValue::from(monster.ranged_range)),
    ("accuracy", JsonValue::from(monster.accuracy)),
    (
      "death_drop",
      monster
        .death_drop
        .map_or(JsonValue::Null, item_kind_to_json),
    ),
  ])
}

fn item_spec_to_json(spec: &drl_protocol::ItemSpawnSpec) -> JsonValue {
  object([
    ("position", position_to_json(spec.position)),
    ("kind", item_kind_to_json(spec.kind)),
  ])
}

fn item_kind_to_json(kind: ItemSpawnKind) -> JsonValue {
  let (name, count) = match kind {
    ItemSpawnKind::Pistol => ("pistol", None),
    ItemSpawnKind::Shotgun => ("shotgun", None),
    ItemSpawnKind::CombatKnife => ("combat_knife", None),
    ItemSpawnKind::Ammo9mm(count) => ("ammo_9mm", Some(count)),
    ItemSpawnKind::AmmoShells(count) => ("ammo_shells", Some(count)),
    ItemSpawnKind::SmallMedPack => ("small_medpack", None),
    ItemSpawnKind::LargeMedPack => ("large_medpack", None),
    ItemSpawnKind::GreenArmor => ("green_armor", None),
    ItemSpawnKind::PhaseDevice => ("phase_device", None),
  };
  let mut map = BTreeMap::new();
  map.insert("kind".to_string(), JsonValue::from(name));
  if let Some(count) = count {
    map.insert("count".to_string(), JsonValue::from(count));
  }
  JsonValue::Object(map)
}

fn tile_kind_to_json(kind: TileKind) -> JsonValue {
  JsonValue::from(match kind {
    TileKind::Floor => "floor",
    TileKind::Wall => "wall",
    TileKind::DoorClosed => "door_closed",
    TileKind::DoorOpen => "door_open",
    TileKind::StairsDown => "stairs_down",
  })
}

fn command_to_json(command: &Command) -> JsonValue {
  match command {
    Command::Move(direction) => action_with_direction("move", *direction),
    Command::AttackMelee(direction) => action_with_direction("attack_melee", *direction),
    Command::AttackRanged(position) => object([
      ("action", JsonValue::from("fire")),
      ("target_x", JsonValue::from(position.x)),
      ("target_y", JsonValue::from(position.y)),
    ]),
    Command::Wait => action("wait"),
    Command::Pickup => action("pickup"),
    Command::Drop(item_id) => item_action("drop", item_id.as_u64()),
    Command::Equip(item_id) => item_action("equip", item_id.as_u64()),
    Command::Unequip(slot) => object([
      ("action", JsonValue::from("unequip")),
      (
        "slot",
        JsonValue::from(match slot {
          drl_protocol::EquipmentSlot::Weapon => "Weapon",
          drl_protocol::EquipmentSlot::Armor => "Armor",
        }),
      ),
    ]),
    Command::Use(item_id) => item_action("use", item_id.as_u64()),
    Command::Reload => action("reload"),
    Command::Descend => action("descend"),
  }
}

fn action(name: &str) -> JsonValue {
  object([("action", JsonValue::from(name))])
}

fn action_with_direction(name: &str, direction: Direction) -> JsonValue {
  object([
    ("action", JsonValue::from(name)),
    ("direction", JsonValue::from(direction_name(direction))),
  ])
}

fn item_action(name: &str, item_id: u64) -> JsonValue {
  object([
    ("action", JsonValue::from(name)),
    ("item_id", uint64_to_json(item_id)),
  ])
}

fn uint64_to_json(value: u64) -> JsonValue {
  JsonValue::RawNumber(value.to_string())
}

fn direction_name(direction: Direction) -> &'static str {
  match direction {
    Direction::None => "None",
    Direction::North => "North",
    Direction::NorthEast => "NorthEast",
    Direction::East => "East",
    Direction::SouthEast => "SouthEast",
    Direction::South => "South",
    Direction::SouthWest => "SouthWest",
    Direction::West => "West",
    Direction::NorthWest => "NorthWest",
  }
}

fn damage_to_json((min, max): (u32, u32)) -> JsonValue {
  object([("min", JsonValue::from(min)), ("max", JsonValue::from(max))])
}

fn position_to_json(position: drl_protocol::Position) -> JsonValue {
  object([
    ("x", JsonValue::from(position.x)),
    ("y", JsonValue::from(position.y)),
  ])
}

fn object<const N: usize>(entries: [(&str, JsonValue); N]) -> JsonValue {
  JsonValue::Object(
    entries
      .into_iter()
      .map(|(key, value)| (key.to_string(), value))
      .collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use drl_protocol::{EquipmentSlot, ItemId, Position};

  #[test]
  fn command_export_covers_every_variant_with_typed_fields() {
    let commands = [
      Command::Move(Direction::North),
      Command::AttackMelee(Direction::SouthWest),
      Command::AttackRanged(Position::new(i32::MIN, i32::MAX)),
      Command::Wait,
      Command::Pickup,
      Command::Drop(ItemId::new(9_007_199_254_740_992)),
      Command::Equip(ItemId::new(7)),
      Command::Unequip(EquipmentSlot::Armor),
      Command::Use(ItemId::new(8)),
      Command::Reload,
      Command::Descend,
    ];
    let values: Vec<_> = commands.iter().map(command_to_json).collect();
    assert_eq!(
      values[0].get("action").and_then(JsonValue::as_str),
      Some("move")
    );
    assert_eq!(
      values[2].get("target_x").and_then(JsonValue::as_i64),
      Some(i64::from(i32::MIN))
    );
    assert_eq!(
      values[5].get("item_id").and_then(JsonValue::as_u64),
      Some(9_007_199_254_740_992)
    );
    assert_eq!(
      values[7].get("slot").and_then(JsonValue::as_str),
      Some("Armor")
    );
    assert_eq!(
      values[10].get("action").and_then(JsonValue::as_str),
      Some("descend")
    );
  }

  #[test]
  fn replay_export_is_complete_and_repeatable() {
    let player_config = PlayerSpawnConfig {
      hp: 30,
      max_hp: 60,
      speed: 90,
      initial_items: vec![
        ItemSpawnKind::Pistol,
        ItemSpawnKind::Shotgun,
        ItemSpawnKind::CombatKnife,
        ItemSpawnKind::Ammo9mm(20),
        ItemSpawnKind::AmmoShells(10),
        ItemSpawnKind::SmallMedPack,
        ItemSpawnKind::LargeMedPack,
        ItemSpawnKind::GreenArmor,
        ItemSpawnKind::PhaseDevice,
      ],
      equipped_weapon: Some(ItemSpawnKind::Shotgun),
      equipped_armor: Some(ItemSpawnKind::GreenArmor),
    };
    let mut replay = ReplayLog::new(42, 12, 10, Position::new(1, 2))
      .with_procedural_config(ProceduralGenerationConfig {
        max_rooms: 5,
        min_room_size: 4,
        max_room_size: 8,
        max_monsters_per_room: 2,
        max_items_per_room: 2,
      })
      .with_player_config(player_config)
      .with_metadata(ReplayMetadata {
        version: drl_protocol::ReplayVersion::V1,
        engine_name: "fixture-engine".to_string(),
        engine_version: "test".to_string(),
      });
    replay.record_stairs(Position::new(4, 5));
    replay.record_monster(
      MonsterSpawnSpec::new(Position::new(6, 7), "Imp", 20, 100, (3, 8))
        .with_ranged_combat((2, 5), 7, 70)
        .with_death_drop(Some(ItemSpawnKind::LargeMedPack)),
    );
    replay.record_item(drl_protocol::ItemSpawnSpec::new(
      Position::new(2, 3),
      ItemSpawnKind::Ammo9mm(20),
    ));
    for (position, kind) in [
      (Position::new(1, 1), TileKind::Floor),
      (Position::new(2, 1), TileKind::Wall),
      (Position::new(3, 1), TileKind::DoorClosed),
      (Position::new(4, 1), TileKind::DoorOpen),
      (Position::new(5, 1), TileKind::StairsDown),
    ] {
      replay.record_tile(position, kind);
    }
    replay.commands = vec![
      Command::Move(Direction::North),
      Command::AttackMelee(Direction::SouthWest),
      Command::AttackRanged(Position::new(-4, 5)),
      Command::Wait,
      Command::Pickup,
      Command::Drop(drl_protocol::ItemId::new(1)),
      Command::Equip(drl_protocol::ItemId::new(2)),
      Command::Unequip(drl_protocol::EquipmentSlot::Armor),
      Command::Use(drl_protocol::ItemId::new(3)),
      Command::Reload,
      Command::Descend,
    ];
    let first = to_json_value(&replay).to_compact_string();
    let second = to_json_value(&replay).to_compact_string();
    assert_eq!(first, second);
    assert!(JsonValue::parse(&first).is_ok());
    let decoded = from_json_value(&JsonValue::parse(&first).unwrap()).unwrap();
    assert_eq!(decoded, replay);
    let value = to_json_value(&replay);
    assert_eq!(
      value.get("format").and_then(JsonValue::as_str),
      Some(FORMAT)
    );
    assert!(value.get("metadata").is_some());
    assert!(value.get("player_config").is_some());
    assert!(value.get("procedural_config").is_some());
    assert!(value.get("initial_monsters").is_some());
    assert!(value.get("initial_items").is_some());
    assert!(value.get("custom_tiles").is_some());
    assert_eq!(
      value
        .get("commands")
        .and_then(JsonValue::as_array)
        .map(Vec::len),
      Some(11)
    );
  }

  #[test]
  fn replay_export_preserves_u64_values_above_json_safe_integer_limit() {
    let seed = 9_007_199_254_740_993;
    let item_id = 9_007_199_254_740_994;
    let mut replay = ReplayLog::new(seed, 4, 4, Position::new(1, 1));
    replay.commands.push(Command::Drop(ItemId::new(item_id)));

    let value = to_json_value(&replay);
    let decoded = from_json_value(&value).unwrap();
    assert_eq!(decoded.seed, seed);
    assert_eq!(decoded.commands, vec![Command::Drop(ItemId::new(item_id))]);
    let mut extended = value.clone();
    extended
      .as_object_mut()
      .unwrap()
      .insert("future_field".to_string(), JsonValue::Bool(true));
    assert_eq!(from_json_value(&extended).unwrap(), replay);
    assert_eq!(
      value.get("seed").map(JsonValue::to_compact_string),
      Some(seed.to_string())
    );
    assert_eq!(
      value
        .get("commands")
        .and_then(JsonValue::as_array)
        .and_then(|commands| commands.first())
        .and_then(|command| command.get("item_id"))
        .map(JsonValue::to_compact_string),
      Some(item_id.to_string())
    );
  }
}
