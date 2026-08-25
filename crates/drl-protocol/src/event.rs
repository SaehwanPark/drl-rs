//! Simulation game events emitted during turn processing.

use crate::item::EquipmentSlot;
use crate::types::{
  ActionCost, AttackOutcome, DamageSource, DeathCause, EntityId, ItemId, LevelId, Position, Turn,
};

/// Game event emitted deterministically by the simulation core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
  /// A new turn has started.
  TurnStarted { turn: Turn },
  /// An entity moved from one cell to another.
  EntityMoved {
    entity_id: EntityId,
    from: Position,
    to: Position,
  },
  /// An entity waited in place for a turn.
  EntityWaited {
    entity_id: EntityId,
    position: Position,
  },
  /// An attack was resolved between an attacker and target.
  AttackResolved {
    attacker_id: EntityId,
    target_id: EntityId,
    outcome: AttackOutcome,
    is_ranged: bool,
  },
  /// Damage was applied to an entity.
  DamageApplied {
    target_id: EntityId,
    amount: u32,
    remaining_hp: u32,
    source: DamageSource,
  },
  /// An actor was destroyed or killed.
  ActorDied {
    entity_id: EntityId,
    cause: DeathCause,
  },
  /// An actor paid action cost / energy.
  ActionCostPaid {
    entity_id: EntityId,
    cost: ActionCost,
  },
  /// An item was picked up from the ground into inventory.
  ItemPickedUp {
    entity_id: EntityId,
    item_id: ItemId,
    item_name: String,
  },
  /// An item was dropped from inventory to the ground.
  ItemDropped {
    entity_id: EntityId,
    item_id: ItemId,
    item_name: String,
    position: Position,
  },
  /// An item was equipped to an equipment slot.
  ItemEquipped {
    entity_id: EntityId,
    item_id: ItemId,
    slot: EquipmentSlot,
  },
  /// An item was unequipped from an equipment slot back into inventory.
  ItemUnequipped {
    entity_id: EntityId,
    item_id: ItemId,
    slot: EquipmentSlot,
  },
  /// An item was used or consumed (e.g. MedPack).
  ItemUsed {
    entity_id: EntityId,
    item_id: ItemId,
    item_name: String,
  },
  /// A weapon was reloaded with ammunition from inventory.
  WeaponReloaded {
    entity_id: EntityId,
    ammo_loaded: u32,
    current_clip: u32,
    max_clip: u32,
  },
  /// Medical Powerarmor restored one HP and spent one durability point.
  MedicalPowerarmorRepaired {
    entity_id: EntityId,
    item_id: ItemId,
    healed: u32,
    remaining_hp: u32,
    durability_remaining: u32,
    timer: u32,
  },
  /// The player descended stairs and transitioned to a new level.
  LevelTransitioned {
    from_level: LevelId,
    to_level: LevelId,
  },
  /// The player teleported to a new grid position (e.g. via Phase Device).
  PlayerTeleported { from: Position, to: Position },
  /// An actor was knocked back from one position to another by an attack.
  ActorKnockedBack {
    entity_id: EntityId,
    from: Position,
    to: Position,
  },
  /// The current turn completed.
  TurnEnded { turn: Turn },
}
