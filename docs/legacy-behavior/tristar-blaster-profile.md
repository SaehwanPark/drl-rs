# Tristar Blaster ordinary-fire volley evidence

Status: active ordinary-fire volley profile target for `0.2.225`; spread,
explosion, and presentation behavior remain explicitly deferred.

The pinned legacy revision is
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` in the adjacent
`doom-the-roughlike-original` checkout.

## Pinned declaration

`bin/data/drl/items/eitems.lua:337-363` declares `utristar` as an exotic Cell
weapon with a 45-cell clip, `shots = 3`, and `shotcost = 5`. Its spread and
delayed explosion fields are retained as unresolved legacy behavior rather than
being inferred by the current direct-target path.

## Projectile and cost derivation

Legacy `src/dfitem.pas:247-255` loads the explicit `shots = 3` value and
`src/dfbeing.pas:1477-1485` uses that count for ordinary fire. Legacy
`src/dfitem.pas:627-634` computes `Max(ShotCost, 1) * aShots` before any
callback multiplier, so the current three-projectile contract charges fifteen
cells (`3 × 5`) for one accepted command.

## Rust boundary

`TRISTAR_BLASTER_BEHAVIOR` records ordered three-projectile and five-cell
per-projectile fragments; generic ranged execution remains authoritative for
target/LOS/range checks, damage/RNG, event ordering, and transactional clip
mutation. Clips below fifteen reject before clip or RNG mutation and are covered
by exact `Game` equality. Spread routing, delayed explosion geometry, callback
parity, and controlled runtime or audiovisual comparison remain `NOT_RUN`.
