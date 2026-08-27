//! Simulation game events emitted during turn processing.

use crate::item::EquipmentSlot;
use crate::item::WeaponFireMode;
use crate::types::{
  ActionCost, AttackOutcome, DamageSource, DamageType, DeathCause, EntityId, HitPoints, ItemId,
  LevelId, Position, Turn,
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
    /// Optional typed damage family; populated for bounded environment hazards
    /// while legacy/actor damage remains intentionally unclassified.
    damage_type: Option<DamageType>,
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
  /// A self-recharging weapon restored ammunition without using reserve ammo.
  WeaponRecharged {
    entity_id: EntityId,
    item_id: ItemId,
    ammo_recharged: u32,
    current_clip: u32,
    max_clip: u32,
    timer: u32,
  },
  /// Acid Spitter drew one rocket from an Acid tile and converted it to Water.
  AcidSpitterReloaded {
    entity_id: EntityId,
    item_id: ItemId,
    position: Position,
    ammo_loaded: u32,
    current_clip: u32,
    max_clip: u32,
    score_count_remaining: i32,
  },
  /// A Subtle Knife invoke paid its actor cost and selected visible targets.
  SubtleKnifeInvoked {
    entity_id: EntityId,
    item_id: ItemId,
    targets: Vec<EntityId>,
    remaining_hp: u32,
    score_count_remaining: i32,
  },
  /// A confirmed Trigun alternate reload paid its actor-side costs.
  TrigunAltReloaded {
    entity_id: EntityId,
    item_id: ItemId,
    remaining_hp: HitPoints,
    score_count_remaining: i32,
  },
  /// A Grammaton alternate reload cycled its typed fire mode.
  GrammatonFireModeChanged {
    entity_id: EntityId,
    item_id: ItemId,
    mode: WeaponFireMode,
    score_count_remaining: i32,
  },
  /// A Jackhammer alternate reload toggled its typed fire mode.
  JackhammerFireModeChanged {
    entity_id: EntityId,
    item_id: ItemId,
    mode: WeaponFireMode,
    score_count_remaining: i32,
  },
  /// Null Pointer hit a target and applied its target-dependent score branch.
  NullPointerHit {
    entity_id: EntityId,
    item_id: ItemId,
    target_id: EntityId,
    target_is_boss: bool,
    score_count_remaining: i32,
  },
  /// Null Pointer scheduled its evidence-backed deferred splash explosion.
  NullPointerExplosionScheduled {
    entity_id: EntityId,
    target_id: EntityId,
    delay: u32,
    radius: u32,
    damage: u32,
  },
  /// A typed level nuke was scheduled at an accepted command boundary.
  NukeActivated { level_id: LevelId, countdown: u32 },
  /// The scheduled level nuke resolved before its internal player damage.
  LevelNuked { level_id: LevelId },
  /// Medical Powerarmor restored one HP and spent one durability point.
  MedicalPowerarmorRepaired {
    entity_id: EntityId,
    item_id: ItemId,
    healed: u32,
    remaining_hp: u32,
    durability_remaining: u32,
    timer: u32,
  },
  /// Lava Armor restored durability after its lava recharge interval.
  LavaArmorRecharged {
    entity_id: EntityId,
    item_id: ItemId,
    durability_restored: u32,
    durability_remaining: u32,
    timer: u32,
  },
  /// Malek's Armor restored durability after its recharge interval.
  MalekArmorRecharged {
    entity_id: EntityId,
    item_id: ItemId,
    durability_restored: u32,
    durability_remaining: u32,
    timer: u32,
  },
  /// A confirmed nuclear weapon overload armed a typed level nuke.
  NuclearWeaponOverloaded {
    entity_id: EntityId,
    item_id: ItemId,
    countdown: u32,
    score_count_remaining: i32,
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
