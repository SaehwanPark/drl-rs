# BFG 10K delayed explosion evidence

Status: bounded typed schedule metadata target for `0.2.201`; explosion
damage/geometry, controlled runtime comparison, and audiovisual parity remain
`NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/uitems.lua:730-765` defines `ubfg10k` with an explosion
  payload carrying `delay = 25`, `radius = 2`, and `knockback = 16`.
- `src/dfbeing.pas:476-510` routes scatter weapons through the projectile
  path, where the delayed explosion is resolved separately from the direct
  ranged-hit calculation.

## DRL-Rust boundary

Gameplay semantics `39` (project version `0.2.201`) records one typed
`Bfg10kExplosionScheduled` event after each direct-target volley hit, carrying
the pinned delay, radius, and knockback metadata. The event is deterministic
and replay-visible; the `0.2.264` first-level chainfire slice preserves the same
schedule for each successful chainfire hit. Explosion geometry, splash damage,
knockback application, projectile routing, higher chainfire levels, mods,
controlled legacy runtime, and audiovisual comparison remain separate slices
or `NOT_RUN`.
