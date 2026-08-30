# BFG 10K chainfire-level evidence

Pinned legacy revision: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

## Observed legacy contract

- `bin/data/drl/items/uitems.lua:730-769` defines `ubfg10k` with the
  `perk_altfire_chainfire` perk, five configured shots, and a five-cell
  per-projectile shot cost.
- `src/dfbeing.pas:1486-1493` applies the chainfire adjustment by warm-up
  level: level zero uses `shots - (shots div 3)`, level one leaves `shots`
  unchanged, and level two and later use `shots + (shots div 2)`. With the
  BFG 10K's five configured shots, the first nineteen bounded levels therefore
  emit four, five, and seventeen seven-projectile bursts;
  `src/dfbeing.pas:1496-1515` charges the resulting projectile count through
  the existing shot-cost helper.
- The same item remains `IF_EXACTHIT` and carries a delayed explosion payload;
  those policies are retained for each successful direct-target chainfire hit
  without claiming scatter, projectile routing, or delayed explosion geometry.

## DRL-Rust boundary

Gameplay semantics `112` (project version `0.2.303`) extends the typed BFG 10K
chainfire command with the pinned nineteenth warm-up level. At level zero, a valid
visible target and at least twenty loaded cells produce four ordered exact-hit
outcomes and advance warm-up to level one. At level one, a valid visible target
and at least twenty-five loaded cells produce five ordered exact-hit outcomes,
consume twenty-five cells, and advance warm-up to level two. After reloads,
levels two through eighteen accept a valid visible target and at least thirty-five
loaded cells, produce seven ordered exact-hit outcomes each, consume thirty-five
cells per burst, preserve the existing per-hit `Bfg10kExplosionScheduled`
metadata, and advance warm-up through levels three through nineteen. Post-lethal
slots remain deterministic no-op misses, ordinary fire resets the warm-up
state, and twentieth-level and later requests remain atomic rejections.
Direct-core, reload-backed replay, ScenarioRunner, MCP legal-action/JSON,
physical browser key, and BrowserSession parity plus atomic nineteenth-level and under-supply
rejection are covered. Twentieth and later levels, scatter/target rotation,
projectile routing, delayed explosion damage/geometry/knockback, controlled
legacy runtime, browser capture, and audiovisual comparison remain `NOT_RUN` or
open.
