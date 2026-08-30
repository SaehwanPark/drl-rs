//! MCP semantic session manager, legal action synthesis, and simulation bridge.

use crate::json::JsonValue;
use crate::replay_json::MAX_REPLAY_DIMENSION;
use drl_core::generator::LevelGeneratorConfig;
use drl_core::grid::Tile;
use drl_core::scenario::Scenario;
use drl_core::{
  Game, ReplayEngine, bfg10k_chainfire_profile, chaingun_chainfire_profile,
  laser_rifle_chainfire_profile, minigun_chainfire_profile, nuclear_plasma_chainfire_profile,
  plasma_rifle_chainfire_profile,
};
use drl_protocol::{
  Command, Direction, EpisodeMetrics, EquipmentSlot, GameEvent, ItemArchetype, ItemCategory,
  ItemId, ItemView, OmniscientObservation, PlayerObservation, Position, ProceduralGenerationConfig,
  ReplayLog, RunOutcome, TileKind,
};
use std::collections::BTreeMap;

/// Represents a legal semantic action that an MCP agent or AI player can submit.
#[derive(Debug, Clone, PartialEq)]
pub struct LegalAction {
  /// Category / action name string (e.g. "Move", "AttackRanged", "Reload", "Wait").
  pub action: String,
  /// Human-readable explanation of what this action achieves.
  pub description: String,
  /// Underlying typed engine command.
  pub command: Command,
  /// Structured parameters representation for MCP tool arguments.
  pub params: JsonValue,
}

fn chainfire_profile(archetype: ItemArchetype, level: u8) -> Option<(u32, u32)> {
  match archetype {
    ItemArchetype::Bfg10k => bfg10k_chainfire_profile(level),
    ItemArchetype::Chaingun => chaingun_chainfire_profile(level),
    ItemArchetype::Minigun => minigun_chainfire_profile(level),
    ItemArchetype::PlasmaRifle => plasma_rifle_chainfire_profile(level),
    ItemArchetype::LaserRifle => laser_rifle_chainfire_profile(level),
    ItemArchetype::NuclearPlasmaRifle => nuclear_plasma_chainfire_profile(level),
    _ => None,
  }
}

impl LegalAction {
  /// Converts this legal action into a JSON object.
  #[must_use]
  pub fn to_json_value(&self) -> JsonValue {
    let mut map = BTreeMap::new();
    map.insert("action".to_string(), JsonValue::String(self.action.clone()));
    map.insert(
      "description".to_string(),
      JsonValue::String(self.description.clone()),
    );
    map.insert("params".to_string(), self.params.clone());
    JsonValue::Object(map)
  }
}

/// Computes the fair, currently advertised actions from player observation.
///
/// This catalog intentionally leaves hidden geometry and dynamic core checks to
/// `drl_core::Game::step` rather than claiming to be an exhaustive rules engine.
#[must_use]
pub fn compute_legal_actions(obs: &PlayerObservation) -> Vec<LegalAction> {
  let mut actions = Vec::new();

  // 1. Wait is always available
  let mut wait_params = BTreeMap::new();
  wait_params.insert("action".to_string(), JsonValue::from("wait"));
  actions.push(LegalAction {
    action: "Wait".to_string(),
    description: "Wait in place for 1 turn (costs standard energy)".to_string(),
    command: Command::Wait,
    params: JsonValue::Object(wait_params),
  });

  // 2. Cardinal and diagonal movement / bump-attack directions
  let directions = [
    (Direction::North, "North"),
    (Direction::South, "South"),
    (Direction::East, "East"),
    (Direction::West, "West"),
    (Direction::NorthEast, "NorthEast"),
    (Direction::NorthWest, "NorthWest"),
    (Direction::SouthEast, "SouthEast"),
    (Direction::SouthWest, "SouthWest"),
  ];

  for (dir, dir_name) in directions {
    let target_pos = obs.player_position + dir;

    // Check if target position is walkable or occupied by visible monster
    let is_walkable_tile = obs
      .visible_tiles
      .iter()
      .any(|t| t.position == target_pos && t.is_walkable);
    let monster_at_pos = obs
      .visible_actors
      .iter()
      .find(|a| a.position == target_pos && !a.is_player && a.is_alive);

    if is_walkable_tile || monster_at_pos.is_some() {
      let desc = if let Some(m) = monster_at_pos {
        format!("Melee bump-attack {} to the {dir_name}", m.name)
      } else {
        format!("Step {dir_name} to ({}, {})", target_pos.x, target_pos.y)
      };

      let mut p = BTreeMap::new();
      p.insert("action".to_string(), JsonValue::from("move"));
      p.insert("direction".to_string(), JsonValue::from(dir_name));

      actions.push(LegalAction {
        action: "Move".to_string(),
        description: desc,
        command: Command::Move(dir),
        params: JsonValue::Object(p),
      });

      if let Some(monster) = monster_at_pos {
        let mut melee_params = BTreeMap::new();
        melee_params.insert("action".to_string(), JsonValue::from("attack_melee"));
        melee_params.insert("direction".to_string(), JsonValue::from(dir_name));
        actions.push(LegalAction {
          action: "AttackMelee".to_string(),
          description: format!("Direct melee attack {} to the {dir_name}", monster.name),
          command: Command::AttackMelee(dir),
          params: JsonValue::Object(melee_params),
        });
      }
    }
  }

  // 3. Ranged attacks (if ranged weapon equipped with ammo)
  if let Some(ref weapon) = obs.equipped_weapon {
    let has_ammo = weapon.clip.is_none_or(|(loaded, _)| loaded > 0);
    if has_ammo {
      for actor in &obs.visible_actors {
        if !actor.is_player && actor.is_alive {
          let mut p = BTreeMap::new();
          p.insert("action".to_string(), JsonValue::from("fire"));
          p.insert("target_x".to_string(), JsonValue::from(actor.position.x));
          p.insert("target_y".to_string(), JsonValue::from(actor.position.y));

          actions.push(LegalAction {
            action: "Fire".to_string(),
            description: format!(
              "Fire {} at {} at ({}, {})",
              weapon.name, actor.name, actor.position.x, actor.position.y
            ),
            command: Command::AttackRanged(actor.position),
            params: JsonValue::Object(p),
          });

          if let Some((projectile_count, ammo_cost)) =
            chainfire_profile(weapon.archetype, weapon.chainfire_level)
            && weapon.clip.is_some_and(|(loaded, _)| loaded >= ammo_cost)
          {
            let mut chainfire_params = BTreeMap::new();
            chainfire_params.insert("action".to_string(), JsonValue::from("chainfire"));
            chainfire_params.insert("target_x".to_string(), JsonValue::from(actor.position.x));
            chainfire_params.insert("target_y".to_string(), JsonValue::from(actor.position.y));
            actions.push(LegalAction {
              action: "Chainfire".to_string(),
              description: format!(
                "Chainfire {} at {} at ({}, {}) ({projectile_count} projectiles, {ammo_cost} rounds)",
                weapon.name, actor.name, actor.position.x, actor.position.y
              ),
              command: Command::AttackRangedChainfire(actor.position),
              params: JsonValue::Object(chainfire_params),
            });
          }

          if matches!(
            weapon.archetype,
            ItemArchetype::Pistol
              | ItemArchetype::CombatPistol
              | ItemArchetype::Blaster
              | ItemArchetype::Trigun
              | ItemArchetype::AntiFreakJackal
          ) {
            let mut aimed_params = BTreeMap::new();
            aimed_params.insert("action".to_string(), JsonValue::from("aimed_fire"));
            aimed_params.insert("target_x".to_string(), JsonValue::from(actor.position.x));
            aimed_params.insert("target_y".to_string(), JsonValue::from(actor.position.y));
            actions.push(LegalAction {
              action: "AimedFire".to_string(),
              description: format!(
                "Aim and fire {} at {} at ({}, {}) (+3 accuracy, 2x time)",
                weapon.name, actor.name, actor.position.x, actor.position.y
              ),
              command: Command::AttackRangedAimed(actor.position),
              params: JsonValue::Object(aimed_params),
            });
          }
        }
      }
    }
  }

  // 3a. Typed Subtle Knife alternate invoke.
  if let Some(weapon) = obs.equipped_weapon.as_ref()
    && weapon.archetype == ItemArchetype::SubtleKnife
  {
    let mut p = BTreeMap::new();
    p.insert("action".to_string(), JsonValue::from("invoke"));
    p.insert("item_id".to_string(), JsonValue::from(weapon.id.as_u64()));
    actions.push(LegalAction {
      action: "Invoke".to_string(),
      description: "Invoke the equipped Subtle Knife against visible targets".to_string(),
      command: Command::Invoke(weapon.id),
      params: JsonValue::Object(p),
    });
  }

  // 3b. Typed Trigun alternate reload (the caller must explicitly confirm).
  if let Some(weapon) = obs.equipped_weapon.as_ref()
    && weapon.archetype == ItemArchetype::Trigun
    && obs.player_hp.is_some_and(|hp| hp.max > 10)
  {
    let mut p = BTreeMap::new();
    p.insert("action".to_string(), JsonValue::from("alt_reload"));
    p.insert("item_id".to_string(), JsonValue::from(weapon.id.as_u64()));
    p.insert("confirmed".to_string(), JsonValue::Bool(true));
    actions.push(LegalAction {
      action: "AltReload".to_string(),
      description: "Confirm the equipped Trigun alternate reload and level nuke".to_string(),
      command: Command::AltReload {
        item_id: weapon.id,
        confirmed: true,
      },
      params: JsonValue::Object(p),
    });
  }

  // 3c. Typed Grammaton fire-mode cycle.
  if let Some(weapon) = obs.equipped_weapon.as_ref()
    && weapon.archetype == ItemArchetype::GrammatonBeretta
  {
    let mut p = BTreeMap::new();
    p.insert("action".to_string(), JsonValue::from("alt_reload"));
    p.insert("item_id".to_string(), JsonValue::from(weapon.id.as_u64()));
    p.insert("confirmed".to_string(), JsonValue::Bool(true));
    actions.push(LegalAction {
      action: "AltReload".to_string(),
      description: "Cycle the equipped Grammaton fire mode".to_string(),
      command: Command::AltReload {
        item_id: weapon.id,
        confirmed: true,
      },
      params: JsonValue::Object(p),
    });
  }

  // 3d. Typed Jackhammer burst/single fire-mode toggle.
  if let Some(weapon) = obs.equipped_weapon.as_ref()
    && weapon.archetype == ItemArchetype::Jackhammer
  {
    let mut p = BTreeMap::new();
    p.insert("action".to_string(), JsonValue::from("alt_reload"));
    p.insert("item_id".to_string(), JsonValue::from(weapon.id.as_u64()));
    p.insert("confirmed".to_string(), JsonValue::Bool(true));
    actions.push(LegalAction {
      action: "AltReload".to_string(),
      description: "Toggle the equipped Jackhammer fire mode".to_string(),
      command: Command::AltReload {
        item_id: weapon.id,
        confirmed: true,
      },
      params: JsonValue::Object(p),
    });
  }

  // 3e. Typed Assault Shotgun alternate/full reload.
  if let Some(weapon) = obs.equipped_weapon.as_ref()
    && weapon.archetype == ItemArchetype::AssaultShotgun
  {
    let mut p = BTreeMap::new();
    p.insert("action".to_string(), JsonValue::from("alt_reload"));
    p.insert("item_id".to_string(), JsonValue::from(weapon.id.as_u64()));
    p.insert("confirmed".to_string(), JsonValue::Bool(false));
    actions.push(LegalAction {
      action: "AltReload".to_string(),
      description: "Fully reload the equipped Assault Shotgun from loose shells".to_string(),
      command: Command::AltReload {
        item_id: weapon.id,
        confirmed: false,
      },
      params: JsonValue::Object(p),
    });
  }

  // 3f. Typed Combat Shotgun alternate/full reload.
  if let Some(weapon) = obs.equipped_weapon.as_ref()
    && weapon.archetype == ItemArchetype::CombatShotgun
  {
    let mut p = BTreeMap::new();
    p.insert("action".to_string(), JsonValue::from("alt_reload"));
    p.insert("item_id".to_string(), JsonValue::from(weapon.id.as_u64()));
    p.insert("confirmed".to_string(), JsonValue::Bool(false));
    actions.push(LegalAction {
      action: "AltReload".to_string(),
      description: "Fully reload the equipped Combat Shotgun from loose shells".to_string(),
      command: Command::AltReload {
        item_id: weapon.id,
        confirmed: false,
      },
      params: JsonValue::Object(p),
    });
  }

  // 3g. Typed Missile Launcher alternate/full reload.
  if let Some(weapon) = obs.equipped_weapon.as_ref()
    && weapon.archetype == ItemArchetype::MissileLauncher
    && weapon
      .clip
      .is_some_and(|(loaded, max_clip)| loaded < max_clip)
    && obs
      .inventory
      .iter()
      .any(|item| item.archetype == ItemArchetype::AmmoRockets && item.count > 0)
  {
    let mut p = BTreeMap::new();
    p.insert("action".to_string(), JsonValue::from("alt_reload"));
    p.insert("item_id".to_string(), JsonValue::from(weapon.id.as_u64()));
    p.insert("confirmed".to_string(), JsonValue::Bool(false));
    actions.push(LegalAction {
      action: "AltReload".to_string(),
      description: "Fully reload the equipped Missile Launcher from loose rockets".to_string(),
      command: Command::AltReload {
        item_id: weapon.id,
        confirmed: false,
      },
      params: JsonValue::Object(p),
    });
  }

  // 3h. Typed nuclear-weapon overload (the caller must explicitly confirm
  // the destructive action). The cloned core probe below filters pending-
  // nuke and any other state not visible through observation.
  if let Some(weapon) = obs.equipped_weapon.as_ref()
    && matches!(
      weapon.archetype,
      ItemArchetype::NuclearPlasmaRifle | ItemArchetype::NuclearBfg9000
    )
    && weapon
      .clip
      .is_some_and(|(loaded, max_clip)| loaded >= max_clip)
    && !obs
      .visible_tiles
      .iter()
      .any(|tile| tile.position == obs.player_position && tile.kind == TileKind::StairsDown)
  {
    let mut p = BTreeMap::new();
    p.insert("action".to_string(), JsonValue::from("alt_reload"));
    p.insert("item_id".to_string(), JsonValue::from(weapon.id.as_u64()));
    p.insert("confirmed".to_string(), JsonValue::Bool(true));
    actions.push(LegalAction {
      action: "AltReload".to_string(),
      description: format!("Confirm the equipped {} overload", weapon.name),
      command: Command::AltReload {
        item_id: weapon.id,
        confirmed: true,
      },
      params: JsonValue::Object(p),
    });
  }

  // 4. Reload weapon (if weapon not full and matching ammo exists in inventory)
  if let Some(ref weapon) = obs.equipped_weapon
    && let Some((loaded, max_clip)) = weapon.clip
  {
    let has_matching_ammo = obs
      .inventory
      .iter()
      .any(|item| item.category == ItemCategory::Ammo && item.count > 0);
    if loaded < max_clip && has_matching_ammo {
      let mut p = BTreeMap::new();
      p.insert("action".to_string(), JsonValue::from("reload"));
      actions.push(LegalAction {
        action: "Reload".to_string(),
        description: format!(
          "Reload {} ({loaded}/{max_clip}) from inventory ammo",
          weapon.name
        ),
        command: Command::Reload,
        params: JsonValue::Object(p),
      });
    }
  }

  // 5. Pickup ground items (if standing on ground item)
  for ground in &obs.ground_items {
    if ground.position == obs.player_position {
      let mut p = BTreeMap::new();
      p.insert("action".to_string(), JsonValue::from("pickup"));
      actions.push(LegalAction {
        action: "Pickup".to_string(),
        description: format!("Pick up {} from floor", ground.item.name),
        command: Command::Pickup,
        params: JsonValue::Object(p),
      });
    }
  }

  // 6. Use / consume inventory items (MedPacks, Phase Devices)
  for item in &obs.inventory {
    if item.category == ItemCategory::MedPack || item.category == ItemCategory::PhaseDevice {
      let mut p = BTreeMap::new();
      p.insert("action".to_string(), JsonValue::from("use"));
      p.insert("item_id".to_string(), JsonValue::from(item.id.as_u64()));

      actions.push(LegalAction {
        action: "Use".to_string(),
        description: format!("Use/consume item {} ({})", item.name, item.id.as_u64()),
        command: Command::Use(item.id),
        params: JsonValue::Object(p),
      });
    }
  }

  // 7. Equip items from inventory
  for item in &obs.inventory {
    if item.category == ItemCategory::Weapon {
      let mut p = BTreeMap::new();
      p.insert("action".to_string(), JsonValue::from("equip"));
      p.insert("item_id".to_string(), JsonValue::from(item.id.as_u64()));
      p.insert("slot".to_string(), JsonValue::from("Weapon"));

      actions.push(LegalAction {
        action: "Equip".to_string(),
        description: format!("Equip weapon {} into Weapon slot", item.name),
        command: Command::Equip(item.id),
        params: JsonValue::Object(p),
      });
    } else if item.category == ItemCategory::Armor {
      let mut p = BTreeMap::new();
      p.insert("action".to_string(), JsonValue::from("equip"));
      p.insert("item_id".to_string(), JsonValue::from(item.id.as_u64()));
      p.insert("slot".to_string(), JsonValue::from("Armor"));

      actions.push(LegalAction {
        action: "Equip".to_string(),
        description: format!("Equip armor {} into Armor slot", item.name),
        command: Command::Equip(item.id),
        params: JsonValue::Object(p),
      });
    }
  }

  // 8. Unequip currently equipped weapon and armor.
  for (slot, label, item) in [
    (
      EquipmentSlot::Weapon,
      "Weapon",
      obs.equipped_weapon.as_ref(),
    ),
    (EquipmentSlot::Armor, "Armor", obs.equipped_armor.as_ref()),
  ] {
    let Some(item) = item else {
      continue;
    };
    let mut p = BTreeMap::new();
    p.insert("action".to_string(), JsonValue::from("unequip"));
    p.insert("slot".to_string(), JsonValue::from(label));
    actions.push(LegalAction {
      action: "Unequip".to_string(),
      description: format!("Unequip {} from the {label} slot", item.name),
      command: Command::Unequip(slot),
      params: JsonValue::Object(p),
    });
  }

  // 9. Drop items from inventory
  for item in &obs.inventory {
    let mut p = BTreeMap::new();
    p.insert("action".to_string(), JsonValue::from("drop"));
    p.insert("item_id".to_string(), JsonValue::from(item.id.as_u64()));

    actions.push(LegalAction {
      action: "Drop".to_string(),
      description: format!("Drop {} onto the ground", item.name),
      command: Command::Drop(item.id),
      params: JsonValue::Object(p),
    });
  }

  // 10. Descend stairs (if standing on StairsDown)
  let on_stairs = obs
    .visible_tiles
    .iter()
    .any(|t| t.position == obs.player_position && t.kind == TileKind::StairsDown);
  if on_stairs {
    let mut p = BTreeMap::new();
    p.insert("action".to_string(), JsonValue::from("descend"));

    actions.push(LegalAction {
      action: "Descend".to_string(),
      description: "Descend stairs to transition to the next dungeon level".to_string(),
      command: Command::Descend,
      params: JsonValue::Object(p),
    });
  }

  actions
}

/// Parses a direction string (e.g. "North", "n", "East", "e") into a typed `Direction`.
pub fn parse_direction(s: &str) -> Option<Direction> {
  match s.to_lowercase().as_str() {
    "north" | "n" | "up" | "k" => Some(Direction::North),
    "south" | "s" | "down" | "j" => Some(Direction::South),
    "east" | "e" | "right" | "l" => Some(Direction::East),
    "west" | "w" | "left" | "h" => Some(Direction::West),
    "northeast" | "ne" | "u" => Some(Direction::NorthEast),
    "northwest" | "nw" | "y" => Some(Direction::NorthWest),
    "southeast" | "se" | "n_key" | "b" => Some(Direction::SouthEast),
    "southwest" | "sw" | "m" => Some(Direction::SouthWest),
    "none" | "wait" | "." => Some(Direction::None),
    _ => None,
  }
}

/// Parses a JSON value into a simulation `Command`.
pub fn json_to_command(val: &JsonValue) -> Result<Command, String> {
  let obj = val
    .as_object()
    .ok_or_else(|| "Action arguments must be a JSON object".to_string())?;

  let action = obj
    .get("action")
    .or_else(|| obj.get("command"))
    .and_then(|v| v.as_str())
    .ok_or_else(|| "Missing 'action' or 'command' field in arguments".to_string())?
    .to_lowercase();

  match action.as_str() {
    "move" => {
      let dir_str = obj
        .get("direction")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'direction' parameter for move action".to_string())?;
      let dir =
        parse_direction(dir_str).ok_or_else(|| format!("Invalid direction value: '{dir_str}'"))?;
      if dir == Direction::None {
        Ok(Command::Wait)
      } else {
        Ok(Command::Move(dir))
      }
    }
    "attack_melee" | "melee" => {
      let dir_str = obj
        .get("direction")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'direction' parameter for melee attack".to_string())?;
      let dir =
        parse_direction(dir_str).ok_or_else(|| format!("Invalid direction value: '{dir_str}'"))?;
      Ok(Command::AttackMelee(dir))
    }
    "attack_ranged" | "fire" | "shoot" => {
      let target_x = required_exact_i32(obj, &["target_x", "x"], "target_x / x")?;
      let target_y = required_exact_i32(obj, &["target_y", "y"], "target_y / y")?;
      Ok(Command::AttackRanged(Position::new(target_x, target_y)))
    }
    "aimed_fire" | "attack_ranged_aimed" | "aim" => {
      let target_x = required_exact_i32(obj, &["target_x", "x"], "target_x / x")?;
      let target_y = required_exact_i32(obj, &["target_y", "y"], "target_y / y")?;
      Ok(Command::AttackRangedAimed(Position::new(
        target_x, target_y,
      )))
    }
    "chainfire" | "attack_ranged_chainfire" => {
      let target_x = required_exact_i32(obj, &["target_x", "x"], "target_x / x")?;
      let target_y = required_exact_i32(obj, &["target_y", "y"], "target_y / y")?;
      Ok(Command::AttackRangedChainfire(Position::new(
        target_x, target_y,
      )))
    }
    "wait" => Ok(Command::Wait),
    "pickup" => Ok(Command::Pickup),
    "drop" => {
      let item_id = required_exact_item_id(obj)?;
      Ok(Command::Drop(ItemId::new(item_id)))
    }
    "equip" => {
      let item_id = required_exact_item_id(obj)?;
      Ok(Command::Equip(ItemId::new(item_id)))
    }
    "unequip" => {
      let slot_str = obj
        .get("slot")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'slot' parameter for unequip action".to_string())?;
      let slot = match slot_str.to_lowercase().as_str() {
        "weapon" => EquipmentSlot::Weapon,
        "armor" => EquipmentSlot::Armor,
        other => return Err(format!("Unknown equipment slot '{other}'")),
      };
      Ok(Command::Unequip(slot))
    }
    "use" => {
      let item_id = required_exact_item_id(obj)?;
      Ok(Command::Use(ItemId::new(item_id)))
    }
    "invoke" => {
      let item_id = required_exact_item_id(obj)?;
      Ok(Command::Invoke(ItemId::new(item_id)))
    }
    "alt_reload" => {
      let item_id = required_exact_item_id(obj)?;
      let confirmed = obj
        .get("confirmed")
        .and_then(JsonValue::as_bool)
        .ok_or_else(|| "Missing or invalid 'confirmed' parameter".to_string())?;
      Ok(Command::AltReload {
        item_id: ItemId::new(item_id),
        confirmed,
      })
    }
    "reload" => Ok(Command::Reload),
    "descend" => Ok(Command::Descend),
    other => Err(format!("Unknown action type: '{other}'")),
  }
}

