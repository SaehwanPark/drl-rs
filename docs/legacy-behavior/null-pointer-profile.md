# Charch's Null Pointer ordinary-fire cost evidence

Status: active ordinary-fire cost profile target for `0.2.224`; target-score
and deferred explosion behavior remain implemented separately.

The pinned legacy revision is
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` in the adjacent
`doom-the-roughlike-original` checkout.

## Pinned declaration

`bin/data/drl/items/uitems.lua:78-110` declares `unullpointer` as a ranged
Cell weapon with a 60-cell clip, no explicit `shots` field, and `shotcost = 10`.
The current Rust definition preserves the Cell family, clip, and zero-damage
ordinary-hit boundary; this slice adds the missing typed ordinary-fire cost.

## Projectile and cost derivation

Legacy `src/dfitem.pas:247-255` defaults an absent `shots` field to zero, while
`src/dfbeing.pas:1477-1483` resolves ordinary fire with `Max(aGun.Shots, 1)`,
so the current contract emits one projectile. Legacy
`src/dfitem.pas:627-634` computes `Max(ShotCost, 1) * aShots` before any callback
multiplier; with `shotcost = 10` and one projectile, ordinary fire consumes ten
cells.

## Rust boundary

`NULL_POINTER_BEHAVIOR` records ordered one-projectile and ten-cell fragments;
generic ranged execution remains authoritative for target/LOS/range checks,
the existing target-score branch and deferred explosion events, damage/RNG,
event ordering, and transactional clip mutation. Clips below ten reject before
clip or RNG mutation and are covered by exact `Game` equality. Full callback
parity, delayed explosion geometry, and controlled runtime or audiovisual
comparison remain `NOT_RUN`.
