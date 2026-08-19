//! Inventory management, ammo stack handling, and equipment slots.

use drl_protocol::{AmmoType, CommandError, EquipmentSlot, ItemCategory, ItemId, ItemView};
use std::collections::BTreeMap;

use crate::item::Item;

/// Default player backpack capacity.
pub const DEFAULT_INVENTORY_CAPACITY: usize = 10;

/// Player item inventory with bounded capacity and automatic ammo stacking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
  capacity: usize,
  items: BTreeMap<ItemId, Item>,
}

impl Inventory {
  /// Creates an inventory with the specified capacity limit.
  #[must_use]
  pub fn new(capacity: usize) -> Self {
    Self {
      capacity,
      items: BTreeMap::new(),
    }
  }

  /// Returns current item count.
  #[must_use]
  pub fn len(&self) -> usize {
    self.items.len()
  }

  /// Returns true if inventory contains no items.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.items.is_empty()
  }

  /// Maximum number of items the inventory can hold.
  #[must_use]
  pub const fn capacity(&self) -> usize {
    self.capacity
  }

  /// Returns true if inventory is at full capacity.
  #[must_use]
  pub fn is_full(&self) -> bool {
    self.items.len() >= self.capacity
  }

  /// Reference to the internal item map.
  #[must_use]
  pub const fn items(&self) -> &BTreeMap<ItemId, Item> {
    &self.items
  }

  /// Mutable reference to the internal item map.
  pub fn items_mut(&mut self) -> &mut BTreeMap<ItemId, Item> {
    &mut self.items
  }

  /// Retrieves an item by its unique `ItemId`.
  #[must_use]
  pub fn get_item(&self, id: ItemId) -> Option<&Item> {
    self.items.get(&id)
  }

  /// Retrieves a mutable reference to an item by its unique `ItemId`.
  pub fn get_item_mut(&mut self, id: ItemId) -> Option<&mut Item> {
    self.items.get_mut(&id)
  }

  /// Adds an item to the inventory.
  ///
  /// For ammunition, automatically attempts to merge into existing compatible ammo stacks.
  /// Returns the `ItemId` where the item (or its remaining stack) was placed.
  pub fn add_item(&mut self, mut item: Item) -> Result<ItemId, CommandError> {
    if item.is_ammo() {
      let ammo_type = item.ammo_type().unwrap();
      // Try to merge into existing ammo stacks of the same type
      for existing in self.items.values_mut() {
        if existing.ammo_type() == Some(ammo_type) {
          let space = existing.add_ammo(item.count());
          item.spend_ammo(space);
          if item.count() == 0 {
            return Ok(existing.id());
          }
        }
      }
    }

    if self.is_full() {
      return Err(CommandError::InventoryFull);
    }

    let id = item.id();
    self.items.insert(id, item);
    Ok(id)
  }

  /// Removes and returns an item from the inventory.
  pub fn remove_item(&mut self, id: ItemId) -> Result<Item, CommandError> {
    self.items.remove(&id).ok_or(CommandError::ItemNotFound(id))
  }

  /// Deducts a specified amount of ammunition from available stacks.
  ///
  /// Removes any stacks that become depleted (count reaches 0).
  /// Returns the actual amount of ammo taken.
  pub fn take_ammo(&mut self, ammo_type: AmmoType, needed: u32) -> u32 {
    let mut remaining = needed;
    let mut empty_ids = Vec::new();

    for (id, item) in &mut self.items {
      if item.ammo_type() == Some(ammo_type) {
        let taken = item.spend_ammo(remaining);
        remaining -= taken;
        if item.count() == 0 {
          empty_ids.push(*id);
        }
        if remaining == 0 {
          break;
        }
      }
    }

    for id in empty_ids {
      self.items.remove(&id);
    }

    needed - remaining
  }

  /// Finds the first item of a given category.
  #[must_use]
  pub fn find_first_by_category(&self, category: ItemCategory) -> Option<ItemId> {
    self
      .items
      .values()
      .find(|item| item.category() == category)
      .map(Item::id)
  }

  /// Returns the total quantity of ammo of the given type across all inventory stacks.
  #[must_use]
  pub fn total_ammo(&self, ammo_type: AmmoType) -> u32 {
    self
      .items
      .values()
      .filter(|item| item.ammo_type() == Some(ammo_type))
      .map(Item::count)
      .sum()
  }

  /// Returns true if inventory contains at least `count` rounds of `ammo_type`.
  #[must_use]
  pub fn has_ammo(&self, ammo_type: AmmoType, count: u32) -> bool {
    self.total_ammo(ammo_type) >= count
  }

  /// Converts all inventory items into observation views.
  #[must_use]
  pub fn to_views(&self) -> Vec<ItemView> {
    self.items.values().map(Item::to_view).collect()
  }
}

impl Default for Inventory {
  fn default() -> Self {
    Self::new(DEFAULT_INVENTORY_CAPACITY)
  }
}

