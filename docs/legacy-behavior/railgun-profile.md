# Railgun typed behavior-profile evidence

Status: delivered ordinary-fire cost and bounded clear-ray piercing profile for
`0.2.255`; exact legacy timing/accuracy, controlled runtime comparison, and
audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/uitems.lua:772-804` declares `urailgun` (displayed as
  “Railgun”) as a ranged weapon using the `cell` ammo family, `8d8` damage,
  and `shotcost = 5`. It does not declare a multi-shot `shots` count and sets
  `IF_RAYGUN` plus `IF_PIERCEHIT`.
- `src/dfitem.pas:249-252` defaults absent `shots` to zero and preserves the
  explicit `shotcost`; `src/dfbeing.pas:1477-1481` resolves ordinary fire with
  `iShots := Max(aGun.Shots, 1)`, so the ordinary path emits one projectile.
- `src/dfitem.pas:627-634` applies `math.Max(ShotCost, 1)` and multiplies by
  the resolved shot count, yielding five cells for this one-projectile
  ordinary shot before any callback multiplier.
- `src/dfbeing.pas:2428-2442` rolls one damage value before traversing the
  missile path; `:2453-2494` advances through clear cells and `:2496-2522`
  checks each actor encountered. `:2558-2563` continues after a hit only for
  `IF_PIERCEHIT`, while `:2536-2549` leaves the zero-radius Railgun impact
  without knockback.
- The Rust definition maps the stable family to `AmmoType::Cell`; generic
  ranged execution validates the five-cell clip deficit before consuming clip
  state or combat RNG.

## DRL-Rust boundary

The immutable `drl_core::behavior::RAILGUN_BEHAVIOR` profile records ordered
`AttackEffect::ProjectileCount(1)`, `HitEffect::Pierce`, and
`ResourceCost::Ammo { ammo_type: Cell, amount: 5 }` fragments. The existing
`Item::shot_cost` helper and generic ranged execution remain authoritative for
preflight and transactional clip consumption; the bounded Railgun resolver
owns clear-ray traversal, per-actor hit checks, and the shared damage roll.
Exact legacy ray geometry, spread/falloff, and runtime parity remain outside
this contract.

A controlled legacy runtime comparison, browser capture, and audiovisual parity
remain deferred and are not inferred from source similarity alone.
