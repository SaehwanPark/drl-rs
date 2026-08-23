//! Structural validation for the Rust-owned content tables.
//!
//! This module checks internal invariants before current definitions are used.
//! It intentionally does not compare values with legacy Lua, balance targets,
//! or external captures.

use drl_protocol::{ItemSpawnKind, MonsterDefinition, MonsterKind};

use crate::item_definition::{ItemDefinition, ItemDefinitionKind, definition_for_spawn_kind};
use crate::level_definition::{LEVEL_DEFINITIONS, LevelDefinition};
use crate::loot_definition::{GENERATED_LOOT_DEFINITIONS, GeneratedLootDefinition};
use crate::monster_roll_definition::{GENERATED_MONSTER_DEFINITIONS, GeneratedMonsterDefinition};
use crate::special_level_definition::SPECIAL_LEVEL_DEFINITIONS;

/// A structural defect in one of the current Rust-owned content tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentValidationError {
  /// A required descriptive field is empty.
  EmptyField {
    table: &'static str,
    key: &'static str,
    field: &'static str,
  },
  /// A required positive scalar is zero.
  NonPositive {
    table: &'static str,
    key: &'static str,
    field: &'static str,
  },
  /// A minimum/maximum pair is reversed or empty.
  InvalidRange {
    table: &'static str,
    key: &'static str,
    field: &'static str,
    minimum: u32,
    maximum: u32,
  },
  /// A percentage-like value falls outside its inclusive range.
  InvalidAccuracy { key: &'static str, accuracy: i32 },
  /// A weapon's ranged fields disagree with its weapon kind.
  InvalidWeaponShape {
    key: &'static str,
    field: &'static str,
  },
  /// A roll-bound table is not strictly increasing.
  InvalidRollBounds {
    table: &'static str,
    index: usize,
    previous: u32,
    upper_bound: u32,
  },
  /// A roll-bound table does not cover the complete `0..100` domain.
  IncompleteRollBounds {
    table: &'static str,
    final_bound: u32,
  },
  /// A special-level catalog is not sorted and unique.
  InvalidCatalogOrder {
    previous: &'static str,
    current: &'static str,
  },
}

/// Validates every current Rust-owned content table.
pub fn validate_current_content() -> Result<(), ContentValidationError> {
  for kind in [
    MonsterKind::FormerHuman,
    MonsterKind::FormerSergeant,
    MonsterKind::Imp,
    MonsterKind::Demon,
  ] {
    validate_monster_definition(kind.definition())?;
  }
  for kind in [
    ItemSpawnKind::Pistol,
    ItemSpawnKind::Shotgun,
    ItemSpawnKind::DoubleShotgun,
    ItemSpawnKind::CombatShotgun,
    ItemSpawnKind::Blaster,
    ItemSpawnKind::LaserRifle,
    ItemSpawnKind::MissileLauncher,
    ItemSpawnKind::NuclearPlasmaRifle,
    ItemSpawnKind::NuclearBfg9000,
    ItemSpawnKind::Bfg10k,
    ItemSpawnKind::MegaBuster,
    ItemSpawnKind::GrammatonBeretta,
    ItemSpawnKind::FragShotgun,
    ItemSpawnKind::RevenantsLauncher,
    ItemSpawnKind::Railgun,
    ItemSpawnKind::AcidSpitter,
    ItemSpawnKind::CombatPistol,
    ItemSpawnKind::AssaultShotgun,
    ItemSpawnKind::PlasmaShotgun,
    ItemSpawnKind::Jackhammer,
    ItemSpawnKind::SuperShotgun,
    ItemSpawnKind::TristarBlaster,
    ItemSpawnKind::ButchersCleaver,
    ItemSpawnKind::Mjollnir,
    ItemSpawnKind::SubtleKnife,
    ItemSpawnKind::Chaingun,
    ItemSpawnKind::PlasmaRifle,
    ItemSpawnKind::RocketLauncher,
    ItemSpawnKind::Bfg9000,
    ItemSpawnKind::Chainsaw,
    ItemSpawnKind::CombatKnife,
    ItemSpawnKind::Ammo9mm(0),
    ItemSpawnKind::AmmoShells(0),
    ItemSpawnKind::AmmoRockets(0),
    ItemSpawnKind::AmmoCells(0),
    ItemSpawnKind::AmmoPackRockets,
    ItemSpawnKind::AmmoPackCells,
    ItemSpawnKind::AmmoPack9mm,
    ItemSpawnKind::AmmoPackShells,
    ItemSpawnKind::SmallMedPack,
    ItemSpawnKind::LargeMedPack,
    ItemSpawnKind::GreenArmor,
    ItemSpawnKind::BlueArmor,
    ItemSpawnKind::RedArmor,
    ItemSpawnKind::PhaseDevice,
  ] {
    validate_item_definition(definition_for_spawn_kind(kind))?;
  }

  validate_monster_roll_table(&GENERATED_MONSTER_DEFINITIONS)?;
  validate_loot_roll_table(&GENERATED_LOOT_DEFINITIONS)?;
  for level in LEVEL_DEFINITIONS {
    validate_level_definition(level)?;
  }
  validate_special_level_catalog()
}

