//! Exact decoder for the canonical in-memory V1 replay JSON envelope.

use crate::json::JsonValue;
use crate::replay_json::{
  MAX_CONTENT_PER_ROOM, MAX_PROCEDURAL_ROOMS, MAX_REPLAY_DIMENSION, MAX_ROOM_SIZE,
};
use drl_protocol::{
  Command, Direction, EquipmentSlot, ItemSpawnKind, ItemSpawnSpec, MonsterSpawnSpec,
  PlayerSpawnConfig, Position, ProceduralGenerationConfig, ReplayLog, ReplayMetadata,
  ReplayVersion, TileKind,
};
use std::collections::BTreeMap;

const FORMAT: &str = "drl-rust-replay-v1";
const MAX_INITIAL_ENTITIES: usize = 4_096;
const MAX_CUSTOM_TILES: usize = 65_536;
const MAX_COMMANDS: usize = 100_000;

/// Decodes the exact JSON envelope emitted by `replay_json::to_json_value`.
///
/// Unknown object properties are ignored, but every canonical field and nested
/// value is required to have its documented type. This decoder only constructs
/// an in-memory replay for read-only verification; it never activates a session.
pub fn from_json_value(value: &JsonValue) -> Result<ReplayLog, String> {
  let object = object(value, "replay envelope")?;
  if string(required(object, "format")?, "format")? != FORMAT {
    return Err("replay format must be drl-rust-replay-v1".to_string());
  }
  if u64_value(required(object, "schema_version")?, "schema_version")? != 1 {
    return Err("replay schema_version must be 1".to_string());
  }
  if replay_version(required(object, "version")?, "version")? != ReplayVersion::V1 {
    return Err("replay version must be V1".to_string());
  }

  let metadata = parse_metadata(required(object, "metadata")?)?;
  let player_config = nullable(object, "player_config", parse_player_config)?;
  let procedural_config = nullable(object, "procedural_config", parse_procedural_config)?;
  let max_turns = nullable(object, "max_turns", |value| u64_value(value, "max_turns"))?;
  let seed = u64_value(required(object, "seed")?, "seed")?;
  let width = u32_value(required(object, "width")?, "width")?;
  let height = u32_value(required(object, "height")?, "height")?;
  let player_start = parse_position(required(object, "player_start")?, "player_start")?;
  let initial_stairs = nullable(object, "initial_stairs", |value| {
    parse_position(value, "initial_stairs")
  })?;
  let initial_monsters = bounded_array(
    required(object, "initial_monsters")?,
    "initial_monsters",
    MAX_INITIAL_ENTITIES,
  )?
  .iter()
  .enumerate()
  .map(|(index, value)| parse_monster(value, index))
  .collect::<Result<Vec<_>, _>>()?;
  let initial_items = bounded_array(
    required(object, "initial_items")?,
    "initial_items",
    MAX_INITIAL_ENTITIES,
  )?
  .iter()
  .enumerate()
  .map(|(index, value)| parse_item_spec(value, index))
  .collect::<Result<Vec<_>, _>>()?;
  let custom_tiles = bounded_array(
    required(object, "custom_tiles")?,
    "custom_tiles",
    MAX_CUSTOM_TILES,
  )?
  .iter()
  .enumerate()
  .map(|(index, value)| parse_custom_tile(value, index))
  .collect::<Result<Vec<_>, _>>()?;
  let commands = bounded_array(required(object, "commands")?, "commands", MAX_COMMANDS)?
    .iter()
    .enumerate()
    .map(|(index, value)| parse_command(value, index))
    .collect::<Result<Vec<_>, _>>()?;

  let replay = ReplayLog {
    version: ReplayVersion::V1,
    metadata,
    player_config,
    procedural_config,
    max_turns,
    seed,
    width,
    height,
    player_start,
    initial_stairs,
    initial_monsters,
    initial_items,
    custom_tiles,
    commands,
  };
  validate_replay_safety(&replay)?;
  Ok(replay)
}

fn parse_metadata(value: &JsonValue) -> Result<ReplayMetadata, String> {
  let object = object(value, "metadata")?;
  Ok(ReplayMetadata {
    version: replay_version(required(object, "version")?, "metadata.version")?,
    engine_name: string(required(object, "engine_name")?, "metadata.engine_name")?.to_string(),
    engine_version: string(
      required(object, "engine_version")?,
      "metadata.engine_version",
    )?
    .to_string(),
  })
}

