# Laser Rifle typed behavior-profile evidence

Status: delivered typed ordinary-fire five-shot profile for `0.2.231`;
alternate chainfire, spread/routing, controlled legacy runtime comparison, and
audiovisual parity remain `NOT_RUN`.

## Pinned source

Evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`:

- `bin/data/drl/items/eitems.lua:289-326` defines `ulaser` as an exotic ranged
  weapon using the `cell` family, with a 40-cell capacity and `shots = 5`. No
  `shotcost` field is declared; only the alternate chainfire perk is attached
  during creation.
- `src/dfitem.pas:247-255` loads the declared `Shots` and `ShotCost` fields,
  defaulting absent values to zero.
- `src/dfitem.pas:627-634` clamps the effective per-projectile cost to at
  least one and multiplies it by the resolved shot count before firing.
- `src/dfbeing.pas:1477-1481` resolves ordinary fire to five projectiles;
  chainfire adjustments require alternate fire.
- `src/dfbeing.pas:1496-1514` checks and debits the aggregate ammunition cost
  before emitting the ordered projectile loop at `:498-510`.

## DRL-Rust boundary

The immutable `drl_core::behavior::LASER_RIFLE_BEHAVIOR` profile records
ordered `AttackEffect::ProjectileCount(5)` and
`ResourceCost::Ammo { ammo_type: Cell, amount: 1 }` fragments. The existing
ranged command path remains execution authority for target/LOS/range and
death-drop preflight, damage RNG, event ordering, and transactional clip
consumption. Direct integration tests verify five attack events, five-cell
consumption, atomic below-cost rejection, and deterministic replay.

The legacy alternate chainfire perk, spread/routing, exact timing/accuracy,
controlled runtime comparison, and audiovisual parity remain deferred; source
similarity alone is not parity proof.