fn required_exact_i32(
  obj: &BTreeMap<String, JsonValue>,
  names: &[&str],
  label: &str,
) -> Result<i32, String> {
  let value = names.iter().find_map(|name| obj.get(*name));
  value
    .and_then(exact_i32)
    .ok_or_else(|| format!("Missing or invalid '{label}' parameter"))
}

fn required_exact_item_id(obj: &BTreeMap<String, JsonValue>) -> Result<u64, String> {
  obj
    .get("item_id")
    .and_then(exact_item_id)
    .ok_or_else(|| "Missing or invalid 'item_id' parameter".to_string())
}

fn exact_i32(value: &JsonValue) -> Option<i32> {
  let JsonValue::Number(number) = value else {
    return None;
  };
  if !number.is_finite()
    || number.fract() != 0.0
    || *number < f64::from(i32::MIN)
    || *number > f64::from(i32::MAX)
  {
    return None;
  }
  Some(*number as i32)
}

fn exact_item_id(value: &JsonValue) -> Option<u64> {
  let JsonValue::Number(number) = value else {
    return None;
  };
  if !number.is_finite()
    || *number < 0.0
    || number.fract() != 0.0
    || *number > 9_007_199_254_740_992.0
  {
    return None;
  }
  Some(*number as u64)
}

/// Converts a `PlayerObservation` to a JSON representation.
#[must_use]
pub fn player_observation_to_json(obs: &PlayerObservation) -> JsonValue {
  let mut map = BTreeMap::new();
  map.insert("turn".to_string(), JsonValue::from(obs.turn.count));
  map.insert(
    "player_position".to_string(),
    position_to_json(obs.player_position),
  );

  let mut tiles = Vec::with_capacity(obs.visible_tiles.len());
  for t in &obs.visible_tiles {
    let mut tm = BTreeMap::new();
    tm.insert("position".to_string(), position_to_json(t.position));
    tm.insert("kind".to_string(), JsonValue::from(format!("{:?}", t.kind)));
    tm.insert("walkable".to_string(), JsonValue::Bool(t.is_walkable));
    tm.insert("visible".to_string(), JsonValue::Bool(t.is_visible));
    tiles.push(JsonValue::Object(tm));
  }
  map.insert("visible_tiles".to_string(), JsonValue::Array(tiles));

  let mut actors = Vec::with_capacity(obs.visible_actors.len());
  for a in &obs.visible_actors {
    let mut am = BTreeMap::new();
    am.insert("id".to_string(), JsonValue::from(a.id.as_u64()));
    am.insert("name".to_string(), JsonValue::from(a.name.as_str()));
    am.insert("is_player".to_string(), JsonValue::Bool(a.is_player));
    am.insert("position".to_string(), position_to_json(a.position));
    am.insert("alive".to_string(), JsonValue::Bool(a.is_alive));
    am.insert("speed".to_string(), JsonValue::from(a.speed.as_u32()));
    if let Some(hp) = a.hp {
      let mut hpm = BTreeMap::new();
      hpm.insert("current".to_string(), JsonValue::from(hp.current));
      hpm.insert("max".to_string(), JsonValue::from(hp.max));
      am.insert("hp".to_string(), JsonValue::Object(hpm));
    }
    actors.push(JsonValue::Object(am));
  }
  map.insert("visible_actors".to_string(), JsonValue::Array(actors));

  let mut inv = Vec::with_capacity(obs.inventory.len());
  for item in &obs.inventory {
    inv.push(item_view_to_json(item));
  }
  map.insert("inventory".to_string(), JsonValue::Array(inv));

  if let Some(ref w) = obs.equipped_weapon {
    map.insert("equipped_weapon".to_string(), item_view_to_json(w));
  } else {
    map.insert("equipped_weapon".to_string(), JsonValue::Null);
  }

  if let Some(ref a) = obs.equipped_armor {
    map.insert("equipped_armor".to_string(), item_view_to_json(a));
  } else {
    map.insert("equipped_armor".to_string(), JsonValue::Null);
  }

  let mut ground = Vec::with_capacity(obs.ground_items.len());
  for g in &obs.ground_items {
    let mut gm = BTreeMap::new();
    gm.insert("position".to_string(), position_to_json(g.position));
    gm.insert("item".to_string(), item_view_to_json(&g.item));
    ground.push(JsonValue::Object(gm));
  }
  map.insert("ground_items".to_string(), JsonValue::Array(ground));

  JsonValue::Object(map)
}

/// Converts an `OmniscientObservation` to JSON.
#[must_use]
pub fn omniscient_observation_to_json(obs: &OmniscientObservation) -> JsonValue {
  let mut map = BTreeMap::new();
  map.insert("turn".to_string(), JsonValue::from(obs.turn.count));
  map.insert("width".to_string(), JsonValue::from(obs.width));
  map.insert("height".to_string(), JsonValue::from(obs.height));

  let mut tiles = Vec::with_capacity(obs.tiles.len());
  for t in &obs.tiles {
    let mut tm = BTreeMap::new();
    tm.insert("position".to_string(), position_to_json(t.position));
    tm.insert("kind".to_string(), JsonValue::from(format!("{:?}", t.kind)));
    tm.insert("walkable".to_string(), JsonValue::Bool(t.is_walkable));
    tiles.push(JsonValue::Object(tm));
  }
  map.insert("tiles".to_string(), JsonValue::Array(tiles));

  let mut actors = Vec::with_capacity(obs.actors.len());
  for a in &obs.actors {
    let mut am = BTreeMap::new();
    am.insert("id".to_string(), JsonValue::from(a.id.as_u64()));
    am.insert("name".to_string(), JsonValue::from(a.name.as_str()));
    am.insert("is_player".to_string(), JsonValue::Bool(a.is_player));
    am.insert("position".to_string(), position_to_json(a.position));
    am.insert("alive".to_string(), JsonValue::Bool(a.is_alive));
    if let Some(hp) = a.hp {
      let mut hpm = BTreeMap::new();
      hpm.insert("current".to_string(), JsonValue::from(hp.current));
      hpm.insert("max".to_string(), JsonValue::from(hp.max));
      am.insert("hp".to_string(), JsonValue::Object(hpm));
    }
    actors.push(JsonValue::Object(am));
  }
  map.insert("actors".to_string(), JsonValue::Array(actors));

  let mut ground = Vec::with_capacity(obs.ground_items.len());
  for g in &obs.ground_items {
    let mut gm = BTreeMap::new();
    gm.insert("position".to_string(), position_to_json(g.position));
    gm.insert("item".to_string(), item_view_to_json(&g.item));
    ground.push(JsonValue::Object(gm));
  }
  map.insert("ground_items".to_string(), JsonValue::Array(ground));

  JsonValue::Object(map)
}

fn item_view_to_json(item: &ItemView) -> JsonValue {
  let mut map = BTreeMap::new();
  map.insert("id".to_string(), JsonValue::from(item.id.as_u64()));
  map.insert("name".to_string(), JsonValue::from(item.name.as_str()));
  map.insert(
    "category".to_string(),
    JsonValue::from(item.category.to_string()),
  );
  map.insert("count".to_string(), JsonValue::from(item.count));
  map.insert(
    "description".to_string(),
    JsonValue::from(item.description.as_str()),
  );
  if let Some((curr, max)) = item.clip {
    let mut cm = BTreeMap::new();
    cm.insert("current".to_string(), JsonValue::from(curr));
    cm.insert("max".to_string(), JsonValue::from(max));
    map.insert("clip".to_string(), JsonValue::Object(cm));
  }
  map.insert(
    "chainfire_level".to_string(),
    JsonValue::from(u32::from(item.chainfire_level)),
  );
  if let Some((min_d, max_d)) = item.damage {
    let mut dm = BTreeMap::new();
    dm.insert("min".to_string(), JsonValue::from(min_d));
    dm.insert("max".to_string(), JsonValue::from(max_d));
    map.insert("damage".to_string(), JsonValue::Object(dm));
  }
  if let Some(armor) = item.armor_value {
    map.insert("armor_value".to_string(), JsonValue::from(armor));
  }
  if let Some(heal) = item.heal_amount {
    map.insert("heal_amount".to_string(), JsonValue::from(heal));
  }
  JsonValue::Object(map)
}

fn position_to_json(pos: Position) -> JsonValue {
  let mut map = BTreeMap::new();
  map.insert("x".to_string(), JsonValue::from(pos.x));
  map.insert("y".to_string(), JsonValue::from(pos.y));
  JsonValue::Object(map)
}