/// Equipped weapon and body armor slots.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Equipment {
  weapon: Option<Item>,
  armor: Option<Item>,
}

impl Equipment {
  /// Creates an empty equipment layout.
  #[must_use]
  pub fn new() -> Self {
    Self {
      weapon: None,
      armor: None,
    }
  }

  /// Reference to the equipped weapon if any.
  #[must_use]
  pub const fn weapon(&self) -> Option<&Item> {
    self.weapon.as_ref()
  }

  /// Mutable reference to the equipped weapon if any.
  pub fn weapon_mut(&mut self) -> Option<&mut Item> {
    self.weapon.as_mut()
  }

  /// Reference to the equipped armor if any.
  #[must_use]
  pub const fn armor(&self) -> Option<&Item> {
    self.armor.as_ref()
  }

  /// Mutable reference to the equipped armor if any.
  pub fn armor_mut(&mut self) -> Option<&mut Item> {
    self.armor.as_mut()
  }

  /// Equips an item to its designated slot, returning any previously equipped item.
  pub fn equip(&mut self, slot: EquipmentSlot, item: Item) -> Result<Option<Item>, CommandError> {
    let item_slot = item
      .equipment_slot()
      .ok_or(CommandError::CannotEquip(item.id()))?;

    if item_slot != slot {
      return Err(CommandError::CannotEquip(item.id()));
    }

    match slot {
      EquipmentSlot::Weapon => Ok(self.weapon.replace(item)),
      EquipmentSlot::Armor => Ok(self.armor.replace(item)),
    }
  }

  /// Unequips and returns the item from a given equipment slot.
  pub fn unequip(&mut self, slot: EquipmentSlot) -> Result<Item, CommandError> {
    match slot {
      EquipmentSlot::Weapon => self.weapon.take().ok_or(CommandError::SlotEmpty(slot)),
      EquipmentSlot::Armor => self.armor.take().ok_or(CommandError::SlotEmpty(slot)),
    }
  }

  /// Observation view of the equipped weapon.
  #[must_use]
  pub fn weapon_view(&self) -> Option<ItemView> {
    self.weapon.as_ref().map(Item::to_view)
  }

  /// Observation view of the equipped armor.
  #[must_use]
  pub fn armor_view(&self) -> Option<ItemView> {
    self.armor.as_ref().map(Item::to_view)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_inventory_capacity_and_stacking() {
    let mut inv = Inventory::new(2);
    let ammo1 = Item::ammo_9mm(ItemId::new(1), 20);
    let id1 = inv.add_item(ammo1).unwrap();
    assert_eq!(id1, ItemId::new(1));
    assert_eq!(inv.len(), 1);

    // Adding more 9mm ammo merges into the first stack
    let ammo2 = Item::ammo_9mm(ItemId::new(2), 30);
    let id2 = inv.add_item(ammo2).unwrap();
    assert_eq!(id2, ItemId::new(1));
    assert_eq!(inv.len(), 1);
    assert_eq!(inv.get_item(ItemId::new(1)).unwrap().count(), 50);

    // Add small medpack (second slot)
    let med = Item::small_medpack(ItemId::new(3));
    inv.add_item(med).unwrap();
    assert_eq!(inv.len(), 2);
    assert!(inv.is_full());

    // Adding 3rd item fails due to capacity
    let knife = Item::combat_knife(ItemId::new(4));
    let err = inv.add_item(knife).unwrap_err();
    assert_eq!(err, CommandError::InventoryFull);
  }

  #[test]
  fn test_take_ammo_depletion() {
    let mut inv = Inventory::new(5);
    inv.add_item(Item::ammo_9mm(ItemId::new(1), 15)).unwrap();

    let taken = inv.take_ammo(AmmoType::Ammo9mm, 10);
    assert_eq!(taken, 10);
    assert_eq!(inv.get_item(ItemId::new(1)).unwrap().count(), 5);

    let taken2 = inv.take_ammo(AmmoType::Ammo9mm, 10);
    assert_eq!(taken2, 5);
    assert!(inv.is_empty());
  }

  #[test]
  fn test_equipment_swap_and_unequip() {
    let mut eq = Equipment::new();
    assert!(eq.weapon().is_none());

    let pistol = Item::pistol(ItemId::new(1));
    let prev = eq.equip(EquipmentSlot::Weapon, pistol).unwrap();
    assert!(prev.is_none());
    assert_eq!(eq.weapon().unwrap().name(), "Pistol");

    let shotgun = Item::shotgun(ItemId::new(2));
    let old_weapon = eq.equip(EquipmentSlot::Weapon, shotgun).unwrap();
    assert_eq!(old_weapon.unwrap().name(), "Pistol");
    assert_eq!(eq.weapon().unwrap().name(), "Shotgun");

    let unequipped = eq.unequip(EquipmentSlot::Weapon).unwrap();
    assert_eq!(unequipped.name(), "Shotgun");
    assert!(eq.weapon().is_none());
  }
}
