# Missile Launcher typed behavior-profile evidence

Status: delivered typed behavior profile for `0.2.211`, `0.2.348` direct Fire
mitigation, and `0.2.356` bounded radius-3 Fire fanout. Exact legacy metric /
traversal, rocket-jump, delayed timing, controlled runtime comparison, and
audiovisual parity remain `NOT_RUN` or explicitly deferred.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/eitems.lua:397-434` defines `umbazooka` (Missile
  Launcher) with a four-rocket clip, `IF_SINGLERELOAD`, reload time `12`,
  `damage = "6d6"`, `damagetype = DAMAGE_FIRE`, and an `OnCreate` hook that
  adds `perk_altreload_full`.
- `bin/data/drl/perks.lua:204-220` implements `perk_altreload_full` with
  `being:full_reload(self)` and caps its aggregate score-count cost at `2500`.
- `bin/data/core/being.lua:446-478` supplies the complete-deficit reload loop,
  while `src/dfbeing.pas:1407-1457` limits the ordinary flagged path to one
  rocket per reload action.
- `src/dfbeing.pas:2167, 2178-2182` carries `DAMAGE_FIRE` into target resistance
  mitigation, applying Red Armor's 25% Fire resistance before flat protection.
- `src/dfdata.pas:204-212, 896-905` supplies the omitted explosion knockback
  default of `8`; `src/dfbeing.pas:2430-2442,2536-2550,2629-2644` shows that
  radius weapons skip direct `ApplyDamage` for the explosion branch and invoke
  the final-impact explosion even after a miss or covered shot.
- `src/dflevel.pas:991-1095` records one damage roll per clear cell, line of
  sight, actor de-duplication, knockback before damage, and ordinary item
  destruction when post-falloff damage exceeds `10`. `src/vrltools.pas:897-900`
  uses the legacy octile-like `Distance` metric, yielding 37 radius-three cells;
  the current bounded Rust helper intentionally retains its established
  center-first/Chebyshev 49-cell policy until an explicit migration slice.

## DRL-Rust boundary

The immutable `drl_core::behavior::MISSILE_LAUNCHER_BEHAVIOR` profile records
one-projectile/one-rocket ordinary fire, typed Fire direct damage,
`ScheduleExplosion { delay: 40, radius: 3, knockback: Some(8) }`, ordered
`AlternateAction::Reload`, and `AlternateAction::FullReload { cost_cap: 2500 }`
fragments. Dedicated ordinary reload and `MissileLauncherTransition` planner
paths remain execution authority for one-rocket loading, full-deficit reserve
checks, capped action cost, and transactional rejection behavior.

In `0.2.348`, successful direct target damage is routed through the existing
typed Fire path, so Red Armor's catalog-defined 25% resistance applies before
flat protection (4) while raw rolls and RNG order remain identical between
unarmored and armored runs. In `0.2.356`, an accepted hit emits the distinct
schedule metadata and immediately resolves the bounded splash; a miss emits
the same schedule and splash without a direct damage event. The current Rust
resolver consumes one ordered `6d6` roll per clear cell, applies integer
distance falloff, de-duplicates actors, applies radial `damage / 8` knockback
before typed Fire damage, damages the firing actor, and removes the lowest-ID
ordinary ground item when post-falloff damage exceeds `10`. Rejection
preflights splash death-drop destinations before clip/RNG mutation. Firing with
an empty clip rejects atomically with `CommandError::NoAmmoInClip` before clip
or RNG mutation.

Exact legacy metric/traversal, pending delayed timing, rocket-jump, controlled
runtime comparison, and audiovisual parity remain deferred and are not inferred
from source similarity alone.
