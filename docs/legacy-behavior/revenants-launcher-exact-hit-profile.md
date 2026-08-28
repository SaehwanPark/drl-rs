# Revenant's Launcher typed behavior-profile evidence

Status: delivered typed behavior profile for `0.2.213`; homing, projectile
routing, delayed explosions, controlled legacy runtime comparison, and
audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/uitems.lua:632-664` defines `urbazooka` (Revenant's
  Launcher) with `IF_EXACTHIT`, a one-rocket clip, and `7d6` damage; no
  item-specific range is declared, so the exact-hit path uses default player
  vision range `8`.
- `src/dfbeing.pas:2317-2344` maps `IF_EXACTHIT` to a 100% ranged to-hit
  result while retaining the normal visibility, range, and damage path.
- The same item carries separate homing and delayed-explosion behavior at
  `uitems.lua:660-663`; those effects remain outside this profile.

## DRL-Rust boundary

The immutable `drl_core::behavior::REVENANTS_LAUNCHER_BEHAVIOR` profile records
one `AttackEffect::ExactHit` fragment. Dedicated combat resolution remains
execution authority for LOS/range/clip/action-cost validation, damage RNG, and
ordered attack/damage events. No command, replay, RNG, or generic
callback-dispatch surface is introduced by the profile.

Homing, projectile routing, delayed explosions, exact legacy timing, controlled
runtime comparison, and audiovisual parity remain deferred and are not inferred
from source similarity alone.
