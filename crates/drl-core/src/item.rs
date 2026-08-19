//! Item domain models, weapon properties, armor, ammunition, and consumables.

use drl_protocol::{
  ActionCost, AmmoType, EquipmentSlot, ItemCategory, ItemId, ItemSpawnKind, ItemView,
};

/// Physical properties for a weapon instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeaponProperties {
  pub is_ranged: bool,
  pub ammo_type: Option<AmmoType>,
  pub clip_capacity: u32,
  pub current_clip: u32,
  pub damage: (u32, u32),
  pub range: u32,
  pub accuracy: i32,
  pub knockback: u32,
  pub fire_cost: ActionCost,
  pub reload_cost: ActionCost,
}

/// Physical properties for wearable body armor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmorProperties {
  pub protection: u32,
  pub durability: u32,
  pub max_durability: u32,
}

/// Physical properties for consumable medical supplies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableProperties {
  pub heal_amount: u32,
}

/// Item classification and payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
  /// Weapon item (melee or ranged).
  Weapon(WeaponProperties),
  /// Protective body armor.
  Armor(ArmorProperties),
  /// Stackable ammunition.
  Ammo {
    ammo_type: AmmoType,
    count: u32,
    max_stack: u32,
  },
  /// Usable medical supply.
  MedPack(ConsumableProperties),
  /// Special consumable device (Phase Device).
  PhaseDevice,
}

/// Physical item instance in the simulation world or actor inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
  id: ItemId,
  name: String,
  description: String,
  kind: ItemKind,
}

impl Item {
  /// Constructs a generic item.
  #[must_use]
  pub fn new(
    id: ItemId,
    name: impl Into<String>,
    description: impl Into<String>,
    kind: ItemKind,
  ) -> Self {
    Self {
      id,
      name: name.into(),
      description: description.into(),
      kind,
    }
  }

  /// Returns the unique `ItemId`.
  #[must_use]
  pub const fn id(&self) -> ItemId {
    self.id
  }

  /// Returns the item's display name.
  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Returns the item description.
  #[must_use]
  pub fn description(&self) -> &str {
    &self.description
  }

  /// Reference to the item's kind.
  #[must_use]
  pub const fn kind(&self) -> &ItemKind {
    &self.kind
  }

  /// Mutable reference to the item's kind.
  pub fn kind_mut(&mut self) -> &mut ItemKind {
    &mut self.kind
  }

  /// Returns the semantic category of this item.
  #[must_use]
  pub const fn category(&self) -> ItemCategory {
    match &self.kind {
      ItemKind::Weapon(_) => ItemCategory::Weapon,
      ItemKind::Armor(_) => ItemCategory::Armor,
      ItemKind::Ammo { .. } => ItemCategory::Ammo,
      ItemKind::MedPack(_) => ItemCategory::MedPack,
      ItemKind::PhaseDevice => ItemCategory::PhaseDevice,
    }
  }

  /// Returns true if this item is a weapon.
  #[must_use]
  pub const fn is_weapon(&self) -> bool {
    matches!(&self.kind, ItemKind::Weapon(_))
  }

  /// Returns true if this item is armor.
  #[must_use]
  pub const fn is_armor(&self) -> bool {
    matches!(&self.kind, ItemKind::Armor(_))
  }

  /// Returns true if this item is ammunition.
  #[must_use]
  pub const fn is_ammo(&self) -> bool {
    matches!(&self.kind, ItemKind::Ammo { .. })
  }

  /// Returns true if this item is consumable (MedPack or PhaseDevice).
  #[must_use]
  pub const fn is_consumable(&self) -> bool {
    matches!(&self.kind, ItemKind::MedPack(_) | ItemKind::PhaseDevice)
  }

  /// Returns true if this item is a Phase Device.
  #[must_use]
  pub const fn is_phase_device(&self) -> bool {
    matches!(&self.kind, ItemKind::PhaseDevice)
  }

  /// Returns the equipment slot if this item is equippable.
  #[must_use]
  pub const fn equipment_slot(&self) -> Option<EquipmentSlot> {
    match &self.kind {
      ItemKind::Weapon(_) => Some(EquipmentSlot::Weapon),
      ItemKind::Armor(_) => Some(EquipmentSlot::Armor),
      _ => None,
    }
  }

  /// Returns weapon properties if this item is a weapon.
  #[must_use]
  pub const fn weapon_properties(&self) -> Option<&WeaponProperties> {
    match &self.kind {
      ItemKind::Weapon(props) => Some(props),
      _ => None,
    }
  }

  /// Returns mutable weapon properties if this item is a weapon.
  pub fn weapon_properties_mut(&mut self) -> Option<&mut WeaponProperties> {
    match &mut self.kind {
      ItemKind::Weapon(props) => Some(props),
      _ => None,
    }
  }

  /// Returns armor properties if this item is armor.
  #[must_use]
  pub const fn armor_properties(&self) -> Option<&ArmorProperties> {
    match &self.kind {
      ItemKind::Armor(props) => Some(props),
      _ => None,
    }
  }

