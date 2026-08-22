//! Immutable Rust-owned definitions for the current item spawn families.

use drl_protocol::{ActionCost, AmmoType, ItemArchetype, ItemSpawnKind};

/// Static payload needed to construct one known item family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemDefinitionKind {
  /// A weapon with its immutable combat properties.
  Weapon {
    is_ranged: bool,
    ammo_type: Option<AmmoType>,
    clip_capacity: u32,
    damage: (u32, u32),
    range: u32,
    accuracy: i32,
    knockback: u32,
    fire_cost: ActionCost,
    reload_cost: ActionCost,
  },
  /// A stackable ammunition family; the instance count remains caller-owned.
  Ammo { ammo_type: AmmoType, max_stack: u32 },
  /// A medical consumable with a fixed healing amount.
  MedPack { heal_amount: u32 },
  /// Wearable armor with its baseline durability.
  Armor {
    protection: u32,
    durability: u32,
    max_durability: u32,
  },
  /// The special phase-shift consumable.
  PhaseDevice,
}

/// Immutable metadata for one current Rust-owned item family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemDefinition {
  pub archetype: ItemArchetype,
  pub name: &'static str,
  pub description: &'static str,
  pub kind: ItemDefinitionKind,
}

const PISTOL: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Pistol,
  name: "Pistol",
  description: "Standard 9mm military sidearm. Reliable and accurate.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Ammo9mm),
    clip_capacity: 10,
    damage: (4, 8),
    range: 8,
    accuracy: 75,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const SHOTGUN: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Shotgun,
  name: "Shotgun",
  description: "Pump-action 12-gauge shotgun. Devastating at close range.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Shells),
    clip_capacity: 8,
    damage: (8, 16),
    range: 5,
    accuracy: 65,
    knockback: 1,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::new(1200),
  },
};

const COMBAT_KNIFE: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::CombatKnife,
  name: "Combat Knife",
  description: "Serrated combat blade for close-quarters fighting.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: false,
    ammo_type: None,
    clip_capacity: 0,
    damage: (5, 9),
    range: 1,
    accuracy: 85,
    knockback: 0,
    fire_cost: ActionCost::MELEE_ATTACK,
    reload_cost: ActionCost::new(0),
  },
};

const AMMO_9MM: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Ammo9mm,
  name: "9mm Ammo",
  description: "Standard magazine rounds for 9mm pistols and submachine guns.",
  kind: ItemDefinitionKind::Ammo {
    ammo_type: AmmoType::Ammo9mm,
    max_stack: 100,
  },
};

const AMMO_SHELLS: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::AmmoShells,
  name: "Shotgun Shells",
  description: "Heavy buckshot shells for shotguns.",
  kind: ItemDefinitionKind::Ammo {
    ammo_type: AmmoType::Shells,
    max_stack: 50,
  },
};

const SMALL_MEDPACK: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::SmallMedPack,
  name: "Small MedPack",
  description: "Compact medical kit providing rapid first aid (+10 HP).",
  kind: ItemDefinitionKind::MedPack { heal_amount: 10 },
};

const LARGE_MEDPACK: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::LargeMedPack,
  name: "Large MedPack",
  description: "Comprehensive field surgery kit (+25 HP).",
  kind: ItemDefinitionKind::MedPack { heal_amount: 25 },
};

const GREEN_ARMOR: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::GreenArmor,
  name: "Green Armor",
  description: "Standard security armor suit absorbing incoming damage.",
  kind: ItemDefinitionKind::Armor {
    protection: 5,
    durability: 100,
    max_durability: 100,
  },
};

const PHASE_DEVICE: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::PhaseDevice,
  name: "Phase Device",
  description: "Emergency phase-shift device. Instantly teleports the user across space.",
  kind: ItemDefinitionKind::PhaseDevice,
};

