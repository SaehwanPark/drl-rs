//! Item domain models, weapon properties, armor, ammunition, and consumables.

use drl_protocol::{
  ActionCost, AmmoType, EquipmentSlot, ItemArchetype, ItemCategory, ItemId, ItemSpawnKind, ItemView,
};

use crate::item_definition::{ItemDefinitionKind, definition_for_spawn_kind};

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

/// Fixed payload for a prepared-slot ammunition pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmmoPackProperties {
  pub ammo_type: AmmoType,
  pub amount: u32,
  pub max_amount: u32,
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
  /// Prepared-slot ammunition pack; use behavior is intentionally deferred.
  AmmoPack(AmmoPackProperties),
  /// Usable medical supply.
  MedPack(ConsumableProperties),
  /// Special consumable device (Phase Device).
  PhaseDevice,
}

/// Physical item instance in the simulation world or actor inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
  id: ItemId,
  archetype: ItemArchetype,
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
      archetype: ItemArchetype::Unknown,
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

  /// Assigns the stable presentation archetype used by a factory item.
  #[must_use]
  fn with_archetype(mut self, archetype: ItemArchetype) -> Self {
    self.archetype = archetype;
    self
  }

  /// Returns the semantic category of this item.
  #[must_use]
  pub const fn category(&self) -> ItemCategory {
    match &self.kind {
      ItemKind::Weapon(_) => ItemCategory::Weapon,
      ItemKind::Armor(_) => ItemCategory::Armor,
      ItemKind::Ammo { .. } => ItemCategory::Ammo,
      ItemKind::AmmoPack(_) => ItemCategory::AmmoPack,
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

  /// Returns true if this item is a prepared-slot ammunition pack.
  #[must_use]
  pub const fn is_ammo_pack(&self) -> bool {
    matches!(&self.kind, ItemKind::AmmoPack(_))
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

  /// Returns prepared-slot pack payload when this item is an ammo pack.
  #[must_use]
  pub const fn ammo_pack_properties(&self) -> Option<&AmmoPackProperties> {
    match &self.kind {
      ItemKind::AmmoPack(properties) => Some(properties),
      _ => None,
    }
  }

  /// Returns the count / stack size.
  #[must_use]
  pub const fn count(&self) -> u32 {
    match &self.kind {
      ItemKind::Ammo { count, .. } => *count,
      ItemKind::AmmoPack(properties) => properties.amount,
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
      ItemKind::Ammo { .. } | ItemKind::AmmoPack(_) | ItemKind::PhaseDevice => {
        (None, None, None, None, None)
      }
      ItemKind::MedPack(props) => (None, None, None, Some(props.heal_amount), None),
    };

    ItemView {
      id: self.id,
      archetype: self.archetype(),
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

  /// Returns the stable presentation archetype for this item instance.
  #[must_use]
  pub fn archetype(&self) -> ItemArchetype {
    self.archetype
  }

  // --- Factory constructors for representative DRL items ---

  /// Factory: standard 9mm Pistol.
  #[must_use]
  pub fn pistol(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::Pistol)
  }

  /// Factory: standard pump-action Shotgun.
  #[must_use]
  pub fn shotgun(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::Shotgun)
  }

  /// Factory: chaingun (40-round 9mm clip, 1d6 damage policy).
  #[must_use]
  pub fn chaingun(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::Chaingun)
  }

  /// Factory: plasma rifle (six-cell clip, 1d7 damage policy).
  #[must_use]
  pub fn plasma_rifle(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::PlasmaRifle)
  }

  /// Factory: rocket launcher (one-rocket clip, 6d6 damage policy).
  #[must_use]
  pub fn rocket_launcher(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::RocketLauncher)
  }

  /// Factory: chainsaw (melee, 4d6 damage policy).
  #[must_use]
  pub fn chainsaw(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::Chainsaw)
  }

  /// Factory: BFG 9000 (100-cell clip, 10d6 damage policy).
  #[must_use]
  pub fn bfg9000(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::Bfg9000)
  }

  /// Factory: Combat Knife melee weapon.
  #[must_use]
  pub fn combat_knife(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::CombatKnife)
  }

  /// Factory: 9mm ammunition box.
  #[must_use]
  pub fn ammo_9mm(id: ItemId, count: u32) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::Ammo9mm(count))
  }

  /// Factory: shotgun shells box.
  #[must_use]
  pub fn ammo_shells(id: ItemId, count: u32) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::AmmoShells(count))
  }

  /// Factory: rocket ammunition box for the prepared slot.
  #[must_use]
  pub fn ammo_pack_rockets(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::AmmoPackRockets)
  }

  /// Factory: power-cell ammunition box for the prepared slot.
  #[must_use]
  pub fn ammo_pack_cells(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::AmmoPackCells)
  }

  /// Factory: 10mm ammunition chain for the prepared slot.
  #[must_use]
  pub fn ammo_pack_9mm(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::AmmoPack9mm)
  }

  /// Factory: shell box for the prepared slot.
  #[must_use]
  pub fn ammo_pack_shells(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::AmmoPackShells)
  }

  /// Factory: rocket ammunition box.
  #[must_use]
  pub fn ammo_rockets(id: ItemId, count: u32) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::AmmoRockets(count))
  }

  /// Factory: power-cell ammunition box.
  #[must_use]
  pub fn ammo_cells(id: ItemId, count: u32) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::AmmoCells(count))
  }

  /// Factory: Small MedPack (+10 HP).
  #[must_use]
  pub fn small_medpack(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::SmallMedPack)
  }

  /// Factory: Large MedPack (+25 HP).
  #[must_use]
  pub fn large_medpack(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::LargeMedPack)
  }

  /// Factory: Green Armor (+5 armor protection, 100 durability).
  #[must_use]
  pub fn green_armor(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::GreenArmor)
  }

  /// Factory: blue armor (+2 armor protection, 100 durability).
  #[must_use]
  pub fn blue_armor(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::BlueArmor)
  }

  /// Factory: red armor (+4 armor protection, 100 durability).
  #[must_use]
  pub fn red_armor(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::RedArmor)
  }

  /// Factory: Phase Device (emergency teleportation consumable).
  #[must_use]
  pub fn phase_device(id: ItemId) -> Self {
    Self::from_spawn_kind(id, ItemSpawnKind::PhaseDevice)
  }

  /// Instantiates an `Item` from an `ItemSpawnKind`.
  #[must_use]
  pub fn from_spawn_kind(id: ItemId, kind: ItemSpawnKind) -> Self {
    let count = match kind {
      ItemSpawnKind::Ammo9mm(count)
      | ItemSpawnKind::AmmoShells(count)
      | ItemSpawnKind::AmmoRockets(count)
      | ItemSpawnKind::AmmoCells(count) => count,
      _ => 1,
    };
    let definition = definition_for_spawn_kind(kind);
    let item_kind = match definition.kind {
      ItemDefinitionKind::Weapon {
        is_ranged,
        ammo_type,
        clip_capacity,
        damage,
        range,
        accuracy,
        knockback,
        fire_cost,
        reload_cost,
      } => ItemKind::Weapon(WeaponProperties {
        is_ranged,
        ammo_type,
        clip_capacity,
        current_clip: clip_capacity,
        damage,
        range,
        accuracy,
        knockback,
        fire_cost,
        reload_cost,
      }),
      ItemDefinitionKind::Ammo {
        ammo_type,
        max_stack,
        ..
      } => ItemKind::Ammo {
        ammo_type,
        count,
        max_stack,
      },
      ItemDefinitionKind::AmmoPack {
        ammo_type,
        amount,
        max_amount,
      } => ItemKind::AmmoPack(AmmoPackProperties {
        ammo_type,
        amount,
        max_amount,
      }),
      ItemDefinitionKind::MedPack { heal_amount } => {
        ItemKind::MedPack(ConsumableProperties { heal_amount })
      }
      ItemDefinitionKind::Armor {
        protection,
        durability,
        max_durability,
      } => ItemKind::Armor(ArmorProperties {
        protection,
        durability,
        max_durability,
      }),
      ItemDefinitionKind::PhaseDevice => ItemKind::PhaseDevice,
    };
    Self::new(id, definition.name, definition.description, item_kind)
      .with_archetype(definition.archetype)
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
    assert_eq!(view.archetype, drl_protocol::ItemArchetype::Pistol);
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

    let pack = Item::ammo_pack_rockets(ItemId::new(5));
    assert!(pack.is_ammo_pack());
    assert!(!pack.is_ammo());
    assert_eq!(pack.category(), ItemCategory::AmmoPack);
    assert_eq!(pack.count(), 25);
    assert_eq!(
      pack
        .ammo_pack_properties()
        .map(|props| (props.ammo_type, props.amount, props.max_amount)),
      Some((AmmoType::Rocket, 25, 25))
    );
    assert_eq!(pack.ammo_type(), None);

    let cell_pack = Item::ammo_pack_cells(ItemId::new(6));
    assert!(cell_pack.is_ammo_pack());
    assert_eq!(cell_pack.count(), 120);
    assert_eq!(
      cell_pack.ammo_pack_properties().map(|props| (
        props.ammo_type,
        props.amount,
        props.max_amount
      )),
      Some((AmmoType::Cell, 120, 120))
    );

    let ammo_pack = Item::ammo_pack_9mm(ItemId::new(7));
    assert_eq!(ammo_pack.count(), 250);
    assert_eq!(
      ammo_pack.ammo_pack_properties().map(|props| (
        props.ammo_type,
        props.amount,
        props.max_amount
      )),
      Some((AmmoType::Ammo9mm, 250, 250))
    );
    let shell_pack = Item::ammo_pack_shells(ItemId::new(8));
    assert_eq!(shell_pack.count(), 100);
    assert_eq!(
      shell_pack.ammo_pack_properties().map(|props| (
        props.ammo_type,
        props.amount,
        props.max_amount
      )),
      Some((AmmoType::Shells, 100, 100))
    );

    let device = Item::phase_device(ItemId::new(9));
    assert!(device.is_consumable());
    assert!(device.is_phase_device());
    assert_eq!(device.category(), ItemCategory::PhaseDevice);
    assert_eq!(device.to_view().name, "Phase Device");
  }

  #[test]
  fn convenience_factories_match_definition_backed_factory() {
    let cases = [
      (ItemSpawnKind::Pistol, Item::pistol(ItemId::new(1))),
      (ItemSpawnKind::Shotgun, Item::shotgun(ItemId::new(2))),
      (ItemSpawnKind::Chaingun, Item::chaingun(ItemId::new(3))),
      (
        ItemSpawnKind::PlasmaRifle,
        Item::plasma_rifle(ItemId::new(4)),
      ),
      (
        ItemSpawnKind::RocketLauncher,
        Item::rocket_launcher(ItemId::new(5)),
      ),
      (ItemSpawnKind::Chainsaw, Item::chainsaw(ItemId::new(6))),
      (ItemSpawnKind::Bfg9000, Item::bfg9000(ItemId::new(7))),
      (
        ItemSpawnKind::CombatKnife,
        Item::combat_knife(ItemId::new(8)),
      ),
      (ItemSpawnKind::Ammo9mm(0), Item::ammo_9mm(ItemId::new(9), 0)),
      (
        ItemSpawnKind::Ammo9mm(107),
        Item::ammo_9mm(ItemId::new(10), 107),
      ),
      (
        ItemSpawnKind::AmmoShells(7),
        Item::ammo_shells(ItemId::new(11), 7),
      ),
      (
        ItemSpawnKind::AmmoRockets(3),
        Item::ammo_rockets(ItemId::new(12), 3),
      ),
      (
        ItemSpawnKind::AmmoCells(20),
        Item::ammo_cells(ItemId::new(13), 20),
      ),
      (
        ItemSpawnKind::AmmoPackRockets,
        Item::ammo_pack_rockets(ItemId::new(14)),
      ),
      (
        ItemSpawnKind::AmmoPackCells,
        Item::ammo_pack_cells(ItemId::new(15)),
      ),
      (
        ItemSpawnKind::AmmoPack9mm,
        Item::ammo_pack_9mm(ItemId::new(16)),
      ),
      (
        ItemSpawnKind::AmmoPackShells,
        Item::ammo_pack_shells(ItemId::new(17)),
      ),
      (
        ItemSpawnKind::SmallMedPack,
        Item::small_medpack(ItemId::new(18)),
      ),
      (
        ItemSpawnKind::LargeMedPack,
        Item::large_medpack(ItemId::new(19)),
      ),
      (
        ItemSpawnKind::GreenArmor,
        Item::green_armor(ItemId::new(20)),
      ),
      (ItemSpawnKind::BlueArmor, Item::blue_armor(ItemId::new(21))),
      (ItemSpawnKind::RedArmor, Item::red_armor(ItemId::new(22))),
      (
        ItemSpawnKind::PhaseDevice,
        Item::phase_device(ItemId::new(23)),
      ),
    ];
    for (kind, factory_item) in cases {
      let canonical = Item::from_spawn_kind(factory_item.id(), kind);
      assert_eq!(factory_item.to_view(), canonical.to_view());
      assert_eq!(factory_item.kind(), canonical.kind());
    }
  }

  #[test]
  fn definition_backed_items_preserve_ammo_counts_and_determinism() {
    let ammo_cases = [
      (ItemSpawnKind::Ammo9mm(0), 0, 100),
      (ItemSpawnKind::Ammo9mm(7), 7, 100),
      (ItemSpawnKind::Ammo9mm(101), 101, 100),
      (ItemSpawnKind::AmmoShells(0), 0, 50),
      (ItemSpawnKind::AmmoShells(7), 7, 50),
      (ItemSpawnKind::AmmoShells(51), 51, 50),
      (ItemSpawnKind::AmmoRockets(0), 0, 10),
      (ItemSpawnKind::AmmoRockets(3), 3, 10),
      (ItemSpawnKind::AmmoRockets(11), 11, 10),
      (ItemSpawnKind::AmmoCells(0), 0, 50),
      (ItemSpawnKind::AmmoCells(20), 20, 50),
      (ItemSpawnKind::AmmoCells(51), 51, 50),
    ];
    for (kind, expected_count, expected_max_stack) in ammo_cases {
      let item = Item::from_spawn_kind(ItemId::new(20), kind);
      match item.kind() {
        ItemKind::Ammo {
          count, max_stack, ..
        } => {
          assert_eq!(*count, expected_count);
          assert_eq!(*max_stack, expected_max_stack);
        }
        other => panic!("expected ammunition item, got {other:?}"),
      }
    }

    let kinds = [
      ItemSpawnKind::Pistol,
      ItemSpawnKind::Shotgun,
      ItemSpawnKind::Chaingun,
      ItemSpawnKind::PlasmaRifle,
      ItemSpawnKind::RocketLauncher,
      ItemSpawnKind::Chainsaw,
      ItemSpawnKind::Bfg9000,
      ItemSpawnKind::CombatKnife,
      ItemSpawnKind::Ammo9mm(7),
      ItemSpawnKind::AmmoShells(7),
      ItemSpawnKind::AmmoRockets(3),
      ItemSpawnKind::AmmoCells(20),
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
    ];
    for kind in kinds {
      let first = Item::from_spawn_kind(ItemId::new(21), kind);
      let second = Item::from_spawn_kind(ItemId::new(21), kind);
      assert_eq!(first.to_view(), second.to_view());
      assert_eq!(first.kind(), second.kind());
    }
  }
}
