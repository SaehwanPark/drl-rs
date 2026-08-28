# Acid Spitter typed behavior-profile evidence

Status: delivered typed behavior profile for `0.2.210`; hazard
damage/resistance, fluid movement cost, controlled legacy runtime comparison,
and audiovisual parity remain `NOT_RUN`.

## Pinned source

Evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`:

- `bin/data/drl/items/uitems.lua:666-685` defines the `perk_uacid`
  pre-reload callback.
- `bin/data/drl/items/uitems.lua:687-725` defines the Acid Spitter item.
- `bin/data/drl/cells.lua:387-449` defines the related Acid and Water cells.

The callback rejects a full clip, otherwise requires the actor's current cell
to be Acid, loads one round up to the clip cap, subtracts 1,000 score count,
and changes the cell to Water. Other terrain leaves the clip unchanged.

## DRL-Rust boundary

The immutable `drl_core::behavior::ACID_SPITTER_BEHAVIOR` profile records one
typed `AlternateAction::TerrainReload` requiring `TileKind::Acid`, producing
`TileKind::Water`, and loading one round, followed by a
`ResourceCost::Score` of `1,000`. Dedicated `acid_spitter::apply` remains the
execution authority for clip preflight, terrain validation, saturating score
policy, and the existing command/event boundary. This slice adds no command,
replay, RNG, or generic callback-dispatch surface.

Hazard damage/resistance, fluid movement cost, controlled legacy runtime
capture, and audiovisual parity remain outside this profile and are not
claimed here.