/// Returns the immutable definition for one current spawn family.
#[must_use]
pub const fn definition_for_spawn_kind(kind: ItemSpawnKind) -> &'static ItemDefinition {
  match kind {
    ItemSpawnKind::Pistol => &PISTOL,
    ItemSpawnKind::Shotgun => &SHOTGUN,
    ItemSpawnKind::CombatKnife => &COMBAT_KNIFE,
    ItemSpawnKind::Ammo9mm(_) => &AMMO_9MM,
    ItemSpawnKind::AmmoShells(_) => &AMMO_SHELLS,
    ItemSpawnKind::SmallMedPack => &SMALL_MEDPACK,
    ItemSpawnKind::LargeMedPack => &LARGE_MEDPACK,
    ItemSpawnKind::GreenArmor => &GREEN_ARMOR,
    ItemSpawnKind::PhaseDevice => &PHASE_DEVICE,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn definition_table_covers_every_current_spawn_family() {
    let cases = [
      (ItemSpawnKind::Pistol, ItemArchetype::Pistol, "Pistol"),
      (ItemSpawnKind::Shotgun, ItemArchetype::Shotgun, "Shotgun"),
      (
        ItemSpawnKind::CombatKnife,
        ItemArchetype::CombatKnife,
        "Combat Knife",
      ),
      (
        ItemSpawnKind::Ammo9mm(0),
        ItemArchetype::Ammo9mm,
        "9mm Ammo",
      ),
      (
        ItemSpawnKind::AmmoShells(0),
        ItemArchetype::AmmoShells,
        "Shotgun Shells",
      ),
      (
        ItemSpawnKind::SmallMedPack,
        ItemArchetype::SmallMedPack,
        "Small MedPack",
      ),
      (
        ItemSpawnKind::LargeMedPack,
        ItemArchetype::LargeMedPack,
        "Large MedPack",
      ),
      (
        ItemSpawnKind::GreenArmor,
        ItemArchetype::GreenArmor,
        "Green Armor",
      ),
      (
        ItemSpawnKind::PhaseDevice,
        ItemArchetype::PhaseDevice,
        "Phase Device",
      ),
    ];
    for (kind, archetype, name) in cases {
      let definition = definition_for_spawn_kind(kind);
      assert_eq!(definition.archetype, archetype);
      assert_eq!(definition.name, name);
      assert!(!definition.description.is_empty());
    }
  }

  #[test]
  fn definitions_preserve_current_item_properties() {
    let pistol = definition_for_spawn_kind(ItemSpawnKind::Pistol);
    assert_eq!(pistol.archetype, ItemArchetype::Pistol);
    assert_eq!(pistol.name, "Pistol");
    assert_eq!(
      pistol.kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Ammo9mm),
        clip_capacity: 10,
        damage: (4, 8),
        range: 8,
        accuracy: 75,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );

    let shotgun = definition_for_spawn_kind(ItemSpawnKind::Shotgun);
    assert_eq!(shotgun.archetype, ItemArchetype::Shotgun);
    assert_eq!(
      shotgun.kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Shells),
        clip_capacity: 8,
        damage: (8, 16),
        range: 5,
        accuracy: 65,
        knockback: 1,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::new(1200),
      }
    );

    let knife = definition_for_spawn_kind(ItemSpawnKind::CombatKnife);
    assert_eq!(knife.archetype, ItemArchetype::CombatKnife);
    assert_eq!(
      knife.kind,
      ItemDefinitionKind::Weapon {
        is_ranged: false,
        ammo_type: None,
        clip_capacity: 0,
        damage: (5, 9),
        range: 1,
        accuracy: 85,
        knockback: 0,
        fire_cost: ActionCost::MELEE_ATTACK,
        reload_cost: ActionCost::new(0),
      }
    );

    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::SmallMedPack).kind,
      ItemDefinitionKind::MedPack { heal_amount: 10 }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::LargeMedPack).kind,
      ItemDefinitionKind::MedPack { heal_amount: 25 }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::GreenArmor).kind,
      ItemDefinitionKind::Armor {
        protection: 5,
        durability: 100,
        max_durability: 100,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::PhaseDevice).kind,
      ItemDefinitionKind::PhaseDevice
    );
  }

  #[test]
  fn ammunition_definitions_keep_family_stack_policies() {
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Ammo9mm(7)).kind,
      ItemDefinitionKind::Ammo {
        ammo_type: AmmoType::Ammo9mm,
        max_stack: 100,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::AmmoShells(7)).kind,
      ItemDefinitionKind::Ammo {
        ammo_type: AmmoType::Shells,
        max_stack: 50,
      }
    );
  }
}
