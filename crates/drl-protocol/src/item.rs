//! Item domain types, ammunition categories, equipment slots, and observation views.

use crate::types::{ItemId, Position};
use std::fmt;

/// Ammunition caliber and energy cell types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum AmmoType {
  /// Standard 9mm pistol ammunition.
  #[default]
  Ammo9mm,
  /// 12-gauge shotgun shells.
  Shells,
  /// High-explosive rockets.
  Rocket,
  /// Plasma energy cells.
  Cell,
}

impl fmt::Display for AmmoType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Ammo9mm => write!(f, "9mm ammo"),
      Self::Shells => write!(f, "shotgun shells"),
      Self::Rocket => write!(f, "rockets"),
      Self::Cell => write!(f, "energy cells"),
    }
  }
}

/// Equipment slots for wearable or wieldable items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EquipmentSlot {
  /// Primary wielded weapon (melee or ranged).
  Weapon,
  /// Body armor suit.
  Armor,
}

impl fmt::Display for EquipmentSlot {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Weapon => write!(f, "weapon"),
      Self::Armor => write!(f, "armor"),
    }
  }
}

/// High-level semantic item category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemCategory {
  /// Weapon item (melee or ranged).
  Weapon,
  /// Protective body armor.
  Armor,
  /// Ammunition stack.
  Ammo,
  /// Prepared-slot ammunition pack.
  AmmoPack,
  /// Medical supply or consumable item.
  MedPack,
  /// Special consumable device (e.g. Phase Device).
  PhaseDevice,
}

/// Stable presentation identifier for an item family.
///
/// Unlike the display name, this identifier is safe to use in asset lookup
/// tables and remains stable when localized or reformatted text changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ItemArchetype {
  #[default]
  Unknown,
  Pistol,
  Shotgun,
  CombatKnife,
  Ammo9mm,
  AmmoShells,
  AmmoRockets,
  AmmoCells,
  AmmoPackRockets,
  AmmoPackCells,
  AmmoPack9mm,
  AmmoPackShells,
  SmallMedPack,
  LargeMedPack,
  GreenArmor,
  BlueArmor,
  PhaseDevice,
}

impl fmt::Display for ItemArchetype {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let value = match self {
      Self::Unknown => "unknown",
      Self::Pistol => "pistol",
      Self::Shotgun => "shotgun",
      Self::CombatKnife => "combat_knife",
      Self::Ammo9mm => "ammo_9mm",
      Self::AmmoShells => "ammo_shells",
      Self::AmmoRockets => "ammo_rockets",
      Self::AmmoCells => "ammo_cells",
      Self::AmmoPackRockets => "ammo_pack_rockets",
      Self::AmmoPackCells => "ammo_pack_cells",
      Self::AmmoPack9mm => "ammo_pack_9mm",
      Self::AmmoPackShells => "ammo_pack_shells",
      Self::SmallMedPack => "small_medpack",
      Self::LargeMedPack => "large_medpack",
      Self::GreenArmor => "green_armor",
      Self::BlueArmor => "blue_armor",
      Self::PhaseDevice => "phase_device",
    };
    write!(f, "{value}")
  }
}

impl fmt::Display for ItemCategory {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Weapon => write!(f, "Weapon"),
      Self::Armor => write!(f, "Armor"),
      Self::Ammo => write!(f, "Ammo"),
      Self::AmmoPack => write!(f, "AmmoPack"),
      Self::MedPack => write!(f, "MedPack"),
      Self::PhaseDevice => write!(f, "PhaseDevice"),
    }
  }
}

/// Immutable semantic view of an item for observations and UI frontends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemView {
  pub id: ItemId,
  pub archetype: ItemArchetype,
  pub name: String,
  pub category: ItemCategory,
  pub count: u32,
  pub description: String,
  pub clip: Option<(u32, u32)>,
  pub damage: Option<(u32, u32)>,
  pub armor_value: Option<u32>,
  pub heal_amount: Option<u32>,
  pub knockback: Option<u32>,
}

/// Semantic view of an item lying on the dungeon floor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroundItemView {
  pub position: Position,
  pub item: ItemView,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ammo_type_display() {
    assert_eq!(AmmoType::Ammo9mm.to_string(), "9mm ammo");
    assert_eq!(AmmoType::Shells.to_string(), "shotgun shells");
    assert_eq!(AmmoType::Rocket.to_string(), "rockets");
    assert_eq!(AmmoType::Cell.to_string(), "energy cells");
  }

  #[test]
  fn test_ammo_archetype_display() {
    assert_eq!(ItemArchetype::AmmoRockets.to_string(), "ammo_rockets");
    assert_eq!(ItemArchetype::AmmoCells.to_string(), "ammo_cells");
    assert_eq!(
      ItemArchetype::AmmoPackRockets.to_string(),
      "ammo_pack_rockets"
    );
    assert_eq!(ItemArchetype::AmmoPackCells.to_string(), "ammo_pack_cells");
    assert_eq!(ItemArchetype::AmmoPack9mm.to_string(), "ammo_pack_9mm");
    assert_eq!(
      ItemArchetype::AmmoPackShells.to_string(),
      "ammo_pack_shells"
    );
    assert_eq!(ItemArchetype::BlueArmor.to_string(), "blue_armor");
  }

  #[test]
  fn test_ammo_pack_category_display() {
    assert_eq!(ItemCategory::AmmoPack.to_string(), "AmmoPack");
  }

  #[test]
  fn test_equipment_slot_display() {
    assert_eq!(EquipmentSlot::Weapon.to_string(), "weapon");
    assert_eq!(EquipmentSlot::Armor.to_string(), "armor");
  }
}
