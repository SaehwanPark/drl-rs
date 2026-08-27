# Standard BFG 9000 exact-hit evidence

This note records the source evidence for the bounded M9 exact-hit slice. The
legacy checkout is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

## Observed legacy contract

- `bin/data/drl/items/eitems.lua:94-134` registers the standard `bfg9000`
  with `IF_EXACTHIT`.
- `src/dfbeing.pas:2317-2340` maps `IF_EXACTHIT` to a 100% ranged to-hit
  result.
- `src/dfbeing.pas:485` and `:2567-2568` also contain target-square projectile
  path behavior for exact-hit weapons.

## DRL-Rust boundary

DRL-Rust implements only the typed to-hit bypass for the standard BFG 9000.
Line-of-sight, range, clip, action cost, damage sampling, and existing attack
and damage events remain unchanged. Projectile routing, radius/falloff,
delayed explosions, other exact-hit families, and audiovisual or controlled
legacy runtime parity are separate slices and are not claimed here.

The legacy checkout's controlled runtime capture was not run for this slice.