fn parse_player_config(value: &JsonValue) -> Result<PlayerSpawnConfig, String> {
  let object = object(value, "player_config")?;
  let initial_items = bounded_array(
    required(object, "initial_items")?,
    "player_config.initial_items",
    MAX_INITIAL_ENTITIES,
  )?
  .iter()
  .enumerate()
  .map(|(index, value)| item_kind(value, &format!("player_config.initial_items[{index}]")))
  .collect::<Result<Vec<_>, _>>()?;
  Ok(PlayerSpawnConfig {
    hp: u32_value(required(object, "hp")?, "player_config.hp")?,
    max_hp: u32_value(required(object, "max_hp")?, "player_config.max_hp")?,
    speed: u32_value(required(object, "speed")?, "player_config.speed")?,
    initial_items,
    equipped_weapon: nullable(object, "equipped_weapon", |value| {
      item_kind(value, "player_config.equipped_weapon")
    })?,
    equipped_armor: nullable(object, "equipped_armor", |value| {
      item_kind(value, "player_config.equipped_armor")
    })?,
  })
}

fn parse_procedural_config(value: &JsonValue) -> Result<ProceduralGenerationConfig, String> {
  let object = object(value, "procedural_config")?;
  Ok(ProceduralGenerationConfig {
    max_rooms: u32_value(
      required(object, "max_rooms")?,
      "procedural_config.max_rooms",
    )?,
    min_room_size: u32_value(
      required(object, "min_room_size")?,
      "procedural_config.min_room_size",
    )?,
    max_room_size: u32_value(
      required(object, "max_room_size")?,
      "procedural_config.max_room_size",
    )?,
    max_monsters_per_room: u32_value(
      required(object, "max_monsters_per_room")?,
      "procedural_config.max_monsters_per_room",
    )?,
    max_items_per_room: u32_value(
      required(object, "max_items_per_room")?,
      "procedural_config.max_items_per_room",
    )?,
  })
}

fn parse_monster(value: &JsonValue, index: usize) -> Result<MonsterSpawnSpec, String> {
  let context = format!("initial_monsters[{index}]");
  let object = object(value, &context)?;
  Ok(MonsterSpawnSpec {
    position: parse_position(
      required(object, "position")?,
      &format!("{context}.position"),
    )?,
    name: string(required(object, "name")?, &format!("{context}.name"))?.to_string(),
    hp: u32_value(required(object, "hp")?, &format!("{context}.hp"))?,
    speed: u32_value(required(object, "speed")?, &format!("{context}.speed"))?,
    melee_damage: damage(
      required(object, "melee_damage")?,
      &format!("{context}.melee_damage"),
    )?,
    ranged_damage: nullable(object, "ranged_damage", |value| {
      damage(value, &format!("{context}.ranged_damage"))
    })?,
    ranged_range: u32_value(
      required(object, "ranged_range")?,
      &format!("{context}.ranged_range"),
    )?,
    accuracy: i32_value(
      required(object, "accuracy")?,
      &format!("{context}.accuracy"),
    )?,
    death_drop: nullable(object, "death_drop", |value| {
      item_kind(value, &format!("{context}.death_drop"))
    })?,
  })
}

fn parse_item_spec(value: &JsonValue, index: usize) -> Result<ItemSpawnSpec, String> {
  let context = format!("initial_items[{index}]");
  let object = object(value, &context)?;
  Ok(ItemSpawnSpec::new(
    parse_position(
      required(object, "position")?,
      &format!("{context}.position"),
    )?,
    item_kind(required(object, "kind")?, &format!("{context}.kind"))?,
  ))
}

fn parse_custom_tile(value: &JsonValue, index: usize) -> Result<(Position, TileKind), String> {
  let context = format!("custom_tiles[{index}]");
  let object = object(value, &context)?;
  Ok((
    parse_position(
      required(object, "position")?,
      &format!("{context}.position"),
    )?,
    tile_kind(required(object, "kind")?, &format!("{context}.kind"))?,
  ))
}

fn parse_command(value: &JsonValue, index: usize) -> Result<Command, String> {
  let context = format!("commands[{index}]");
  let object = object(value, &context)?;
  let action = string(required(object, "action")?, &format!("{context}.action"))?;
  match action {
    "move" => Ok(Command::Move(direction(
      required(object, "direction")?,
      &format!("{context}.direction"),
    )?)),
    "attack_melee" => Ok(Command::AttackMelee(direction(
      required(object, "direction")?,
      &format!("{context}.direction"),
    )?)),
    "fire" => Ok(Command::AttackRanged(Position::new(
      i32_value(
        required(object, "target_x")?,
        &format!("{context}.target_x"),
      )?,
      i32_value(
        required(object, "target_y")?,
        &format!("{context}.target_y"),
      )?,
    ))),
    "wait" => Ok(Command::Wait),
    "pickup" => Ok(Command::Pickup),
    "drop" => Ok(Command::Drop(drl_protocol::ItemId::new(u64_value(
      required(object, "item_id")?,
      &format!("{context}.item_id"),
    )?))),
    "equip" => Ok(Command::Equip(drl_protocol::ItemId::new(u64_value(
      required(object, "item_id")?,
      &format!("{context}.item_id"),
    )?))),
    "unequip" => Ok(Command::Unequip(slot(
      required(object, "slot")?,
      &format!("{context}.slot"),
    )?)),
    "use" => Ok(Command::Use(drl_protocol::ItemId::new(u64_value(
      required(object, "item_id")?,
      &format!("{context}.item_id"),
    )?))),
    "reload" => Ok(Command::Reload),
    "descend" => Ok(Command::Descend),
    _ => Err(format!(
      "{context}.action has unsupported action '{action}'"
    )),
  }
}

