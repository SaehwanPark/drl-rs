# Rocket Launcher typed behavior-profile evidence

Status: delivered typed profile for `0.2.219`; rocket-jump and explosion
callback semantics, exact legacy timing/accuracy, controlled runtime
comparison, and audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/items.lua:657-688` declares `bazooka` (displayed as
  “rocket launcher”) as a ranged weapon using the `rocket` ammo family and
  does not declare a multi-shot `shots` count. The same record attaches the
  `perk_altfire_rocketjump` callback and defines an explosion payload; both
  remain outside this ordinary-fire slice.
- `src/dfitem.pas:247-252` defaults an absent `shots` field to zero, while
  `src/dfbeing.pas:1477-1480` resolves ordinary fire with
  `iShots := Max(aGun.Shots, 1)`, so the ordinary path emits one projectile.

## DRL-Rust boundary

The immutable `drl_core::behavior::ROCKET_LAUNCHER_BEHAVIOR` profile records
ordered `AttackEffect::ProjectileCount(1)` and
`ResourceCost::Ammo { ammo_type: Rocket, amount: 1 }` fragments. Generic
ranged execution remains authoritative for legality checks, damage RNG, event
ordering, and transactional clip consumption. The profile does not reinterpret
the legacy rocket-jump or delayed explosion callbacks.

A controlled legacy runtime comparison, browser capture, and audiovisual parity
remain deferred and are not inferred from source similarity alone.
