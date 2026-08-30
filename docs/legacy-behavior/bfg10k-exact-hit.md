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

Gameplay semantics `38` (project version `0.2.200`) extends the typed exact-hit
policy to BFG 10K's five-projectile direct-target volley. Valid visible,
in-range shots bypass only the to-hit RNG for each projectile while retaining
clip, action cost, damage RNG, and existing attack/damage events; invalid
commands remain atomic. The `0.2.264` first-level chainfire slice and the
  bounded `0.2.304` twentieth-level extension reuse that exact-hit policy for four,
  five, and eighteen seven-projectile ordered bursts. Scatter, twenty-first-and-later
chainfire levels,
projectile routing, explosions, mods,
controlled legacy runtime, and audiovisual comparison remain separate slices
or `NOT_RUN`.
