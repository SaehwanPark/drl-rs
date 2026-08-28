# Combat Pistol typed behavior-profile evidence

Status: active ordinary-fire profile target for `0.2.220`; aimed-fire callback
semantics, exact legacy timing/accuracy, controlled runtime comparison, and
audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/eitems.lua:171-201` declares `ucpistol` (displayed as
  “combat pistol”) as a ranged weapon using the `ammo` family. It does not
  declare a multi-shot `shots` count or a `shotcost`; its `OnCreate` callback
  attaches `perk_altfire_aimed`, which remains outside this ordinary-fire
  slice.
- `src/dfitem.pas:249-252` defaults absent `shots` and `shotcost` fields to
  zero, and `src/dfbeing.pas:1477-1481` resolves ordinary fire with
  `iShots := Max(aGun.Shots, 1)`, so the ordinary path emits one projectile.
- `src/dfitem.pas:627-634` applies `math.Max(ShotCost, 1)` and multiplies by
  the resolved shot count; with the absent `shotcost` default and one ordinary
  shot, the legacy cost is one round before any callback multiplier.
- The Rust definition maps the stable family to `AmmoType::Ammo9mm`; ordinary
  ranged execution consumes one clip round after complete target, line-of-
  sight, range, and death-drop preflight.

## DRL-Rust boundary

The immutable `drl_core::behavior::COMBAT_PISTOL_BEHAVIOR` profile records
ordered `AttackEffect::ProjectileCount(1)` and
`ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 1 }` fragments. Generic
ranged execution remains authoritative for legality checks, damage RNG, event
ordering, and transactional clip consumption. The profile does not reinterpret
the legacy aimed-fire callback or claim exact legacy accuracy/timing.

A controlled legacy runtime comparison, browser capture, and audiovisual parity
remain deferred and are not inferred from source similarity alone.
