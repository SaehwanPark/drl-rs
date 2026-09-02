# Blue Armor Plasma-resistance profile

## Pinned legacy evidence

- `bin/data/drl/items/items.lua:56-72` registers Blue Armor with
  `resist = { plasma = 20 }`, armor protection `2`, and durability-independent
  item metadata.
- `src/dfbeing.pas:2078-2182` selects resistance by damage family before flat
  armor protection. A nonzero resistance scales damage with integer-rounded
  percentage arithmetic and keeps a nonzero result at one point; resistance
  `100` produces zero damage.

## Current Rust boundary

The item catalog carries `plasma_resistance` into `ArmorProperties`. The typed
actor damage path applies that percentage before the existing flat protection;
the Standard BFG direct-target and actor-splash policies pass
`DamageType::Plasma` through this route.
Integer arithmetic is deterministic and consumes no RNG. Current Fire, Acid,
untyped direct hits, armor durability, body zones, weapon/hook bonuses, and
difficulty modifiers remain outside this bounded slice.

This evidence supports a current-Rust typed Plasma mitigation claim, not full
legacy resistance aggregation or controlled runtime parity.
