# BFG 10K delayed explosion evidence

Status: delivered bounded typed schedule metadata, actor-only radius-2 fanout,
and ordinary loose-ammo destruction in `0.2.266`; active `0.2.336` work types
the five direct target hits as Plasma. Delayed timing/state-machine parity,
terrain/content effects, controlled runtime comparison, and audiovisual parity
remain `NOT_RUN`.

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
  The ground-item branch removes one ordinary loose-ammo stack when rolled
  damage is greater than `10`; item packs and feature/non-ammunition items are
  preserved by the current Rust boundary.

## DRL-Rust boundary

Gameplay semantics `39` (project version `0.2.201`) first recorded one typed
`Bfg10kExplosionScheduled` event after each direct-target volley hit. Gameplay
semantics `74` (project version `0.2.265`) adds the bounded Rust resolver: the
event remains replay-visible, then each successful ordinary or bounded
chainfire hit immediately considers in-bounds, line-of-sight-cleared radius-2
cells in center/ring order. One `6d4` Plasma environment roll is consumed per
cell without distance falloff; each living actor is processed once, radial
knockback uses integer `damage / 16`, and normal damage/death/drop events are
emitted deterministically. Gameplay semantics `75` (project version `0.2.266`)
additionally removes the lowest-ID ordinary loose-ammo stack on a clear blast
cell when that cell's roll exceeds `10`, emitting `GroundItemDestroyed` after
that cell's actor processing and before lethal follow-up; cells without actors
still apply the ground-item rule. The delay remains presentation metadata
rather than a pending queue. Terrain/content mutation, non-ammunition
ground-item destruction, splash immunity, scatter/projectile routing,
sixth-and-later chainfire levels, mods, controlled legacy runtime, and
audiovisual comparison remain separate slices or `NOT_RUN`.

Gameplay semantics `138` (project version `0.2.336`) classifies each successful
BFG 10K direct volley target hit as typed Plasma, preserving the raw `6d4`
damage roll, exact-hit behavior, five-projectile event order, and RNG stream.
Blue Armor therefore applies its catalog-defined 20% Plasma resistance before
flat protection on the direct events. This is a current-Rust classification of
the existing direct-target path: the cited legacy source routes BFG 10K through
its scatter/projectile resolver, so this slice does not claim separate legacy
direct `ApplyDamage` or full scatter/projectile parity. The already delivered
radius-2 splash remains an environment Plasma path; SPLASMA armor divisors,
delayed timing, and broader resistance aggregation remain outside the slice.
