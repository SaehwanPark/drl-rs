# Revenant's Launcher typed behavior-profile evidence

Status: delivered typed exact-hit profile for `0.2.213` and `0.2.354` typed
direct Fire mitigation; homing, projectile routing, delayed explosions,
controlled legacy runtime comparison, and audiovisual parity remain
`NOT_RUN`.

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

## DRL-Rust boundary

The immutable `drl_core::behavior::REVENANTS_LAUNCHER_BEHAVIOR` profile records
one `AttackEffect::ExactHit` fragment. In `0.2.354`, successful direct target
damage is routed through the existing typed Fire path, so Red Armor's
catalog-defined 25% resistance applies before flat protection (4) while raw
`7d6` rolls and RNG order remain identical between unarmored and armored runs.
Dedicated combat resolution remains execution authority for LOS/range/clip/
action-cost validation, damage RNG, and ordered attack/damage events. Firing
with an empty clip rejects atomically with `CommandError::NoAmmoInClip` before
clip or RNG mutation.

Homing, projectile routing, delayed explosions, exact legacy timing, controlled
runtime comparison, and audiovisual parity remain deferred and are not inferred
from source similarity alone.
