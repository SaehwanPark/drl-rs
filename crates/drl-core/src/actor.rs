//! Actor entities representing creatures and the player character.

use drl_protocol::{ActionCost, ActorView, EntityId, HitPoints, Position, Speed};

use crate::inventory::{Equipment, Inventory};
use crate::item::Item;

/// Simulation actor instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
  id: EntityId,
  position: Position,
  name: String,
  is_player: bool,
  blocks_movement: bool,
  hp: HitPoints,
  speed: Speed,
  energy: i32,
  is_alive: bool,
  melee_damage: (u32, u32),
  ranged_damage: Option<(u32, u32)>,
  ranged_range: u32,
  accuracy: i32,
  inventory: Inventory,
  equipment: Equipment,
}

impl Actor {
  /// Creates a new default actor.
  #[must_use]
  pub fn new(id: EntityId, position: Position, name: impl Into<String>, is_player: bool) -> Self {
    let max_hp = if is_player { 50 } else { 20 };
    Self {
      id,
      position,
      name: name.into(),
      is_player,
      blocks_movement: true,
      hp: HitPoints::full(max_hp),
      speed: Speed::NORMAL,
      energy: 0,
      is_alive: true,
      melee_damage: if is_player { (3, 6) } else { (2, 4) },
      ranged_damage: if is_player { Some((4, 8)) } else { None },
      ranged_range: if is_player { 8 } else { 0 },
      accuracy: 75,
      inventory: Inventory::default(),
      equipment: Equipment::new(),
    }
  }

  /// Builder for configuring custom combat stats.
  #[must_use]
  pub fn with_stats(
    mut self,
    hp: HitPoints,
    speed: Speed,
    melee_damage: (u32, u32),
    ranged_damage: Option<(u32, u32)>,
    ranged_range: u32,
    accuracy: i32,
  ) -> Self {
    self.hp = hp;
    self.speed = speed;
    self.melee_damage = melee_damage;
    self.ranged_damage = ranged_damage;
    self.ranged_range = ranged_range;
    self.accuracy = accuracy;
    self
  }

  /// Returns the actor's unique EntityId.
  #[must_use]
  pub const fn id(&self) -> EntityId {
    self.id
  }

  /// Returns the actor's current grid position.
  #[must_use]
  pub const fn position(&self) -> Position {
    self.position
  }

  /// Updates the actor's grid position.
  pub fn set_position(&mut self, pos: Position) {
    self.position = pos;
  }

  /// Returns the actor's display name.
  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Returns true if this actor is the player character.
  #[must_use]
  pub const fn is_player(&self) -> bool {
    self.is_player
  }

  /// Returns true if this actor blocks movement into its tile.
  #[must_use]
  pub const fn blocks_movement(&self) -> bool {
    self.is_alive && self.blocks_movement
  }

  /// Sets whether this actor blocks movement.
  pub fn set_blocks_movement(&mut self, blocks: bool) {
    self.blocks_movement = blocks;
  }

  /// Current hit points.
  #[must_use]
  pub const fn hp(&self) -> HitPoints {
    self.hp
  }

  /// Mutable reference to hit points.
  pub fn hp_mut(&mut self) -> &mut HitPoints {
    &mut self.hp
  }

  /// Returns true if the actor is alive.
  #[must_use]
  pub const fn is_alive(&self) -> bool {
    self.is_alive
  }

  /// Applies damage to this actor, mitigated by equipped armor protection.
  /// Returns actual damage taken and whether the blow was lethal.
  pub fn take_damage(&mut self, raw_amount: u32) -> (u32, bool) {
    if !self.is_alive {
      return (0, false);
    }
    let armor_prot = self.armor_protection();
    let net_amount = raw_amount.saturating_sub(armor_prot).max(1);
    let taken = self.hp.take_damage(net_amount);
    if self.hp.is_dead() {
      self.is_alive = false;
      self.blocks_movement = false;
      (taken, true)
    } else {
      (taken, false)
    }
  }

  /// Heals this actor up to maximum HP. Returns actual health restored.
  pub fn heal(&mut self, amount: u32) -> u32 {
    if !self.is_alive {
      return 0;
    }
    self.hp.heal(amount)
  }

  /// Speed of this actor.
  #[must_use]
  pub const fn speed(&self) -> Speed {
    self.speed
  }

  /// Sets the actor's speed.
  pub fn set_speed(&mut self, speed: Speed) {
    self.speed = speed;
  }

  /// Current energy balance.
  #[must_use]
  pub const fn energy(&self) -> i32 {
    self.energy
  }

  /// Sets current energy balance.
  pub fn set_energy(&mut self, energy: i32) {
    self.energy = energy;
  }

  /// Accumulates energy.
  pub fn add_energy(&mut self, amount: i32) {
    self.energy += amount;
  }