fn item_kind(value: &JsonValue, context: &str) -> Result<ItemSpawnKind, String> {
  let object = object(value, context)?;
  let kind = string(required(object, "kind")?, &format!("{context}.kind"))?;
  match kind {
    "pistol" => Ok(ItemSpawnKind::Pistol),
    "shotgun" => Ok(ItemSpawnKind::Shotgun),
    "combat_knife" => Ok(ItemSpawnKind::CombatKnife),
    "ammo_9mm" => Ok(ItemSpawnKind::Ammo9mm(u32_value(
      required(object, "count")?,
      &format!("{context}.count"),
    )?)),
    "ammo_shells" => Ok(ItemSpawnKind::AmmoShells(u32_value(
      required(object, "count")?,
      &format!("{context}.count"),
    )?)),
    "ammo_rockets" => Ok(ItemSpawnKind::AmmoRockets(u32_value(
      required(object, "count")?,
      &format!("{context}.count"),
    )?)),
    "ammo_cells" => Ok(ItemSpawnKind::AmmoCells(u32_value(
      required(object, "count")?,
      &format!("{context}.count"),
    )?)),
    "ammo_pack_rockets" => Ok(ItemSpawnKind::AmmoPackRockets),
    "ammo_pack_cells" => Ok(ItemSpawnKind::AmmoPackCells),
    "small_medpack" => Ok(ItemSpawnKind::SmallMedPack),
    "large_medpack" => Ok(ItemSpawnKind::LargeMedPack),
    "green_armor" => Ok(ItemSpawnKind::GreenArmor),
    "phase_device" => Ok(ItemSpawnKind::PhaseDevice),
    _ => Err(format!("{context}.kind has unsupported item kind '{kind}'")),
  }
}

fn tile_kind(value: &JsonValue, context: &str) -> Result<TileKind, String> {
  let kind = string(value, context)?;
  match kind {
    "floor" => Ok(TileKind::Floor),
    "wall" => Ok(TileKind::Wall),
    "door_closed" => Ok(TileKind::DoorClosed),
    "door_open" => Ok(TileKind::DoorOpen),
    "stairs_down" => Ok(TileKind::StairsDown),
    _ => Err(format!("{context} has unsupported tile kind '{kind}'")),
  }
}

fn direction(value: &JsonValue, context: &str) -> Result<Direction, String> {
  let direction = string(value, context)?;
  match direction {
    "None" => Ok(Direction::None),
    "North" => Ok(Direction::North),
    "NorthEast" => Ok(Direction::NorthEast),
    "East" => Ok(Direction::East),
    "SouthEast" => Ok(Direction::SouthEast),
    "South" => Ok(Direction::South),
    "SouthWest" => Ok(Direction::SouthWest),
    "West" => Ok(Direction::West),
    "NorthWest" => Ok(Direction::NorthWest),
    _ => Err(format!("{context} has unsupported direction '{direction}'")),
  }
}

fn slot(value: &JsonValue, context: &str) -> Result<EquipmentSlot, String> {
  let slot = string(value, context)?;
  match slot {
    "Weapon" => Ok(EquipmentSlot::Weapon),
    "Armor" => Ok(EquipmentSlot::Armor),
    _ => Err(format!("{context} has unsupported equipment slot '{slot}'")),
  }
}

fn damage(value: &JsonValue, context: &str) -> Result<(u32, u32), String> {
  let object = object(value, context)?;
  Ok((
    u32_value(required(object, "min")?, &format!("{context}.min"))?,
    u32_value(required(object, "max")?, &format!("{context}.max"))?,
  ))
}

fn parse_position(value: &JsonValue, context: &str) -> Result<Position, String> {
  let object = object(value, context)?;
  Ok(Position::new(
    i32_value(required(object, "x")?, &format!("{context}.x"))?,
    i32_value(required(object, "y")?, &format!("{context}.y"))?,
  ))
}

fn replay_version(value: &JsonValue, context: &str) -> Result<ReplayVersion, String> {
  match u64_value(value, context)? {
    1 => Ok(ReplayVersion::V1),
    _ => Err(format!("{context} must be replay version V1")),
  }
}

