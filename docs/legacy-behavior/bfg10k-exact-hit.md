# BFG 10K Exact-Hit Evidence

Pinned legacy revision: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

## Observed legacy contract

- `bin/data/drl/items/uitems.lua:730-765` defines `ubfg10k` with
  `IF_EXACTHIT`, alongside `IF_SCATTER`, five configured shots, and a
  five-cell shot cost.
- `src/dfbeing.pas:2317-2344` maps `IF_EXACTHIT` to a 100% ranged to-hit
  result.
- `src/dfbeing.pas:476-510` handles scatter and shot-count behavior through a
  separate projectile path; delayed explosion behavior is separate as well.

## DRL-Rust boundary

Gameplay semantics `36` (project version `0.2.189`) extends the typed exact-hit
policy to BFG 10K. Valid visible, in-range shots bypass only the to-hit RNG
while retaining clip, action cost, damage RNG, and existing attack/damage
events; invalid commands remain atomic. Five-shot volley, scatter, chainfire,
five-cell shot cost, projectile routing, explosions, mods, controlled legacy
runtime, and audiovisual comparison are separate slices or `NOT_RUN`.
