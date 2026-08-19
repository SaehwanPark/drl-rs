# Specification

## Document Contract

The [project roadmap](docs/DRL-Rust_Project_Roadmap.md) is the canonical plan
for milestone scope, order, status, and exit criteria. This file expands only
the active implementation slice into observable outcomes and verification. It
does not replace or duplicate the full roadmap.

## Past

- The repository, Rust 2024 binary scaffold, license, proposal, roadmap, and
  local legacy-asset research location were established before this
  specification workflow was adopted.
- The Milestone 0 documentation and harness foundation established durable
  agent guidance, team contracts, check scripts, and repository workflow.
- Milestone 0 multi-crate Cargo workspace and initial crates boundary scaffolding
  were established and validated with architectural boundary tests.
- Milestone 1 established the deterministic headless simulation kernel in `drl-core`
  and shared protocol contracts in `drl-protocol`, including 2D grid maps, seedable RNG,
  movement validation, and replay determinism.
- Milestone 4 established Field of View (FOV) calculation, line-of-sight raycasting,
  fog-of-war exploration memory, entity observation filtering, and line-of-fire obstacle blocking.

## Present

### Milestone 4: Item Domain Models, Inventory Management, Equipment Slots, and Weapon/Ammo Mechanics

Status: Active

This slice implements DRL's item domain models, player inventory capacity, equipment slots
(weapon and armor), ground item placement, pickup and drop actions, medpack consumption,
ammunition tracking, weapon reload mechanics, and ammo-dependent ranged combat.

Observable outcomes:

- `drl-protocol` defines domain types and enumerations for items:
  - `AmmoType`: 9mm (`Ammo9mm`), Shotgun shells (`Shells`), Rockets (`Rocket`), Plasma cells (`Cell`);
  - `ItemCategory`: Weapon, Armor, Ammo, MedPack / Consumable;
  - `EquipmentSlot`: Weapon, Armor;
  - `ItemView` and `GroundItemView` representing observed items and ground stacks;
- `drl-protocol` defines new semantic player commands and error conditions:
  - `Command::Pickup` (picks up item from current cell into inventory);
  - `Command::Drop(ItemId)` (drops item from inventory to the ground at current cell);
  - `Command::Equip(ItemId)` (equips an item from inventory to its designated slot);
  - `Command::Unequip(EquipmentSlot)` (unequips item back into inventory);
  - `Command::Use(ItemId)` (consumes a usable item like a MedPack);
  - `Command::Reload` (reloads the currently equipped weapon using matching ammo in inventory);
  - Typed error variants in `CommandError` (`InventoryFull`, `ItemNotFound`, `NoItemAtPosition`,
    `CannotEquip`, `CannotUse`, `SlotEmpty`, `NoEquippedWeapon`, `NoAmmoInClip`, `NoMatchingAmmo`,
    `ClipAlreadyFull`);
- `drl-protocol` defines new semantic game events:
  - `GameEvent::ItemPickedUp`, `GameEvent::ItemDropped`, `GameEvent::ItemEquipped`,
    `GameEvent::ItemUnequipped`, `GameEvent::ItemUsed`, `GameEvent::WeaponReloaded`;
- `drl-protocol` expands `PlayerObservation` and `OmniscientObservation` to include player
  inventory, equipped items, and ground items (with perception filtering for fog of war / FOV);
- `drl-core` implements an isolated `item` and `inventory` module:
  - `Inventory`: capacity-constrained container with stack management for ammunition;
  - `Equipment`: slot management for equipped weapon and armor;
  - Representative weapons: Pistol (9mm, clip 10), Shotgun (Shells, clip 8), Combat Knife (melee);
  - Representative ammo: 9mm (packs of 10-30), Shells (packs of 8-16);
  - Representative consumables: Small MedPack (+10 HP), Large MedPack (+25 HP);
  - Representative armor: Green Armor (+5 armor protection);
- `drl-core` integrates weapons and ammo into combat resolution:
  - `Command::AttackRanged` deducts 1 ammunition from the equipped weapon's clip;
  - Firing with an empty clip fails with `CommandError::NoAmmoInClip`;
  - Ranged damage, range, and accuracy derive from the equipped weapon;
  - `Command::Reload` transfers ammo from inventory stacks up to weapon clip capacity;
  - `Command::Use` on a MedPack restores player HP up to maximum and consumes the item;
- `drl-core` tracks ground items in `World` with FOV/fog-of-war perception filtering;
- `drl-app` demonstrates inventory management, weapon reloading, ground pickup, and healing;
- `sh scripts/check-repository.sh` runs all checks, formatting, clippy, and tests cleanly.

Verification:

- `sh scripts/check-repository.sh` succeeds locally;
- `cargo test --locked --workspace` passes all unit, integration, boundary, combat,
  visibility, inventory, and replay determinism tests;
- integration tests in `crates/drl-core/tests/inventory.rs` verify pickup/drop, inventory
  capacity limits, equip/unequip cycles, medpack healing, weapon ammo consumption, and reloading;
- `cargo run` executes the headless demo demonstrating item pickups, weapon firing, reloading,
  and replay reproducibility.

Out of scope:

- procedural level generation algorithms;
- live Lua scripting integration;
- MCP transport servers;
- presentation/GUI rendering and audio.

## Future

Proceed with procedural level generation, stairs, and level flow in Milestone 4.
