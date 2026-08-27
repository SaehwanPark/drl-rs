# Nuclear BFG 9000 Shot-Cost Evidence

Pinned legacy revision: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

## Observed legacy contract

- `bin/data/drl/items/eitems.lua:474-518` defines `unbfg9000` with a
  40-cell clip, one implicit shot, `shotcost=40`, and separate exact-hit,
  recharge, and alternate-overload hooks.
- `src/dfitem.pas:627-634` clamps a base shot cost to at least one cell and
  multiplies it by the configured projectile count.
- `src/dfbeing.pas:1496-1515` computes and validates total ammo cost before
  debiting the clip, then emits the configured projectile count.

## DRL-Rust boundary

Gameplay semantics `35` (project version `0.2.188`) extends the existing typed
shot-cost policy to Nuclear BFG 9000. A valid visible, in-range one-shot with
40 cells consumes exactly 40 cells; clips below 40 reject before mutation.
Exact-hit, recharge, alternate overload, action cost, damage RNG, and existing
attack/damage events remain unchanged. Projectile routing, explosions, NukeRun,
other shot-cost families, controlled legacy runtime, and audiovisual comparison
are separate slices or `NOT_RUN`.