/// Converts a `GameEvent` to JSON.
#[must_use]
pub fn game_event_to_json(event: &GameEvent) -> JsonValue {
  let mut map = BTreeMap::new();
  match event {
    GameEvent::TurnStarted { turn } => {
      map.insert("type".to_string(), JsonValue::from("TurnStarted"));
      map.insert("turn".to_string(), JsonValue::from(turn.count));
    }
    GameEvent::EntityMoved {
      entity_id,
      from,
      to,
    } => {
      map.insert("type".to_string(), JsonValue::from("EntityMoved"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("from".to_string(), position_to_json(*from));
      map.insert("to".to_string(), position_to_json(*to));
    }
    GameEvent::EntityWaited {
      entity_id,
      position,
    } => {
      map.insert("type".to_string(), JsonValue::from("EntityWaited"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("position".to_string(), position_to_json(*position));
    }
    GameEvent::AttackResolved {
      attacker_id,
      target_id,
      outcome,
      is_ranged,
    } => {
      map.insert("type".to_string(), JsonValue::from("AttackResolved"));
      map.insert(
        "attacker_id".to_string(),
        JsonValue::from(attacker_id.as_u64()),
      );
      map.insert("target_id".to_string(), JsonValue::from(target_id.as_u64()));
      map.insert("is_ranged".to_string(), JsonValue::Bool(*is_ranged));
      match outcome {
        drl_protocol::AttackOutcome::Hit { damage, is_lethal } => {
          map.insert("hit".to_string(), JsonValue::Bool(true));
          map.insert("damage".to_string(), JsonValue::from(*damage));
          map.insert("is_lethal".to_string(), JsonValue::Bool(*is_lethal));
        }
        drl_protocol::AttackOutcome::Miss => {
          map.insert("hit".to_string(), JsonValue::Bool(false));
          map.insert("damage".to_string(), JsonValue::from(0));
        }
        drl_protocol::AttackOutcome::Blocked => {
          map.insert("hit".to_string(), JsonValue::Bool(false));
          map.insert("blocked".to_string(), JsonValue::Bool(true));
          map.insert("damage".to_string(), JsonValue::from(0));
        }
      }
    }
    GameEvent::DamageApplied {
      target_id,
      amount,
      remaining_hp,
      damage_type,
      ..
    } => {
      map.insert("type".to_string(), JsonValue::from("DamageApplied"));
      map.insert("target_id".to_string(), JsonValue::from(target_id.as_u64()));
      map.insert("amount".to_string(), JsonValue::from(*amount));
      map.insert("remaining_hp".to_string(), JsonValue::from(*remaining_hp));
      if let Some(damage_type) = damage_type {
        map.insert(
          "damage_type".to_string(),
          JsonValue::from(format!("{damage_type:?}")),
        );
      }
    }
    GameEvent::ActorDied { entity_id, cause } => {
      map.insert("type".to_string(), JsonValue::from("ActorDied"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("cause".to_string(), JsonValue::from(format!("{cause:?}")));
    }
    GameEvent::ItemPickedUp {
      entity_id,
      item_name,
      ..
    } => {
      map.insert("type".to_string(), JsonValue::from("ItemPickedUp"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_name".to_string(), JsonValue::from(item_name.as_str()));
    }
    GameEvent::ItemDropped {
      entity_id,
      item_name,
      position,
      ..
    } => {
      map.insert("type".to_string(), JsonValue::from("ItemDropped"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_name".to_string(), JsonValue::from(item_name.as_str()));
      map.insert("position".to_string(), position_to_json(*position));
    }
    GameEvent::ItemEquipped {
      entity_id,
      slot,
      item_id,
    } => {
      map.insert("type".to_string(), JsonValue::from("ItemEquipped"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("slot".to_string(), JsonValue::from(slot.to_string()));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
    }
    GameEvent::ItemUnequipped {
      entity_id,
      slot,
      item_id,
    } => {
      map.insert("type".to_string(), JsonValue::from("ItemUnequipped"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("slot".to_string(), JsonValue::from(slot.to_string()));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
    }
    GameEvent::ItemUsed {
      entity_id,
      item_name,
      ..
    } => {
      map.insert("type".to_string(), JsonValue::from("ItemUsed"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_name".to_string(), JsonValue::from(item_name.as_str()));
    }
    GameEvent::GroundItemDestroyed { item_id, position } => {
      map.insert("type".to_string(), JsonValue::from("GroundItemDestroyed"));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
      map.insert("position".to_string(), position_to_json(*position));
    }
    GameEvent::WeaponReloaded {
      entity_id,
      ammo_loaded,
      current_clip,
      max_clip,
    } => {
      map.insert("type".to_string(), JsonValue::from("WeaponReloaded"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("ammo_loaded".to_string(), JsonValue::from(*ammo_loaded));
      map.insert("current_clip".to_string(), JsonValue::from(*current_clip));
      map.insert("max_clip".to_string(), JsonValue::from(*max_clip));
    }
    GameEvent::WeaponRecharged {
      entity_id,
      item_id,
      ammo_recharged,
      current_clip,
      max_clip,
      timer,
    } => {
      map.insert("type".to_string(), JsonValue::from("WeaponRecharged"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
      map.insert(
        "ammo_recharged".to_string(),
        JsonValue::from(*ammo_recharged),
      );
      map.insert("current_clip".to_string(), JsonValue::from(*current_clip));
      map.insert("max_clip".to_string(), JsonValue::from(*max_clip));
      map.insert("timer".to_string(), JsonValue::from(*timer));
    }
    GameEvent::AcidSpitterReloaded {
      entity_id,
      item_id,
      position,
      ammo_loaded,
      current_clip,
      max_clip,
      score_count_remaining,
    } => {
      map.insert("type".to_string(), JsonValue::from("AcidSpitterReloaded"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
      map.insert("position".to_string(), position_to_json(*position));
      map.insert("ammo_loaded".to_string(), JsonValue::from(*ammo_loaded));
      map.insert("current_clip".to_string(), JsonValue::from(*current_clip));
      map.insert("max_clip".to_string(), JsonValue::from(*max_clip));
      map.insert(
        "score_count_remaining".to_string(),
        JsonValue::from(*score_count_remaining),
      );
    }
    GameEvent::MedicalPowerarmorRepaired {
      entity_id,
      item_id,
      healed,
      remaining_hp,
      durability_remaining,
      timer,
    } => {
      map.insert(
        "type".to_string(),
        JsonValue::from("MedicalPowerarmorRepaired"),
      );
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
      map.insert("healed".to_string(), JsonValue::from(*healed));
      map.insert("remaining_hp".to_string(), JsonValue::from(*remaining_hp));
      map.insert(
        "durability_remaining".to_string(),
        JsonValue::from(*durability_remaining),
      );
      map.insert("timer".to_string(), JsonValue::from(*timer));
    }
    GameEvent::LavaArmorRecharged {
      entity_id,
      item_id,
      durability_restored,
      durability_remaining,
      timer,
    } => {
      map.insert("type".to_string(), JsonValue::from("LavaArmorRecharged"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
      map.insert(
        "durability_restored".to_string(),
        JsonValue::from(*durability_restored),
      );
      map.insert(
        "durability_remaining".to_string(),
        JsonValue::from(*durability_remaining),
      );
      map.insert("timer".to_string(), JsonValue::from(*timer));
    }
    GameEvent::MalekArmorRecharged {
      entity_id,
      item_id,
      durability_restored,
      durability_remaining,
      timer,
    } => {
      map.insert("type".to_string(), JsonValue::from("MalekArmorRecharged"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
      map.insert(
        "durability_restored".to_string(),
        JsonValue::from(*durability_restored),
      );
      map.insert(
        "durability_remaining".to_string(),
        JsonValue::from(*durability_remaining),
      );
      map.insert("timer".to_string(), JsonValue::from(*timer));
    }
    GameEvent::NuclearWeaponOverloaded {
      entity_id,
      item_id,
      countdown,
      score_count_remaining,
    } => {
      map.insert(
        "type".to_string(),
        JsonValue::from("NuclearWeaponOverloaded"),
      );
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
      map.insert("countdown".to_string(), JsonValue::from(*countdown));
      map.insert(
        "score_count_remaining".to_string(),
        JsonValue::from(*score_count_remaining),
      );
    }
    GameEvent::SubtleKnifeInvoked {
      entity_id,
      item_id,
      targets,
      remaining_hp,
      score_count_remaining,
    } => {
      map.insert("type".to_string(), JsonValue::from("SubtleKnifeInvoked"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
      map.insert(
        "targets".to_string(),
        JsonValue::Array(
          targets
            .iter()
            .map(|target| JsonValue::from(target.as_u64()))
            .collect(),
        ),
      );
      map.insert("remaining_hp".to_string(), JsonValue::from(*remaining_hp));
      map.insert(
        "score_count_remaining".to_string(),
        JsonValue::from(*score_count_remaining),
      );
    }
    GameEvent::TrigunAltReloaded {
      entity_id,
      item_id,
      remaining_hp,
      score_count_remaining,
    } => {
      map.insert("type".to_string(), JsonValue::from("TrigunAltReloaded"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
      let mut hp = BTreeMap::new();
      hp.insert("current".to_string(), JsonValue::from(remaining_hp.current));
      hp.insert("max".to_string(), JsonValue::from(remaining_hp.max));
      map.insert("remaining_hp".to_string(), JsonValue::Object(hp));
      map.insert(
        "score_count_remaining".to_string(),
        JsonValue::from(*score_count_remaining),
      );
    }
    GameEvent::GrammatonFireModeChanged {
      entity_id,
      item_id,
      mode,
      score_count_remaining,
    } => {
      map.insert(
        "type".to_string(),
        JsonValue::from("GrammatonFireModeChanged"),
      );
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
      let mode_name = match mode {
        drl_protocol::WeaponFireMode::Single => "single",
        drl_protocol::WeaponFireMode::Burst => "burst",
        drl_protocol::WeaponFireMode::Auto => "auto",
      };
      map.insert("mode".to_string(), JsonValue::from(mode_name));
      map.insert(
        "score_count_remaining".to_string(),
        JsonValue::from(*score_count_remaining),
      );
    }
    GameEvent::JackhammerFireModeChanged {
      entity_id,
      item_id,
      mode,
      score_count_remaining,
    } => {
      map.insert(
        "type".to_string(),
        JsonValue::from("JackhammerFireModeChanged"),
      );
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
      let mode_name = match mode {
        drl_protocol::WeaponFireMode::Single => "single",
        drl_protocol::WeaponFireMode::Burst => "burst",
        drl_protocol::WeaponFireMode::Auto => "auto",
      };
      map.insert("mode".to_string(), JsonValue::from(mode_name));
      map.insert(
        "score_count_remaining".to_string(),
        JsonValue::from(*score_count_remaining),
      );
    }
    GameEvent::NullPointerHit {
      entity_id,
      item_id,
      target_id,
      target_is_boss,
      score_count_remaining,
    } => {
      map.insert("type".to_string(), JsonValue::from("NullPointerHit"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("item_id".to_string(), JsonValue::from(item_id.as_u64()));
      map.insert("target_id".to_string(), JsonValue::from(target_id.as_u64()));
      map.insert(
        "target_is_boss".to_string(),
        JsonValue::from(*target_is_boss),
      );
      map.insert(
        "score_count_remaining".to_string(),
        JsonValue::from(*score_count_remaining),
      );
    }
    GameEvent::NullPointerExplosionScheduled {
      entity_id,
      target_id,
      delay,
      radius,
      damage,
    } => {
      map.insert(
        "type".to_string(),
        JsonValue::from("NullPointerExplosionScheduled"),
      );
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("target_id".to_string(), JsonValue::from(target_id.as_u64()));
      map.insert("delay".to_string(), JsonValue::from(*delay));
      map.insert("radius".to_string(), JsonValue::from(*radius));
      map.insert("damage".to_string(), JsonValue::from(*damage));
    }
    GameEvent::AntiFreakJackalExplosionScheduled {
      entity_id,
      target_id,
      delay,
      radius,
      knockback,
    } => {
      map.insert(
        "type".to_string(),
        JsonValue::from("AntiFreakJackalExplosionScheduled"),
      );
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("target_id".to_string(), JsonValue::from(target_id.as_u64()));
      map.insert("delay".to_string(), JsonValue::from(*delay));
      map.insert("radius".to_string(), JsonValue::from(*radius));
      map.insert("knockback".to_string(), JsonValue::from(*knockback));
    }
    GameEvent::Bfg10kExplosionScheduled {
      entity_id,
      target_id,
      delay,
      radius,
      knockback,
    } => {
      map.insert(
        "type".to_string(),
        JsonValue::from("Bfg10kExplosionScheduled"),
      );
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("target_id".to_string(), JsonValue::from(target_id.as_u64()));
      map.insert("delay".to_string(), JsonValue::from(*delay));
      map.insert("radius".to_string(), JsonValue::from(*radius));
      map.insert("knockback".to_string(), JsonValue::from(*knockback));
    }
    GameEvent::Bfg9000ExplosionScheduled {
      entity_id,
      target_id,
      delay,
      radius,
      knockback,
    } => {
      map.insert(
        "type".to_string(),
        JsonValue::from("Bfg9000ExplosionScheduled"),
      );
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("target_id".to_string(), JsonValue::from(target_id.as_u64()));
      map.insert("delay".to_string(), JsonValue::from(*delay));
      map.insert("radius".to_string(), JsonValue::from(*radius));
      map.insert("knockback".to_string(), JsonValue::from(*knockback));
    }
    GameEvent::NuclearBfg9000ExplosionScheduled {
      entity_id,
      target_id,
      delay,
      radius,
      knockback,
    } => {
      map.insert(
        "type".to_string(),
        JsonValue::from("NuclearBfg9000ExplosionScheduled"),
      );
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("target_id".to_string(), JsonValue::from(target_id.as_u64()));
      map.insert("delay".to_string(), JsonValue::from(*delay));
      map.insert("radius".to_string(), JsonValue::from(*radius));
      map.insert("knockback".to_string(), JsonValue::from(*knockback));
    }
    GameEvent::NukeActivated {
      level_id,
      countdown,
    } => {
      map.insert("type".to_string(), JsonValue::from("NukeActivated"));
      map.insert("level_id".to_string(), JsonValue::from(level_id.0));
      map.insert("countdown".to_string(), JsonValue::from(*countdown));
    }
    GameEvent::LevelNuked { level_id } => {
      map.insert("type".to_string(), JsonValue::from("LevelNuked"));
      map.insert("level_id".to_string(), JsonValue::from(level_id.0));
    }
    GameEvent::LevelTransitioned {
      from_level,
      to_level,
    } => {
      map.insert("type".to_string(), JsonValue::from("LevelTransitioned"));
      map.insert("from_level".to_string(), JsonValue::from(from_level.0));
      map.insert("to_level".to_string(), JsonValue::from(to_level.0));
    }
    GameEvent::PlayerTeleported { from, to } => {
      map.insert("type".to_string(), JsonValue::from("PlayerTeleported"));
      map.insert("from".to_string(), position_to_json(*from));
      map.insert("to".to_string(), position_to_json(*to));
    }
    GameEvent::ActorKnockedBack {
      entity_id,
      from,
      to,
    } => {
      map.insert("type".to_string(), JsonValue::from("ActorKnockedBack"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("from".to_string(), position_to_json(*from));
      map.insert("to".to_string(), position_to_json(*to));
    }
    GameEvent::ActionCostPaid { entity_id, cost } => {
      map.insert("type".to_string(), JsonValue::from("ActionCostPaid"));
      map.insert("entity_id".to_string(), JsonValue::from(entity_id.as_u64()));
      map.insert("cost".to_string(), JsonValue::from(cost.as_u32()));
    }
    GameEvent::TurnEnded { turn } => {
      map.insert("type".to_string(), JsonValue::from("TurnEnded"));
      map.insert("turn".to_string(), JsonValue::from(turn.count));
    }
  }
  JsonValue::Object(map)
}

/// Converts `EpisodeMetrics` to JSON.
#[must_use]
pub fn episode_metrics_to_json(metrics: &EpisodeMetrics) -> JsonValue {
  let mut map = BTreeMap::new();
  map.insert(
    "turns_survived".to_string(),
    JsonValue::from(metrics.turns_survived),
  );
  map.insert(
    "damage_dealt".to_string(),
    JsonValue::from(metrics.damage_dealt),
  );
  map.insert(
    "damage_taken".to_string(),
    JsonValue::from(metrics.damage_taken),
  );
  map.insert(
    "enemies_killed".to_string(),
    JsonValue::from(metrics.enemies_killed),
  );
  map.insert(
    "shots_fired".to_string(),
    JsonValue::from(metrics.shots_fired),
  );
  map.insert("shots_hit".to_string(), JsonValue::from(metrics.shots_hit));
  map.insert(
    "items_picked_up".to_string(),
    JsonValue::from(metrics.items_picked_up),
  );
  map.insert(
    "items_used".to_string(),
    JsonValue::from(metrics.items_used),
  );
  map.insert(
    "level_reached".to_string(),
    JsonValue::from(metrics.level_reached.0),
  );

  let outcome_str = match &metrics.outcome {
    RunOutcome::InProgress => "InProgress".to_string(),
    RunOutcome::Victory => "Victory".to_string(),
    RunOutcome::Death { cause } => format!("Death({cause:?})"),
    RunOutcome::TurnLimitReached => "TurnLimitReached".to_string(),
    RunOutcome::Stalled => "Stalled".to_string(),
  };
  map.insert("outcome".to_string(), JsonValue::from(outcome_str));

  JsonValue::Object(map)
}

/// Session configuration snapshot.
#[derive(Debug, Clone)]
pub struct SessionConfig {
  pub seed: u64,
  pub max_turns: Option<u64>,
  pub scenario_ascii: Option<String>,
  pub width: Option<u32>,
  pub height: Option<u32>,
}

/// Active MCP game session manager.
#[derive(Debug)]
pub struct McpSession {
  game: Option<Game>,
  config: Option<SessionConfig>,
  dev_mode: bool,
  max_turns: Option<u64>,
  turn_count: u64,
  replay_log: Option<ReplayLog>,
  loaded_replay_source: Option<ReplayLog>,
  metrics: EpisodeMetrics,
  recent_events: Vec<GameEvent>,
}

impl Default for McpSession {
  fn default() -> Self {
    Self::new()
  }
}

impl McpSession {
  /// Creates a new inactive MCP session.
  #[must_use]
  pub fn new() -> Self {
    Self {
      game: None,
      config: None,
      dev_mode: false,
      max_turns: None,
      turn_count: 0,
      replay_log: None,
      loaded_replay_source: None,
      metrics: EpisodeMetrics::new(),
      recent_events: Vec::new(),
    }
  }

  /// Sets developer mode. When disabled (default), omniscient world access is forbidden.
  pub fn set_dev_mode(&mut self, enabled: bool) {
    self.dev_mode = enabled;
  }

  /// Returns true if developer mode is enabled.
  #[must_use]
  pub fn is_dev_mode(&self) -> bool {
    self.dev_mode
  }

  /// Returns true if a game session is currently initialized and active.
  #[must_use]
  pub fn is_active(&self) -> bool {
    self.game.is_some()
  }

  /// Starts a new procedural dungeon game session.
  pub fn start_game(
    &mut self,
    seed: u64,
    max_turns: Option<u64>,
    width: Option<u32>,
    height: Option<u32>,
  ) -> Result<PlayerObservation, String> {
    let w = width.unwrap_or(40);
    let h = height.unwrap_or(20);
    validate_replay_dimensions(w, h)?;

    let config = LevelGeneratorConfig {
      width: w,
      height: h,
      max_rooms: 5,
      min_room_size: 4,
      max_room_size: 8,
      max_monsters_per_room: 2,
      max_items_per_room: 2,
    };
    let replay_config = ProceduralGenerationConfig {
      max_rooms: config.max_rooms,
      min_room_size: config.min_room_size,
      max_room_size: config.max_room_size,
      max_monsters_per_room: config.max_monsters_per_room,
      max_items_per_room: config.max_items_per_room,
    };

    let game = Game::new_procedural(seed, config)
      .map_err(|e| format!("Failed to generate procedural game: {e}"))?;

    let p_pos = game.observe_player().player_position;
    let replay = ReplayLog::new(seed, w, h, p_pos)
      .with_procedural_config(replay_config)
      .with_max_turns(max_turns);

    self.config = Some(SessionConfig {
      seed,
      max_turns,
      scenario_ascii: None,
      width: Some(w),
      height: Some(h),
    });
    self.max_turns = max_turns;
    self.turn_count = 0;
    self.metrics = EpisodeMetrics::new();
    self.recent_events.clear();
    self.replay_log = Some(replay);
    self.loaded_replay_source = None;

    let obs = game.observe_player();
    self.game = Some(game);
    Ok(obs)
  }

  /// Loads an explicit scenario fixture into this session.
  pub fn load_scenario(
    &mut self,
    ascii_map: &str,
    max_turns: Option<u64>,
  ) -> Result<PlayerObservation, String> {
    validate_ascii_dimensions(ascii_map)?;
    let scenario = Scenario::from_ascii("McpScenario", "Scenario loaded via MCP", ascii_map)
      .map_err(|e| format!("Failed to parse scenario ASCII: {e}"))?;
    validate_replay_dimensions(scenario.width, scenario.height)?;

    let game = scenario
      .instantiate()
      .map_err(|e| format!("Failed to instantiate scenario: {e}"))?;

    let w = game.world().map().width();
    let h = game.world().map().height();
    let replay = replay_log_for_scenario(&scenario).with_max_turns(max_turns);

    self.config = Some(SessionConfig {
      seed: 0,
      max_turns,
      scenario_ascii: Some(ascii_map.to_string()),
      width: Some(w),
      height: Some(h),
    });
    self.max_turns = max_turns;
    self.turn_count = 0;
    self.metrics = EpisodeMetrics::new();
    self.recent_events.clear();
    self.replay_log = Some(replay);
    self.loaded_replay_source = None;

    let obs = game.observe_player();
    self.game = Some(game);
    Ok(obs)
  }

  /// Resets the current game session back to its starting configuration.
  pub fn reset(&mut self) -> Result<PlayerObservation, String> {
    if let Some(replay) = self.loaded_replay_source.clone() {
      return self.restore_replay(replay);
    }

    let cfg = self
      .config
      .clone()
      .ok_or_else(|| "No active session configuration to reset".to_string())?;

    if let Some(ref ascii) = cfg.scenario_ascii {
      self.load_scenario(ascii, cfg.max_turns)
    } else {
      self.start_game(cfg.seed, cfg.max_turns, cfg.width, cfg.height)
    }
  }

  /// Restores a canonical V2 replay transactionally into the current session.
  ///
  /// Replay execution builds a temporary core game and metrics first. The live
  /// session is replaced only after the decoder and complete command sequence
  /// succeed, so malformed or simulation-invalid input cannot partially mutate
  /// an existing game.
  pub fn load_replay(&mut self, replay: ReplayLog) -> Result<PlayerObservation, String> {
    self.restore_replay(replay)
  }

  fn restore_replay(&mut self, replay: ReplayLog) -> Result<PlayerObservation, String> {
    let (game, events, mut metrics) =
      ReplayEngine::run_with_diagnostics(&replay).map_err(|error| error.to_string())?;
    if matches!(metrics.outcome, RunOutcome::InProgress)
      && replay
        .max_turns
        .is_some_and(|max_turns| max_turns > 0 && replay.commands.len() as u64 >= max_turns)
    {
      metrics.outcome = RunOutcome::TurnLimitReached;
    }
    let observation = game.observe_player();
    let turn_count = metrics.turns_survived;
    let config = SessionConfig {
      seed: replay.seed,
      max_turns: replay.max_turns,
      scenario_ascii: None,
      width: Some(replay.width),
      height: Some(replay.height),
    };

    self.game = Some(game);
    self.config = Some(config);
    self.max_turns = replay.max_turns;
    self.turn_count = turn_count;
    self.replay_log = Some(replay.clone());
    self.loaded_replay_source = Some(replay);
    self.metrics = metrics;
    self.recent_events = events
      .iter()
      .rposition(|event| matches!(event, GameEvent::TurnStarted { .. }))
      .map_or_else(|| events.clone(), |index| events[index..].to_vec());
    Ok(observation)
  }

  /// Retrieves the current player observation.
  pub fn get_observation(&self) -> Result<PlayerObservation, String> {
    let game = self
      .game
      .as_ref()
      .ok_or_else(|| "No active game session".to_string())?;
    Ok(game.observe_player())
  }

  /// Returns the currently executable actions visible through the fair
  /// observation boundary.
  ///
  /// Candidate generation remains observation-only. Each candidate is then
  /// probed against a cloned core game so geometry, line of sight, range, and
  /// inventory rules remain owned by `drl_core::Game::step`; the live game is
  /// never mutated by enumeration.
  pub fn legal_actions(&self) -> Result<Vec<LegalAction>, String> {
    if !matches!(self.metrics.outcome, RunOutcome::InProgress) {
      return Ok(Vec::new());
    }

    let game = self
      .game
      .as_ref()
      .ok_or_else(|| "No active game session".to_string())?;
    let observation = game.observe_player();
    let candidates = compute_legal_actions(&observation);

    Ok(
      candidates
        .into_iter()
        .filter(|candidate| {
          let mut probe = game.clone();
          probe.step(candidate.command).is_ok()
        })
        .collect(),
    )
  }

  /// Retrieves the omniscient world state (developer mode required).
  pub fn get_dev_state(&self) -> Result<OmniscientObservation, String> {
    if !self.dev_mode {
      return Err(
        "Developer mode is disabled. Omniscient observation access is forbidden.".to_string(),
      );
    }
    let game = self
      .game
      .as_ref()
      .ok_or_else(|| "No active game session".to_string())?;
    Ok(game.observe_omniscient())
  }

  /// Executes a single currently advertised semantic command in the simulation.
  ///
  /// The fair legal-action catalog is checked before dispatch; the core remains
  /// authoritative for constraints that the observation cannot prove.
  pub fn step(
    &mut self,
    command: Command,
  ) -> Result<(Vec<GameEvent>, PlayerObservation, Option<RunOutcome>), String> {
    if !matches!(self.metrics.outcome, RunOutcome::InProgress) {
      return Err(format!(
        "Game session already ended with outcome {:?}",
        self.metrics.outcome
      ));
    }

    if self.game.is_none() {
      return Err("No active game session".to_string());
    }

    let legal_actions = self.legal_actions()?;
    if !legal_actions
      .iter()
      .any(|legal_action| legal_action.command == command)
    {
      return Err("Command is not currently advertised as legal".to_string());
    }

    let game = self
      .game
      .as_mut()
      .ok_or_else(|| "No active game session".to_string())?;

    let player_id = game
      .world()
      .player_id()
      .ok_or_else(|| "No player entity in world".to_string())?;

    let events = game
      .step(command)
      .map_err(|e| format!("Simulation step error: {e}"))?;

    // Ingest events into metrics
    for ev in &events {
      self.metrics.record_event(player_id, ev);
    }

    self.turn_count += 1;
    self.recent_events.clone_from(&events);

    if let Some(ref mut replay) = self.replay_log {
      replay.record_command(command);
    }

    let obs = game.observe_player();

    // Check terminal conditions
    let player_alive = game
      .world()
      .get_actor(player_id)
      .is_some_and(|p| p.is_alive());
    let reached_next_level = events.iter().any(|event| {
      matches!(
        event,
        GameEvent::LevelTransitioned { to_level, .. } if to_level.0 > 1
      )
    });
    let outcome = if !player_alive {
      let death_outcome = self.metrics.outcome.clone();
      Some(death_outcome)
    } else if reached_next_level {
      self.metrics.outcome = RunOutcome::Victory;
      Some(RunOutcome::Victory)
    } else if let Some(max_t) = self.max_turns {
      if self.turn_count >= max_t {
        self.metrics.outcome = RunOutcome::TurnLimitReached;
        Some(RunOutcome::TurnLimitReached)
      } else {
        None
      }
    } else {
      None
    };

    Ok((events, obs, outcome))
  }

  /// Retrieves current episode telemetry metrics.
  #[must_use]
  pub fn get_metrics(&self) -> &EpisodeMetrics {
    &self.metrics
  }

  /// Exports the recorded replay log for the current session.
  pub fn export_replay(&self) -> Result<&ReplayLog, String> {
    self
      .replay_log
      .as_ref()
      .ok_or_else(|| "No replay log available for session".to_string())
  }

  /// Retrieves recent game events.
  #[must_use]
  pub fn recent_events(&self) -> &[GameEvent] {
    &self.recent_events
  }
}

fn validate_replay_dimensions(width: u32, height: u32) -> Result<(), String> {
  if !(3..=MAX_REPLAY_DIMENSION).contains(&width) || !(3..=MAX_REPLAY_DIMENSION).contains(&height) {
    return Err(format!(
      "Map dimensions must be within 3..={MAX_REPLAY_DIMENSION}"
    ));
  }
  Ok(())
}

fn validate_ascii_dimensions(ascii: &str) -> Result<(), String> {
  let mut height = 0_u32;
  let mut width = 0_u32;
  for line in ascii.lines().filter(|line| !line.is_empty()) {
    height = height.saturating_add(1);
    let line_width = line.chars().take(MAX_REPLAY_DIMENSION as usize + 1).count() as u32;
    width = width.max(line_width);
    if height > MAX_REPLAY_DIMENSION || line_width > MAX_REPLAY_DIMENSION {
      return Err(format!(
        "ASCII scenario dimensions must be within 3..={MAX_REPLAY_DIMENSION}"
      ));
    }
  }
  validate_replay_dimensions(width, height)
}

fn replay_log_for_scenario(scenario: &Scenario) -> ReplayLog {
  let mut replay = ReplayLog::new(
    scenario.seed,
    scenario.width,
    scenario.height,
    scenario.player_start,
  );
  if let Some(stairs) = scenario.stairs {
    replay.record_stairs(stairs);
  }
  for (&position, &tile) in &scenario.tiles {
    let kind = match tile {
      Tile::Wall => TileKind::Wall,
      Tile::Floor => TileKind::Floor,
      Tile::StairsDown => TileKind::StairsDown,
      Tile::DoorClosed => TileKind::DoorClosed,
      Tile::DoorOpen => TileKind::DoorOpen,
      Tile::Lava => TileKind::Lava,
      Tile::Acid => TileKind::Acid,
      Tile::Water => TileKind::Water,
      Tile::Mud => TileKind::Mud,
    };
    replay.record_tile(position, kind);
  }
  for monster in &scenario.monsters {
    replay.record_monster(monster.clone());
  }
  for item in &scenario.items {
    replay.record_item(item.clone());
  }
  if let Some(config) = &scenario.player_config {
    replay = replay.with_player_config(config.clone());
  }
  replay
}

#[cfg(test)]
mod tests {
  use super::*;

  fn assert_bfg10k_volley_events(
    events: &[GameEvent],
    attacker_id: drl_protocol::EntityId,
    target_id: drl_protocol::EntityId,
  ) {
    let mut attacks = Vec::new();
    let mut damages = Vec::new();
    let mut schedules = Vec::new();
    for (index, event) in events.iter().enumerate() {
      match event {
        GameEvent::AttackResolved {
          attacker_id: event_attacker,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { damage, .. },
          is_ranged: true,
        } if *event_attacker == attacker_id && *event_target == target_id => {
          attacks.push((index, *damage));
        }
        GameEvent::DamageApplied {
          target_id: event_target,
          amount,
          source: drl_protocol::DamageSource::Actor(_),
          damage_type: None,
          ..
        } if *event_target == target_id => damages.push((index, *amount)),
        GameEvent::Bfg10kExplosionScheduled {
          entity_id,
          target_id: event_target,
          delay,
          radius,
          knockback,
        } if *entity_id == attacker_id && *event_target == target_id => {
          schedules.push((index, *delay, *radius, *knockback));
        }
        _ => {}
      }
    }

    assert_eq!(attacks.len(), 5, "BFG 10K volley must resolve five hits");
    assert_eq!(
      damages.len(),
      5,
      "BFG 10K volley must apply five damage events"
    );
    assert_eq!(
      attacks
        .iter()
        .map(|(_, damage)| *damage)
        .collect::<Vec<_>>(),
      damages
        .iter()
        .map(|(_, amount)| *amount)
        .collect::<Vec<_>>(),
      "each projectile's resolved damage must be applied in order"
    );
    assert_eq!(
      schedules.len(),
      5,
      "BFG 10K volley must schedule five delayed explosions"
    );
    assert!(
      schedules
        .iter()
        .all(|(_, delay, radius, knockback)| (*delay, *radius, *knockback) == (25, 2, 16)),
      "each BFG 10K schedule must preserve delay 25, radius 2, and knockback 16"
    );
    for (((attack_index, _), (damage_index, _)), (schedule_index, _, _, _)) in
      attacks.iter().zip(damages.iter()).zip(schedules.iter())
    {
      assert_eq!(
        *damage_index,
        *attack_index + 1,
        "each BFG 10K attack must be followed immediately by its damage event"
      );
      assert_eq!(
        *schedule_index,
        *damage_index + 1,
        "each BFG 10K damage event must be followed immediately by its schedule"
      );
    }
  }

  fn assert_double_shotgun_dual_shot_events(
    events: &[GameEvent],
    attacker_id: drl_protocol::EntityId,
    target_id: drl_protocol::EntityId,
  ) {
    let mut attacks = Vec::new();
    let mut damages = Vec::new();
    for (index, event) in events.iter().enumerate() {
      match event {
        GameEvent::AttackResolved {
          attacker_id: event_attacker,
          target_id: event_target,
          outcome,
          is_ranged: true,
        } if *event_attacker == attacker_id && *event_target == target_id => {
          attacks.push((index, *outcome));
        }
        GameEvent::DamageApplied {
          target_id: event_target,
          amount,
          ..
        } if *event_target == target_id => damages.push((index, *amount)),
        _ => {}
      }
    }

    assert_eq!(
      attacks.len(),
      2,
      "Double Shotgun must resolve two projectiles"
    );
    assert_eq!(
      damages.len(),
      1,
      "the seeded dual-shot fixture must hit once"
    );
    let hit = attacks
      .iter()
      .find_map(|(index, outcome)| match outcome {
        drl_protocol::AttackOutcome::Hit { damage, .. } => Some((*index, *damage)),
        _ => None,
      })
      .expect("the seeded dual-shot fixture must contain one hit");
    assert_eq!(hit.1, 26, "the seeded dual-shot damage must remain stable");
    assert_eq!(damages[0].1, hit.1);
    assert_eq!(damages[0].0, hit.0 + 1);
  }

  fn assert_standard_bfg_schedule_event(
    events: &[GameEvent],
    attacker_id: drl_protocol::EntityId,
    target_id: drl_protocol::EntityId,
  ) {
    let attack_index = events
      .iter()
      .position(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id: event_attacker,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *event_attacker == attacker_id && *event_target == target_id
        )
      })
      .expect("standard BFG shot must resolve a hit");
    let damage_index = events
      .iter()
      .position(|event| {
        matches!(
          event,
          GameEvent::DamageApplied { target_id: event_target, .. }
            if *event_target == target_id
        )
      })
      .expect("standard BFG shot must apply damage");
    let (schedule_index, delay, radius, knockback) = events
      .iter()
      .enumerate()
      .find_map(|(index, event)| match event {
        GameEvent::Bfg9000ExplosionScheduled {
          entity_id,
          target_id: event_target,
          delay,
          radius,
          knockback,
        } if *entity_id == attacker_id && *event_target == target_id => {
          Some((index, *delay, *radius, *knockback))
        }
        _ => None,
      })
      .expect("standard BFG shot must schedule its delayed explosion");
    assert_eq!(damage_index, attack_index + 1);
    assert_eq!(schedule_index, damage_index + 1);
    assert_eq!((delay, radius, knockback), (33, 8, 16));
    assert_eq!(
      events
        .iter()
        .filter(|event| matches!(event, GameEvent::Bfg9000ExplosionScheduled { .. }))
        .count(),
      1
    );
  }

  fn assert_nuclear_bfg_schedule_event(
    events: &[GameEvent],
    attacker_id: drl_protocol::EntityId,
    target_id: drl_protocol::EntityId,
  ) {
    let attack_index = events
      .iter()
      .position(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id: event_attacker,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *event_attacker == attacker_id && *event_target == target_id
        )
      })
      .expect("Nuclear BFG shot must resolve a hit");
    let damage_index = events
      .iter()
      .position(|event| {
        matches!(
          event,
          GameEvent::DamageApplied { target_id: event_target, .. }
            if *event_target == target_id
        )
      })
      .expect("Nuclear BFG shot must apply damage");
    let (schedule_index, delay, radius, knockback) = events
      .iter()
      .enumerate()
      .find_map(|(index, event)| match event {
        GameEvent::NuclearBfg9000ExplosionScheduled {
          entity_id,
          target_id: event_target,
          delay,
          radius,
          knockback,
        } if *entity_id == attacker_id && *event_target == target_id => {
          Some((index, *delay, *radius, *knockback))
        }
        _ => None,
      })
      .expect("Nuclear BFG shot must schedule its delayed explosion");
    assert_eq!(damage_index, attack_index + 1);
    assert_eq!(schedule_index, damage_index + 1);
    assert_eq!((delay, radius, knockback), (33, 8, 16));
    assert_eq!(
      events
        .iter()
        .filter(|event| matches!(event, GameEvent::NuclearBfg9000ExplosionScheduled { .. }))
        .count(),
      1
    );
  }

  #[test]
  fn acid_spitter_reload_event_projects_to_mcp_json() {
    let value = game_event_to_json(&GameEvent::AcidSpitterReloaded {
      entity_id: drl_protocol::EntityId::new(1),
      item_id: ItemId::new(2),
      position: Position::new(3, 4),
      ammo_loaded: 1,
      current_clip: 2,
      max_clip: 10,
      score_count_remaining: 500,
    });
    let JsonValue::Object(map) = value else {
      panic!("event projection must be an object");
    };
    assert_eq!(
      map.get("type").and_then(JsonValue::as_str),
      Some("AcidSpitterReloaded")
    );
    assert_eq!(map.get("ammo_loaded").and_then(JsonValue::as_i64), Some(1));
    assert_eq!(
      map.get("score_count_remaining").and_then(JsonValue::as_i64),
      Some(500)
    );
    assert_eq!(
      map
        .get("position")
        .and_then(|value| match value {
          JsonValue::Object(position) => Some(position),
          _ => None,
        })
        .and_then(|position| position.get("x"))
        .and_then(JsonValue::as_i64),
      Some(3)
    );
  }

  #[test]
  fn bfg10k_explosion_schedule_event_projects_to_mcp_json() {
    let value = game_event_to_json(&GameEvent::Bfg10kExplosionScheduled {
      entity_id: drl_protocol::EntityId::new(1),
      target_id: drl_protocol::EntityId::new(2),
      delay: 25,
      radius: 2,
      knockback: 16,
    });
    let JsonValue::Object(map) = value else {
      panic!("event projection must be an object");
    };
    assert_eq!(
      map.get("type").and_then(JsonValue::as_str),
      Some("Bfg10kExplosionScheduled")
    );
    assert_eq!(map.get("entity_id").and_then(JsonValue::as_i64), Some(1));
    assert_eq!(map.get("target_id").and_then(JsonValue::as_i64), Some(2));
    assert_eq!(map.get("delay").and_then(JsonValue::as_i64), Some(25));
    assert_eq!(map.get("radius").and_then(JsonValue::as_i64), Some(2));
    assert_eq!(map.get("knockback").and_then(JsonValue::as_i64), Some(16));
  }

  #[test]
  fn anti_freak_jackal_explosion_schedule_event_projects_to_mcp_json() {
    let value = game_event_to_json(&GameEvent::AntiFreakJackalExplosionScheduled {
      entity_id: drl_protocol::EntityId::new(1),
      target_id: drl_protocol::EntityId::new(2),
      delay: 40,
      radius: 1,
      knockback: 8,
    });
    let JsonValue::Object(map) = value else {
      panic!("event projection must be an object");
    };
    assert_eq!(
      map.get("type").and_then(JsonValue::as_str),
      Some("AntiFreakJackalExplosionScheduled")
    );
    assert_eq!(map.get("entity_id").and_then(JsonValue::as_i64), Some(1));
    assert_eq!(map.get("target_id").and_then(JsonValue::as_i64), Some(2));
    assert_eq!(map.get("delay").and_then(JsonValue::as_i64), Some(40));
    assert_eq!(map.get("radius").and_then(JsonValue::as_i64), Some(1));
    assert_eq!(map.get("knockback").and_then(JsonValue::as_i64), Some(8));
  }

  #[test]
  fn actor_knockback_event_projects_to_mcp_json() {
    let value = game_event_to_json(&GameEvent::ActorKnockedBack {
      entity_id: drl_protocol::EntityId::new(2),
      from: drl_protocol::Position::new(3, 3),
      to: drl_protocol::Position::new(4, 2),
    });
    let JsonValue::Object(map) = value else {
      panic!("event projection must be an object");
    };
    assert_eq!(
      map.get("type").and_then(JsonValue::as_str),
      Some("ActorKnockedBack")
    );
    assert_eq!(map.get("entity_id").and_then(JsonValue::as_i64), Some(2));
    assert_eq!(
      map.get("from"),
      Some(&position_to_json(drl_protocol::Position::new(3, 3)))
    );
    assert_eq!(
      map.get("to"),
      Some(&position_to_json(drl_protocol::Position::new(4, 2)))
    );
  }

  #[test]
  fn ground_item_destroyed_event_projects_to_mcp_json() {
    let value = game_event_to_json(&GameEvent::GroundItemDestroyed {
      item_id: drl_protocol::ItemId::new(7),
      position: drl_protocol::Position::new(4, 2),
    });
    let JsonValue::Object(map) = value else {
      panic!("event projection must be an object");
    };
    assert_eq!(
      map.get("type").and_then(JsonValue::as_str),
      Some("GroundItemDestroyed")
    );
    assert_eq!(map.get("item_id").and_then(JsonValue::as_i64), Some(7));
    assert_eq!(
      map.get("position"),
      Some(&position_to_json(drl_protocol::Position::new(4, 2)))
    );
  }

  #[test]
  fn bfg9000_explosion_schedule_event_projects_to_mcp_json() {
    let value = game_event_to_json(&GameEvent::Bfg9000ExplosionScheduled {
      entity_id: drl_protocol::EntityId::new(1),
      target_id: drl_protocol::EntityId::new(2),
      delay: 33,
      radius: 8,
      knockback: 16,
    });
    let JsonValue::Object(map) = value else {
      panic!("event projection must be an object");
    };
    assert_eq!(
      map.get("type").and_then(JsonValue::as_str),
      Some("Bfg9000ExplosionScheduled")
    );
    assert_eq!(map.get("entity_id").and_then(JsonValue::as_i64), Some(1));
    assert_eq!(map.get("target_id").and_then(JsonValue::as_i64), Some(2));
    assert_eq!(map.get("delay").and_then(JsonValue::as_i64), Some(33));
    assert_eq!(map.get("radius").and_then(JsonValue::as_i64), Some(8));
    assert_eq!(map.get("knockback").and_then(JsonValue::as_i64), Some(16));
  }

  #[test]
  fn nuclear_bfg9000_explosion_schedule_event_projects_to_mcp_json() {
    let value = game_event_to_json(&GameEvent::NuclearBfg9000ExplosionScheduled {
      entity_id: drl_protocol::EntityId::new(1),
      target_id: drl_protocol::EntityId::new(2),
      delay: 33,
      radius: 8,
      knockback: 16,
    });
    let JsonValue::Object(map) = value else {
      panic!("event projection must be an object");
    };
    assert_eq!(
      map.get("type").and_then(JsonValue::as_str),
      Some("NuclearBfg9000ExplosionScheduled")
    );
    assert_eq!(map.get("entity_id").and_then(JsonValue::as_i64), Some(1));
    assert_eq!(map.get("target_id").and_then(JsonValue::as_i64), Some(2));
    assert_eq!(map.get("delay").and_then(JsonValue::as_i64), Some(33));
    assert_eq!(map.get("radius").and_then(JsonValue::as_i64), Some(8));
    assert_eq!(map.get("knockback").and_then(JsonValue::as_i64), Some(16));
  }

  #[test]
  fn weapon_recharged_event_projects_to_mcp_json() {
    let value = game_event_to_json(&GameEvent::WeaponRecharged {
      entity_id: drl_protocol::EntityId::new(1),
      item_id: ItemId::new(2),
      ammo_recharged: 1,
      current_clip: 10,
      max_clip: 10,
      timer: 30,
    });
    let JsonValue::Object(map) = value else {
      panic!("event projection must be an object");
    };
    assert_eq!(
      map.get("type").and_then(JsonValue::as_str),
      Some("WeaponRecharged")
    );
    assert_eq!(map.get("entity_id").and_then(JsonValue::as_i64), Some(1));
    assert_eq!(map.get("item_id").and_then(JsonValue::as_i64), Some(2));
    assert_eq!(
      map.get("current_clip").and_then(JsonValue::as_i64),
      Some(10)
    );
    assert_eq!(map.get("max_clip").and_then(JsonValue::as_i64), Some(10));
  }

  #[test]
  fn maleks_armor_recharged_event_projects_to_mcp_json() {
    let value = game_event_to_json(&GameEvent::MalekArmorRecharged {
      entity_id: drl_protocol::EntityId::new(1),
      item_id: ItemId::new(2),
      durability_restored: 1,
      durability_remaining: 100,
      timer: 50,
    });
    let JsonValue::Object(map) = value else {
      panic!("event projection must be an object");
    };
    assert_eq!(
      map.get("type").and_then(JsonValue::as_str),
      Some("MalekArmorRecharged")
    );
    assert_eq!(map.get("entity_id").and_then(JsonValue::as_i64), Some(1));
    assert_eq!(map.get("item_id").and_then(JsonValue::as_i64), Some(2));
    assert_eq!(
      map.get("durability_restored").and_then(JsonValue::as_i64),
      Some(1)
    );
    assert_eq!(
      map.get("durability_remaining").and_then(JsonValue::as_i64),
      Some(100)
    );
    assert_eq!(map.get("timer").and_then(JsonValue::as_i64), Some(50));
  }

  #[test]
  fn nuclear_weapon_overloaded_event_projects_to_mcp_json() {
    let value = game_event_to_json(&GameEvent::NuclearWeaponOverloaded {
      entity_id: drl_protocol::EntityId::new(1),
      item_id: ItemId::new(2),
      countdown: 100,
      score_count_remaining: -1_000,
    });
    let JsonValue::Object(map) = value else {
      panic!("event projection must be an object");
    };
    assert_eq!(
      map.get("type").and_then(JsonValue::as_str),
      Some("NuclearWeaponOverloaded")
    );
    assert_eq!(map.get("entity_id").and_then(JsonValue::as_i64), Some(1));
    assert_eq!(map.get("item_id").and_then(JsonValue::as_i64), Some(2));
    assert_eq!(map.get("countdown").and_then(JsonValue::as_i64), Some(100));
    assert_eq!(
      map.get("score_count_remaining").and_then(JsonValue::as_i64),
      Some(-1_000)
    );
  }

  #[test]
  fn environment_damage_type_projects_to_mcp_json() {
    let value = game_event_to_json(&GameEvent::DamageApplied {
      target_id: drl_protocol::EntityId::new(1),
      amount: 6,
      remaining_hp: 44,
      source: drl_protocol::DamageSource::Environment,
      damage_type: Some(drl_protocol::DamageType::Acid),
    });
    let JsonValue::Object(map) = value else {
      panic!("event projection must be an object");
    };
    assert_eq!(
      map.get("damage_type").and_then(JsonValue::as_str),
      Some("Acid")
    );
  }

  #[test]
  fn load_scenario_accepts_acid_water_and_mud_glyphs() {
    let mut session = McpSession::new();
    let observation = session
      .load_scenario("\n#######\n#@xwu.#\n#######\n", None)
      .unwrap();

    assert_eq!(
      observation
        .visible_tiles
        .iter()
        .find(|tile| tile.position == Position::new(2, 1))
        .map(|tile| tile.kind),
      Some(TileKind::Acid)
    );
    assert_eq!(
      observation
        .visible_tiles
        .iter()
        .find(|tile| tile.position == Position::new(3, 1))
        .map(|tile| tile.kind),
      Some(TileKind::Water)
    );
    assert_eq!(
      observation
        .visible_tiles
        .iter()
        .find(|tile| tile.position == Position::new(4, 1))
        .map(|tile| tile.kind),
      Some(TileKind::Mud)
    );
  }

  #[test]
  fn test_legal_actions_synthesis() {
    let mut session = McpSession::new();
    session
      .start_game(42, Some(100), Some(20), Some(15))
      .unwrap();
    let actions = session.legal_actions().unwrap();
    assert!(!actions.is_empty());
    assert!(actions.iter().any(|a| a.action == "Wait"));
    assert_eq!(actions, session.legal_actions().unwrap());

    let game = session.game.as_ref().unwrap();
    for action in &actions {
      let mut probe = game.clone();
      assert!(
        probe.step(action.command).is_ok(),
        "filtered action failed core probe: {:?}",
        action.command
      );
    }
  }

  #[test]
  fn test_legal_actions_filter_core_rejections_without_mutation() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n#######\n#@.h..#\n#######\n", None)
      .unwrap();

    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::combat_knife(ItemId::new(100)),
      )
      .unwrap();
    let ammo_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .inventory_mut()
      .add_item(drl_core::item::Item::ammo_9mm(ammo_id, 40))
      .unwrap();

    let observation = session.get_observation().unwrap();
    assert!(compute_legal_actions(&observation).iter().any(|action| {
      action.action == "Fire" && action.command == Command::AttackRanged(Position::new(3, 1))
    }));

    let observation_before = observation.clone();
    let game_before = session.game.clone();
    let metrics_before = session.get_metrics().clone();
    let replay_before = session.export_replay().unwrap().clone();
    let events_before = session.recent_events().to_vec();
    let actions = session.legal_actions().unwrap();

    assert!(!actions.iter().any(|action| action.action == "Fire"));
    assert!(actions.iter().any(|action| action.action == "Wait"));
    assert_eq!(
      session
        .step(Command::AttackRanged(Position::new(3, 1)))
        .unwrap_err(),
      "Command is not currently advertised as legal"
    );
    assert_eq!(session.get_observation().unwrap(), observation_before);
    assert_eq!(session.game, game_before);
    assert_eq!(session.get_metrics(), &metrics_before);
    assert_eq!(session.export_replay().unwrap(), &replay_before);
    assert_eq!(session.recent_events(), events_before.as_slice());
  }

  #[test]
  fn test_legal_action_catalog_advertises_chaingun_chainfire_levels() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n#######\n#@.h..#\n#######\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::chaingun(ItemId::new(100)),
      )
      .unwrap();

    let target_id = session
      .game
      .as_ref()
      .unwrap()
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("Chaingun chainfire target")
      .id();
    let target = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(target_id)
      .expect("Chaingun chainfire target");
    target.hp_mut().max = 10_000;
    target.hp_mut().current = 10_000;

    let observation = session.get_observation().unwrap();
    let chainfire = compute_legal_actions(&observation)
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("first Chaingun chainfire should be advertised");
    assert_eq!(
      chainfire.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(chainfire.description.contains("3 projectiles, 3 rounds"));
    session.step(chainfire.command).expect("chainfire action");

    let second = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("second Chaingun chainfire should be advertised");
    assert_eq!(
      second.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(second.description.contains("4 projectiles, 4 rounds"));
    session
      .step(second.command)
      .expect("second chainfire action");
    let third = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("third Chaingun chainfire should be advertised");
    assert_eq!(
      third.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(third.description.contains("6 projectiles, 6 rounds"));
    session.step(third.command).expect("third chainfire action");
    let fourth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("fourth Chaingun chainfire should be advertised");
    assert_eq!(
      fourth.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(fourth.description.contains("6 projectiles, 6 rounds"));
    session
      .step(fourth.command)
      .expect("fourth chainfire action");
    let fifth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("fifth Chaingun chainfire should be advertised");
    assert_eq!(
      fifth.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(fifth.description.contains("6 projectiles, 6 rounds"));
    session.step(fifth.command).expect("fifth chainfire action");
    let sixth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("sixth Chaingun chainfire should be advertised");
    assert_eq!(
      sixth.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(sixth.description.contains("6 projectiles, 6 rounds"));
    session.step(sixth.command).expect("sixth chainfire action");
    let seventh = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("seventh Chaingun chainfire should be advertised");
    assert_eq!(
      seventh.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(seventh.description.contains("6 projectiles, 6 rounds"));
    session
      .step(seventh.command)
      .expect("seventh chainfire action");
    let reload = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Reload")
      .expect("Chaingun reload should be advertised before eighth chainfire");
    assert_eq!(reload.command, Command::Reload);
    session
      .step(reload.command)
      .expect("reload before eighth chainfire action");
    let eighth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("eighth Chaingun chainfire should be advertised");
    assert_eq!(
      eighth.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(eighth.description.contains("6 projectiles, 6 rounds"));
    session
      .step(eighth.command)
      .expect("eighth chainfire action");
    let ninth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("ninth Chaingun chainfire should be advertised");
    assert_eq!(
      ninth.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(ninth.description.contains("6 projectiles, 6 rounds"));
    session.step(ninth.command).expect("ninth chainfire action");
    let tenth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("tenth Chaingun chainfire should be advertised");
    assert_eq!(
      tenth.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(tenth.description.contains("6 projectiles, 6 rounds"));
    session.step(tenth.command).expect("tenth chainfire action");
    assert!(
      !compute_legal_actions(&session.get_observation().unwrap())
        .iter()
        .any(|action| action.action == "Chainfire")
    );
  }

  #[test]
  fn chaingun_chainfire_mcp_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(3, 1);
    let player_config = drl_protocol::PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![drl_protocol::ItemSpawnKind::Ammo9mm(40)],
      equipped_weapon: Some(drl_protocol::ItemSpawnKind::Chaingun),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(2_647, 8, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      target_position,
      "Static Target",
      10_000,
      0,
      (1, 7),
    ));

    let mut session = McpSession::new();
    session
      .load_replay(setup_replay)
      .expect("load Chaingun chainfire replay setup");
    let initial = session.game.as_ref().unwrap().clone();
    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("Chaingun chainfire target")
      .id();
    let command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&initial.observe_player())
        .iter()
        .any(|action| action.command == command)
    );

    let mut direct = initial.clone();
    let mut expected_events = direct
      .step(command)
      .expect("direct Chaingun chainfire command");
    let (events, observation, outcome) = session
      .step(command)
      .expect("MCP Chaingun chainfire command");
    assert_eq!(events, expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(observation, direct.observe_player());
    assert_eq!(outcome, None);
    assert_eq!(
      events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      3
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      37
    );
    let second_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == second_command)
    );
    let second_expected_events = direct
      .step(second_command)
      .expect("direct second Chaingun chainfire command");
    let (second_events, second_observation, second_outcome) = session
      .step(second_command)
      .expect("MCP second Chaingun chainfire command");
    assert_eq!(second_events, second_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(second_observation, direct.observe_player());
    assert_eq!(second_outcome, None);
    assert_eq!(
      second_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      4
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      33
    );
    let third_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == third_command)
    );
    let third_expected_events = direct
      .step(third_command)
      .expect("direct third Chaingun chainfire command");
    let (third_events, third_observation, third_outcome) = session
      .step(third_command)
      .expect("MCP third Chaingun chainfire command");
    assert_eq!(third_events, third_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(third_observation, direct.observe_player());
    assert_eq!(third_outcome, None);
    assert_eq!(
      third_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      6
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      27
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      3
    );
    let fourth_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == fourth_command)
    );
    let fourth_expected_events = direct
      .step(fourth_command)
      .expect("direct fourth Chaingun chainfire command");
    let (fourth_events, fourth_observation, fourth_outcome) = session
      .step(fourth_command)
      .expect("MCP fourth Chaingun chainfire command");
    assert_eq!(fourth_events, fourth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fourth_observation, direct.observe_player());
    assert_eq!(fourth_outcome, None);
    assert_eq!(
      fourth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      6
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      21
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      4
    );
    let fifth_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == fifth_command)
    );
    let fifth_expected_events = direct
      .step(fifth_command)
      .expect("direct fifth Chaingun chainfire command");
    let (fifth_events, fifth_observation, fifth_outcome) = session
      .step(fifth_command)
      .expect("MCP fifth Chaingun chainfire command");
    assert_eq!(fifth_events, fifth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fifth_observation, direct.observe_player());
    assert_eq!(fifth_outcome, None);
    assert_eq!(
      fifth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      6
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      5
    );
    let sixth_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == sixth_command)
    );
    let sixth_expected_events = direct
      .step(sixth_command)
      .expect("direct sixth Chaingun chainfire command");
    let (sixth_events, sixth_observation, sixth_outcome) = session
      .step(sixth_command)
      .expect("MCP sixth Chaingun chainfire command");
    assert_eq!(sixth_events, sixth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(sixth_observation, direct.observe_player());
    assert_eq!(sixth_outcome, None);
    assert_eq!(
      sixth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      6
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      9
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      6
    );
    let seventh_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == seventh_command)
    );
    let seventh_expected_events = direct
      .step(seventh_command)
      .expect("direct seventh Chaingun chainfire command");
    let (seventh_events, seventh_observation, seventh_outcome) = session
      .step(seventh_command)
      .expect("MCP seventh Chaingun chainfire command");
    assert_eq!(seventh_events, seventh_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(seventh_observation, direct.observe_player());
    assert_eq!(seventh_outcome, None);
    assert_eq!(
      seventh_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      6
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      3
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      7
    );
    let reload_command = Command::Reload;
    let reload_expected_events = direct
      .step(reload_command)
      .expect("direct Chaingun chainfire reload");
    let (reload_events, reload_observation, reload_outcome) = session
      .step(reload_command)
      .expect("MCP Chaingun chainfire reload");
    assert_eq!(reload_events, reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(reload_observation, direct.observe_player());
    assert_eq!(reload_outcome, None);
    assert!(reload_events.iter().any(|event| matches!(
      event,
      GameEvent::WeaponReloaded {
        entity_id,
        ammo_loaded: 37,
        current_clip: 40,
        max_clip: 40,
        ..
      } if *entity_id == player_id
    )));
    let eighth_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == eighth_command)
    );
    let eighth_expected_events = direct
      .step(eighth_command)
      .expect("direct eighth Chaingun chainfire command");
    let (eighth_events, eighth_observation, eighth_outcome) = session
      .step(eighth_command)
      .expect("MCP eighth Chaingun chainfire command");
    assert_eq!(eighth_events, eighth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(eighth_observation, direct.observe_player());
    assert_eq!(eighth_outcome, None);
    assert_eq!(
      eighth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      6
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      34
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      8
    );
    let ninth_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == ninth_command)
    );
    let ninth_expected_events = direct
      .step(ninth_command)
      .expect("direct ninth Chaingun chainfire command");
    let (ninth_events, ninth_observation, ninth_outcome) = session
      .step(ninth_command)
      .expect("MCP ninth Chaingun chainfire command");
    assert_eq!(ninth_events, ninth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(ninth_observation, direct.observe_player());
    assert_eq!(ninth_outcome, None);
    assert_eq!(
      ninth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      6
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      28
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      9
    );
    let tenth_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == tenth_command)
    );
    let tenth_expected_events = direct
      .step(tenth_command)
      .expect("direct tenth Chaingun chainfire command");
    let (tenth_events, tenth_observation, tenth_outcome) = session
      .step(tenth_command)
      .expect("MCP tenth Chaingun chainfire command");
    assert_eq!(tenth_events, tenth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(tenth_observation, direct.observe_player());
    assert_eq!(tenth_outcome, None);
    assert_eq!(
      tenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      6
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      22
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      10
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );
    expected_events.extend(second_events);
    expected_events.extend(third_events);
    expected_events.extend(fourth_events);
    expected_events.extend(fifth_events);
    expected_events.extend(sixth_events);
    expected_events.extend(seventh_events);
    expected_events.extend(reload_events);
    expected_events.extend(eighth_events);
    expected_events.extend(ninth_events);
    expected_events.extend(tenth_events);

    let replay = session.export_replay().expect("MCP replay export");
    let (replayed, replay_events) =
      ReplayEngine::run(replay).expect("MCP Chaingun chainfire replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(ReplayEngine::verify_determinism(replay).expect("replay determinism"));
  }

  #[test]
  fn test_legal_action_catalog_advertises_minigun_chainfire_levels() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n#######\n#@.h..#\n#######\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::minigun(ItemId::new(100)),
      )
      .unwrap();

    let target_id = session
      .game
      .as_ref()
      .unwrap()
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("Minigun chainfire target")
      .id();
    let target = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(target_id)
      .expect("Minigun chainfire target");
    target.hp_mut().max = 10_000;
    target.hp_mut().current = 10_000;

    let observation = session.get_observation().unwrap();
    let chainfire = compute_legal_actions(&observation)
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("first Minigun chainfire should be advertised");
    assert_eq!(
      chainfire.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(chainfire.description.contains("6 projectiles, 6 rounds"));
    session.step(chainfire.command).expect("chainfire action");
    let second = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("second Minigun chainfire should be advertised");
    assert_eq!(
      second.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(second.description.contains("8 projectiles, 8 rounds"));
    session
      .step(second.command)
      .expect("second Minigun chainfire action");
    let third = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("third Minigun chainfire should be advertised");
    assert_eq!(
      third.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(third.description.contains("12 projectiles, 12 rounds"));
    session
      .step(third.command)
      .expect("third Minigun chainfire action");
    assert!(
      !compute_legal_actions(&session.get_observation().unwrap())
        .iter()
        .any(|action| action.action == "Chainfire")
    );
  }

  #[test]
  fn test_legal_action_catalog_advertises_plasma_rifle_chainfire_levels() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n#######\n#@.h..#\n#######\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::plasma_rifle(ItemId::new(100)),
      )
      .unwrap();

    let target_id = session
      .game
      .as_ref()
      .unwrap()
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("Plasma Rifle chainfire target")
      .id();
    let target = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(target_id)
      .expect("Plasma Rifle chainfire target");
    target.hp_mut().max = 10_000;
    target.hp_mut().current = 10_000;

    let observation = session.get_observation().unwrap();
    let chainfire = compute_legal_actions(&observation)
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("first Plasma Rifle chainfire should be advertised");
    assert_eq!(
      chainfire.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(chainfire.description.contains("4 projectiles, 4 rounds"));
    session.step(chainfire.command).expect("chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 6;
    let second = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("second Plasma Rifle chainfire should be advertised");
    assert_eq!(
      second.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(second.description.contains("6 projectiles, 6 rounds"));
    session
      .step(second.command)
      .expect("second Plasma Rifle chainfire action");
    assert!(
      !compute_legal_actions(&session.get_observation().unwrap())
        .iter()
        .any(|action| action.action == "Chainfire")
    );
  }

  #[test]
  fn test_legal_action_catalog_advertises_laser_rifle_chainfire_levels() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n#######\n#@.h..#\n#######\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::laser_rifle(ItemId::new(100)),
      )
      .unwrap();

    let target_id = session
      .game
      .as_ref()
      .unwrap()
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("Laser Rifle chainfire target")
      .id();
    let target = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(target_id)
      .expect("Laser Rifle chainfire target");
    target.hp_mut().max = 10_000;
    target.hp_mut().current = 10_000;

    let observation = session.get_observation().unwrap();
    let chainfire = compute_legal_actions(&observation)
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("first Laser Rifle chainfire should be advertised");
    assert_eq!(
      chainfire.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(chainfire.description.contains("4 projectiles, 4 rounds"));
    session.step(chainfire.command).expect("chainfire action");
    let second = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("second Laser Rifle chainfire should be advertised");
    assert_eq!(
      second.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(second.description.contains("5 projectiles, 5 rounds"));
    session
      .step(second.command)
      .expect("second Laser Rifle chainfire action");
    let third = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("third Laser Rifle chainfire should be advertised");
    assert_eq!(
      third.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(third.description.contains("7 projectiles, 7 rounds"));
    session
      .step(third.command)
      .expect("third Laser Rifle chainfire action");
    let fourth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("fourth Laser Rifle chainfire should be advertised");
    assert_eq!(
      fourth.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(fourth.description.contains("7 projectiles, 7 rounds"));
    session
      .step(fourth.command)
      .expect("fourth Laser Rifle chainfire action");
    let fifth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("fifth Laser Rifle chainfire should be advertised");
    assert_eq!(
      fifth.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(fifth.description.contains("7 projectiles, 7 rounds"));
    session
      .step(fifth.command)
      .expect("fifth Laser Rifle chainfire action");
    let sixth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("sixth Laser Rifle chainfire should be advertised");
    assert_eq!(
      sixth.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(sixth.description.contains("7 projectiles, 7 rounds"));
    session
      .step(sixth.command)
      .expect("sixth Laser Rifle chainfire action");
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 7;
    let seventh = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("seventh Laser Rifle chainfire should be advertised");
    assert_eq!(
      seventh.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(seventh.description.contains("7 projectiles, 7 rounds"));
    session
      .step(seventh.command)
      .expect("seventh Laser Rifle chainfire action");
    assert!(
      !compute_legal_actions(&session.get_observation().unwrap())
        .iter()
        .any(|action| action.action == "Chainfire")
    );
  }

  #[test]
  fn test_legal_action_catalog_advertises_bfg10k_chainfire_warmup_levels() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n#######\n#@.h..#\n#######\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::bfg10k(ItemId::new(100)),
      )
      .unwrap();
    let target_id = session
      .game
      .as_ref()
      .unwrap()
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("scenario target")
      .id();
    let target = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(target_id)
      .expect("scenario target");
    target.set_speed(drl_protocol::Speed::new(0));
    target.hp_mut().max = 1_000_000;
    target.hp_mut().current = 1_000_000;

    let observation = session.get_observation().unwrap();
    let chainfire = compute_legal_actions(&observation)
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("first BFG 10K chainfire should be advertised");
    assert_eq!(
      chainfire.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(chainfire.description.contains("4 projectiles, 20 rounds"));
    session.step(chainfire.command).expect("chainfire action");
    let second = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("second BFG 10K chainfire should be advertised");
    assert!(second.description.contains("5 projectiles, 25 rounds"));
    session
      .step(second.command)
      .expect("second chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 35;
    let third = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("third BFG 10K chainfire should be advertised");
    assert!(third.description.contains("7 projectiles, 35 rounds"));
    session.step(third.command).expect("third chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let fourth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("fourth BFG 10K chainfire should be advertised");
    assert!(fourth.description.contains("7 projectiles, 35 rounds"));
    session
      .step(fourth.command)
      .expect("fourth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let fifth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("fifth BFG 10K chainfire should be advertised");
    assert!(fifth.description.contains("7 projectiles, 35 rounds"));
    session.step(fifth.command).expect("fifth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let sixth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("sixth BFG 10K chainfire should be advertised");
    assert!(matches!(sixth.command, Command::AttackRangedChainfire(_)));
    assert!(sixth.description.contains("7 projectiles, 35 rounds"));
    session.step(sixth.command).expect("sixth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let seventh = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("seventh BFG 10K chainfire should be advertised");
    assert!(matches!(seventh.command, Command::AttackRangedChainfire(_)));
    assert!(seventh.description.contains("7 projectiles, 35 rounds"));
    session
      .step(seventh.command)
      .expect("seventh chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let eighth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("eighth BFG 10K chainfire should be advertised");
    assert!(matches!(eighth.command, Command::AttackRangedChainfire(_)));
    assert!(eighth.description.contains("7 projectiles, 35 rounds"));
    session
      .step(eighth.command)
      .expect("eighth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let ninth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("ninth BFG 10K chainfire should be advertised");
    assert!(matches!(ninth.command, Command::AttackRangedChainfire(_)));
    assert!(ninth.description.contains("7 projectiles, 35 rounds"));
    session.step(ninth.command).expect("ninth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let tenth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("tenth BFG 10K chainfire should be advertised");
    assert!(matches!(tenth.command, Command::AttackRangedChainfire(_)));
    assert!(tenth.description.contains("7 projectiles, 35 rounds"));
    session.step(tenth.command).expect("tenth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let eleventh = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("eleventh BFG 10K chainfire should be advertised");
    assert!(matches!(
      eleventh.command,
      Command::AttackRangedChainfire(_)
    ));
    assert!(eleventh.description.contains("7 projectiles, 35 rounds"));
    session
      .step(eleventh.command)
      .expect("eleventh chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let twelfth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("twelfth BFG 10K chainfire should be advertised");
    assert!(matches!(twelfth.command, Command::AttackRangedChainfire(_)));
    assert!(twelfth.description.contains("7 projectiles, 35 rounds"));
    session
      .step(twelfth.command)
      .expect("twelfth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let thirteenth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("thirteenth BFG 10K chainfire should be advertised");
    assert!(matches!(
      thirteenth.command,
      Command::AttackRangedChainfire(_)
    ));
    assert!(thirteenth.description.contains("7 projectiles, 35 rounds"));
    session
      .step(thirteenth.command)
      .expect("thirteenth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let fourteenth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("fourteenth BFG 10K chainfire should be advertised");
    assert!(matches!(
      fourteenth.command,
      Command::AttackRangedChainfire(_)
    ));
    assert!(fourteenth.description.contains("7 projectiles, 35 rounds"));
    session
      .step(fourteenth.command)
      .expect("fourteenth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let fifteenth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("fifteenth BFG 10K chainfire should be advertised");
    assert!(matches!(
      fifteenth.command,
      Command::AttackRangedChainfire(_)
    ));
    assert!(fifteenth.description.contains("7 projectiles, 35 rounds"));
    session
      .step(fifteenth.command)
      .expect("fifteenth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let sixteenth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("sixteenth BFG 10K chainfire should be advertised");
    assert!(matches!(
      sixteenth.command,
      Command::AttackRangedChainfire(_)
    ));
    assert!(sixteenth.description.contains("7 projectiles, 35 rounds"));
    session
      .step(sixteenth.command)
      .expect("sixteenth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let seventeenth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("seventeenth BFG 10K chainfire should be advertised");
    assert!(matches!(
      seventeenth.command,
      Command::AttackRangedChainfire(_)
    ));
    assert!(seventeenth.description.contains("7 projectiles, 35 rounds"));
    session
      .step(seventeenth.command)
      .expect("seventeenth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let eighteenth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("eighteenth BFG 10K chainfire should be advertised");
    assert!(matches!(
      eighteenth.command,
      Command::AttackRangedChainfire(_)
    ));
    assert!(eighteenth.description.contains("7 projectiles, 35 rounds"));
    session
      .step(eighteenth.command)
      .expect("eighteenth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let nineteenth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("nineteenth BFG 10K chainfire should be advertised");
    assert!(matches!(
      nineteenth.command,
      Command::AttackRangedChainfire(_)
    ));
    assert!(nineteenth.description.contains("7 projectiles, 35 rounds"));
    session
      .step(nineteenth.command)
      .expect("nineteenth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let twentieth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("twentieth BFG 10K chainfire should be advertised");
    assert!(matches!(
      twentieth.command,
      Command::AttackRangedChainfire(_)
    ));
    assert!(twentieth.description.contains("7 projectiles, 35 rounds"));
    session
      .step(twentieth.command)
      .expect("twentieth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    let twenty_first = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("twenty-first BFG 10K chainfire should be advertised");
    assert!(matches!(
      twenty_first.command,
      Command::AttackRangedChainfire(_)
    ));
    assert!(
      twenty_first
        .description
        .contains("7 projectiles, 35 rounds")
    );
    session
      .step(twenty_first.command)
      .expect("twenty-first chainfire action");
    assert!(
      !compute_legal_actions(&session.get_observation().unwrap())
        .iter()
        .any(|action| action.action == "Chainfire")
    );
  }

  #[test]
  fn test_legal_action_catalog_advertises_nuclear_plasma_chainfire_levels() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n#######\n#@.h..#\n#######\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::nuclear_plasma_rifle(ItemId::new(100)),
      )
      .unwrap();
    let target_id = session
      .game
      .as_ref()
      .unwrap()
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("Nuclear Plasma chainfire target")
      .id();
    let target = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(target_id)
      .expect("Nuclear Plasma chainfire target");
    target.hp_mut().max = 10_000;
    target.hp_mut().current = 10_000;

    let observation = session.get_observation().unwrap();
    let chainfire = compute_legal_actions(&observation)
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("first Nuclear Plasma chainfire should be advertised");
    assert_eq!(
      chainfire.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(chainfire.description.contains("4 projectiles, 4 rounds"));
    session.step(chainfire.command).expect("chainfire action");
    let second = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("second Nuclear Plasma chainfire should be advertised");
    assert_eq!(
      second.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(second.description.contains("6 projectiles, 6 rounds"));
    session
      .step(second.command)
      .expect("second chainfire action");
    let third = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("third Nuclear Plasma chainfire should be advertised");
    assert_eq!(
      third.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(third.description.contains("9 projectiles, 9 rounds"));
    session.step(third.command).expect("third chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 24;
    let fourth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("fourth Nuclear Plasma chainfire should be advertised");
    assert_eq!(
      fourth.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(fourth.description.contains("9 projectiles, 9 rounds"));
    session
      .step(fourth.command)
      .expect("fourth chainfire action");
    let fifth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("fifth Nuclear Plasma chainfire should be advertised");
    assert_eq!(
      fifth.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(fifth.description.contains("9 projectiles, 9 rounds"));
    session.step(fifth.command).expect("fifth chainfire action");
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 24;
    let sixth = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("sixth Nuclear Plasma chainfire should be advertised");
    assert_eq!(
      sixth.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(sixth.description.contains("9 projectiles, 9 rounds"));
    session.step(sixth.command).expect("sixth chainfire action");
    let seventh = compute_legal_actions(&session.get_observation().unwrap())
      .into_iter()
      .find(|action| action.action == "Chainfire")
      .expect("seventh Nuclear Plasma chainfire should be advertised");
    assert_eq!(
      seventh.command,
      Command::AttackRangedChainfire(Position::new(3, 1))
    );
    assert!(seventh.description.contains("9 projectiles, 9 rounds"));
    session
      .step(seventh.command)
      .expect("seventh chainfire action");
    assert!(
      !compute_legal_actions(&session.get_observation().unwrap())
        .iter()
        .any(|action| action.action == "Chainfire")
    );
  }

  #[test]
  fn pistol_aimed_fire_is_advertised_and_executed_through_mcp() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n########\n#@..h..#\n########\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::pistol(weapon_id),
      )
      .unwrap();

    let target = Position::new(4, 1);
    let command = Command::AttackRangedAimed(target);
    let actions = session.legal_actions().unwrap();
    assert!(
      actions
        .iter()
        .any(|action| { action.action == "AimedFire" && action.command == command })
    );

    let (events, _, _) = session.step(command).unwrap();
    assert!(events.iter().any(|event| {
      matches!(
        event,
        GameEvent::ActionCostPaid {
          cost: drl_protocol::ActionCost(2_000),
          ..
        }
      )
    }));
    assert_eq!(session.export_replay().unwrap().commands, vec![command]);
  }

  #[test]
  fn combat_pistol_aimed_fire_is_advertised_and_executed_through_mcp() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n########\n#@..h..#\n########\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::combat_pistol(weapon_id),
      )
      .unwrap();

    let target = Position::new(4, 1);
    let command = Command::AttackRangedAimed(target);
    assert!(
      session
        .legal_actions()
        .unwrap()
        .iter()
        .any(|action| { action.action == "AimedFire" && action.command == command })
    );
    let (events, _, _) = session.step(command).unwrap();
    assert!(events.iter().any(|event| {
      matches!(
        event,
        GameEvent::ActionCostPaid {
          cost: drl_protocol::ActionCost(2_000),
          ..
        }
      )
    }));
    assert_eq!(session.export_replay().unwrap().commands, vec![command]);
  }

  #[test]
  fn blaster_aimed_fire_is_advertised_and_executed_through_mcp() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n########\n#@..h..#\n########\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::blaster(weapon_id),
      )
      .unwrap();

    let target = Position::new(4, 1);
    let command = Command::AttackRangedAimed(target);
    assert!(
      session
        .legal_actions()
        .unwrap()
        .iter()
        .any(|action| { action.action == "AimedFire" && action.command == command })
    );
    let (events, _, _) = session.step(command).unwrap();
    assert!(events.iter().any(|event| {
      matches!(
        event,
        GameEvent::ActionCostPaid {
          cost: drl_protocol::ActionCost(2_000),
          ..
        }
      )
    }));
    assert_eq!(session.export_replay().unwrap().commands, vec![command]);
    assert_eq!(
      session
        .game
        .as_ref()
        .unwrap()
        .world()
        .player()
        .unwrap()
        .weapon_recharge_timer(),
      1
    );
  }

  #[test]
  fn trigun_aimed_fire_is_advertised_and_executed_through_mcp() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n########\n#@..h..#\n########\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::trigun(weapon_id),
      )
      .unwrap();

    let target = Position::new(4, 1);
    let command = Command::AttackRangedAimed(target);
    assert!(
      session
        .legal_actions()
        .unwrap()
        .iter()
        .any(|action| { action.action == "AimedFire" && action.command == command })
    );
    let (events, _, _) = session.step(command).unwrap();
    assert!(events.iter().any(|event| {
      matches!(
        event,
        GameEvent::ActionCostPaid {
          cost: drl_protocol::ActionCost(2_000),
          ..
        }
      )
    }));
    assert_eq!(session.export_replay().unwrap().commands, vec![command]);
    assert_eq!(
      session
        .game
        .as_ref()
        .unwrap()
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      5
    );
  }

  #[test]
  fn anti_freak_jackal_aimed_fire_is_advertised_and_executed_through_mcp() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n########\n#@..h..#\n########\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::anti_freak_jackal(weapon_id),
      )
      .unwrap();

    let target = Position::new(4, 1);
    let command = Command::AttackRangedAimed(target);
    assert!(
      session
        .legal_actions()
        .unwrap()
        .iter()
        .any(|action| action.action == "AimedFire" && action.command == command)
    );
    let (events, _, _) = session.step(command).unwrap();
    assert!(events.iter().any(|event| {
      matches!(
        event,
        GameEvent::ActionCostPaid {
          cost: drl_protocol::ActionCost(2_000),
          ..
        }
      )
    }));
    assert_eq!(session.export_replay().unwrap().commands, vec![command]);
    assert_eq!(
      session
        .game
        .as_ref()
        .unwrap()
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      5
    );
  }

  #[test]
  fn test_legal_action_catalog_includes_explicit_melee_and_unequip() {
    let mut session = McpSession::new();
    let obs = session
      .load_scenario("\n#####\n#@h.#\n#####\n", None)
      .unwrap();
    let actions = compute_legal_actions(&obs);
    assert!(actions.iter().any(|action| {
      action.action == "AttackMelee" && action.command == Command::AttackMelee(Direction::East)
    }));

    session
      .load_scenario("\n######\n#@p..#\n######\n", None)
      .unwrap();
    let (_, obs, _) = session.step(Command::Move(Direction::East)).unwrap();
    assert!(
      obs
        .ground_items
        .iter()
        .any(|item| item.position == obs.player_position)
    );
    let (_, obs, _) = session.step(Command::Pickup).unwrap();
    let pistol_id = obs
      .inventory
      .iter()
      .find(|item| item.category == ItemCategory::Weapon)
      .expect("scenario pistol in inventory")
      .id;
    let (_, obs, _) = session.step(Command::Equip(pistol_id)).unwrap();
    assert!(compute_legal_actions(&obs).iter().any(|action| {
      action.action == "Unequip" && action.command == Command::Unequip(EquipmentSlot::Weapon)
    }));
  }

  #[test]
  fn test_legal_action_catalog_and_events_include_grammaton_mode_cycle() {
    let mut session = McpSession::new();
    session.start_game(797, None, None, None).unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let item_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::grammaton_beretta(item_id),
      )
      .unwrap();

    let observation = session.get_observation().unwrap();
    assert!(compute_legal_actions(&observation).iter().any(|action| {
      action.action == "AltReload"
        && action.command
          == Command::AltReload {
            item_id,
            confirmed: true,
          }
    }));

    let (events, _, _) = session
      .step(Command::AltReload {
        item_id,
        confirmed: true,
      })
      .unwrap();
    assert!(
      events
        .iter()
        .any(|event| matches!(event, GameEvent::GrammatonFireModeChanged { .. }))
    );
  }

  #[test]
  fn test_legal_action_catalog_and_events_include_jackhammer_mode_toggle() {
    let mut session = McpSession::new();
    session.start_game(798, None, None, None).unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let item_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    let player = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap();
    player.set_score_count(5);
    player
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::jackhammer(item_id),
      )
      .unwrap();

    let observation = session.get_observation().unwrap();
    assert!(compute_legal_actions(&observation).iter().any(|action| {
      action.action == "AltReload"
        && action.command
          == Command::AltReload {
            item_id,
            confirmed: true,
          }
    }));

    let (events, _, _) = session
      .step(Command::AltReload {
        item_id,
        confirmed: true,
      })
      .unwrap();
    assert!(events.iter().any(|event| matches!(
      event,
      GameEvent::JackhammerFireModeChanged {
        mode: drl_protocol::WeaponFireMode::Single,
        score_count_remaining: 4,
        ..
      }
    )));
  }

  #[test]
  fn test_legal_action_catalog_and_events_include_assault_shotgun_alt_reload() {
    let mut session = McpSession::new();
    session.start_game(799, None, None, None).unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    let shells_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    let player = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap();
    player
      .inventory_mut()
      .add_item(drl_core::item::Item::ammo_shells(shells_id, 6))
      .unwrap();
    player
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::assault_shotgun(weapon_id),
      )
      .unwrap();
    player
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 0;

    let observation = session.get_observation().unwrap();
    let command = Command::AltReload {
      item_id: weapon_id,
      confirmed: false,
    };
    assert!(
      compute_legal_actions(&observation)
        .iter()
        .any(|action| { action.action == "AltReload" && action.command == command })
    );

    let (events, _, _) = session.step(command).unwrap();
    assert!(events.iter().any(|event| matches!(
      event,
      GameEvent::WeaponReloaded {
        entity_id,
        ammo_loaded: 6,
        current_clip: 6,
        max_clip: 6,
      } if *entity_id == player_id
    )));
    assert!(events.iter().any(|event| matches!(
      event,
      GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(2_500),
        } if *entity_id == player_id
    )));
    assert!(
      !session
        .legal_actions()
        .unwrap()
        .iter()
        .any(|action| action.command == command)
    );
  }

  #[test]
  fn test_legal_action_catalog_and_events_include_combat_shotgun_alt_reload() {
    let mut session = McpSession::new();
    session.start_game(800, None, None, None).unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    let shells_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    let player = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap();
    player
      .inventory_mut()
      .add_item(drl_core::item::Item::ammo_shells(shells_id, 5))
      .unwrap();
    player
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::combat_shotgun(weapon_id),
      )
      .unwrap();
    player
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 0;

    let observation = session.get_observation().unwrap();
    let command = Command::AltReload {
      item_id: weapon_id,
      confirmed: false,
    };
    assert!(
      compute_legal_actions(&observation)
        .iter()
        .any(|action| action.action == "AltReload" && action.command == command)
    );

    let (events, _, _) = session.step(command).unwrap();
    assert!(events.iter().any(|event| matches!(
      event,
      GameEvent::WeaponReloaded {
        entity_id,
        ammo_loaded: 5,
        current_clip: 5,
        max_clip: 5,
      } if *entity_id == player_id
    )));
    assert!(events.iter().any(|event| matches!(
      event,
      GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(2_500),
      } if *entity_id == player_id
    )));
    assert!(
      !session
        .legal_actions()
        .unwrap()
        .iter()
        .any(|action| action.command == command)
    );
  }

  #[test]
  fn test_legal_action_catalog_and_events_include_missile_launcher_alt_reload() {
    let mut session = McpSession::new();
    session.start_game(801, None, None, None).unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    let rockets_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    let player = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap();
    player
      .inventory_mut()
      .add_item(drl_core::item::Item::ammo_rockets(rockets_id, 4))
      .unwrap();
    player
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::missile_launcher(weapon_id),
      )
      .unwrap();
    player
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 0;

    let observation = session.get_observation().unwrap();
    let command = Command::AltReload {
      item_id: weapon_id,
      confirmed: false,
    };
    assert!(
      compute_legal_actions(&observation)
        .iter()
        .any(|action| action.action == "AltReload" && action.command == command)
    );

    let (events, _, _) = session.step(command).unwrap();
    assert!(events.iter().any(|event| matches!(
      event,
      GameEvent::WeaponReloaded {
        entity_id,
        ammo_loaded: 4,
        current_clip: 4,
        max_clip: 4,
      } if *entity_id == player_id
    )));
    assert!(events.iter().any(|event| matches!(
      event,
      GameEvent::ActionCostPaid {
        entity_id,
        cost: drl_protocol::ActionCost(2_500),
      } if *entity_id == player_id
    )));
    assert!(
      !session
        .legal_actions()
        .unwrap()
        .iter()
        .any(|action| action.command == command)
    );
  }

  #[test]
  fn standard_bfg_exact_hit_is_exposed_through_mcp_fire_action() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n########\n#@..h..#\n########\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::bfg9000(weapon_id),
      )
      .unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .accuracy = 0;

    let target = Position::new(4, 1);
    let observation = session.get_observation().unwrap();
    let command = Command::AttackRanged(target);
    assert!(
      compute_legal_actions(&observation)
        .iter()
        .any(|action| action.action == "Fire" && action.command == command)
    );

    let (events, _, _) = session.step(command).unwrap();
    let target_id = events
      .iter()
      .find_map(|event| match event {
        GameEvent::AttackResolved { target_id, .. } => Some(*target_id),
        _ => None,
      })
      .expect("standard BFG attack should identify a target");
    assert_standard_bfg_schedule_event(&events, player_id, target_id);
    assert!(events.iter().any(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          target_id: _,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *attacker_id == player_id
      )
    }));
    assert_eq!(
      session
        .game
        .as_ref()
        .unwrap()
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      60
    );
  }

  #[test]
  fn nuclear_bfg_exact_hit_is_exposed_through_mcp_fire_action() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n########\n#@..h..#\n########\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::nuclear_bfg9000(weapon_id),
      )
      .unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .accuracy = 0;

    let target = Position::new(4, 1);
    let observation = session.get_observation().unwrap();
    let command = Command::AttackRanged(target);
    assert!(
      compute_legal_actions(&observation)
        .iter()
        .any(|action| action.action == "Fire" && action.command == command)
    );

    let (events, _, _) = session.step(command).unwrap();
    let target_id = events
      .iter()
      .find_map(|event| match event {
        GameEvent::AttackResolved { target_id, .. } => Some(*target_id),
        _ => None,
      })
      .expect("Nuclear BFG attack should identify a target");
    assert_nuclear_bfg_schedule_event(&events, player_id, target_id);
    assert!(events.iter().any(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
          ..
        } if *attacker_id == player_id
      )
    }));
    assert_eq!(
      session
        .game
        .as_ref()
        .unwrap()
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      0
    );
  }

  #[test]
  fn bfg_family_exact_hit_vertical_mcp_boundaries_match_direct_core() {
    for (kind, expected_clip) in [
      (drl_protocol::ItemSpawnKind::Bfg9000, 60),
      (drl_protocol::ItemSpawnKind::NuclearBfg9000, 0),
    ] {
      let player_position = Position::new(1, 1);
      let target_position = Position::new(5, 1);
      let player_config = drl_protocol::PlayerSpawnConfig {
        hp: 50,
        max_hp: 50,
        speed: 100,
        initial_items: Vec::new(),
        equipped_weapon: Some(kind),
        equipped_armor: None,
        equipped_armor_durability: None,
      };
      let mut setup_replay =
        ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config);
      setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
        target_position,
        "Static Target",
        500,
        1,
        (2, 4),
      ));

      let mut session = McpSession::new();
      session
        .load_replay(setup_replay.clone())
        .expect("load BFG exact-hit replay setup");
      let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
      let initial = session.game.as_ref().unwrap().clone();
      let command = Command::AttackRanged(target_position);
      let mut direct = initial.clone();
      let expected_events = direct.step(command).expect("direct BFG exact-hit command");
      let (events, observation, outcome) = session.step(command).expect("MCP BFG exact-hit shot");

      assert_eq!(events, expected_events);
      assert_eq!(session.game.as_ref().unwrap(), &direct);
      assert_eq!(observation, direct.observe_player());
      assert_eq!(outcome, None);
      assert!(events.iter().any(|event| {
        matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
            ..
          } if *attacker_id == player_id
        )
      }));
      assert_eq!(
        direct
          .world()
          .player()
          .unwrap()
          .equipment()
          .weapon()
          .unwrap()
          .weapon_properties()
          .unwrap()
          .current_clip,
        expected_clip
      );
      let attack_index = events
        .iter()
        .position(|event| matches!(event, GameEvent::AttackResolved { .. }))
        .expect("MCP BFG exact-hit shot must resolve an attack");
      let cost_index = events
        .iter()
        .position(|event| matches!(event, GameEvent::ActionCostPaid { .. }))
        .expect("MCP BFG exact-hit shot must pay an action cost");
      let turn_end_index = events
        .iter()
        .position(|event| matches!(event, GameEvent::TurnEnded { .. }))
        .expect("MCP BFG exact-hit shot must end its turn");
      assert!(attack_index < cost_index);
      assert!(cost_index < turn_end_index);

      let replay = session.export_replay().expect("MCP replay export");
      assert_eq!(replay.commands, vec![command]);
      let (replayed, replay_events) = ReplayEngine::run(replay).expect("MCP BFG exact-hit replay");
      assert_eq!(replayed, direct);
      assert_eq!(replay_events, expected_events);
      assert!(ReplayEngine::verify_determinism(replay).expect("replay determinism"));
    }
  }

  #[test]
  fn revenants_launcher_exact_hit_is_exposed_through_mcp_fire_action() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n########\n#@..h..#\n########\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::revenants_launcher(weapon_id),
      )
      .unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .accuracy = 0;

    let target = Position::new(4, 1);
    let observation = session.get_observation().unwrap();
    let command = Command::AttackRanged(target);
    assert!(
      compute_legal_actions(&observation)
        .iter()
        .any(|action| action.action == "Fire" && action.command == command)
    );

    let (events, _, _) = session.step(command).unwrap();
    assert!(events.iter().any(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
          ..
        } if *attacker_id == player_id
      )
    }));
  }

  #[test]
  fn bfg10k_exact_hit_is_exposed_through_mcp_fire_action() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n########\n#@..h..#\n########\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::bfg10k(weapon_id),
      )
      .unwrap();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .accuracy = 0;

    let target = Position::new(4, 1);
    let observation = session.get_observation().unwrap();
    let command = Command::AttackRanged(target);
    assert!(
      compute_legal_actions(&observation)
        .iter()
        .any(|action| action.action == "Fire" && action.command == command)
    );

    let (events, _, _) = session.step(command).unwrap();
    assert!(events.iter().any(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
          ..
        } if *attacker_id == player_id
      )
    }));
    assert_eq!(
      session
        .game
        .as_ref()
        .unwrap()
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      25
    );
  }

  #[test]
  fn bfg10k_shot_cost_vertical_mcp_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(5, 1);
    let player_config = drl_protocol::PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(drl_protocol::ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      target_position,
      "Static Target",
      500,
      1,
      (2, 4),
    ));
    setup_replay.record_item(drl_protocol::ItemSpawnSpec::new(
      target_position,
      drl_protocol::ItemSpawnKind::AmmoCells(2),
    ));

    let mut session = McpSession::new();
    session
      .load_replay(setup_replay.clone())
      .expect("load canonical BFG 10K replay setup");
    let initial = session.game.as_ref().unwrap().clone();
    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("BFG 10K target")
      .id();
    let command = Command::AttackRanged(target_position);
    let mut direct = initial.clone();
    let expected_events = direct
      .step(command)
      .expect("direct BFG 10K shot-cost command");
    let (events, observation, outcome) = session.step(command).expect("MCP BFG 10K shot");

    assert_eq!(events, expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(observation, direct.observe_player());
    assert_eq!(outcome, None);
    assert_eq!(
      events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            is_ranged: true,
            ..
          }
        ))
        .count(),
      5
    );
    assert_bfg10k_volley_events(&events, player_id, target_id);
    assert!(events.iter().any(|event| {
      matches!(
        event,
        GameEvent::GroundItemDestroyed { position, .. } if *position == target_position
      )
    }));
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      25
    );
    let attack_index = events
      .iter()
      .position(|event| matches!(event, GameEvent::AttackResolved { .. }))
      .expect("MCP BFG 10K shot must resolve an attack");
    let cost_index = events
      .iter()
      .position(|event| matches!(event, GameEvent::ActionCostPaid { .. }))
      .expect("MCP BFG 10K shot must pay an action cost");
    let turn_end_index = events
      .iter()
      .position(|event| matches!(event, GameEvent::TurnEnded { .. }))
      .expect("MCP BFG 10K shot must end its turn");
    assert!(attack_index < cost_index);
    assert!(cost_index < turn_end_index);

    let replay = session.export_replay().expect("MCP replay export");
    let (replayed, replay_events) = ReplayEngine::run(replay).expect("MCP BFG 10K shot replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(ReplayEngine::verify_determinism(replay).expect("replay determinism"));
  }

  #[test]
  fn plasma_rifle_volley_mcp_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(3, 1);
    let player_config = drl_protocol::PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![drl_protocol::ItemSpawnKind::AmmoCells(6)],
      equipped_weapon: Some(drl_protocol::ItemSpawnKind::PlasmaRifle),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(2_269, 8, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      target_position,
      "Static Target",
      500,
      100,
      (1, 7),
    ));

    let mut session = McpSession::new();
    session
      .load_replay(setup_replay)
      .expect("load Plasma Rifle volley replay setup");
    let initial = session.game.as_ref().unwrap().clone();
    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("Plasma Rifle target")
      .id();
    let command = Command::AttackRanged(target_position);
    let mut direct = initial.clone();
    let expected_events = direct
      .step(command)
      .expect("direct Plasma Rifle volley command");
    let (events, observation, outcome) = session.step(command).expect("MCP Plasma Rifle volley");

    assert_eq!(events, expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(observation, direct.observe_player());
    assert_eq!(outcome, None);
    assert_eq!(
      events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      6
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      0
    );

    let replay = session.export_replay().expect("MCP replay export");
    let (replayed, replay_events) =
      ReplayEngine::run(replay).expect("MCP Plasma Rifle volley replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(ReplayEngine::verify_determinism(replay).expect("replay determinism"));
  }

  #[test]
  fn plasma_rifle_chainfire_mcp_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(3, 1);
    let player_config = drl_protocol::PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![drl_protocol::ItemSpawnKind::AmmoCells(4)],
      equipped_weapon: Some(drl_protocol::ItemSpawnKind::PlasmaRifle),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(2_448, 8, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      target_position,
      "Static Target",
      500,
      0,
      (1, 7),
    ));

    let mut session = McpSession::new();
    session
      .load_replay(setup_replay)
      .expect("load Plasma Rifle chainfire replay setup");
    let initial = session.game.as_ref().unwrap().clone();
    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("Plasma Rifle chainfire target")
      .id();
    let command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&initial.observe_player())
        .iter()
        .any(|action| action.command == command)
    );
    let mut direct = initial.clone();
    let mut expected_events = direct
      .step(command)
      .expect("direct Plasma Rifle chainfire command");
    let (events, observation, outcome) = session
      .step(command)
      .expect("MCP Plasma Rifle chainfire command");

    assert_eq!(events, expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(observation, direct.observe_player());
    assert_eq!(outcome, None);
    assert_eq!(
      events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      4
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      2
    );

    let reload_command = Command::Reload;
    let reload_expected_events = direct
      .step(reload_command)
      .expect("direct Plasma Rifle chainfire reload");
    let (reload_events, reload_observation, reload_outcome) = session
      .step(reload_command)
      .expect("MCP Plasma Rifle chainfire reload");
    assert_eq!(reload_events, reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(reload_observation, direct.observe_player());
    assert_eq!(reload_outcome, None);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      6
    );
    expected_events.extend(reload_events);

    let second_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == second_command)
    );
    let second_expected_events = direct
      .step(second_command)
      .expect("direct second Plasma Rifle chainfire command");
    let (second_events, second_observation, second_outcome) = session
      .step(second_command)
      .expect("MCP second Plasma Rifle chainfire command");
    assert_eq!(second_events, second_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(second_observation, direct.observe_player());
    assert_eq!(second_outcome, None);
    assert_eq!(
      second_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      6
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      0
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      2
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );
    expected_events.extend(second_events);

    let replay = session.export_replay().expect("MCP replay export");
    let (replayed, replay_events) =
      ReplayEngine::run(replay).expect("MCP Plasma Rifle chainfire replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(ReplayEngine::verify_determinism(replay).expect("replay determinism"));
  }

  #[test]
  fn laser_rifle_chainfire_mcp_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(3, 1);
    let player_config = drl_protocol::PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![drl_protocol::ItemSpawnKind::AmmoCells(7)],
      equipped_weapon: Some(drl_protocol::ItemSpawnKind::LaserRifle),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(2_548, 8, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      target_position,
      "Static Target",
      500,
      0,
      (1, 7),
    ));

    let mut session = McpSession::new();
    session
      .load_replay(setup_replay)
      .expect("load Laser Rifle chainfire replay setup");
    let initial = session.game.as_ref().unwrap().clone();
    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("Laser Rifle chainfire target")
      .id();
    let command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&initial.observe_player())
        .iter()
        .any(|action| action.command == command)
    );
    let mut direct = initial.clone();
    let mut expected_events = direct
      .step(command)
      .expect("direct Laser Rifle chainfire command");
    let (events, observation, outcome) = session
      .step(command)
      .expect("MCP Laser Rifle chainfire command");

    assert_eq!(events, expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(observation, direct.observe_player());
    assert_eq!(outcome, None);
    assert_eq!(
      events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      4
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      36
    );

    let second_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == second_command)
    );
    let second_expected_events = direct
      .step(second_command)
      .expect("direct second Laser Rifle chainfire command");
    let (second_events, second_observation, second_outcome) = session
      .step(second_command)
      .expect("MCP second Laser Rifle chainfire command");
    assert_eq!(second_events, second_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(second_observation, direct.observe_player());
    assert_eq!(second_outcome, None);
    assert_eq!(
      second_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      5
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      31
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      2
    );
    expected_events.extend(second_events);

    let third_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == third_command)
    );
    let third_expected_events = direct
      .step(third_command)
      .expect("direct third Laser Rifle chainfire command");
    let (third_events, third_observation, third_outcome) = session
      .step(third_command)
      .expect("MCP third Laser Rifle chainfire command");
    assert_eq!(third_events, third_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(third_observation, direct.observe_player());
    assert_eq!(third_outcome, None);
    assert_eq!(
      third_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      24
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      3
    );
    expected_events.extend(third_events);

    let fourth_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == fourth_command)
    );
    let fourth_expected_events = direct
      .step(fourth_command)
      .expect("direct fourth Laser Rifle chainfire command");
    let (fourth_events, fourth_observation, fourth_outcome) = session
      .step(fourth_command)
      .expect("MCP fourth Laser Rifle chainfire command");
    assert_eq!(fourth_events, fourth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fourth_observation, direct.observe_player());
    assert_eq!(fourth_outcome, None);
    assert_eq!(
      fourth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      17
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      4
    );
    expected_events.extend(fourth_events);

    let fifth_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == fifth_command)
    );
    let fifth_expected_events = direct
      .step(fifth_command)
      .expect("direct fifth Laser Rifle chainfire command");
    let (fifth_events, fifth_observation, fifth_outcome) = session
      .step(fifth_command)
      .expect("MCP fifth Laser Rifle chainfire command");
    assert_eq!(fifth_events, fifth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fifth_observation, direct.observe_player());
    assert_eq!(fifth_outcome, None);
    assert_eq!(
      fifth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      10
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      5
    );
    expected_events.extend(fifth_events);

    let sixth_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == sixth_command)
    );
    let sixth_expected_events = direct
      .step(sixth_command)
      .expect("direct sixth Laser Rifle chainfire command");
    let (sixth_events, sixth_observation, sixth_outcome) = session
      .step(sixth_command)
      .expect("MCP sixth Laser Rifle chainfire command");
    assert_eq!(sixth_events, sixth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(sixth_observation, direct.observe_player());
    assert_eq!(sixth_outcome, None);
    assert_eq!(
      sixth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      3
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      6
    );
    let reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct Laser Rifle reload");
    let (reload_events, reload_observation, reload_outcome) = session
      .step(Command::Reload)
      .expect("MCP Laser Rifle reload");
    assert_eq!(reload_events, reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(reload_observation, direct.observe_player());
    assert_eq!(reload_outcome, None);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      10
    );
    expected_events.extend(sixth_events);
    expected_events.extend(reload_events);
    let seventh_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == seventh_command)
    );
    let seventh_expected_events = direct
      .step(seventh_command)
      .expect("direct seventh Laser Rifle chainfire command");
    let (seventh_events, seventh_observation, seventh_outcome) = session
      .step(seventh_command)
      .expect("MCP seventh Laser Rifle chainfire command");
    assert_eq!(seventh_events, seventh_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(seventh_observation, direct.observe_player());
    assert_eq!(seventh_outcome, None);
    assert_eq!(
      seventh_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      3
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      7
    );
    expected_events.extend(seventh_events);
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );

    let replay = session.export_replay().expect("MCP replay export");
    let (replayed, replay_events) =
      ReplayEngine::run(replay).expect("MCP Laser Rifle chainfire replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(ReplayEngine::verify_determinism(replay).expect("replay determinism"));
  }

  #[test]
  fn bfg10k_chainfire_mcp_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(7, 1);
    let player_config = drl_protocol::PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![drl_protocol::ItemSpawnKind::AmmoCells(675)],
      equipped_weapon: Some(drl_protocol::ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(2_706, 8, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      target_position,
      "Static Target",
      10_000,
      0,
      (1, 7),
    ));

    let mut session = McpSession::new();
    session
      .load_replay(setup_replay)
      .expect("load BFG 10K chainfire replay setup");
    let initial = session.game.as_ref().unwrap().clone();
    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("BFG 10K chainfire target")
      .id();
    let command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&initial.observe_player())
        .iter()
        .any(|action| action.command == command)
    );
    let mut direct = initial.clone();
    let mut expected_events = direct
      .step(command)
      .expect("direct BFG 10K chainfire command");
    let (events, observation, outcome) = session
      .step(command)
      .expect("MCP BFG 10K chainfire command");

    assert_eq!(events, expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(observation, direct.observe_player());
    assert_eq!(outcome, None);
    assert_eq!(
      events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      4
    );
    assert_eq!(
      events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      4
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      30
    );

    let second_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive")
      .position();
    let second_command = Command::AttackRangedChainfire(second_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == second_command)
    );
    let second_expected_events = direct
      .step(second_command)
      .expect("direct second BFG 10K chainfire command");
    let (second_events, second_observation, second_outcome) = session
      .step(second_command)
      .expect("MCP second BFG 10K chainfire command");
    assert_eq!(second_events, second_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(second_observation, direct.observe_player());
    assert_eq!(second_outcome, None);
    assert_eq!(
      second_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      5
    );
    assert_eq!(
      second_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      5
    );
    expected_events.extend(second_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      5
    );

    let reload_command = Command::Reload;
    let reload_expected_events = direct
      .step(reload_command)
      .expect("direct BFG 10K chainfire reload");
    let (reload_events, reload_observation, reload_outcome) = session
      .step(reload_command)
      .expect("MCP BFG 10K chainfire reload");
    assert_eq!(reload_events, reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(reload_observation, direct.observe_player());
    assert_eq!(reload_outcome, None);
    expected_events.extend(reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let third_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive reload")
      .position();
    let third_command = Command::AttackRangedChainfire(third_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == third_command)
    );
    let third_expected_events = direct
      .step(third_command)
      .expect("direct third BFG 10K chainfire command");
    let (third_events, third_observation, third_outcome) = session
      .step(third_command)
      .expect("MCP third BFG 10K chainfire command");
    assert_eq!(third_events, third_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(third_observation, direct.observe_player());
    assert_eq!(third_outcome, None);
    assert_eq!(
      third_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      third_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(third_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      3
    );

    let second_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct second BFG 10K chainfire reload");
    let (second_reload_events, second_reload_observation, second_reload_outcome) = session
      .step(Command::Reload)
      .expect("MCP second BFG 10K chainfire reload");
    assert_eq!(second_reload_events, second_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(second_reload_observation, direct.observe_player());
    assert_eq!(second_reload_outcome, None);
    expected_events.extend(second_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let fourth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive second reload")
      .position();
    let fourth_command = Command::AttackRangedChainfire(fourth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == fourth_command)
    );
    let fourth_expected_events = direct
      .step(fourth_command)
      .expect("direct fourth BFG 10K chainfire command");
    let (fourth_events, fourth_observation, fourth_outcome) = session
      .step(fourth_command)
      .expect("MCP fourth BFG 10K chainfire command");
    assert_eq!(fourth_events, fourth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fourth_observation, direct.observe_player());
    assert_eq!(fourth_outcome, None);
    assert_eq!(
      fourth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      fourth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(fourth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      4
    );
    let third_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct third BFG 10K chainfire reload");
    let (third_reload_events, third_reload_observation, third_reload_outcome) = session
      .step(Command::Reload)
      .expect("MCP third BFG 10K chainfire reload");
    assert_eq!(third_reload_events, third_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(third_reload_observation, direct.observe_player());
    assert_eq!(third_reload_outcome, None);
    expected_events.extend(third_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let fifth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive third reload")
      .position();
    let fifth_command = Command::AttackRangedChainfire(fifth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == fifth_command)
    );
    let fifth_expected_events = direct
      .step(fifth_command)
      .expect("direct fifth BFG 10K chainfire command");
    let (fifth_events, fifth_observation, fifth_outcome) = session
      .step(fifth_command)
      .expect("MCP fifth BFG 10K chainfire command");
    assert_eq!(fifth_events, fifth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fifth_observation, direct.observe_player());
    assert_eq!(fifth_outcome, None);
    assert_eq!(
      fifth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      fifth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(fifth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      5
    );

    let fourth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct fourth BFG 10K chainfire reload");
    let (fourth_reload_events, fourth_reload_observation, fourth_reload_outcome) = session
      .step(Command::Reload)
      .expect("MCP fourth BFG 10K chainfire reload");
    assert_eq!(fourth_reload_events, fourth_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fourth_reload_observation, direct.observe_player());
    assert_eq!(fourth_reload_outcome, None);
    expected_events.extend(fourth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let sixth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive fourth reload")
      .position();
    let sixth_command = Command::AttackRangedChainfire(sixth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == sixth_command)
    );
    let sixth_expected_events = direct
      .step(sixth_command)
      .expect("direct sixth BFG 10K chainfire command");
    let (sixth_events, sixth_observation, sixth_outcome) = session
      .step(sixth_command)
      .expect("MCP sixth BFG 10K chainfire command");
    assert_eq!(sixth_events, sixth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(sixth_observation, direct.observe_player());
    assert_eq!(sixth_outcome, None);
    assert_eq!(
      sixth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      sixth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(sixth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      6
    );

    let fifth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct fifth BFG 10K chainfire reload");
    let (fifth_reload_events, fifth_reload_observation, fifth_reload_outcome) = session
      .step(Command::Reload)
      .expect("MCP fifth BFG 10K chainfire reload");
    assert_eq!(fifth_reload_events, fifth_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fifth_reload_observation, direct.observe_player());
    assert_eq!(fifth_reload_outcome, None);
    expected_events.extend(fifth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let seventh_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive fifth reload")
      .position();
    let seventh_command = Command::AttackRangedChainfire(seventh_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == seventh_command)
    );
    let seventh_expected_events = direct
      .step(seventh_command)
      .expect("direct seventh BFG 10K chainfire command");
    let (seventh_events, seventh_observation, seventh_outcome) = session
      .step(seventh_command)
      .expect("MCP seventh BFG 10K chainfire command");
    assert_eq!(seventh_events, seventh_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(seventh_observation, direct.observe_player());
    assert_eq!(seventh_outcome, None);
    assert_eq!(
      seventh_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      seventh_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(seventh_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      7
    );

    let sixth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct sixth BFG 10K chainfire reload");
    let (sixth_reload_events, sixth_reload_observation, sixth_reload_outcome) = session
      .step(Command::Reload)
      .expect("MCP sixth BFG 10K chainfire reload");
    assert_eq!(sixth_reload_events, sixth_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(sixth_reload_observation, direct.observe_player());
    assert_eq!(sixth_reload_outcome, None);
    expected_events.extend(sixth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let eighth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive sixth reload")
      .position();
    let eighth_command = Command::AttackRangedChainfire(eighth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == eighth_command)
    );
    let eighth_expected_events = direct
      .step(eighth_command)
      .expect("direct eighth BFG 10K chainfire command");
    let (eighth_events, eighth_observation, eighth_outcome) = session
      .step(eighth_command)
      .expect("MCP eighth BFG 10K chainfire command");
    assert_eq!(eighth_events, eighth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(eighth_observation, direct.observe_player());
    assert_eq!(eighth_outcome, None);
    assert_eq!(
      eighth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      eighth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(eighth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      8
    );

    let seventh_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct seventh BFG 10K chainfire reload");
    let (seventh_reload_events, seventh_reload_observation, seventh_reload_outcome) = session
      .step(Command::Reload)
      .expect("MCP seventh BFG 10K chainfire reload");
    assert_eq!(seventh_reload_events, seventh_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(seventh_reload_observation, direct.observe_player());
    assert_eq!(seventh_reload_outcome, None);
    expected_events.extend(seventh_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let ninth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive seventh reload")
      .position();
    let ninth_command = Command::AttackRangedChainfire(ninth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == ninth_command)
    );
    let ninth_expected_events = direct
      .step(ninth_command)
      .expect("direct ninth BFG 10K chainfire command");
    let (ninth_events, ninth_observation, ninth_outcome) = session
      .step(ninth_command)
      .expect("MCP ninth BFG 10K chainfire command");
    assert_eq!(ninth_events, ninth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(ninth_observation, direct.observe_player());
    assert_eq!(ninth_outcome, None);
    assert_eq!(
      ninth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      ninth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(ninth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      9
    );

    let ninth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct ninth BFG 10K chainfire reload");
    let (ninth_reload_events, ninth_reload_observation, ninth_reload_outcome) = session
      .step(Command::Reload)
      .expect("MCP ninth BFG 10K chainfire reload");
    assert_eq!(ninth_reload_events, ninth_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(ninth_reload_observation, direct.observe_player());
    assert_eq!(ninth_reload_outcome, None);
    expected_events.extend(ninth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let tenth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive ninth reload")
      .position();
    let tenth_command = Command::AttackRangedChainfire(tenth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == tenth_command)
    );
    let tenth_expected_events = direct
      .step(tenth_command)
      .expect("direct tenth BFG 10K chainfire command");
    let (tenth_events, tenth_observation, tenth_outcome) = session
      .step(tenth_command)
      .expect("MCP tenth BFG 10K chainfire command");
    assert_eq!(tenth_events, tenth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(tenth_observation, direct.observe_player());
    assert_eq!(tenth_outcome, None);
    assert_eq!(
      tenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      tenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(tenth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      10
    );

    let tenth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct tenth BFG 10K chainfire reload");
    let (tenth_reload_events, tenth_reload_observation, tenth_reload_outcome) = session
      .step(Command::Reload)
      .expect("MCP tenth BFG 10K chainfire reload");
    assert_eq!(tenth_reload_events, tenth_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(tenth_reload_observation, direct.observe_player());
    assert_eq!(tenth_reload_outcome, None);
    expected_events.extend(tenth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let eleventh_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive tenth reload")
      .position();
    let eleventh_command = Command::AttackRangedChainfire(eleventh_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == eleventh_command)
    );
    let eleventh_expected_events = direct
      .step(eleventh_command)
      .expect("direct eleventh BFG 10K chainfire command");
    let (eleventh_events, eleventh_observation, eleventh_outcome) = session
      .step(eleventh_command)
      .expect("MCP eleventh BFG 10K chainfire command");
    assert_eq!(eleventh_events, eleventh_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(eleventh_observation, direct.observe_player());
    assert_eq!(eleventh_outcome, None);
    assert_eq!(
      eleventh_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      eleventh_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(eleventh_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      11
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );

    let twelfth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct twelfth BFG 10K chainfire reload");
    let (twelfth_reload_events, twelfth_reload_observation, twelfth_reload_outcome) = session
      .step(Command::Reload)
      .expect("MCP twelfth BFG 10K chainfire reload");
    assert_eq!(twelfth_reload_events, twelfth_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(twelfth_reload_observation, direct.observe_player());
    assert_eq!(twelfth_reload_outcome, None);
    expected_events.extend(twelfth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let twelfth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive eleventh reload")
      .position();
    let twelfth_command = Command::AttackRangedChainfire(twelfth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == twelfth_command)
    );
    let twelfth_expected_events = direct
      .step(twelfth_command)
      .expect("direct twelfth BFG 10K chainfire command");
    let (twelfth_events, twelfth_observation, twelfth_outcome) = session
      .step(twelfth_command)
      .expect("MCP twelfth BFG 10K chainfire command");
    assert_eq!(twelfth_events, twelfth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(twelfth_observation, direct.observe_player());
    assert_eq!(twelfth_outcome, None);
    assert_eq!(
      twelfth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      twelfth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(twelfth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      12
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );

    let thirteenth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct thirteenth BFG 10K chainfire reload");
    let (thirteenth_reload_events, thirteenth_reload_observation, thirteenth_reload_outcome) =
      session
        .step(Command::Reload)
        .expect("MCP thirteenth BFG 10K chainfire reload");
    assert_eq!(thirteenth_reload_events, thirteenth_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(thirteenth_reload_observation, direct.observe_player());
    assert_eq!(thirteenth_reload_outcome, None);
    expected_events.extend(thirteenth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let thirteenth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive twelfth reload")
      .position();
    let thirteenth_command = Command::AttackRangedChainfire(thirteenth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == thirteenth_command)
    );
    let thirteenth_expected_events = direct
      .step(thirteenth_command)
      .expect("direct thirteenth BFG 10K chainfire command");
    let (thirteenth_events, thirteenth_observation, thirteenth_outcome) = session
      .step(thirteenth_command)
      .expect("MCP thirteenth BFG 10K chainfire command");
    assert_eq!(thirteenth_events, thirteenth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(thirteenth_observation, direct.observe_player());
    assert_eq!(thirteenth_outcome, None);
    assert_eq!(
      thirteenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      thirteenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(thirteenth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      13
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );

    let fourteenth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct fourteenth BFG 10K chainfire reload");
    let (fourteenth_reload_events, fourteenth_reload_observation, fourteenth_reload_outcome) =
      session
        .step(Command::Reload)
        .expect("MCP fourteenth BFG 10K chainfire reload");
    assert_eq!(fourteenth_reload_events, fourteenth_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fourteenth_reload_observation, direct.observe_player());
    assert_eq!(fourteenth_reload_outcome, None);
    expected_events.extend(fourteenth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let fourteenth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive thirteenth reload")
      .position();
    let fourteenth_command = Command::AttackRangedChainfire(fourteenth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == fourteenth_command)
    );
    let fourteenth_expected_events = direct
      .step(fourteenth_command)
      .expect("direct fourteenth BFG 10K chainfire command");
    let (fourteenth_events, fourteenth_observation, fourteenth_outcome) = session
      .step(fourteenth_command)
      .expect("MCP fourteenth BFG 10K chainfire command");
    assert_eq!(fourteenth_events, fourteenth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fourteenth_observation, direct.observe_player());
    assert_eq!(fourteenth_outcome, None);
    assert_eq!(
      fourteenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      fourteenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(fourteenth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      14
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );

    let fifteenth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct fifteenth BFG 10K chainfire reload");
    let (fifteenth_reload_events, fifteenth_reload_observation, fifteenth_reload_outcome) = session
      .step(Command::Reload)
      .expect("MCP fifteenth BFG 10K chainfire reload");
    assert_eq!(fifteenth_reload_events, fifteenth_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fifteenth_reload_observation, direct.observe_player());
    assert_eq!(fifteenth_reload_outcome, None);
    expected_events.extend(fifteenth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let fifteenth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive fourteenth reload")
      .position();
    let fifteenth_command = Command::AttackRangedChainfire(fifteenth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == fifteenth_command)
    );
    let fifteenth_expected_events = direct
      .step(fifteenth_command)
      .expect("direct fifteenth BFG 10K chainfire command");
    let (fifteenth_events, fifteenth_observation, fifteenth_outcome) = session
      .step(fifteenth_command)
      .expect("MCP fifteenth BFG 10K chainfire command");
    assert_eq!(fifteenth_events, fifteenth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fifteenth_observation, direct.observe_player());
    assert_eq!(fifteenth_outcome, None);
    assert_eq!(
      fifteenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      fifteenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(fifteenth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      15
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );

    let sixteenth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct sixteenth BFG 10K chainfire reload");
    let (sixteenth_reload_events, sixteenth_reload_observation, sixteenth_reload_outcome) = session
      .step(Command::Reload)
      .expect("MCP sixteenth BFG 10K chainfire reload");
    assert_eq!(sixteenth_reload_events, sixteenth_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(sixteenth_reload_observation, direct.observe_player());
    assert_eq!(sixteenth_reload_outcome, None);
    expected_events.extend(sixteenth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let sixteenth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive fifteenth reload")
      .position();
    let sixteenth_command = Command::AttackRangedChainfire(sixteenth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == sixteenth_command)
    );
    let sixteenth_expected_events = direct
      .step(sixteenth_command)
      .expect("direct sixteenth BFG 10K chainfire command");
    let (sixteenth_events, sixteenth_observation, sixteenth_outcome) = session
      .step(sixteenth_command)
      .expect("MCP sixteenth BFG 10K chainfire command");
    assert_eq!(sixteenth_events, sixteenth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(sixteenth_observation, direct.observe_player());
    assert_eq!(sixteenth_outcome, None);
    assert_eq!(
      sixteenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      sixteenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(sixteenth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      16
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );

    let seventeenth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct seventeenth BFG 10K chainfire reload");
    let (seventeenth_reload_events, seventeenth_reload_observation, seventeenth_reload_outcome) =
      session
        .step(Command::Reload)
        .expect("MCP seventeenth BFG 10K chainfire reload");
    assert_eq!(
      seventeenth_reload_events,
      seventeenth_reload_expected_events
    );
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(seventeenth_reload_observation, direct.observe_player());
    assert_eq!(seventeenth_reload_outcome, None);
    expected_events.extend(seventeenth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let seventeenth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive sixteenth reload")
      .position();
    let seventeenth_command = Command::AttackRangedChainfire(seventeenth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == seventeenth_command)
    );
    let seventeenth_expected_events = direct
      .step(seventeenth_command)
      .expect("direct seventeenth BFG 10K chainfire command");
    let (seventeenth_events, seventeenth_observation, seventeenth_outcome) = session
      .step(seventeenth_command)
      .expect("MCP seventeenth BFG 10K chainfire command");
    assert_eq!(seventeenth_events, seventeenth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(seventeenth_observation, direct.observe_player());
    assert_eq!(seventeenth_outcome, None);
    assert_eq!(
      seventeenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      seventeenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(seventeenth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      17
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );

    let eighteenth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct eighteenth BFG 10K chainfire reload");
    let (eighteenth_reload_events, eighteenth_reload_observation, eighteenth_reload_outcome) =
      session
        .step(Command::Reload)
        .expect("MCP eighteenth BFG 10K chainfire reload");
    assert_eq!(eighteenth_reload_events, eighteenth_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(eighteenth_reload_observation, direct.observe_player());
    assert_eq!(eighteenth_reload_outcome, None);
    expected_events.extend(eighteenth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let eighteenth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive seventeenth reload")
      .position();
    let eighteenth_command = Command::AttackRangedChainfire(eighteenth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == eighteenth_command)
    );
    let eighteenth_expected_events = direct
      .step(eighteenth_command)
      .expect("direct eighteenth BFG 10K chainfire command");
    let (eighteenth_events, eighteenth_observation, eighteenth_outcome) = session
      .step(eighteenth_command)
      .expect("MCP eighteenth BFG 10K chainfire command");
    assert_eq!(eighteenth_events, eighteenth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(eighteenth_observation, direct.observe_player());
    assert_eq!(eighteenth_outcome, None);
    assert_eq!(
      eighteenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      eighteenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(eighteenth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      18
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );

    let nineteenth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct nineteenth BFG 10K chainfire reload");
    let (nineteenth_reload_events, nineteenth_reload_observation, nineteenth_reload_outcome) =
      session
        .step(Command::Reload)
        .expect("MCP nineteenth BFG 10K chainfire reload");
    assert_eq!(nineteenth_reload_events, nineteenth_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(nineteenth_reload_observation, direct.observe_player());
    assert_eq!(nineteenth_reload_outcome, None);
    expected_events.extend(nineteenth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let nineteenth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive eighteenth reload")
      .position();
    let nineteenth_command = Command::AttackRangedChainfire(nineteenth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == nineteenth_command)
    );
    let nineteenth_expected_events = direct
      .step(nineteenth_command)
      .expect("direct nineteenth BFG 10K chainfire command");
    let (nineteenth_events, nineteenth_observation, nineteenth_outcome) = session
      .step(nineteenth_command)
      .expect("MCP nineteenth BFG 10K chainfire command");
    assert_eq!(nineteenth_events, nineteenth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(nineteenth_observation, direct.observe_player());
    assert_eq!(nineteenth_outcome, None);
    assert_eq!(
      nineteenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      nineteenth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(nineteenth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      19
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );

    let twentieth_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct twentieth BFG 10K chainfire reload");
    let (twentieth_reload_events, twentieth_reload_observation, twentieth_reload_outcome) = session
      .step(Command::Reload)
      .expect("MCP twentieth BFG 10K chainfire reload");
    assert_eq!(twentieth_reload_events, twentieth_reload_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(twentieth_reload_observation, direct.observe_player());
    assert_eq!(twentieth_reload_outcome, None);
    expected_events.extend(twentieth_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let twentieth_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive nineteenth reload")
      .position();
    let twentieth_command = Command::AttackRangedChainfire(twentieth_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == twentieth_command)
    );
    let twentieth_expected_events = direct
      .step(twentieth_command)
      .expect("direct twentieth BFG 10K chainfire command");
    let (twentieth_events, twentieth_observation, twentieth_outcome) = session
      .step(twentieth_command)
      .expect("MCP twentieth BFG 10K chainfire command");
    assert_eq!(twentieth_events, twentieth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(twentieth_observation, direct.observe_player());
    assert_eq!(twentieth_outcome, None);
    assert_eq!(
      twentieth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      twentieth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(twentieth_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      20
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );

    let twenty_first_reload_expected_events = direct
      .step(Command::Reload)
      .expect("direct twenty-first BFG 10K chainfire reload");
    let (twenty_first_reload_events, twenty_first_reload_observation, twenty_first_reload_outcome) =
      session
        .step(Command::Reload)
        .expect("MCP twenty-first BFG 10K chainfire reload");
    assert_eq!(
      twenty_first_reload_events,
      twenty_first_reload_expected_events
    );
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(twenty_first_reload_observation, direct.observe_player());
    assert_eq!(twenty_first_reload_outcome, None);
    expected_events.extend(twenty_first_reload_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      50
    );

    let twenty_first_target = direct
      .world()
      .get_actor(target_id)
      .expect("BFG 10K chainfire target should survive twentieth reload")
      .position();
    let twenty_first_command = Command::AttackRangedChainfire(twenty_first_target);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == twenty_first_command)
    );
    let twenty_first_expected_events = direct
      .step(twenty_first_command)
      .expect("direct twenty-first BFG 10K chainfire command");
    let (twenty_first_events, twenty_first_observation, twenty_first_outcome) = session
      .step(twenty_first_command)
      .expect("MCP twenty-first BFG 10K chainfire command");
    assert_eq!(twenty_first_events, twenty_first_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(twenty_first_observation, direct.observe_player());
    assert_eq!(twenty_first_outcome, None);
    assert_eq!(
      twenty_first_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            outcome: drl_protocol::AttackOutcome::Hit { .. },
            is_ranged: true,
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    assert_eq!(
      twenty_first_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::Bfg10kExplosionScheduled {
            entity_id,
            target_id: event_target,
            delay: 25,
            radius: 2,
            knockback: 16,
          } if *entity_id == player_id && *event_target == target_id
        ))
        .count(),
      7
    );
    expected_events.extend(twenty_first_events);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      15
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      21
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );

    let replay = session.export_replay().expect("MCP replay export");
    let (replayed, replay_events) =
      ReplayEngine::run(replay).expect("MCP BFG 10K chainfire replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(ReplayEngine::verify_determinism(replay).expect("replay determinism"));
  }

  #[test]
  fn nuclear_plasma_chainfire_mcp_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(3, 1);
    let player_config = drl_protocol::PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![drl_protocol::ItemSpawnKind::AmmoCells(10)],
      equipped_weapon: Some(drl_protocol::ItemSpawnKind::NuclearPlasmaRifle),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(2_648, 8, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      target_position,
      "Static Target",
      500,
      0,
      (1, 7),
    ));

    let mut session = McpSession::new();
    session
      .load_replay(setup_replay)
      .expect("load Nuclear Plasma chainfire replay setup");
    let initial = session.game.as_ref().unwrap().clone();
    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("Nuclear Plasma chainfire target")
      .id();
    let command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&initial.observe_player())
        .iter()
        .any(|action| action.command == command)
    );
    let mut direct = initial.clone();
    let mut expected_events = direct
      .step(command)
      .expect("direct Nuclear Plasma chainfire command");
    let (events, observation, outcome) = session
      .step(command)
      .expect("MCP Nuclear Plasma chainfire command");

    assert_eq!(events, expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(observation, direct.observe_player());
    assert_eq!(outcome, None);
    assert_eq!(
      events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      4
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      20
    );

    let second_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == second_command)
    );
    let second_expected_events = direct
      .step(second_command)
      .expect("direct second Nuclear Plasma chainfire command");
    let (second_events, second_observation, second_outcome) = session
      .step(second_command)
      .expect("MCP second Nuclear Plasma chainfire command");
    assert_eq!(second_events, second_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(second_observation, direct.observe_player());
    assert_eq!(second_outcome, None);
    assert_eq!(
      second_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      6
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      14
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      2
    );
    expected_events.extend(second_events);

    let third_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == third_command)
    );
    let third_expected_events = direct
      .step(third_command)
      .expect("direct third Nuclear Plasma chainfire command");
    let (third_events, third_observation, third_outcome) = session
      .step(third_command)
      .expect("MCP third Nuclear Plasma chainfire command");
    assert_eq!(third_events, third_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(third_observation, direct.observe_player());
    assert_eq!(third_outcome, None);
    assert_eq!(
      third_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      9
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      5
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      3
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );
    expected_events.extend(third_events);

    for _ in 0..47 {
      let wait_expected_events = direct
        .step(Command::Wait)
        .expect("direct Nuclear Plasma recharge wait");
      let (wait_events, wait_observation, wait_outcome) = session
        .step(Command::Wait)
        .expect("MCP Nuclear Plasma recharge wait");
      assert_eq!(wait_events, wait_expected_events);
      assert_eq!(session.game.as_ref().unwrap(), &direct);
      assert_eq!(wait_observation, direct.observe_player());
      assert_eq!(wait_outcome, None);
      expected_events.extend(wait_events);
    }
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      9
    );

    let fourth_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == fourth_command)
    );
    let fourth_expected_events = direct
      .step(fourth_command)
      .expect("direct fourth Nuclear Plasma chainfire command");
    let (fourth_events, fourth_observation, fourth_outcome) = session
      .step(fourth_command)
      .expect("MCP fourth Nuclear Plasma chainfire command");
    assert_eq!(fourth_events, fourth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fourth_observation, direct.observe_player());
    assert_eq!(fourth_outcome, None);
    assert_eq!(
      fourth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      9
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      0
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      4
    );
    expected_events.extend(fourth_events);

    for _ in 0..57 {
      let wait_expected_events = direct
        .step(Command::Wait)
        .expect("direct Nuclear Plasma fifth-level recharge wait");
      let (wait_events, wait_observation, wait_outcome) = session
        .step(Command::Wait)
        .expect("MCP Nuclear Plasma fifth-level recharge wait");
      assert_eq!(wait_events, wait_expected_events);
      assert_eq!(session.game.as_ref().unwrap(), &direct);
      assert_eq!(wait_observation, direct.observe_player());
      assert_eq!(wait_outcome, None);
      expected_events.extend(wait_events);
    }
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      9
    );

    let fifth_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == fifth_command)
    );
    let fifth_expected_events = direct
      .step(fifth_command)
      .expect("direct fifth Nuclear Plasma chainfire command");
    let (fifth_events, fifth_observation, fifth_outcome) = session
      .step(fifth_command)
      .expect("MCP fifth Nuclear Plasma chainfire command");
    assert_eq!(fifth_events, fifth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(fifth_observation, direct.observe_player());
    assert_eq!(fifth_outcome, None);
    assert_eq!(
      fifth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      9
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      0
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      5
    );
    expected_events.extend(fifth_events);

    for _ in 0..57 {
      let wait_expected_events = direct
        .step(Command::Wait)
        .expect("direct Nuclear Plasma sixth-level recharge wait");
      let (wait_events, wait_observation, wait_outcome) = session
        .step(Command::Wait)
        .expect("MCP Nuclear Plasma sixth-level recharge wait");
      assert_eq!(wait_events, wait_expected_events);
      assert_eq!(session.game.as_ref().unwrap(), &direct);
      assert_eq!(wait_observation, direct.observe_player());
      assert_eq!(wait_outcome, None);
      expected_events.extend(wait_events);
    }
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      9
    );

    let sixth_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == sixth_command)
    );
    let sixth_expected_events = direct
      .step(sixth_command)
      .expect("direct sixth Nuclear Plasma chainfire command");
    let (sixth_events, sixth_observation, sixth_outcome) = session
      .step(sixth_command)
      .expect("MCP sixth Nuclear Plasma chainfire command");
    assert_eq!(sixth_events, sixth_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(sixth_observation, direct.observe_player());
    assert_eq!(sixth_outcome, None);
    assert_eq!(
      sixth_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      9
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      0
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      6
    );
    expected_events.extend(sixth_events);

    for _ in 0..57 {
      let wait_expected_events = direct
        .step(Command::Wait)
        .expect("direct Nuclear Plasma seventh-level recharge wait");
      let (wait_events, wait_observation, wait_outcome) = session
        .step(Command::Wait)
        .expect("MCP Nuclear Plasma seventh-level recharge wait");
      assert_eq!(wait_events, wait_expected_events);
      assert_eq!(session.game.as_ref().unwrap(), &direct);
      assert_eq!(wait_observation, direct.observe_player());
      assert_eq!(wait_outcome, None);
      expected_events.extend(wait_events);
    }
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      9
    );

    let seventh_command = Command::AttackRangedChainfire(target_position);
    assert!(
      compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.command == seventh_command)
    );
    let seventh_expected_events = direct
      .step(seventh_command)
      .expect("direct seventh Nuclear Plasma chainfire command");
    let (seventh_events, seventh_observation, seventh_outcome) = session
      .step(seventh_command)
      .expect("MCP seventh Nuclear Plasma chainfire command");
    assert_eq!(seventh_events, seventh_expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(seventh_observation, direct.observe_player());
    assert_eq!(seventh_outcome, None);
    assert_eq!(
      seventh_events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      9
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      0
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .chainfire_level,
      7
    );
    assert!(
      !compute_legal_actions(&direct.observe_player())
        .iter()
        .any(|action| action.action == "Chainfire")
    );
    expected_events.extend(seventh_events);

    let replay = session.export_replay().expect("MCP replay export");
    let (replayed, replay_events) =
      ReplayEngine::run(replay).expect("MCP Nuclear Plasma chainfire replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(ReplayEngine::verify_determinism(replay).expect("replay determinism"));
  }

  #[test]
  fn nuclear_plasma_volley_mcp_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(3, 1);
    let player_config = drl_protocol::PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![drl_protocol::ItemSpawnKind::AmmoCells(6)],
      equipped_weapon: Some(drl_protocol::ItemSpawnKind::NuclearPlasmaRifle),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(2_272, 8, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      target_position,
      "Static Target",
      1_000,
      100,
      (1, 7),
    ));

    let mut session = McpSession::new();
    session
      .load_replay(setup_replay)
      .expect("load Nuclear Plasma volley replay setup");
    let initial = session.game.as_ref().unwrap().clone();
    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("Nuclear Plasma target")
      .id();
    let command = Command::AttackRanged(target_position);
    assert!(
      session
        .legal_actions()
        .unwrap()
        .iter()
        .any(|action| action.action == "Fire" && action.command == command)
    );
    let mut direct = initial.clone();
    let expected_events = direct
      .step(command)
      .expect("direct Nuclear Plasma volley command");
    let (events, observation, outcome) = session.step(command).expect("MCP Nuclear Plasma volley");

    assert_eq!(events, expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(observation, direct.observe_player());
    assert_eq!(outcome, None);
    assert_eq!(
      events
        .iter()
        .filter(|event| matches!(
          event,
          GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      6
    );
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      18
    );

    let replay = session.export_replay().expect("MCP replay export");
    let (replayed, replay_events) =
      ReplayEngine::run(replay).expect("MCP Nuclear Plasma volley replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(ReplayEngine::verify_determinism(replay).expect("replay determinism"));
  }

  #[test]
  fn double_shotgun_dual_shot_mcp_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(7, 1);
    let player_config = drl_protocol::PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(drl_protocol::ItemSpawnKind::DoubleShotgun),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(1, 9, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      target_position,
      "Static Target",
      500,
      1,
      (9, 27),
    ));

    let mut session = McpSession::new();
    session
      .load_replay(setup_replay)
      .expect("load canonical Double Shotgun replay setup");
    let initial = session.game.as_ref().unwrap().clone();
    let player_id = initial.world().player_id().expect("player identity");
    let target_id = initial
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .expect("Double Shotgun target")
      .id();
    let command = Command::AttackRanged(target_position);
    let mut direct = initial.clone();
    let expected_events = direct
      .step(command)
      .expect("direct Double Shotgun dual-shot command");
    let (events, observation, outcome) = session.step(command).expect("MCP Double Shotgun shot");

    assert_eq!(events, expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(observation, direct.observe_player());
    assert_eq!(outcome, None);
    assert_double_shotgun_dual_shot_events(&events, player_id, target_id);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      0
    );

    let replay = session.export_replay().expect("MCP replay export");
    let (replayed, replay_events) = ReplayEngine::run(replay).expect("MCP dual-shot replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(ReplayEngine::verify_determinism(replay).expect("replay determinism"));
  }

  #[test]
  fn standard_bfg_shot_cost_vertical_mcp_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(5, 1);
    let player_config = drl_protocol::PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(drl_protocol::ItemSpawnKind::Bfg9000),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      target_position,
      "Static Target",
      500,
      1,
      (2, 4),
    ));
    let splash_target_position = Position::new(6, 1);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      splash_target_position,
      "Splash Target",
      500,
      1,
      (2, 4),
    ));
    setup_replay.record_item(drl_protocol::ItemSpawnSpec::new(
      target_position,
      drl_protocol::ItemSpawnKind::SmallMedPack,
    ));

    let mut session = McpSession::new();
    session
      .load_replay(setup_replay.clone())
      .expect("load canonical standard BFG replay setup");
    let initial = session.game.as_ref().unwrap().clone();
    let command = Command::AttackRanged(target_position);
    let mut direct = initial.clone();
    let expected_events = direct
      .step(command)
      .expect("direct standard BFG shot-cost command");
    let (events, observation, outcome) = session.step(command).expect("MCP standard BFG shot");

    let player_id = direct.world().player_id().unwrap();
    let target_id = direct
      .world()
      .actors()
      .values()
      .find(|actor| actor.name() == "Static Target")
      .unwrap()
      .id();
    let splash_target_id = direct
      .world()
      .actors()
      .values()
      .find(|actor| actor.name() == "Splash Target")
      .unwrap()
      .id();
    assert_standard_bfg_schedule_event(&events, player_id, target_id);
    assert_eq!(
      events
        .iter()
        .filter(|event| {
          matches!(
            event,
            GameEvent::DamageApplied {
              target_id: event_target,
              source: drl_protocol::DamageSource::Environment,
              damage_type: Some(drl_protocol::DamageType::Plasma),
              ..
            } if *event_target == splash_target_id
          )
        })
        .count(),
      1,
      "MCP standard BFG splash must damage the second actor exactly once"
    );
    let splash_damage_index = events
      .iter()
      .position(|event| {
        matches!(
          event,
          GameEvent::DamageApplied {
            target_id: event_target,
            source: drl_protocol::DamageSource::Environment,
            damage_type: Some(drl_protocol::DamageType::Plasma),
            ..
          } if *event_target == target_id
        )
      })
      .expect("MCP standard BFG center actor should receive splash damage");
    let destroyed_index = events
      .iter()
      .position(|event| {
        matches!(
          event,
          GameEvent::GroundItemDestroyed { position, .. } if *position == target_position
        )
      })
      .expect("MCP standard BFG should destroy the center ground item");
    assert!(splash_damage_index < destroyed_index);

    assert_eq!(events, expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(observation, direct.observe_player());
    assert_eq!(outcome, None);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      60
    );
    let attack_index = events
      .iter()
      .position(|event| matches!(event, GameEvent::AttackResolved { .. }))
      .expect("MCP standard BFG shot must resolve an attack");
    let cost_index = events
      .iter()
      .position(|event| matches!(event, GameEvent::ActionCostPaid { .. }))
      .expect("MCP standard BFG shot must pay an action cost");
    let turn_end_index = events
      .iter()
      .position(|event| matches!(event, GameEvent::TurnEnded { .. }))
      .expect("MCP standard BFG shot must end its turn");
    assert!(attack_index < cost_index);
    assert!(cost_index < turn_end_index);

    let replay = session.export_replay().expect("MCP replay export");
    let (replayed, replay_events) =
      ReplayEngine::run(replay).expect("MCP standard BFG shot replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(ReplayEngine::verify_determinism(replay).expect("replay determinism"));
  }

  #[test]
  fn nuclear_bfg_shot_cost_vertical_mcp_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(5, 1);
    let player_config = drl_protocol::PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(drl_protocol::ItemSpawnKind::NuclearBfg9000),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(0, 8, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      target_position,
      "Static Target",
      500,
      1,
      (2, 4),
    ));
    let splash_target_position = Position::new(6, 1);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      splash_target_position,
      "Splash Target",
      500,
      1,
      (2, 4),
    ));
    setup_replay.record_item(drl_protocol::ItemSpawnSpec::new(
      target_position,
      drl_protocol::ItemSpawnKind::SmallMedPack,
    ));

    let mut session = McpSession::new();
    session
      .load_replay(setup_replay.clone())
      .expect("load canonical Nuclear BFG replay setup");
    let initial = session.game.as_ref().unwrap().clone();
    let command = Command::AttackRanged(target_position);
    let mut direct = initial.clone();
    let expected_events = direct
      .step(command)
      .expect("direct Nuclear BFG shot-cost command");
    let (events, observation, outcome) = session.step(command).expect("MCP Nuclear BFG shot");

    let player_id = direct.world().player_id().unwrap();
    let target_id = direct
      .world()
      .actors()
      .values()
      .find(|actor| !actor.is_player())
      .unwrap()
      .id();
    let splash_target_id = direct
      .world()
      .actors()
      .values()
      .find(|actor| actor.name() == "Splash Target")
      .unwrap()
      .id();
    assert_nuclear_bfg_schedule_event(&events, player_id, target_id);
    assert_eq!(
      events
        .iter()
        .filter(|event| {
          matches!(
            event,
            GameEvent::DamageApplied {
              target_id: event_target,
              source: drl_protocol::DamageSource::Environment,
              damage_type: Some(drl_protocol::DamageType::Plasma),
              ..
            } if *event_target == splash_target_id
          )
        })
        .count(),
      1,
      "MCP Nuclear BFG splash must damage the second actor exactly once"
    );
    let splash_damage_index = events
      .iter()
      .position(|event| {
        matches!(
          event,
          GameEvent::DamageApplied {
            target_id: event_target,
            source: drl_protocol::DamageSource::Environment,
            damage_type: Some(drl_protocol::DamageType::Plasma),
            ..
          } if *event_target == target_id
        )
      })
      .expect("MCP Nuclear BFG center actor should receive splash damage");
    let destroyed_index = events
      .iter()
      .position(|event| {
        matches!(
          event,
          GameEvent::GroundItemDestroyed { position, .. } if *position == target_position
        )
      })
      .expect("MCP Nuclear BFG should destroy the center ground item");
    assert!(splash_damage_index < destroyed_index);

    assert_eq!(events, expected_events);
    assert_eq!(session.game.as_ref().unwrap(), &direct);
    assert_eq!(observation, direct.observe_player());
    assert_eq!(outcome, None);
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      0
    );
    let attack_index = events
      .iter()
      .position(|event| matches!(event, GameEvent::AttackResolved { .. }))
      .expect("MCP Nuclear BFG shot must resolve an attack");
    let cost_index = events
      .iter()
      .position(|event| matches!(event, GameEvent::ActionCostPaid { .. }))
      .expect("MCP Nuclear BFG shot must pay an action cost");
    let turn_end_index = events
      .iter()
      .position(|event| matches!(event, GameEvent::TurnEnded { .. }))
      .expect("MCP Nuclear BFG shot must end its turn");
    assert!(attack_index < cost_index);
    assert!(cost_index < turn_end_index);

    let replay = session.export_replay().expect("MCP replay export");
    let (replayed, replay_events) = ReplayEngine::run(replay).expect("MCP Nuclear BFG shot replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(ReplayEngine::verify_determinism(replay).expect("replay determinism"));
  }

  #[test]
  fn nuclear_bfg_recharge_vertical_mcp_boundary_matches_direct_core() {
    let player_position = Position::new(1, 1);
    let target_position = Position::new(2, 1);
    let player_config = drl_protocol::PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: Vec::new(),
      equipped_weapon: Some(drl_protocol::ItemSpawnKind::NuclearBfg9000),
      equipped_armor: None,
      equipped_armor_durability: None,
    };
    let mut setup_replay =
      ReplayLog::new(33, 8, 4, player_position).with_player_config(player_config);
    setup_replay.record_monster(drl_protocol::MonsterSpawnSpec::new(
      target_position,
      "Recharge Target",
      1_000,
      1,
      (0, 0),
    ));

    let mut session = McpSession::new();
    session
      .load_replay(setup_replay.clone())
      .expect("load canonical Nuclear BFG recharge replay setup");
    let initial = session.game.as_ref().unwrap().clone();
    let mut direct = initial.clone();
    let commands = [
      Command::AttackRanged(target_position),
      Command::Wait,
      Command::Wait,
      Command::Wait,
      Command::Wait,
    ];
    let mut expected_events = Vec::new();
    for (index, command) in commands.iter().copied().enumerate() {
      let direct_events = direct
        .step(command)
        .expect("direct Nuclear BFG recharge command");
      let (events, observation, outcome) = session
        .step(command)
        .expect("MCP Nuclear BFG recharge command");
      assert_eq!(events, direct_events);
      assert_eq!(session.game.as_ref().unwrap(), &direct);
      assert_eq!(observation, direct.observe_player());
      assert_eq!(outcome, None);
      if index < 4 {
        assert!(
          !events
            .iter()
            .any(|event| { matches!(event, GameEvent::WeaponRecharged { .. }) })
        );
      } else {
        assert!(events.iter().any(|event| {
          matches!(
            event,
            GameEvent::WeaponRecharged {
              ammo_recharged: 1,
              current_clip: 1,
              max_clip: 40,
              timer: 0,
              ..
            }
          )
        }));
      }
      expected_events.extend(events);
    }
    assert_eq!(
      direct
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .weapon_properties()
        .unwrap()
        .current_clip,
      1
    );

    let replay = session.export_replay().expect("MCP replay export");
    assert_eq!(replay.commands, commands);
    let (replayed, replay_events) =
      ReplayEngine::run(replay).expect("MCP Nuclear BFG recharge replay");
    assert_eq!(replayed, direct);
    assert_eq!(replay_events, expected_events);
    assert!(ReplayEngine::verify_determinism(replay).expect("replay determinism"));
  }

  #[test]
  fn test_legal_action_catalog_and_events_include_nuclear_plasma_overload() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n#######\n#@....#\n#######\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::nuclear_plasma_rifle(weapon_id),
      )
      .unwrap();

    let observation = session.get_observation().unwrap();
    let command = Command::AltReload {
      item_id: weapon_id,
      confirmed: true,
    };
    assert!(
      compute_legal_actions(&observation)
        .iter()
        .any(|action| action.action == "AltReload" && action.command == command)
    );

    let (events, _, _) = session.step(command).unwrap();
    assert!(events.iter().any(|event| matches!(
      event,
      GameEvent::NuclearWeaponOverloaded {
        entity_id,
        item_id,
        countdown: 100,
        score_count_remaining: -1_000,
      } if *entity_id == player_id && *item_id == weapon_id
    )));
    assert!(
      events
        .iter()
        .any(|event| matches!(event, GameEvent::NukeActivated { countdown: 100, .. }))
    );
    assert!(
      !session
        .legal_actions()
        .unwrap()
        .iter()
        .any(|action| action.command == command)
    );
  }

  #[test]
  fn test_legal_action_catalog_and_events_include_nuclear_bfg_overload() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n#######\n#@....#\n#######\n", None)
      .unwrap();
    let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
    let weapon_id = session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .allocate_item_id();
    session
      .game
      .as_mut()
      .unwrap()
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .equip(
        EquipmentSlot::Weapon,
        drl_core::item::Item::nuclear_bfg9000(weapon_id),
      )
      .unwrap();

    let observation = session.get_observation().unwrap();
    let command = Command::AltReload {
      item_id: weapon_id,
      confirmed: true,
    };
    assert!(
      compute_legal_actions(&observation)
        .iter()
        .any(|action| action.action == "AltReload" && action.command == command)
    );

    let (events, _, _) = session.step(command).unwrap();
    assert!(events.iter().any(|event| matches!(
      event,
      GameEvent::NuclearWeaponOverloaded {
        entity_id,
        item_id,
        countdown: 100,
        score_count_remaining: -1_000,
      } if *entity_id == player_id && *item_id == weapon_id
    )));
    assert!(
      events
        .iter()
        .any(|event| matches!(event, GameEvent::NukeActivated { countdown: 100, .. }))
    );
    assert!(
      !session
        .legal_actions()
        .unwrap()
        .iter()
        .any(|action| action.command == command)
    );
  }

  #[test]
  fn test_unadvertised_commands_are_rejected_without_mutation() {
    let mut session = McpSession::new();
    session
      .load_scenario("\n#####\n#@..#\n#####\n", None)
      .unwrap();

    let rejected = [
      Command::Move(Direction::North),
      Command::AttackMelee(Direction::East),
      Command::Drop(ItemId::new(999)),
      Command::Use(ItemId::new(999)),
      Command::Unequip(EquipmentSlot::Armor),
      Command::Descend,
    ];
    for command in rejected {
      let observation_before = session.get_observation().unwrap();
      let metrics_before = session.get_metrics().clone();
      let replay_before = session.export_replay().unwrap().clone();
      let events_before = session.recent_events().to_vec();

      assert_eq!(
        session.step(command).unwrap_err(),
        "Command is not currently advertised as legal"
      );
      assert_eq!(session.get_observation().unwrap(), observation_before);
      assert_eq!(session.get_metrics(), &metrics_before);
      assert_eq!(session.export_replay().unwrap(), &replay_before);
      assert_eq!(session.recent_events(), events_before.as_slice());
    }

    let (_, observation, _) = session.step(Command::Wait).unwrap();
    assert_eq!(observation.turn.count, 1);
    assert_eq!(session.export_replay().unwrap().commands.len(), 1);
  }

  #[test]
  fn if_noreload_reload_is_not_advertised_and_session_is_unchanged() {
    for (seed, kind) in [
      (1_763, drl_protocol::ItemSpawnKind::Blaster),
      (1_764, drl_protocol::ItemSpawnKind::NuclearPlasmaRifle),
      (1_765, drl_protocol::ItemSpawnKind::NuclearBfg9000),
    ] {
      let mut session = McpSession::new();
      session.start_game(seed, None, Some(10), Some(10)).unwrap();
      let player_id = session.game.as_ref().unwrap().world().player_id().unwrap();
      let weapon_id = session
        .game
        .as_mut()
        .unwrap()
        .world_mut()
        .allocate_item_id();
      let cells_id = session
        .game
        .as_mut()
        .unwrap()
        .world_mut()
        .allocate_item_id();
      let player = session
        .game
        .as_mut()
        .unwrap()
        .world_mut()
        .get_actor_mut(player_id)
        .unwrap();
      player
        .inventory_mut()
        .add_item(drl_core::item::Item::from_spawn_kind(weapon_id, kind))
        .unwrap();
      let weapon = player.inventory_mut().remove_item(weapon_id).unwrap();
      player
        .equipment_mut()
        .equip(EquipmentSlot::Weapon, weapon)
        .unwrap();
      player
        .inventory_mut()
        .add_item(drl_core::item::Item::ammo_cells(cells_id, 20))
        .unwrap();
      player
        .equipment_mut()
        .weapon_mut()
        .unwrap()
        .weapon_properties_mut()
        .unwrap()
        .current_clip = 1;

      let command = Command::Reload;
      let observation_before = session.get_observation().unwrap();
      let metrics_before = session.get_metrics().clone();
      let replay_before = session.export_replay().unwrap().clone();
      let events_before = session.recent_events().to_vec();

      assert!(
        !session
          .legal_actions()
          .unwrap()
          .iter()
          .any(|action| action.command == command)
      );
      assert_eq!(
        session.step(command).unwrap_err(),
        "Command is not currently advertised as legal"
      );
      assert_eq!(session.get_observation().unwrap(), observation_before);
      assert_eq!(session.get_metrics(), &metrics_before);
      assert_eq!(session.export_replay().unwrap(), &replay_before);
      assert_eq!(session.recent_events(), events_before.as_slice());
    }
  }

  #[test]
  fn test_json_to_command_parsing() {
    let raw = r#"{"action":"move","direction":"North"}"#;
    let val = JsonValue::parse(raw).unwrap();
    let cmd = json_to_command(&val).unwrap();
    assert_eq!(cmd, Command::Move(Direction::North));

    let raw_fire = r#"{"action":"fire","target_x":5,"target_y":10}"#;
    let val_fire = JsonValue::parse(raw_fire).unwrap();
    let cmd_fire = json_to_command(&val_fire).unwrap();
    assert_eq!(cmd_fire, Command::AttackRanged(Position::new(5, 10)));

    let alias_fire = JsonValue::parse(r#"{"action":"shoot","x":-3,"y":4}"#).unwrap();
    assert_eq!(
      json_to_command(&alias_fire).unwrap(),
      Command::AttackRanged(Position::new(-3, 4))
    );

    let aimed_fire =
      JsonValue::parse(r#"{"action":"aimed_fire","target_x":7,"target_y":11}"#).unwrap();
    assert_eq!(
      json_to_command(&aimed_fire).unwrap(),
      Command::AttackRangedAimed(Position::new(7, 11))
    );

    let chainfire =
      JsonValue::parse(r#"{"action":"chainfire","target_x":7,"target_y":11}"#).unwrap();
    assert_eq!(
      json_to_command(&chainfire).unwrap(),
      Command::AttackRangedChainfire(Position::new(7, 11))
    );

    let command_alias = JsonValue::parse(r#"{"command":"wait"}"#).unwrap();
    assert_eq!(json_to_command(&command_alias).unwrap(), Command::Wait);

    let boundary =
      JsonValue::parse(r#"{"action":"fire","target_x":-2147483648,"target_y":2147483647}"#)
        .unwrap();
    assert_eq!(
      json_to_command(&boundary).unwrap(),
      Command::AttackRanged(Position::new(i32::MIN, i32::MAX))
    );

    let item_boundary = JsonValue::parse(r#"{"action":"use","item_id":9007199254740992}"#).unwrap();
    assert_eq!(
      json_to_command(&item_boundary).unwrap(),
      Command::Use(ItemId::new(9_007_199_254_740_992))
    );

    let invoke = JsonValue::parse(r#"{"action":"invoke","item_id":42}"#).unwrap();
    assert_eq!(
      json_to_command(&invoke).unwrap(),
      Command::Invoke(ItemId::new(42))
    );

    let alt_reload =
      JsonValue::parse(r#"{"action":"alt_reload","item_id":43,"confirmed":true}"#).unwrap();
    assert_eq!(
      json_to_command(&alt_reload).unwrap(),
      Command::AltReload {
        item_id: ItemId::new(43),
        confirmed: true,
      }
    );

    for alias in ["none", "wait", "."] {
      let value =
        JsonValue::parse(&format!(r#"{{"action":"move","direction":"{alias}"}}"#)).unwrap();
      assert_eq!(json_to_command(&value).unwrap(), Command::Wait);
    }
  }

  #[test]
  fn test_json_to_command_rejects_unsafe_numeric_arguments() {
    for raw in [
      r#"{"action":"fire","target_x":2147483648,"target_y":0}"#,
      r#"{"action":"fire","target_x":-2147483649,"target_y":0}"#,
      r#"{"action":"fire","target_x":1.5,"target_y":0}"#,
      r#"{"action":"fire","target_x":"1","target_y":0}"#,
      r#"{"action":"use","item_id":-1}"#,
      r#"{"action":"use","item_id":9007199254740993}"#,
      r#"{"action":"drop","item_id":true}"#,
    ] {
      let value = JsonValue::parse(raw).unwrap();
      assert!(
        json_to_command(&value).is_err(),
        "accepted invalid action: {raw}"
      );
    }
  }

  #[test]
  fn test_security_dev_state_boundary() {
    let mut session = McpSession::new();
    session.start_game(123, None, None, None).unwrap();
    assert!(!session.is_dev_mode());
    assert!(session.get_dev_state().is_err());

    session.set_dev_mode(true);
    assert!(session.is_dev_mode());
    assert!(session.get_dev_state().is_ok());
  }
}