  /// Returns consumable properties if this item is a MedPack.
  #[must_use]
  pub const fn consumable_properties(&self) -> Option<&ConsumableProperties> {
    match &self.kind {
      ItemKind::MedPack(props) => Some(props),
      _ => None,
    }
  }

  /// Returns the ammo type if this item is an ammunition stack.
  #[must_use]
  pub const fn ammo_type(&self) -> Option<AmmoType> {
    match &self.kind {
      ItemKind::Ammo { ammo_type, .. } => Some(*ammo_type),
      _ => None,
    }
  }

  /// Returns the count / stack size.
  #[must_use]
  pub const fn count(&self) -> u32 {
    match &self.kind {
      ItemKind::Ammo { count, .. } => *count,
      _ => 1,
    }
  }

  /// Deducts ammunition from this ammo stack. Returns the amount actually deducted.
  pub fn spend_ammo(&mut self, amount: u32) -> u32 {
    match &mut self.kind {
      ItemKind::Ammo { count, .. } => {
        let spent = (*count).min(amount);
        *count -= spent;
        spent
      }
      _ => 0,
    }
  }

  /// Adds ammunition to this stack up to `max_stack`. Returns the amount added.
  pub fn add_ammo(&mut self, amount: u32) -> u32 {
    match &mut self.kind {
      ItemKind::Ammo {
        count, max_stack, ..
      } => {
        let space = max_stack.saturating_sub(*count);
        let added = space.min(amount);
        *count += added;
        added
      }
      _ => 0,
    }
  }

  /// Loads ammo into a weapon's clip up to its capacity. Returns the amount loaded.
  pub fn load_ammo_into_clip(&mut self, amount: u32) -> u32 {
    if let Some(props) = self.weapon_properties_mut() {
      let needed = props.clip_capacity.saturating_sub(props.current_clip);
      let loaded = needed.min(amount);
      props.current_clip += loaded;
      loaded
    } else {
      0
    }
  }

  /// Consumes 1 or more rounds from a weapon's clip. Returns true if successful.
  pub fn consume_clip_ammo(&mut self, amount: u32) -> bool {
    if let Some(props) = self.weapon_properties_mut() {
      if props.current_clip >= amount {
        props.current_clip -= amount;
        true
      } else {
        false
      }
    } else {
      false
    }
  }

  /// Converts this item into an immutable `ItemView` for observations.
  #[must_use]
  pub fn to_view(&self) -> ItemView {
    let (clip, damage, armor_val, heal_val, knockback) = match &self.kind {
      ItemKind::Weapon(props) => (
        if props.is_ranged {
          Some((props.current_clip, props.clip_capacity))
        } else {
          None
        },
        Some(props.damage),
        None,
        None,
        if props.knockback > 0 {
          Some(props.knockback)
        } else {
          None
        },
      ),
      ItemKind::Armor(props) => (None, None, Some(props.protection), None, None),
      ItemKind::Ammo { .. } | ItemKind::PhaseDevice => (None, None, None, None, None),
      ItemKind::MedPack(props) => (None, None, None, Some(props.heal_amount), None),
    };

    ItemView {
      id: self.id,
      name: self.name.clone(),
      category: self.category(),
      count: self.count(),
      description: self.description.clone(),
      clip,
      damage,
      armor_value: armor_val,
      heal_amount: heal_val,
      knockback,
    }
  }

  // --- Factory constructors for representative DRL items ---

  /// Factory: standard 9mm Pistol.
  #[must_use]
  pub fn pistol(id: ItemId) -> Self {
    Self::new(
      id,
      "Pistol",
      "Standard 9mm military sidearm. Reliable and accurate.",
      ItemKind::Weapon(WeaponProperties {
        is_ranged: true,
        ammo_type: Some(AmmoType::Ammo9mm),
        clip_capacity: 10,
        current_clip: 10,
        damage: (4, 8),
        range: 8,
        accuracy: 75,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }),
    )
  }

  /// Factory: standard pump-action Shotgun.
  #[must_use]
  pub fn shotgun(id: ItemId) -> Self {
    Self::new(
      id,
      "Shotgun",
      "Pump-action 12-gauge shotgun. Devastating at close range.",
      ItemKind::Weapon(WeaponProperties {
        is_ranged: true,
        ammo_type: Some(AmmoType::Shells),
        clip_capacity: 8,
        current_clip: 8,
        damage: (8, 16),
        range: 5,
        accuracy: 65,
        knockback: 1,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::new(1200),
      }),
    )
  }

  /// Factory: Combat Knife melee weapon.
  #[must_use]
  pub fn combat_knife(id: ItemId) -> Self {
    Self::new(
      id,
      "Combat Knife",
      "Serrated combat blade for close-quarters fighting.",
      ItemKind::Weapon(WeaponProperties {
        is_ranged: false,
        ammo_type: None,
        clip_capacity: 0,
        current_clip: 0,
        damage: (5, 9),
        range: 1,
        accuracy: 85,
        knockback: 0,
        fire_cost: ActionCost::MELEE_ATTACK,
        reload_cost: ActionCost::new(0),
      }),
    )
  }

