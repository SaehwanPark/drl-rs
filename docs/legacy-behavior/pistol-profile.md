# Pistol typed behavior-profile evidence

Status: delivered typed profile for `0.2.218`; aimed-fire callback semantics,
exact legacy timing/accuracy, controlled runtime comparison, and audiovisual
parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/items.lua` declares `pistol` as a ranged weapon using
  the `ammo` family and attaches the `perk_altfire_aimed` callback, which
  remains outside this slice.
- `src/dfitem.pas:247-252` defaults an absent `shots` field to zero, and
  `src/dfbeing.pas:1477-1480` resolves ordinary ranged fire with
  `iShots := Max(aGun.Shots, 1)`, so this path emits one projectile.
- The Rust definition maps the stable family to `AmmoType::Ammo9mm`; ordinary
  ranged execution consumes one clip round after complete target, line-of-
  sight, range, and death-drop preflight.

## DRL-Rust boundary

The immutable `drl_core::behavior::PISTOL_BEHAVIOR` profile records ordered
`AttackEffect::ProjectileCount(1)` and
`ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 1 }` fragments. Generic
ranged execution remains authoritative for legality checks, damage RNG, event
ordering, and transactional clip consumption. The profile does not reinterpret
the legacy aimed-fire callback or claim exact legacy accuracy/timing.

A controlled legacy runtime comparison, browser capture, and audiovisual parity
remain deferred and are not inferred from source similarity alone.
