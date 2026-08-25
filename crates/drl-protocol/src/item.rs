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

/// Typed fire mode for the Grammaton Cleric Beretta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponFireMode {
  /// One round using the single-fire profile.
  Single,
  /// Three-round burst profile.
  Burst,
  /// Six-round full-auto profile.
  Auto,
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
  DoubleShotgun,
  CombatShotgun,
  Blaster,
  LaserRifle,
  MissileLauncher,
  NuclearPlasmaRifle,
  NuclearBfg9000,
  Bfg10k,
  MegaBuster,
  GrammatonBeretta,
  FragShotgun,
  RevenantsLauncher,
  Railgun,
  AcidSpitter,
  CombatPistol,
  AssaultShotgun,
  PlasmaShotgun,
  Jackhammer,
  SuperShotgun,
  TristarBlaster,
  ButchersCleaver,
  Mjollnir,
  SubtleKnife,
  Trigun,
  AntiFreakJackal,
  Minigun,
  Chaingun,
  RocketLauncher,
  PlasmaRifle,
  Bfg9000,
  Chainsaw,
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
  RedArmor,
  OnyxArmor,
  PhaseshiftArmor,
  GothicArmor,
  MaleksArmor,
  CyberneticArmor,
  Necroarmor,
  MedicalPowerarmor,
  LavaArmor,
  ShieldedArmor,
  PhaseDevice,
}

impl ItemArchetype {
  /// All stable item archetypes currently registered by the protocol.
  ///
  /// This is a presentation/contract catalog, not a gameplay definition
  /// table. Core definitions remain authoritative for balance and behavior.
  pub const ALL: &[Self] = &[
    Self::Unknown,
    Self::Pistol,
    Self::Shotgun,
    Self::DoubleShotgun,
    Self::CombatShotgun,
    Self::Blaster,
    Self::LaserRifle,
    Self::MissileLauncher,
    Self::NuclearPlasmaRifle,
    Self::NuclearBfg9000,
    Self::Bfg10k,
    Self::MegaBuster,
    Self::GrammatonBeretta,
    Self::FragShotgun,
    Self::RevenantsLauncher,
    Self::Railgun,
    Self::AcidSpitter,
    Self::CombatPistol,
    Self::AssaultShotgun,
    Self::PlasmaShotgun,
    Self::Jackhammer,
    Self::SuperShotgun,
    Self::TristarBlaster,
    Self::ButchersCleaver,
    Self::Mjollnir,
    Self::SubtleKnife,
    Self::Trigun,
    Self::AntiFreakJackal,
    Self::Minigun,
    Self::Chaingun,
    Self::RocketLauncher,
    Self::PlasmaRifle,
    Self::Bfg9000,
    Self::Chainsaw,
    Self::CombatKnife,
    Self::Ammo9mm,
    Self::AmmoShells,
    Self::AmmoRockets,
    Self::AmmoCells,
    Self::AmmoPackRockets,
    Self::AmmoPackCells,
    Self::AmmoPack9mm,
    Self::AmmoPackShells,
    Self::SmallMedPack,
    Self::LargeMedPack,
    Self::GreenArmor,
    Self::BlueArmor,
    Self::RedArmor,
    Self::OnyxArmor,
    Self::PhaseshiftArmor,
    Self::GothicArmor,
    Self::MaleksArmor,
    Self::CyberneticArmor,
    Self::Necroarmor,
    Self::MedicalPowerarmor,
    Self::LavaArmor,
    Self::ShieldedArmor,
    Self::PhaseDevice,
  ];

  /// Returns whether replay JSON must carry an explicit loose-ammo count.
  ///
  /// This routine wire-shape projection belongs to the stable archetype
  /// catalog so decoders do not maintain a second ammo-family list.
  #[must_use]
  pub const fn requires_stack_count(self) -> bool {
    matches!(
      self,
      Self::Ammo9mm | Self::AmmoShells | Self::AmmoRockets | Self::AmmoCells
    )
  }
}

impl fmt::Display for ItemArchetype {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.stable_name())
  }
}

