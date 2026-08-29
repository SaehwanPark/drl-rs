# Minigun typed behavior-profile evidence

Status: delivered typed ordinary-fire eight-shot profile for `0.2.229`;
alternate chainfire, spread/routing, controlled legacy runtime comparison, and
audiovisual parity remain `NOT_RUN`.

## Pinned source

Evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`:

- `bin/data/drl/items/eitems.lua:363-395` defines `uminigun` as a ranged
  weapon using the `ammo` family, with a 200-round capacity and `shots = 8`.
  No `shotcost` field is declared; only the alternate chainfire perk is
  attached during creation.
- `src/dfitem.pas:247-255` loads the declared `Shots` and `ShotCost` fields,
  defaulting absent values to zero.
- `src/dfitem.pas:627-634` clamps the effective per-projectile cost to at
  least one and multiplies it by the resolved shot count before firing.
- `src/dfbeing.pas:1477-1481` resolves ordinary fire to eight projectiles;
  chainfire adjustments require alternate fire.
- `src/dfbeing.pas:1496-1514` checks and debits the aggregate ammunition cost
  before emitting the ordered projectile loop at `:498-510`.

## DRL-Rust boundary

The immutable `drl_core::behavior::MINIGUN_BEHAVIOR` profile records ordered
`AttackEffect::ProjectileCount(8)` and
`ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 1 }` fragments. The existing
ranged command path remains execution authority for target/LOS/range and
death-drop preflight, damage RNG, event ordering, and transactional clip
consumption. Direct integration tests verify eight attack events, eight-round
consumption, atomic below-cost rejection, and deterministic replay.

The legacy alternate chainfire perk, spread/routing, exact timing/accuracy,
controlled runtime comparison, and audiovisual parity remain deferred; source
similarity alone is not parity proof.