  /// Factory: 9mm ammunition box.
  #[must_use]
  pub fn ammo_9mm(id: ItemId, count: u32) -> Self {
    Self::new(
      id,
      "9mm Ammo",
      "Standard magazine rounds for 9mm pistols and submachine guns.",
      ItemKind::Ammo {
        ammo_type: AmmoType::Ammo9mm,
        count,
        max_stack: 100,
      },
    )
  }

  /// Factory: shotgun shells box.
  #[must_use]
  pub fn ammo_shells(id: ItemId, count: u32) -> Self {
    Self::new(
      id,
      "Shotgun Shells",
      "Heavy buckshot shells for shotguns.",
      ItemKind::Ammo {
        ammo_type: AmmoType::Shells,
        count,
        max_stack: 50,
      },
    )
  }

  /// Factory: Small MedPack (+10 HP).
  #[must_use]
  pub fn small_medpack(id: ItemId) -> Self {
    Self::new(
      id,
      "Small MedPack",
      "Compact medical kit providing rapid first aid (+10 HP).",
      ItemKind::MedPack(ConsumableProperties { heal_amount: 10 }),
    )
  }

  /// Factory: Large MedPack (+25 HP).
  #[must_use]
  pub fn large_medpack(id: ItemId) -> Self {
    Self::new(
      id,
      "Large MedPack",
      "Comprehensive field surgery kit (+25 HP).",
      ItemKind::MedPack(ConsumableProperties { heal_amount: 25 }),
    )
  }

  /// Factory: Green Armor (+5 armor protection, 100 durability).
  #[must_use]
  pub fn green_armor(id: ItemId) -> Self {
    Self::new(
      id,
      "Green Armor",
      "Standard security armor suit absorbing incoming damage.",
      ItemKind::Armor(ArmorProperties {
        protection: 5,
        durability: 100,
        max_durability: 100,
      }),
    )
  }

  /// Factory: Phase Device (emergency teleportation consumable).
  #[must_use]
  pub fn phase_device(id: ItemId) -> Self {
    Self::new(
      id,
      "Phase Device",
      "Emergency phase-shift device. Instantly teleports the user across space.",
      ItemKind::PhaseDevice,
    )
  }

  /// Instantiates an `Item` from an `ItemSpawnKind`.
  #[must_use]
  pub fn from_spawn_kind(id: ItemId, kind: ItemSpawnKind) -> Self {
    match kind {
      ItemSpawnKind::Pistol => Self::pistol(id),
      ItemSpawnKind::Shotgun => Self::shotgun(id),
      ItemSpawnKind::CombatKnife => Self::combat_knife(id),
      ItemSpawnKind::Ammo9mm(count) => Self::ammo_9mm(id, count),
      ItemSpawnKind::AmmoShells(count) => Self::ammo_shells(id, count),
      ItemSpawnKind::SmallMedPack => Self::small_medpack(id),
      ItemSpawnKind::LargeMedPack => Self::large_medpack(id),
      ItemSpawnKind::GreenArmor => Self::green_armor(id),
      ItemSpawnKind::PhaseDevice => Self::phase_device(id),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_pistol_properties_and_clip_consumption() {
    let mut pistol = Item::pistol(ItemId::new(1));
    assert!(pistol.is_weapon());
    assert_eq!(pistol.category(), ItemCategory::Weapon);
    assert_eq!(pistol.equipment_slot(), Some(EquipmentSlot::Weapon));

    let view = pistol.to_view();
    assert_eq!(view.name, "Pistol");
    assert_eq!(view.clip, Some((10, 10)));
    assert_eq!(view.damage, Some((4, 8)));

    assert!(pistol.consume_clip_ammo(1));
    assert_eq!(pistol.weapon_properties().unwrap().current_clip, 9);

    assert_eq!(pistol.load_ammo_into_clip(5), 1);
    assert_eq!(pistol.weapon_properties().unwrap().current_clip, 10);
  }

  #[test]
  fn test_ammo_stacking_and_spending() {
    let mut ammo = Item::ammo_9mm(ItemId::new(2), 20);
    assert!(ammo.is_ammo());
    assert_eq!(ammo.count(), 20);
    assert_eq!(ammo.ammo_type(), Some(AmmoType::Ammo9mm));

    let spent = ammo.spend_ammo(5);
    assert_eq!(spent, 5);
    assert_eq!(ammo.count(), 15);

    let added = ammo.add_ammo(10);
    assert_eq!(added, 10);
    assert_eq!(ammo.count(), 25);
  }

  #[test]
  fn test_medpack_and_armor_views() {
    let med = Item::small_medpack(ItemId::new(3));
    assert!(med.is_consumable());
    assert_eq!(med.to_view().heal_amount, Some(10));

    let armor = Item::green_armor(ItemId::new(4));
    assert!(armor.is_armor());
    assert_eq!(armor.equipment_slot(), Some(EquipmentSlot::Armor));
    assert_eq!(armor.to_view().armor_value, Some(5));

    let device = Item::phase_device(ItemId::new(5));
    assert!(device.is_consumable());
    assert!(device.is_phase_device());
    assert_eq!(device.category(), ItemCategory::PhaseDevice);
    assert_eq!(device.to_view().name, "Phase Device");
  }
}
