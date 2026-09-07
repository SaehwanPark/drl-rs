# Missile Launcher typed behavior-profile evidence

Status: delivered typed behavior profile for `0.2.211` and `0.2.348` typed
direct Fire mitigation; rocket-jump, explosion, controlled legacy runtime
comparison, and audiovisual parity remain `NOT_RUN`.

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

## DRL-Rust boundary

The immutable `drl_core::behavior::MISSILE_LAUNCHER_BEHAVIOR` profile records
ordered `AlternateAction::Reload` and
`AlternateAction::FullReload { cost_cap: 2500 }` fragments. Dedicated ordinary
reload and `MissileLauncherTransition` planner paths remain execution
authority for one-rocket loading, full-deficit reserve checks, capped action
cost, and transactional rejection behavior.

In `0.2.348`, successful direct target damage is routed through the existing
typed Fire path, so Red Armor's catalog-defined 25% resistance applies before
flat protection (4) while raw rolls and RNG order remain identical between
unarmored and armored runs. Firing with an empty clip rejects atomically with
`CommandError::NoAmmoInClip` before clip or RNG mutation.

Rocket-jump, radius-3 explosion splash, exact legacy timing, controlled runtime
comparison, and audiovisual parity remain deferred and are not inferred from
source similarity alone.
