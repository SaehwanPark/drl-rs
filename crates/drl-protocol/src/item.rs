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

/// Typed fire mode for weapons with explicit mode behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponFireMode {
  /// One round using the single-fire profile.
  Single,
  /// Three-round burst profile.
  Burst,
  /// Six-round full-auto profile.
  Auto,
}

/// Declares the routine stable item identity and replay spawn projections from
/// one source. The spawn variant, normalized catalog value, and matching
/// pattern are kept together so count-sensitive payloads stay explicit without
/// duplicating the family list in another module.
macro_rules! define_item_catalog {
  (
    $unknown:ident => $unknown_name:literal;
    $(
      $variant:ident => $stable_name:literal
      => $spawn_variant:ident $( ( $($spawn_payload:tt)* ) )?
      => $spawn_value:expr
      => $spawn_pattern:pat
    ),+ $(,)?
  ) => {
    /// Stable presentation identifier for an item family.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub enum ItemArchetype {
      #[default]
      $unknown,
      $( $variant, )+
    }

    /// Stable replay/scenario spawn family for an item.
    ///
    /// This type is generated beside [`ItemArchetype`] so routine identity
    /// registration has one source. Gameplay definitions and presentation
    /// mappings remain explicit in their owning crates.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ItemSpawnKind {
      $( $spawn_variant $( ( $($spawn_payload)* ) )?, )+
    }

    impl ItemArchetype {
      /// All stable item archetypes currently registered by the protocol.
      pub const ALL: &[Self] = &[
        Self::$unknown,
        $( Self::$variant, )+
      ];

      /// Returns the stable wire identifier for this archetype.
      #[must_use]
      pub const fn stable_name(self) -> &'static str {
        match self {
          Self::$unknown => $unknown_name,
          $( Self::$variant => $stable_name, )+
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

      /// Returns whether replay JSON must carry an explicit loose-ammo count.
      #[must_use]
      pub const fn requires_stack_count(self) -> bool {
        matches!(
          self,
          Self::Ammo9mm | Self::AmmoShells | Self::AmmoRockets | Self::AmmoCells
        )
      }
    }

    impl ItemSpawnKind {
      /// All stable spawn families with normalized representative values.
      ///
      /// Loose-ammo counts are intentionally zero here; callers own the
      /// amount for a concrete item instance.
      pub const ALL: &[Self] = &[
        $( $spawn_value, )+
      ];

      /// Returns the stable presentation/replay archetype for this family.
      #[must_use]
      pub const fn archetype(self) -> ItemArchetype {
        match self {
          $( $spawn_pattern => ItemArchetype::$variant, )+
        }
      }

    }
  };
}

