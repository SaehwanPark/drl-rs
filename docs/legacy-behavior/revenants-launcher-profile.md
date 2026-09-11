# Revenant's Launcher typed behavior-profile evidence

Status: delivered typed exact-hit/direct Fire profile through `0.2.354` and
bounded radius-3 Fire fanout in `0.2.355`; homing, projectile routing, exact
legacy timing, controlled legacy runtime comparison, and audiovisual parity
remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/uitems.lua:632-664` defines `urbazooka` (Revenant's
  Launcher) with `IF_EXACTHIT`, a one-rocket clip (`ammomax = 1`),
  `damage = "7d6"`, `damagetype = DAMAGE_FIRE`, radius `3`, and a delay-`40`
  explosion record.
- `src/dfbeing.pas:2166-2167` carries `Damage_Fire` into the target's fire
  resistance lookup, and `src/dfbeing.pas:2178-2182` applies percentage
  resistance before the subsequent flat protection, matching Red Armor's
  catalog-defined 25% Fire resistance.
- `src/dflevel.pas:991-1095` rolls the explosion payload once per clear cell,
  applies integer distance falloff and radial knockback before damage, de-dupes
  actors, and destroys ordinary ground items only when post-falloff damage is
  greater than `10`.

## DRL-Rust boundary

The immutable `drl_core::behavior::REVENANTS_LAUNCHER_BEHAVIOR` profile records
the exact-hit fragment and delay-40/radius-3/knockback-8 schedule metadata. In
`0.2.354`, successful direct target damage was routed through typed Fire; in
`0.2.355`, the accepted shot then emits the distinct schedule event and applies
an immediate bounded radius-3 Fire fanout. The fanout consumes one ordered
`7d6` roll per clear cell, applies the shared distance falloff and radial
`damage / 8` knockback, and uses the ordinary ground-item `>10` threshold.
Delay remains presentation metadata; no pending queue or callback registry is
introduced. Dedicated combat resolution remains execution authority for
LOS/range/clip/action-cost validation, damage RNG, event ordering, splash
mutation, and atomic death-drop preflight. Firing with an empty clip rejects
atomically with `CommandError::NoAmmoInClip` before clip or RNG mutation.

Homing, projectile routing, delayed explosions, exact legacy timing, controlled
runtime comparison, and audiovisual parity remain deferred and are not inferred
from source similarity alone.
