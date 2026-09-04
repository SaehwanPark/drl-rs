# Anti-Freak Jackal typed Fire-mitigation evidence

Status: delivered radius-1 splash mitigation in `0.2.330`; `0.2.344` also
routes ordinary and aimed direct target hits through the typed Fire damage
path. The bounded profile does not claim complete legacy callback or
resistance aggregation parity.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/uitems.lua:321-357` defines `ujackal` with direct
  `damage = "5d3"`, `damagetype = DAMAGE_FIRE`, and `radius = 1`.
- `src/dfbeing.pas:1477-1515` carries the weapon damage family into the
  direct-hit `ApplyDamage` path; the shared aimed callback changes accuracy
  and action cost but does not replace the weapon's damage type.
- `src/dflevel.pas:1039-1080` rolls each eligible blast cell and passes
  `aData.DamageType` to `TBeing.ApplyDamage` for the actor at that cell.
- `src/dfbeing.pas:2135-2165` selects the resistance family from the typed
  damage value, applies percentage scaling before flat armor protection, rounds
  positive values, and retains minimum-one/full-immunity behavior.

## DRL-Rust boundary

Generic ranged execution remains authoritative for direct target legality,
accuracy, raw damage RNG, clip/action cost, event ordering, and rejection
atomicity. Its direct damage call now uses
`World::apply_damage_typed(..., DamageType::Fire)` for the Anti-Freak Jackal,
so catalog-defined Red Armor resistance is applied by `ArmorProperties` before
flat protection. The existing `anti_freak` resolver still owns the bounded
center-plus-eight-neighbor geometry, one `5d3` roll per cell, radial knockback,
raw-damage ground-ammo threshold, and event/death ordering; its actor damage
call uses the same typed Fire path. Resistance does not consume RNG, and the
same-seed replay pair proves the direct and splash armored/unarmored paths
share their raw roll sequences.

## Evidence limit

The current-Rust tests establish typed event and replay behavior only. They do
not establish legacy runtime, body-zone/equipment aggregation, callbacks,
terrain destruction, audiovisual behavior, balance, or human-play parity.
