# Rocket Launcher typed behavior-profile evidence

Status: delivered typed profile plus a bounded actor-only radius-4 direct-hit
fanout in `0.2.327`. Rocket-jump, projectile/cell/ground-item explosion
callbacks, exact delayed timing/accuracy, controlled runtime comparison, and
audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/items.lua:657-688` declares `bazooka` (displayed as
  “rocket launcher”) as a ranged weapon using the `rocket` ammo family, with
  one-shot `ammomax`, `6d6` `DAMAGE_FIRE`, radius `4`, and a delay-`40`
  explosion payload. It does not declare a multi-shot `shots` count. The same
  record attaches the `perk_altfire_rocketjump` callback; alternate fire and
  other callback state remain outside this slice.
- `src/dfitem.pas:247-252` defaults an absent `shots` field to zero, while
  `src/dfbeing.pas:1477-1480` resolves ordinary fire with
  `iShots := Max(aGun.Shots, 1)`, so the ordinary path emits one projectile.
- `src/dfbeing.pas:2629-2644` builds the explosion roll from the weapon's
  damage dice/type and passes its radius and delay to `TLevel.Explosion` after
  a successful projectile hit.
- `src/dflevel.pas:991-1095` iterates clear in-bounds cells, rolls damage per
  cell, applies distance falloff `(distance + 1) div 2`, de-duplicates actors,
  applies radial knockback using the weapon's default knockback, and then
  applies actor damage. The same routine can mutate terrain and ground items;
  those branches are intentionally not claimed by the Rust slice.

## DRL-Rust boundary

The immutable `drl_core::behavior::ROCKET_LAUNCHER_BEHAVIOR` profile records
ordered `AttackEffect::ProjectileCount(1)` and
`ResourceCost::Ammo { ammo_type: Rocket, amount: 1 }` fragments. Generic
ranged execution remains authoritative for legality checks, direct-hit damage,
event ordering, and transactional clip consumption. On a successful direct hit,
`drl_core::rocket_launcher` supplies the typed radius-4 geometry, one `6d6`
roll per clear cell, legacy distance falloff, and `damage / 8` radial
knockback. `Game` emits `RocketLauncherExplosionScheduled` as presentation
metadata and immediately resolves the bounded actor-only fanout: the source is
not self-safe, actors are processed once, and normal death/drop handling is
retained. Ground items, terrain mutation, projectile routing, delayed queues,
rocket-jump, and generic legacy callbacks are not reimplemented.

The current-Rust behavior is covered by direct-core, replay/scenario,
MCP/audio/metrics/render, BrowserSession, and workspace tests. A controlled
legacy runtime comparison, browser capture, and audiovisual parity remain
deferred and are not inferred from source similarity alone.
