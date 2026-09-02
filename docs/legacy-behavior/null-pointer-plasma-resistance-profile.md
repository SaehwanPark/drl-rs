# Charch's Null Pointer typed Plasma mitigation evidence

Status: active slice in `0.2.331`; the existing Null Pointer target-score and
actor-only radius-1 splash behavior remains delivered separately.

## Pinned legacy evidence

The pinned legacy revision is
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` in the adjacent
`doom-the-roughlike-original` checkout.

- `bin/data/drl/items/uitems.lua:63-112` schedules a range-1 `10d1`
  `DAMAGE_SPLASMA` explosion and identifies the weapon as Plasma-family content.
- `src/dflevel.pas:1039-1080` rolls each clear blast cell and passes the
  explosion's damage type into `TBeing.ApplyDamage`.
- `src/dfbeing.pas:2170-2245` selects the Plasma resistance family before flat
  armor protection for both Plasma and SPLASMA damage. The legacy SPLASMA
  armor-value divisor and body-zone aggregation are not imported into this
  bounded Rust slice.

## Rust contract

At the audited base `ca31143`, the resolver already emitted a typed
`DamageApplied` event but called untyped `World::apply_damage`, so Blue Armor's
catalog-defined 20% Plasma resistance did not affect the splash. The active
slice routes that one call through `apply_damage_typed` with the existing
`DamageType::Plasma` vocabulary. Resistance uses the shared integer rounding,
minimum-one, then flat-protection policy and consumes no RNG. Geometry,
deduplication, fixed damage, event ordering, death/drop handling, transaction
rollback, and boundary projections remain unchanged; delayed timing, terrain
and item destruction, splash immunity, legacy runtime, audiovisual parity, and
the legacy SPLASMA divisor remain `NOT_RUN` or separate work.