impl ItemArchetype {
  /// Returns the stable wire identifier for this archetype.
  #[must_use]
  pub const fn stable_name(self) -> &'static str {
    match self {
      Self::Unknown => "unknown",
      Self::Pistol => "pistol",
      Self::Shotgun => "shotgun",
      Self::DoubleShotgun => "double_shotgun",
      Self::CombatShotgun => "combat_shotgun",
      Self::Blaster => "blaster",
      Self::LaserRifle => "laser_rifle",
      Self::MissileLauncher => "missile_launcher",
      Self::NuclearPlasmaRifle => "nuclear_plasma_rifle",
      Self::NuclearBfg9000 => "nuclear_bfg9000",
      Self::Bfg10k => "bfg_10k",
      Self::MegaBuster => "mega_buster",
      Self::GrammatonBeretta => "grammaton_beretta",
      Self::FragShotgun => "frag_shotgun",
      Self::RevenantsLauncher => "revenants_launcher",
      Self::Railgun => "railgun",
      Self::AcidSpitter => "acid_spitter",
      Self::CombatPistol => "combat_pistol",
      Self::AssaultShotgun => "assault_shotgun",
      Self::PlasmaShotgun => "plasma_shotgun",
      Self::Jackhammer => "jackhammer",
      Self::SuperShotgun => "super_shotgun",
      Self::TristarBlaster => "tristar_blaster",
      Self::ButchersCleaver => "butchers_cleaver",
      Self::Mjollnir => "mjollnir",
      Self::SubtleKnife => "subtle_knife",
      Self::Trigun => "trigun",
      Self::AntiFreakJackal => "anti_freak_jackal",
      Self::Minigun => "minigun",
      Self::Chaingun => "chaingun",
      Self::RocketLauncher => "rocket_launcher",
      Self::PlasmaRifle => "plasma_rifle",
      Self::Bfg9000 => "bfg9000",
      Self::Chainsaw => "chainsaw",
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
      Self::RedArmor => "red_armor",
      Self::OnyxArmor => "onyx_armor",
      Self::PhaseshiftArmor => "phaseshift_armor",
      Self::GothicArmor => "gothic_armor",
      Self::MaleksArmor => "maleks_armor",
      Self::CyberneticArmor => "cybernetic_armor",
      Self::Necroarmor => "necroarmor",
      Self::MedicalPowerarmor => "medical_powerarmor",
      Self::LavaArmor => "lava_armor",
      Self::ShieldedArmor => "shielded_armor",
      Self::PhaseDevice => "phase_device",
    }
  }

  /// Parses the stable wire identifier emitted by `Display`.
  #[must_use]
  pub fn from_stable_name(name: &str) -> Option<Self> {
    Self::ALL
      .iter()
      .copied()
      .find(|archetype| archetype.stable_name() == name)
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
  use std::collections::BTreeSet;

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
    assert_eq!(ItemArchetype::RedArmor.to_string(), "red_armor");
    assert_eq!(ItemArchetype::Blaster.to_string(), "blaster");
    assert_eq!(ItemArchetype::LaserRifle.to_string(), "laser_rifle");
    assert_eq!(
      ItemArchetype::MissileLauncher.to_string(),
      "missile_launcher"
    );
    assert_eq!(
      ItemArchetype::NuclearPlasmaRifle.to_string(),
      "nuclear_plasma_rifle"
    );
    assert_eq!(ItemArchetype::NuclearBfg9000.to_string(), "nuclear_bfg9000");
    assert_eq!(ItemArchetype::Bfg10k.to_string(), "bfg_10k");
    assert_eq!(ItemArchetype::PlasmaRifle.to_string(), "plasma_rifle");
    assert_eq!(ItemArchetype::RocketLauncher.to_string(), "rocket_launcher");
  }

  #[test]
  fn stable_archetype_names_round_trip() {
    for archetype in [
      ItemArchetype::Pistol,
      ItemArchetype::Bfg10k,
      ItemArchetype::AmmoCells,
      ItemArchetype::PhaseDevice,
    ] {
      assert_eq!(
        ItemArchetype::from_stable_name(&archetype.to_string()),
        Some(archetype)
      );
    }
    assert_eq!(ItemArchetype::from_stable_name("not-an-item"), None);
  }

  #[test]
  fn stable_archetype_catalog_names_are_unique_and_round_trip() {
    let names = ItemArchetype::ALL
      .iter()
      .map(|archetype| archetype.stable_name())
      .collect::<BTreeSet<_>>();
    assert_eq!(names.len(), ItemArchetype::ALL.len());

    for archetype in ItemArchetype::ALL.iter().copied() {
      assert_eq!(archetype.to_string(), archetype.stable_name());
      assert_eq!(
        ItemArchetype::from_stable_name(archetype.stable_name()),
        Some(archetype)
      );
    }
  }

  #[test]
  fn loose_ammo_catalog_shape_is_explicit() {
    assert!(ItemArchetype::Ammo9mm.requires_stack_count());
    assert!(ItemArchetype::AmmoShells.requires_stack_count());
    assert!(ItemArchetype::AmmoRockets.requires_stack_count());
    assert!(ItemArchetype::AmmoCells.requires_stack_count());
    assert!(!ItemArchetype::Pistol.requires_stack_count());
    assert!(!ItemArchetype::AmmoPackCells.requires_stack_count());
    assert!(!ItemArchetype::SmallMedPack.requires_stack_count());
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