fn validate_monster_definition(
  definition: MonsterDefinition,
) -> Result<(), ContentValidationError> {
  let key = definition.name;
  require_nonempty("monster", key, "name", definition.name)?;
  require_positive("monster", key, "hp", definition.hp)?;
  require_positive("monster", key, "speed", definition.speed)?;
  require_range("monster", key, "melee_damage", definition.melee_damage)?;
  if let Some(ranged_damage) = definition.ranged_damage {
    require_range("monster", key, "ranged_damage", ranged_damage)?;
    require_positive("monster", key, "ranged_range", definition.ranged_range)?;
  } else if definition.ranged_range != 0 {
    return Err(ContentValidationError::InvalidWeaponShape {
      key,
      field: "ranged_range",
    });
  }
  if !(0..=100).contains(&definition.accuracy) {
    return Err(ContentValidationError::InvalidAccuracy {
      key,
      accuracy: definition.accuracy,
    });
  }
  Ok(())
}

fn validate_item_definition(definition: &ItemDefinition) -> Result<(), ContentValidationError> {
  let key = definition.name;
  require_nonempty("item", key, "name", definition.name)?;
  require_nonempty("item", key, "description", definition.description)?;
  match definition.kind {
    ItemDefinitionKind::Weapon {
      is_ranged,
      ammo_type,
      clip_capacity,
      damage,
      range,
      accuracy,
      ..
    } => {
      require_range("item", key, "damage", damage)?;
      if !(0..=100).contains(&accuracy) {
        return Err(ContentValidationError::InvalidAccuracy { key, accuracy });
      }
      if is_ranged {
        if ammo_type.is_none() {
          return Err(ContentValidationError::InvalidWeaponShape {
            key,
            field: "ammo_type",
          });
        }
        require_positive("item", key, "clip_capacity", clip_capacity)?;
        require_positive("item", key, "range", range)?;
      } else {
        if ammo_type.is_some() || clip_capacity != 0 {
          return Err(ContentValidationError::InvalidWeaponShape {
            key,
            field: "melee_ammo",
          });
        }
        require_positive("item", key, "range", range)?;
      }
    }
    ItemDefinitionKind::Ammo {
      max_stack,
      initial_amount,
      ..
    } => {
      require_positive("item", key, "max_stack", max_stack)?;
      if let Some(initial_amount) = initial_amount
        && initial_amount > max_stack
      {
        return Err(ContentValidationError::InvalidRange {
          table: "item",
          key,
          field: "initial_amount",
          minimum: initial_amount,
          maximum: max_stack,
        });
      }
    }
    ItemDefinitionKind::AmmoPack {
      amount, max_amount, ..
    } => {
      require_positive("item", key, "amount", amount)?;
      require_positive("item", key, "max_amount", max_amount)?;
      if amount > max_amount {
        return Err(ContentValidationError::InvalidRange {
          table: "item",
          key,
          field: "amount",
          minimum: amount,
          maximum: max_amount,
        });
      }
    }
    ItemDefinitionKind::MedPack { heal_amount } => {
      require_positive("item", key, "heal_amount", heal_amount)?;
    }
    ItemDefinitionKind::Armor {
      protection,
      durability,
      max_durability,
    } => {
      require_positive("item", key, "protection", protection)?;
      require_positive("item", key, "max_durability", max_durability)?;
      if durability > max_durability {
        return Err(ContentValidationError::InvalidRange {
          table: "item",
          key,
          field: "durability",
          minimum: durability,
          maximum: max_durability,
        });
      }
    }
    ItemDefinitionKind::PhaseDevice => {}
  }
  Ok(())
}