  /// Deducts action cost from energy.
  pub fn spend_energy(&mut self, cost: ActionCost) {
    self.energy -= cost.as_u32() as i32;
  }

  /// Reference to actor's inventory.
  #[must_use]
  pub const fn inventory(&self) -> &Inventory {
    &self.inventory
  }

  /// Mutable reference to actor's inventory.
  pub fn inventory_mut(&mut self) -> &mut Inventory {
    &mut self.inventory
  }

  /// Reference to actor's equipment.
  #[must_use]
  pub const fn equipment(&self) -> &Equipment {
    &self.equipment
  }

  /// Mutable reference to actor's equipment.
  pub fn equipment_mut(&mut self) -> &mut Equipment {
    &mut self.equipment
  }

  /// Returns equipped armor protection value, if any.
  #[must_use]
  pub fn armor_protection(&self) -> u32 {
    self
      .equipment
      .armor()
      .and_then(|a| a.armor_properties())
      .map_or(0, |p| p.protection)
  }

  /// Melee damage range `(min, max)`, factoring in equipped weapon.
  #[must_use]
  pub fn melee_damage(&self) -> (u32, u32) {
    if let Some(props) = self.equipment.weapon().and_then(Item::weapon_properties)
      && !props.is_ranged
    {
      props.damage
    } else {
      self.melee_damage
    }
  }

  /// Ranged damage range `(min, max)` if equipped with ranged weapon.
  #[must_use]
  pub fn ranged_damage(&self) -> Option<(u32, u32)> {
    if let Some(props) = self.equipment.weapon().and_then(Item::weapon_properties)
      && props.is_ranged
    {
      Some(props.damage)
    } else {
      self.ranged_damage
    }
  }

  /// Maximum ranged attack distance.
  #[must_use]
  pub fn ranged_range(&self) -> u32 {
    if let Some(props) = self.equipment.weapon().and_then(Item::weapon_properties)
      && props.is_ranged
    {
      props.range
    } else {
      self.ranged_range
    }
  }

  /// Base accuracy percentage (e.g. 75).
  #[must_use]
  pub fn accuracy(&self) -> i32 {
    if let Some(props) = self.equipment.weapon().and_then(Item::weapon_properties) {
      props.accuracy
    } else {
      self.accuracy
    }
  }

  /// Converts this actor to an immutable `ActorView` for observations.
  #[must_use]
  pub fn to_view(&self) -> ActorView {
    ActorView {
      id: self.id,
      position: self.position,
      is_player: self.is_player,
      name: self.name.clone(),
      hp: Some(self.hp),
      is_alive: self.is_alive,
      speed: self.speed,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::item::Item;
  use drl_protocol::{EquipmentSlot, ItemId};

  #[test]
  fn test_actor_creation_and_view() {
    let actor = Actor::new(EntityId::new(1), Position::new(3, 4), "Marine", true);
    assert_eq!(actor.id(), EntityId::new(1));
    assert_eq!(actor.position(), Position::new(3, 4));
    assert!(actor.is_player());
    assert!(actor.blocks_movement());
    assert!(actor.is_alive());
    assert_eq!(actor.hp().current, 50);

    let view = actor.to_view();
    assert_eq!(view.id, EntityId::new(1));
    assert_eq!(view.name, "Marine");
    assert_eq!(view.hp, Some(HitPoints::full(50)));
    assert!(view.is_alive);
  }

  #[test]
  fn test_actor_damage_and_death() {
    let mut actor = Actor::new(EntityId::new(2), Position::new(1, 1), "Former Human", false);
    assert_eq!(actor.hp().current, 20);

    let (taken, lethal) = actor.take_damage(15);
    assert_eq!(taken, 15);
    assert!(!lethal);
    assert!(actor.is_alive());
    assert_eq!(actor.hp().current, 5);

    let (taken2, lethal2) = actor.take_damage(10);
    assert_eq!(taken2, 5);
    assert!(lethal2);
    assert!(!actor.is_alive());
    assert!(!actor.blocks_movement());
    assert!(actor.hp().is_dead());
  }

  #[test]
  fn test_actor_equipment_and_armor_protection() {
    let mut actor = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true);
    assert_eq!(actor.armor_protection(), 0);

    let armor = Item::green_armor(ItemId::new(10));
    actor
      .equipment_mut()
      .equip(EquipmentSlot::Armor, armor)
      .unwrap();
    assert_eq!(actor.armor_protection(), 5);

    // Damage of 10 mitigated by 5 armor = 5 net damage
    let (taken, _) = actor.take_damage(10);
    assert_eq!(taken, 5);
    assert_eq!(actor.hp().current, 45);
  }

  #[test]
  fn test_actor_energy_spending() {
    let mut actor = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true);
    actor.set_energy(1500);
    actor.spend_energy(ActionCost::MOVE);
    assert_eq!(actor.energy(), 500);
  }
}
