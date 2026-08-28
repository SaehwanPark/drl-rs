# Lava Armor typed behavior-profile evidence

Status: delivered typed behavior profile for `0.2.207`; hazard
damage/resistance, controlled legacy runtime comparison, and audiovisual
parity remain `NOT_RUN`.

## Pinned source

Evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`:

- `bin/data/drl/items/uitems.lua:924-967` defines the
  `perk_ulavaarmor` recharge callback and Lava Armor's scalar properties.
- `bin/data/drl/cells.lua:445-477` provides the related hazard definition;
  hazard damage and resistance are outside this profile slice.

The callback initializes its recharge timer at zero, advances it on each item
tick while durability is below maximum, and after five ticks checks the
owner's current cell. Lava restores up to three durability points and resets
the timer; the fifth tick away from Lava also resets the timer without repair.

## DRL-Rust boundary

The immutable `drl_core::behavior::LAVA_ARMOR_BEHAVIOR` profile records one
typed `PeriodicEffect::TerrainRecharge` fragment for `TileKind::Lava` with
interval `5` and amount `3`. Dedicated `LavaRechargeState` remains the
execution authority for accepted-command ticking, terrain checks, durability
clamping, timer resets, and the `GameEvent::LavaArmorRecharged` boundary. This
slice adds no command, event, replay, RNG, or generic callback-dispatch
surface.

Hazard damage/resistance policy, movement/knockback modifiers, exact legacy
actor-tick cadence, controlled legacy runtime capture, and audiovisual parity
remain outside this profile and are not claimed here.