fn validate_monster_roll_table(
  definitions: &[GeneratedMonsterDefinition],
) -> Result<(), ContentValidationError> {
  let bounds: Vec<_> = definitions
    .iter()
    .map(|definition| definition.upper_bound)
    .collect();
  validate_roll_bounds("monster-roll", &bounds)?;
  for definition in definitions {
    validate_monster_definition(definition.kind.definition())?;
  }
  Ok(())
}

fn validate_loot_roll_table(
  definitions: &[GeneratedLootDefinition],
) -> Result<(), ContentValidationError> {
  let bounds: Vec<_> = definitions
    .iter()
    .map(|definition| definition.upper_bound)
    .collect();
  validate_roll_bounds("loot-roll", &bounds)?;
  for definition in definitions {
    validate_item_definition(definition_for_spawn_kind(definition.spawn_kind))?;
  }
  Ok(())
}

fn validate_roll_bounds(table: &'static str, bounds: &[u32]) -> Result<(), ContentValidationError> {
  let mut previous = 0;
  for (index, &upper_bound) in bounds.iter().enumerate() {
    if upper_bound <= previous {
      return Err(ContentValidationError::InvalidRollBounds {
        table,
        index,
        previous,
        upper_bound,
      });
    }
    previous = upper_bound;
  }
  if previous != 100 {
    return Err(ContentValidationError::IncompleteRollBounds {
      table,
      final_bound: previous,
    });
  }
  Ok(())
}

fn validate_level_definition(level: LevelDefinition) -> Result<(), ContentValidationError> {
  require_nonempty("level", level.key, "key", level.key)?;
  for (field, value) in [
    ("width", level.width),
    ("height", level.height),
    ("max_rooms", level.max_rooms),
    ("min_room_size", level.min_room_size),
    ("max_room_size", level.max_room_size),
    ("max_monsters_per_room", level.max_monsters_per_room),
    ("max_items_per_room", level.max_items_per_room),
  ] {
    require_positive("level", level.key, field, value)?;
  }
  if level.min_room_size > level.max_room_size {
    return Err(ContentValidationError::InvalidRange {
      table: "level",
      key: level.key,
      field: "room_size",
      minimum: level.min_room_size,
      maximum: level.max_room_size,
    });
  }
  Ok(())
}

fn validate_special_level_catalog() -> Result<(), ContentValidationError> {
  for definition in &SPECIAL_LEVEL_DEFINITIONS {
    require_nonempty("special-level", definition.id, "name", definition.name)?;
  }
  for pair in SPECIAL_LEVEL_DEFINITIONS.windows(2) {
    if pair[0].id >= pair[1].id {
      return Err(ContentValidationError::InvalidCatalogOrder {
        previous: pair[0].id,
        current: pair[1].id,
      });
    }
  }
  Ok(())
}

fn require_nonempty(
  table: &'static str,
  key: &'static str,
  field: &'static str,
  value: &str,
) -> Result<(), ContentValidationError> {
  if value.is_empty() {
    Err(ContentValidationError::EmptyField { table, key, field })
  } else {
    Ok(())
  }
}

fn require_positive(
  table: &'static str,
  key: &'static str,
  field: &'static str,
  value: u32,
) -> Result<(), ContentValidationError> {
  if value == 0 {
    Err(ContentValidationError::NonPositive { table, key, field })
  } else {
    Ok(())
  }
}

fn require_range(
  table: &'static str,
  key: &'static str,
  field: &'static str,
  range: (u32, u32),
) -> Result<(), ContentValidationError> {
  if range.0 == 0 || range.0 > range.1 {
    Err(ContentValidationError::InvalidRange {
      table,
      key,
      field,
      minimum: range.0,
      maximum: range.1,
    })
  } else {
    Ok(())
  }
}

#[cfg(test)]
#[path = "content_validation_tests.rs"]
mod content_validation_tests;
