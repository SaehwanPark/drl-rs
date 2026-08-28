# BFG 10K Shot-Cost Evidence

Pinned legacy revision: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

`bin/data/drl/items/uitems.lua:730-765` registers `ubfg10k` with a 50-cell
clip, `shots=5`, and `shotcost=5`. The legacy `TItem.getShotCost` helper in
`src/dfitem.pas:627-634` clamps the base cost to at least one and scales it by
the projectile count. `TBeing.FireRanged` preflights and debits this total in
`src/dfbeing.pas:1496-1515`, separately from projectile resolution.

This DRL-Rust slice implements the typed five-cell per-projectile clip cost for
the existing five-projectile direct-target Rust path. A valid visible,
in-range BFG 10K volley with a full clip consumes twenty-five cells; clips
below twenty-five reject atomically. Scatter, chainfire, projectile routing,
explosions, and mod behavior remain separate slices, so this is not a claim of
full BFG 10K runtime parity. Controlled runtime and audiovisual comparison are
`NOT_RUN`.