fn u64_value(value: &JsonValue, context: &str) -> Result<u64, String> {
  match value {
    JsonValue::Number(number)
      if number.is_finite()
        && *number >= 0.0
        && number.fract() == 0.0
        && *number <= 9_007_199_254_740_992.0 =>
    {
      Ok(*number as u64)
    }
    JsonValue::RawNumber(raw) => raw
      .parse::<u64>()
      .map_err(|_| format!("{context} must be an exact non-negative u64")),
    _ => Err(format!("{context} must be an exact non-negative u64")),
  }
}

fn u32_value(value: &JsonValue, context: &str) -> Result<u32, String> {
  u64_value(value, context)
    .and_then(|value| u32::try_from(value).map_err(|_| format!("{context} must fit in u32")))
}

fn i32_value(value: &JsonValue, context: &str) -> Result<i32, String> {
  match value {
    JsonValue::Number(number)
      if number.is_finite()
        && number.fract() == 0.0
        && *number >= f64::from(i32::MIN)
        && *number <= f64::from(i32::MAX) =>
    {
      Ok(*number as i32)
    }
    JsonValue::RawNumber(raw) => raw
      .parse::<i64>()
      .ok()
      .and_then(|value| i32::try_from(value).ok())
      .ok_or_else(|| format!("{context} must be an exact i32")),
    _ => Err(format!("{context} must be an exact i32")),
  }
}

fn string<'a>(value: &'a JsonValue, context: &str) -> Result<&'a str, String> {
  value
    .as_str()
    .ok_or_else(|| format!("{context} must be a string"))
}

fn object<'a>(
  value: &'a JsonValue,
  context: &str,
) -> Result<&'a BTreeMap<String, JsonValue>, String> {
  value
    .as_object()
    .ok_or_else(|| format!("{context} must be an object"))
}

fn array<'a>(value: &'a JsonValue, context: &str) -> Result<&'a Vec<JsonValue>, String> {
  value
    .as_array()
    .ok_or_else(|| format!("{context} must be an array"))
}

fn bounded_array<'a>(
  value: &'a JsonValue,
  context: &str,
  max_len: usize,
) -> Result<&'a Vec<JsonValue>, String> {
  let values = array(value, context)?;
  if values.len() > max_len {
    return Err(format!("{context} exceeds the bounded length {max_len}"));
  }
  Ok(values)
}

fn validate_replay_safety(replay: &ReplayLog) -> Result<(), String> {
  if !(3..=MAX_REPLAY_DIMENSION).contains(&replay.width)
    || !(3..=MAX_REPLAY_DIMENSION).contains(&replay.height)
  {
    return Err(format!(
      "replay dimensions must be within 3..={MAX_REPLAY_DIMENSION}"
    ));
  }
  let in_bounds = |position: Position| {
    position.x >= 0
      && position.y >= 0
      && (position.x as u32) < replay.width
      && (position.y as u32) < replay.height
  };
  if !in_bounds(replay.player_start) {
    return Err("player_start is outside replay dimensions".to_string());
  }
  if replay
    .initial_stairs
    .is_some_and(|position| !in_bounds(position))
  {
    return Err("initial_stairs is outside replay dimensions".to_string());
  }
  if replay
    .initial_monsters
    .iter()
    .any(|monster| !in_bounds(monster.position))
  {
    return Err("initial_monsters contains an out-of-bounds position".to_string());
  }
  if replay
    .initial_items
    .iter()
    .any(|item| !in_bounds(item.position))
  {
    return Err("initial_items contains an out-of-bounds position".to_string());
  }
  if replay
    .custom_tiles
    .iter()
    .any(|(position, _)| !in_bounds(*position))
  {
    return Err("custom_tiles contains an out-of-bounds position".to_string());
  }
  if let Some(config) = &replay.procedural_config
    && (config.max_rooms > MAX_PROCEDURAL_ROOMS
      || config.min_room_size == 0
      || config.min_room_size > config.max_room_size
      || config.max_room_size > MAX_ROOM_SIZE
      || config.max_monsters_per_room > MAX_CONTENT_PER_ROOM
      || config.max_items_per_room > MAX_CONTENT_PER_ROOM)
  {
    return Err("procedural replay configuration exceeds safe bounds".to_string());
  }
  Ok(())
}

fn required<'a>(
  object: &'a BTreeMap<String, JsonValue>,
  name: &str,
) -> Result<&'a JsonValue, String> {
  object
    .get(name)
    .ok_or_else(|| format!("missing required replay field '{name}'"))
}

fn nullable<T>(
  object: &BTreeMap<String, JsonValue>,
  name: &str,
  parser: impl FnOnce(&JsonValue) -> Result<T, String>,
) -> Result<Option<T>, String> {
  match required(object, name)? {
    JsonValue::Null => Ok(None),
    value => parser(value).map(Some),
  }
}
