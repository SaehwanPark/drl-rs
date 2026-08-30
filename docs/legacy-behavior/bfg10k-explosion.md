# BFG 10K delayed explosion evidence

Status: delivered bounded typed schedule metadata and actor-only radius-2
fanout in `0.2.265`; delayed timing/state-machine parity, terrain/content
effects, controlled runtime comparison, and audiovisual parity remain
`NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/uitems.lua:730-765` defines `ubfg10k` with an explosion
  payload carrying `delay = 25`, `radius = 2`, `knockback = 16`, and the
  weapon's `6d4` `DAMAGE_SPLASMA` roll.
- `src/dfbeing.pas:476-510` routes scatter weapons through the projectile
  path, where the delayed explosion is resolved separately from the direct
  ranged-hit calculation. Its radius loop rolls one damage result per clear
  cell, omits distance falloff for `EFNODISTANCEDROP`, applies knockback before
  damage, de-duplicates actors, and can destroy ground items or mutate cells.

## DRL-Rust boundary

Gameplay semantics `39` (project version `0.2.201`) first recorded one typed
`Bfg10kExplosionScheduled` event after each direct-target volley hit. Gameplay
semantics `74` (project version `0.2.265`) adds the bounded Rust resolver: the
event remains replay-visible, then each successful ordinary or first-level
chainfire hit immediately considers in-bounds, line-of-sight-cleared radius-2
cells in center/ring order. One `6d4` Plasma environment roll is consumed per
cell without distance falloff; each living actor is processed once, radial
knockback uses integer `damage / 16`, and normal damage/death/drop events are
emitted deterministically. The delay remains presentation metadata rather than
a pending queue. Terrain/content mutation, ground-item destruction, splash
immunity, scatter/projectile routing, higher chainfire levels, mods, controlled
legacy runtime, and audiovisual comparison remain separate slices or
`NOT_RUN`.
