# Standard BFG 9000 Shot Cost Evidence

Pinned legacy revision: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

## Observed legacy contract

- `bin/data/drl/items/eitems.lua:84-117` defines the standard `bfg9000` with
  a 100-cell clip, one implicit shot, and `shotcost=40`.
- `src/dfitem.pas:627-634` derives a minimum one-cell shot cost and scales it
  by the configured shot count.
- `src/dfbeing.pas:1496-1515` computes and validates total ammo cost before
  debiting ammo, then emits the configured projectile count.

Thus a one-shot standard BFG attack accepts a clip of 40 or more and consumes
exactly 40 cells; a clip of 39 rejects before mutation. Projectile routing,
radius/falloff, delayed explosions, and `NukeRun` effects are separate legacy
behaviors and are not part of this slice.

## DRL-Rust boundary

Gameplay semantics `33` (project version `0.2.186`) adds a typed standard-BFG
shot-cost policy at the core prepare/commit boundary. It keeps the delivered
exact-hit resolver, line-of-sight/range/target checks, one attack event,
damage RNG, and action cost unchanged, while debiting 40 clip cells atomically.
Nuclear BFG and other shot-cost families remain open. Controlled legacy runtime,
audio, and visual comparison are `NOT_RUN`.
