# Acid Spitter typed behavior-profile evidence

Status: terrain-reload profile delivered in `0.2.210`, extended with the
ordinary-fire cost in `0.2.226`; hazard damage/resistance, fluid movement cost,
controlled legacy runtime comparison, and audiovisual parity remain `NOT_RUN`.

## Pinned source

Evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`:

- `bin/data/drl/items/uitems.lua:666-685` defines the `perk_uacid`
  pre-reload callback.
- `bin/data/drl/items/uitems.lua:687-725` defines the Acid Spitter item.
- `bin/data/drl/cells.lua:387-449` defines the related Acid and Water cells.
- `src/dfitem.pas:627-634` computes the effective shot cost as at least one,
  multiplying the pinned `ShotCost` by the resolved shot count before any
  callback multiplier.
- `src/dfbeing.pas:1477-1481` resolves a default one-shot count when the item
  does not declare a `shots` value.

The callback rejects a full clip, otherwise requires the actor's current cell
to be Acid, loads one round up to the clip cap, subtracts 1,000 score count,
and changes the cell to Water. Other terrain leaves the clip unchanged.

## DRL-Rust boundary

The immutable `drl_core::behavior::ACID_SPITTER_BEHAVIOR` profile records one
ordered `AttackEffect::ProjectileCount(1)` and
`ResourceCost::Ammo { ammo_type: Rocket, amount: 10 }` for ordinary fire,
followed by the existing typed `AlternateAction::TerrainReload` and
`ResourceCost::Score` fragments. Generic ranged execution remains the
authority for target/LOS/range validation, damage RNG, event ordering, and
transactional ten-rocket clip consumption. Dedicated `acid_spitter::apply`
remains the execution authority for terrain reload clip/terrain preflight,
saturating score policy, and its existing command/event boundary. This slice
adds no command, replay wire field, RNG algorithm, or generic callback-dispatch
surface.

Hazard damage/resistance, fluid movement cost, explosion geometry/content,
spread/falloff, controlled legacy runtime capture, and audiovisual parity
remain outside this profile and are not claimed here.
