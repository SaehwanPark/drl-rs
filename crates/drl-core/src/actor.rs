use drl_protocol::{
  ActionCost, ActorView, DamageType, EntityId, HitPoints, ItemArchetype, ItemSpawnKind,
  MonsterKind, Position, Speed,
};

use crate::behavior::{LavaRechargeOutcome, MedicalRepairOutcome, WeaponRechargeOutcome};
use crate::inventory::{Equipment, Inventory};
use crate::item::Item;
use crate::malek_armor::MalekRechargeOutcome;
use crate::subtle_knife::{SubtleKnifeCost, SubtleKnifeError, SubtleKnifeTransition, TiredStatus};
use crate::trigun::{TrigunCost, TrigunError, TrigunTransition};

/// Simulation actor instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
  id: EntityId,
  position: Position,
  name: String,
  is_player: bool,
  blocks_movement: bool,
  hp: HitPoints,
  tired: TiredStatus,
  score_count: i32,
  speed: Speed,
  energy: i32,
  is_alive: bool,
  melee_damage: (u32, u32),
  ranged_damage: Option<(u32, u32)>,
  ranged_range: u32,
  accuracy: i32,
  knockback: u32,
  monster_kind: Option<MonsterKind>,
  is_boss: bool,
  death_drop: Option<ItemSpawnKind>,
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
      tired: TiredStatus::Ready,
      score_count: 0,
      speed: Speed::NORMAL,
      energy: 0,
      is_alive: true,
      melee_damage: if is_player { (3, 6) } else { (2, 4) },
      ranged_damage: if is_player { Some((4, 8)) } else { None },
      ranged_range: if is_player { 8 } else { 0 },
      accuracy: 75,
      knockback: 0,
      monster_kind: None,
      is_boss: false,
      death_drop: None,
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

  /// Sets the innate knockback power.
  #[must_use]
  pub fn with_knockback(mut self, knockback: u32) -> Self {
    self.knockback = knockback;
    self
  }

  /// Sets the monster archetype.
  #[must_use]
  pub fn with_monster_kind(mut self, kind: MonsterKind) -> Self {
    self.monster_kind = Some(kind);
    self
  }

  /// Marks this actor as a boss for target-dependent item behavior.
  #[must_use]
  pub fn with_boss(mut self, is_boss: bool) -> Self {
    self.is_boss = is_boss;
    self
  }

  /// Returns whether this actor has the explicit boss property.
  #[must_use]
  pub const fn is_boss(&self) -> bool {
    self.is_boss
  }

  /// Sets the explicit boss property for deterministic fixtures.
  pub fn set_boss(&mut self, is_boss: bool) {
    self.is_boss = is_boss;
  }

  /// Sets the death loot drop specification.
  #[must_use]
  pub fn with_death_drop(mut self, drop: Option<ItemSpawnKind>) -> Self {
    self.death_drop = drop;
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

  /// Returns whether the actor has the one-use-per-perk tired condition.
  #[must_use]
  pub const fn is_tired(&self) -> bool {
    matches!(self.tired, TiredStatus::Tired)
  }

  /// Adds the typed tired condition used by Subtle Knife invoke.
  pub fn set_tired(&mut self, tired: bool) {
    self.tired = if tired {
      TiredStatus::Tired
    } else {
      TiredStatus::Ready
    };
  }

  /// Current score-count balance used by callback-derived item behaviors.
  #[must_use]
  pub const fn score_count(&self) -> i32 {
    self.score_count
  }

  /// Sets score count for deterministic scenarios and replay fixtures.
  pub fn set_score_count(&mut self, score_count: i32) {
    self.score_count = score_count;
  }

  /// Mutable score-count balance for explicit target-dependent transitions.
  pub fn score_count_mut(&mut self) -> &mut i32 {
    &mut self.score_count
  }

  /// Spends score count without permitting an underflow.
  pub fn spend_score_count(&mut self, amount: i32) -> i32 {
    self.score_count = self.score_count.saturating_sub(amount);
    self.score_count
  }

  /// Applies the typed Subtle Knife actor-side transition.
  pub fn invoke_subtle_knife(&mut self) -> Result<SubtleKnifeCost, SubtleKnifeError> {
    SubtleKnifeTransition::apply(&mut self.hp, &mut self.tired, &mut self.score_count)
  }

  /// Applies the typed Trigun alternate-reload actor transition.
  pub fn alt_reload_trigun(&mut self) -> Result<TrigunCost, TrigunError> {
    TrigunTransition::apply(&mut self.hp, &mut self.score_count)
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
    self.apply_protected_damage(raw_amount)
  }

  /// Applies typed damage after the armor's family-specific resistance and
  /// existing flat protection. Resistance is intentionally read from the
  /// equipped catalog-derived armor instance rather than from an archetype
  /// match in combat code.
  pub fn take_damage_typed(&mut self, raw_amount: u32, damage_type: DamageType) -> (u32, bool) {
    if !self.is_alive {
      return (0, false);
    }
    let resistance = self
      .equipment
      .armor()
      .and_then(Item::armor_properties)
      .map_or(0, |armor| armor.resistance(damage_type));
    if raw_amount == 0 || resistance >= 100 {
      return (0, false);
    }
    let resisted_amount = crate::resistance::apply_damage_resistance(raw_amount, resistance);
    self.apply_protected_damage(resisted_amount)
  }

  fn apply_protected_damage(&mut self, raw_amount: u32) -> (u32, bool) {
    let net_amount = raw_amount.saturating_sub(self.armor_protection()).max(1);
    let taken = self.hp.take_damage(net_amount);
    if let Some(armor) = self
      .equipment
      .armor_mut()
      .and_then(Item::armor_properties_mut)
    {
      armor.reset_malek_recharge();
    }
    if self.hp.is_dead() {
      self.is_alive = false;
      self.blocks_movement = false;
      (taken, true)
    } else {
      (taken, false)
    }
  }

  /// Applies fixed internal damage without armor mitigation.
  pub fn take_internal_damage(&mut self, amount: u32) -> (u32, bool) {
    if !self.is_alive {
      return (0, false);
    }
    let taken = self.hp.take_damage(amount);
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

  /// Current Medical Powerarmor repair timer for deterministic inspection.
  #[must_use]
  pub fn medical_repair_timer(&self) -> u32 {
    self
      .equipment
      .armor()
      .and_then(Item::armor_properties)
      .map_or(0, |armor| armor.medical_repair_timer())
  }

  /// Advances the equipped Medical Powerarmor behavior, when present.
  ///
  /// The item archetype selects this dedicated typed transition; no generic
  /// callback or string-keyed behavior dispatch is involved.
  pub fn tick_medical_powerarmor(
    &mut self,
  ) -> Option<(drl_protocol::ItemId, MedicalRepairOutcome)> {
    let armor_item = self.equipment.armor()?;
    if armor_item.archetype() != ItemArchetype::MedicalPowerarmor {
      return None;
    }
    let item_id = armor_item.id();
    let armor = self.equipment.armor_mut()?.armor_properties_mut()?;
    let outcome = armor.tick_medical_repair(&mut self.hp);
    Some((item_id, outcome))
  }

  /// Current Lava Armor recharge timer for deterministic inspection.
  #[must_use]
  pub fn lava_recharge_timer(&self) -> u32 {
    self
      .equipment
      .armor()
      .and_then(Item::armor_properties)
      .map_or(0, |armor| armor.lava_recharge_timer())
  }

  /// Advances the equipped Lava Armor behavior, when present.
  pub fn tick_lava_armor(
    &mut self,
    on_lava: bool,
  ) -> Option<(drl_protocol::ItemId, LavaRechargeOutcome)> {
    let armor_item = self.equipment.armor()?;
    if armor_item.archetype() != ItemArchetype::LavaArmor {
      return None;
    }
    let item_id = armor_item.id();
    let armor = self.equipment.armor_mut()?.armor_properties_mut()?;
    let outcome = armor.tick_lava_recharge(on_lava);
    Some((item_id, outcome))
  }

  /// Current Malek's Armor recharge timer for deterministic inspection.
  #[must_use]
  pub fn malek_recharge_timer(&self) -> u32 {
    self
      .equipment
      .armor()
      .and_then(Item::armor_properties)
      .map_or(0, |armor| armor.malek_recharge_timer())
  }

  /// Advances the equipped Malek's Armor behavior, when present.
  pub fn tick_malek_armor(&mut self) -> Option<(drl_protocol::ItemId, MalekRechargeOutcome)> {
    let armor_item = self.equipment.armor()?;
    if armor_item.archetype() != ItemArchetype::MaleksArmor {
      return None;
    }
    let item_id = armor_item.id();
    let armor = self.equipment.armor_mut()?.armor_properties_mut()?;
    let outcome = armor.tick_malek_recharge();
    Some((item_id, outcome))
  }

  /// Current equipped rechargeable-weapon timer, or zero when none is equipped.
  #[must_use]
  pub fn weapon_recharge_timer(&self) -> u32 {
    self
      .equipment
      .weapon()
      .and_then(Item::weapon_recharge_timer)
      .unwrap_or(0)
  }

  /// Advances the equipped rechargeable weapon's periodic behavior.
  pub fn tick_weapon_recharge(&mut self) -> Option<(drl_protocol::ItemId, WeaponRechargeOutcome)> {
    let item_id = self.equipment.weapon()?.id();
    let outcome = self.equipment.weapon_mut()?.tick_weapon_recharge()?;
    Some((item_id, outcome))
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

  /// Returns whether the equipped ranged weapon bypasses to-hit sampling.
  #[must_use]
  pub fn ranged_exact_hit(&self) -> bool {
    self
      .equipment
      .weapon()
      .and_then(Item::weapon_properties)
      .is_some_and(|properties| properties.exact_hit)
  }

  /// Kinetic knockback power (tiles pushed on hit).
  #[must_use]
  pub fn knockback(&self) -> u32 {
    if let Some(props) = self.equipment.weapon().and_then(Item::weapon_properties) {
      props.knockback
    } else {
      self.knockback
    }
  }

  /// Monster archetype classification, if applicable.
  #[must_use]
  pub const fn monster_kind(&self) -> Option<MonsterKind> {
    self.monster_kind
  }

  /// Death loot drop specification, if any.
  #[must_use]
  pub const fn death_drop(&self) -> Option<ItemSpawnKind> {
    self.death_drop
  }

  /// Sets the death loot drop specification.
  pub fn set_death_drop(&mut self, drop: Option<ItemSpawnKind>) {
    self.death_drop = drop;
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
      monster_kind: self.monster_kind,
    }
  }

  // --- Factory constructors for representative DRL monster archetypes ---

  /// Factory: Former Human (pistol zombie).
  #[must_use]
  pub fn former_human(id: EntityId, position: Position) -> Self {
    Self::from_monster_kind(id, position, MonsterKind::FormerHuman)
  }

  /// Factory: Former Sergeant (shotgun sergeant).
  #[must_use]
  pub fn former_sergeant(id: EntityId, position: Position) -> Self {
    Self::from_monster_kind(id, position, MonsterKind::FormerSergeant)
  }

  /// Factory: Demonic Imp (fireball thrower).
  #[must_use]
  pub fn imp(id: EntityId, position: Position) -> Self {
    Self::from_monster_kind(id, position, MonsterKind::Imp)
  }

  /// Factory: Pinky Demon (fast melee rusher).
  #[must_use]
  pub fn demon(id: EntityId, position: Position) -> Self {
    Self::from_monster_kind(id, position, MonsterKind::Demon)
  }

  /// Constructs an actor from a given `MonsterKind`.
  #[must_use]
  pub fn from_monster_kind(id: EntityId, position: Position, kind: MonsterKind) -> Self {
    let definition = kind.definition();
    Self::new(id, position, definition.name, false)
      .with_stats(
        HitPoints::full(definition.hp),
        Speed::new(definition.speed),
        definition.melee_damage,
        definition.ranged_damage,
        definition.ranged_range,
        definition.accuracy,
      )
      .with_knockback(definition.knockback)
      .with_monster_kind(kind)
      .with_death_drop(definition.death_drop)
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
    assert_eq!(view.monster_kind, None);
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
  fn blue_armor_mitigates_plasma_before_flat_protection() {
    let mut actor = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true);
    actor
      .equipment_mut()
      .equip(EquipmentSlot::Armor, Item::blue_armor(ItemId::new(10)))
      .unwrap();

    let (plasma_taken, _) = actor.take_damage_typed(10, DamageType::Plasma);
    assert_eq!(
      plasma_taken, 6,
      "20% resistance leaves 8, then armor blocks 2"
    );

    let (fire_taken, _) = actor.take_damage_typed(10, DamageType::Fire);
    assert_eq!(
      fire_taken, 8,
      "Blue Armor does not resist Fire in this slice"
    );
  }

  #[test]
  fn red_armor_mitigates_fire_before_flat_protection() {
    let mut actor = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true);
    actor
      .equipment_mut()
      .equip(EquipmentSlot::Armor, Item::red_armor(ItemId::new(10)))
      .unwrap();

    let (fire_taken, _) = actor.take_damage_typed(10, DamageType::Fire);
    assert_eq!(
      fire_taken, 4,
      "25% resistance leaves 8, then Red Armor blocks 4"
    );

    let (plasma_taken, _) = actor.take_damage_typed(10, DamageType::Plasma);
    assert_eq!(
      plasma_taken, 6,
      "Red Armor does not resist Plasma in this slice"
    );
  }

  #[test]
  fn test_actor_energy_spending() {
    let mut actor = Actor::new(EntityId::new(1), Position::new(0, 0), "Marine", true);
    actor.set_energy(1500);
    actor.spend_energy(ActionCost::MOVE);
    assert_eq!(actor.energy(), 500);
  }

  #[test]
  fn test_monster_archetypes_and_death_drops() {
    let zombie = Actor::former_human(EntityId::new(10), Position::new(2, 3));
    assert_eq!(zombie.name(), "Former Human");
    assert_eq!(zombie.hp().current, 15);
    assert_eq!(zombie.monster_kind(), Some(MonsterKind::FormerHuman));
    assert_eq!(zombie.death_drop(), Some(ItemSpawnKind::Ammo9mm(10)));
    assert_eq!(zombie.ranged_range(), 7);

    let sergeant = Actor::former_sergeant(EntityId::new(11), Position::new(4, 5));
    assert_eq!(sergeant.name(), "Former Sergeant");
    assert_eq!(sergeant.hp().current, 25);
    assert_eq!(sergeant.speed().as_u32(), 90);
    assert_eq!(sergeant.death_drop(), Some(ItemSpawnKind::AmmoShells(4)));

    let imp = Actor::imp(EntityId::new(12), Position::new(6, 7));
    assert_eq!(imp.name(), "Imp");
    assert_eq!(imp.hp().current, 30);
    assert_eq!(imp.death_drop(), Some(ItemSpawnKind::SmallMedPack));

    let demon = Actor::demon(EntityId::new(13), Position::new(8, 9));
    assert_eq!(demon.name(), "Demon");
    assert_eq!(demon.hp().current, 45);
    assert_eq!(demon.speed().as_u32(), 130);
    assert!(demon.ranged_damage().is_none());
    assert_eq!(demon.death_drop(), None);
  }
}
