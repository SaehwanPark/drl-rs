# Plasma Shotgun typed behavior-profile evidence

Status: delivered ordinary-fire profile and `0.2.242` direct-core/
`BrowserSession` boundary target; full spread/falloff/knockback semantics,
exact legacy timing/accuracy, controlled runtime comparison, and audiovisual
parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/eitems.lua:233-258` declares `upshotgun` (displayed as
  “plasma shotgun”) as a ranged weapon using the `cell` ammo family and sets
  `shotcost = 3`. It does not declare a multi-shot `shots` count; its spread,
  falloff, and knockback fields remain outside this cost-only slice.
- `src/dfitem.pas:249-252` defaults absent `shots` to zero and preserves the
  explicit `shotcost`; `src/dfbeing.pas:1477-1481` resolves ordinary fire with
  `iShots := Max(aGun.Shots, 1)`, so the ordinary path emits one projectile.
- `src/dfitem.pas:627-634` applies `math.Max(ShotCost, 1)` and multiplies by
  the resolved shot count, yielding three cells for this one-projectile
  ordinary shot before any callback multiplier.
- The Rust definition maps the stable family to `AmmoType::Cell`; the generic
  ranged path validates the three-cell clip deficit before consuming clip state
  or combat RNG.

## DRL-Rust boundary

The immutable `drl_core::behavior::PLASMA_SHOTGUN_BEHAVIOR` profile records
ordered `AttackEffect::ProjectileCount(1)` and
`ResourceCost::Ammo { ammo_type: Cell, amount: 3 }` fragments. The existing
`Item::shot_cost` helper and generic ranged execution remain authoritative for
preflight, transactional clip consumption, damage RNG, and event ordering. The
profile intentionally does not claim exact spread/falloff/knockback or runtime
parity.

A controlled legacy runtime comparison, browser capture, and audiovisual parity
remain deferred and are not inferred from source similarity alone.