define_item_catalog! {
  Unknown => "unknown";
  Pistol => "pistol" => Pistol => Self::Pistol => Self::Pistol,
  Shotgun => "shotgun" => Shotgun => Self::Shotgun => Self::Shotgun,
  DoubleShotgun => "double_shotgun" => DoubleShotgun => Self::DoubleShotgun => Self::DoubleShotgun,
  CombatShotgun => "combat_shotgun" => CombatShotgun => Self::CombatShotgun => Self::CombatShotgun,
  Blaster => "blaster" => Blaster => Self::Blaster => Self::Blaster,
  LaserRifle => "laser_rifle" => LaserRifle => Self::LaserRifle => Self::LaserRifle,
  MissileLauncher => "missile_launcher" => MissileLauncher => Self::MissileLauncher => Self::MissileLauncher,
  NuclearPlasmaRifle => "nuclear_plasma_rifle" => NuclearPlasmaRifle => Self::NuclearPlasmaRifle => Self::NuclearPlasmaRifle,
  NuclearBfg9000 => "nuclear_bfg9000" => NuclearBfg9000 => Self::NuclearBfg9000 => Self::NuclearBfg9000,
  Bfg10k => "bfg_10k" => Bfg10k => Self::Bfg10k => Self::Bfg10k,
  MegaBuster => "mega_buster" => MegaBuster => Self::MegaBuster => Self::MegaBuster,
  GrammatonBeretta => "grammaton_beretta" => GrammatonBeretta => Self::GrammatonBeretta => Self::GrammatonBeretta,
  FragShotgun => "frag_shotgun" => FragShotgun => Self::FragShotgun => Self::FragShotgun,
  RevenantsLauncher => "revenants_launcher" => RevenantsLauncher => Self::RevenantsLauncher => Self::RevenantsLauncher,
  Railgun => "railgun" => Railgun => Self::Railgun => Self::Railgun,
  AcidSpitter => "acid_spitter" => AcidSpitter => Self::AcidSpitter => Self::AcidSpitter,
  NullPointer => "null_pointer" => NullPointer => Self::NullPointer => Self::NullPointer,
  CombatPistol => "combat_pistol" => CombatPistol => Self::CombatPistol => Self::CombatPistol,
  AssaultShotgun => "assault_shotgun" => AssaultShotgun => Self::AssaultShotgun => Self::AssaultShotgun,
  PlasmaShotgun => "plasma_shotgun" => PlasmaShotgun => Self::PlasmaShotgun => Self::PlasmaShotgun,
  Jackhammer => "jackhammer" => Jackhammer => Self::Jackhammer => Self::Jackhammer,
  SuperShotgun => "super_shotgun" => SuperShotgun => Self::SuperShotgun => Self::SuperShotgun,
  TristarBlaster => "tristar_blaster" => TristarBlaster => Self::TristarBlaster => Self::TristarBlaster,
  ButchersCleaver => "butchers_cleaver" => ButchersCleaver => Self::ButchersCleaver => Self::ButchersCleaver,
  Mjollnir => "mjollnir" => Mjollnir => Self::Mjollnir => Self::Mjollnir,
  SubtleKnife => "subtle_knife" => SubtleKnife => Self::SubtleKnife => Self::SubtleKnife,
  Trigun => "trigun" => Trigun => Self::Trigun => Self::Trigun,
  AntiFreakJackal => "anti_freak_jackal" => AntiFreakJackal => Self::AntiFreakJackal => Self::AntiFreakJackal,
  Minigun => "minigun" => Minigun => Self::Minigun => Self::Minigun,
  Chaingun => "chaingun" => Chaingun => Self::Chaingun => Self::Chaingun,
  RocketLauncher => "rocket_launcher" => RocketLauncher => Self::RocketLauncher => Self::RocketLauncher,
  PlasmaRifle => "plasma_rifle" => PlasmaRifle => Self::PlasmaRifle => Self::PlasmaRifle,
  Bfg9000 => "bfg9000" => Bfg9000 => Self::Bfg9000 => Self::Bfg9000,
  Chainsaw => "chainsaw" => Chainsaw => Self::Chainsaw => Self::Chainsaw,
  CombatKnife => "combat_knife" => CombatKnife => Self::CombatKnife => Self::CombatKnife,
  Ammo9mm => "ammo_9mm" => Ammo9mm(u32) => Self::Ammo9mm(0) => Self::Ammo9mm(_),
  AmmoShells => "ammo_shells" => AmmoShells(u32) => Self::AmmoShells(0) => Self::AmmoShells(_),
  AmmoRockets => "ammo_rockets" => AmmoRockets(u32) => Self::AmmoRockets(0) => Self::AmmoRockets(_),
  AmmoCells => "ammo_cells" => AmmoCells(u32) => Self::AmmoCells(0) => Self::AmmoCells(_),
  AmmoPackRockets => "ammo_pack_rockets" => AmmoPackRockets => Self::AmmoPackRockets => Self::AmmoPackRockets,
  AmmoPackCells => "ammo_pack_cells" => AmmoPackCells => Self::AmmoPackCells => Self::AmmoPackCells,
  AmmoPack9mm => "ammo_pack_9mm" => AmmoPack9mm => Self::AmmoPack9mm => Self::AmmoPack9mm,
  AmmoPackShells => "ammo_pack_shells" => AmmoPackShells => Self::AmmoPackShells => Self::AmmoPackShells,
  SmallMedPack => "small_medpack" => SmallMedPack => Self::SmallMedPack => Self::SmallMedPack,
  LargeMedPack => "large_medpack" => LargeMedPack => Self::LargeMedPack => Self::LargeMedPack,
  GreenArmor => "green_armor" => GreenArmor => Self::GreenArmor => Self::GreenArmor,
  BlueArmor => "blue_armor" => BlueArmor => Self::BlueArmor => Self::BlueArmor,
  RedArmor => "red_armor" => RedArmor => Self::RedArmor => Self::RedArmor,
  OnyxArmor => "onyx_armor" => OnyxArmor => Self::OnyxArmor => Self::OnyxArmor,
  PhaseshiftArmor => "phaseshift_armor" => PhaseshiftArmor => Self::PhaseshiftArmor => Self::PhaseshiftArmor,
  GothicArmor => "gothic_armor" => GothicArmor => Self::GothicArmor => Self::GothicArmor,
  MaleksArmor => "maleks_armor" => MaleksArmor => Self::MaleksArmor => Self::MaleksArmor,
  CyberneticArmor => "cybernetic_armor" => CyberneticArmor => Self::CyberneticArmor => Self::CyberneticArmor,
  Necroarmor => "necroarmor" => Necroarmor => Self::Necroarmor => Self::Necroarmor,
  MedicalPowerarmor => "medical_powerarmor" => MedicalPowerarmor => Self::MedicalPowerarmor => Self::MedicalPowerarmor,
  LavaArmor => "lava_armor" => LavaArmor => Self::LavaArmor => Self::LavaArmor,
  ShieldedArmor => "shielded_armor" => ShieldedArmor => Self::ShieldedArmor => Self::ShieldedArmor,
  PhaseDevice => "phase_device" => PhaseDevice => Self::PhaseDevice => Self::PhaseDevice,
}

impl fmt::Display for ItemArchetype {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.stable_name())
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
