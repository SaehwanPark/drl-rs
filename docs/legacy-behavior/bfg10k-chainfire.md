# BFG 10K first-level chainfire evidence

Pinned legacy revision: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

## Observed legacy contract

- `bin/data/drl/items/uitems.lua:730-769` defines `ubfg10k` with the
  `perk_altfire_chainfire` perk, five configured shots, and a five-cell
  per-projectile shot cost.
- `src/dfbeing.pas:1486-1493` applies the first chainfire adjustment as
  `shots - (shots div 3)`, yielding four projectiles from the BFG 10K's five;
  `src/dfbeing.pas:1496-1515` charges the resulting projectile count through
  the existing shot-cost helper.
- The same item remains `IF_EXACTHIT` and carries a delayed explosion payload;
  those policies are retained for each successful direct-target chainfire hit
  without claiming scatter, projectile routing, or delayed explosion geometry.

## DRL-Rust boundary

Gameplay semantics `73` (project version `0.2.264`) extends the typed
first-level chainfire command to the BFG 10K. At warm-up level zero, a valid
visible target and at least twenty loaded cells produce four ordered exact-hit
outcomes, consume twenty cells, preserve the existing per-hit
`Bfg10kExplosionScheduled` metadata, and advance warm-up to level one.
Post-lethal slots remain deterministic no-op misses, and ordinary fire resets
the warm-up state. Direct-core, replay, MCP legal-action/JSON, physical browser
key, and BrowserSession parity plus atomic under-supply rejection are covered.
Higher levels, scatter/target rotation, projectile routing, delayed explosion
damage/geometry/knockback, controlled legacy runtime, browser capture, and
audiovisual comparison remain `NOT_RUN` or open.
