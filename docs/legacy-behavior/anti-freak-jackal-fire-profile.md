# Anti-Freak Jackal typed Fire-mitigation evidence

Status: active slice in `0.2.330`; the current branch routes the existing
Anti-Freak radius-1 actor splash through the typed Fire damage path.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/uitems.lua:321-357` defines `ujackal` with `damage =
  "5d3"`, `damagetype = DAMAGE_FIRE`, and `radius = 1`.
- `src/dflevel.pas:1039-1080` rolls each eligible blast cell and passes
  `aData.DamageType` to `TBeing.ApplyDamage` for the actor at that cell.
- `src/dfbeing.pas:2135-2165` selects the resistance family from the typed
  damage value, applies percentage scaling before flat armor protection, rounds
  positive values, and retains minimum-one/full-immunity behavior.

## DRL-Rust boundary

The existing `anti_freak` resolver still owns the bounded center-plus-eight-
neighbor geometry, one `5d3` roll per cell, radial knockback, raw-damage
ground-ammo threshold, and event/death ordering. Its actor damage call now uses
`World::apply_damage_typed(..., DamageType::Fire)`, so catalog-defined Red Armor
resistance is applied by `ArmorProperties` before flat protection. Resistance
does not consume RNG, and the same-seed replay pair proves the armored and
unarmored amounts share the same raw roll sequence.

## Evidence limit

The current-Rust tests establish typed event and replay behavior only. They do
not establish legacy runtime, body-zone/equipment aggregation, callbacks,
terrain destruction, audiovisual behavior, balance, or human-play parity.
