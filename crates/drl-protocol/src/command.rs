//! Semantic command definitions and command failure errors.

use crate::item::EquipmentSlot;
use crate::types::{Direction, EntityId, ItemId, Position};
use std::fmt;

/// Semantic player or actor command submitted to the simulation core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
  /// Move or attempt to step in a direction (initiates bump-attack if enemy present).
  Move(Direction),
  /// Direct melee attack in a direction.
  AttackMelee(Direction),
  /// Direct ranged attack targeting a grid position.
  AttackRanged(Position),
  /// Wait in place for one turn.
  Wait,
  /// Pick up an item lying on the current ground tile into inventory.
  Pickup,
  /// Drop an item from inventory to the ground at the current position.
  Drop(ItemId),
  /// Equip an item from inventory to its designated equipment slot.
  Equip(ItemId),
  /// Unequip an item from a designated slot back into inventory.
  Unequip(EquipmentSlot),
  /// Use/consume an item from inventory (e.g. MedPack).
  Use(ItemId),
  /// Invoke a typed alternate action on an equipped item (e.g. Subtle Knife).
  Invoke(ItemId),
  /// Reload the equipped ranged weapon from inventory ammo stacks.
  Reload,
  /// Descend stairs at the current position to transition to the next level.
  Descend,
}

/// Errors returned when a command fails validation or execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
  /// Target cell is outside the map boundaries.
  OutOfBounds(Position),
  /// Target cell is blocked by impassable terrain (e.g. wall).
  BlockedByTerrain(Position),
  /// Target cell is blocked by an existing entity.
  BlockedByEntity {
    position: Position,
    entity_id: EntityId,
  },
  /// Entity submitting the command does not exist in the world.
  EntityNotFound(EntityId),
  /// Direction supplied is invalid for the requested action.
  InvalidDirection(Direction),
  /// Target position is out of range for the chosen weapon or action.
  TargetOutOfRange(Position),
  /// Target position contains no valid target.
  InvalidTarget(Position),
  /// Line of sight / line of fire to the target position is blocked by an obstacle.
  LineOfSightBlocked(Position),
  /// Dead actor cannot perform actions.
  DeadActorCannotAct(EntityId),
  /// Inventory is full and cannot accept more items.
  InventoryFull,
  /// Specified item ID was not found in inventory or ground.
  ItemNotFound(ItemId),
  /// No item exists on the ground at the specified position.
  NoItemAtPosition(Position),
  /// Item cannot be equipped to an equipment slot.
  CannotEquip(ItemId),
  /// Item cannot be used or consumed.
  CannotUse(ItemId),
  /// Item cannot perform its requested alternate action.
  CannotInvoke(ItemId),
  /// Equipment slot is already empty.
  SlotEmpty(EquipmentSlot),
  /// Action requires an equipped weapon, but none is equipped.
  NoEquippedWeapon,
  /// Weapon has no ammunition loaded in its clip.
  NoAmmoInClip,
  /// No matching ammunition available in inventory for reloading.
  NoMatchingAmmo,
  /// Weapon clip is already full.
  ClipAlreadyFull,
  /// Action requires standing on a stairs tile, but none is present at the current position.
  NotOnStairs(Position),
  /// Generic command validation failure.
  InvalidCommand(String),
}

impl fmt::Display for CommandError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::OutOfBounds(pos) => {
        write!(f, "target position ({}, {}) is out of bounds", pos.x, pos.y)
      }
      Self::BlockedByTerrain(pos) => {
        write!(f, "position ({}, {}) is blocked by terrain", pos.x, pos.y)
      }
      Self::BlockedByEntity {
        position,
        entity_id,
      } => {
        write!(
          f,
          "position ({}, {}) is blocked by entity {}",
          position.x,
          position.y,
          entity_id.as_u64()
        )
      }
      Self::EntityNotFound(id) => write!(f, "entity {} was not found", id.as_u64()),
      Self::InvalidDirection(dir) => write!(f, "invalid direction: {dir:?}"),
      Self::TargetOutOfRange(pos) => {
        write!(f, "target position ({}, {}) is out of range", pos.x, pos.y)
      }
      Self::InvalidTarget(pos) => {
        write!(
          f,
          "target position ({}, {}) contains no valid target",
          pos.x, pos.y
        )
      }
      Self::LineOfSightBlocked(pos) => {
        write!(
          f,
          "line of sight to target ({}, {}) is blocked",
          pos.x, pos.y
        )
      }
      Self::DeadActorCannotAct(id) => {
        write!(f, "dead actor {} cannot perform actions", id.as_u64())
      }
      Self::InventoryFull => write!(f, "inventory is full"),
      Self::ItemNotFound(id) => write!(f, "item {} was not found", id.as_u64()),
      Self::NoItemAtPosition(pos) => {
        write!(f, "no item on ground at ({}, {})", pos.x, pos.y)
      }
      Self::CannotEquip(id) => write!(f, "item {} cannot be equipped", id.as_u64()),
      Self::CannotUse(id) => write!(f, "item {} cannot be used", id.as_u64()),
      Self::CannotInvoke(id) => write!(f, "item {} cannot be invoked", id.as_u64()),
      Self::SlotEmpty(slot) => write!(f, "{slot} slot is empty"),
      Self::NoEquippedWeapon => write!(f, "no weapon equipped"),
      Self::NoAmmoInClip => write!(f, "weapon clip is empty - reload required"),
      Self::NoMatchingAmmo => write!(f, "no matching ammunition in inventory"),
      Self::ClipAlreadyFull => write!(f, "weapon clip is already full"),
      Self::NotOnStairs(pos) => {
        write!(
          f,
          "no stairs present at current position ({}, {})",
          pos.x, pos.y
        )
      }
      Self::InvalidCommand(msg) => write!(f, "invalid command: {msg}"),
    }
  }
}

impl std::error::Error for CommandError {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_command_error_formatting() {
    let err = CommandError::OutOfBounds(Position::new(-1, 5));
    assert_eq!(err.to_string(), "target position (-1, 5) is out of bounds");

    let blocked = CommandError::BlockedByEntity {
      position: Position::new(3, 4),
      entity_id: EntityId::new(42),
    };
    assert_eq!(
      blocked.to_string(),
      "position (3, 4) is blocked by entity 42"
    );

    let dead = CommandError::DeadActorCannotAct(EntityId::new(7));
    assert_eq!(dead.to_string(), "dead actor 7 cannot perform actions");

    let out_range = CommandError::TargetOutOfRange(Position::new(10, 10));
    assert_eq!(
      out_range.to_string(),
      "target position (10, 10) is out of range"
    );

    let los_blocked = CommandError::LineOfSightBlocked(Position::new(8, 9));
    assert_eq!(
      los_blocked.to_string(),
      "line of sight to target (8, 9) is blocked"
    );

    let full = CommandError::InventoryFull;
    assert_eq!(full.to_string(), "inventory is full");

    let no_ammo = CommandError::NoAmmoInClip;
    assert_eq!(
      no_ammo.to_string(),
      "weapon clip is empty - reload required"
    );

    let slot_empty = CommandError::SlotEmpty(EquipmentSlot::Weapon);
    assert_eq!(slot_empty.to_string(), "weapon slot is empty");

    let not_on_stairs = CommandError::NotOnStairs(Position::new(2, 3));
    assert_eq!(
      not_on_stairs.to_string(),
      "no stairs present at current position (2, 3)"
    );
  }
}
