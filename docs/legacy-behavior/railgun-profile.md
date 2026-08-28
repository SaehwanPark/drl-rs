# Railgun typed behavior-profile evidence

Status: active ordinary-fire cost profile target for `0.2.223`; ray/piercing,
exact legacy timing/accuracy, controlled runtime comparison, and audiovisual
parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/uitems.lua:772-804` declares `urailgun` (displayed as
  “Railgun”) as a ranged weapon using the `cell` ammo family and sets
  `shotcost = 5`. It does not declare a multi-shot `shots` count. Its
  `IF_RAYGUN`/`IF_PIERCEHIT` flags and projectile routing remain outside this
  cost-only slice.
- `src/dfitem.pas:249-252` defaults absent `shots` to zero and preserves the
  explicit `shotcost`; `src/dfbeing.pas:1477-1481` resolves ordinary fire with
  `iShots := Max(aGun.Shots, 1)`, so the ordinary path emits one projectile.
- `src/dfitem.pas:627-634` applies `math.Max(ShotCost, 1)` and multiplies by
  the resolved shot count, yielding five cells for this one-projectile
  ordinary shot before any callback multiplier.
- The Rust definition maps the stable family to `AmmoType::Cell`; generic
  ranged execution validates the five-cell clip deficit before consuming clip
  state or combat RNG.

## DRL-Rust boundary

The immutable `drl_core::behavior::RAILGUN_BEHAVIOR` profile records ordered
`AttackEffect::ProjectileCount(1)` and
`ResourceCost::Ammo { ammo_type: Cell, amount: 5 }` fragments. The existing
`Item::shot_cost` helper and generic ranged execution remain authoritative for
preflight, transactional clip consumption, damage RNG, and event ordering. The
profile intentionally does not claim exact ray, piercing, spread/falloff, or
runtime parity.

A controlled legacy runtime comparison, browser capture, and audiovisual parity
remain deferred and are not inferred from source similarity alone.
