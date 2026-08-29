# Super Shotgun typed behavior-profile evidence

Status: delivered typed ordinary-fire dual-shot profile for `0.2.228`;
spread/falloff, controlled legacy runtime comparison, and audiovisual parity
remain `NOT_RUN`.

## Pinned source

Evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`:

- `bin/data/drl/items/eitems.lua:261-287` defines `udshotgun` as a ranged
  weapon using the shell family, with a two-shell capacity and `shots = 2`.
  No `shotcost` field is declared, so the legacy default is one shell per
  projectile.
- `src/dfitem.pas:247-255` loads `Shots` and `ShotCost`, defaulting absent
  values to zero.
- `src/dfitem.pas:627-634` applies `Max(ShotCost, 1)` and multiplies by the
  resolved shot count before the weapon is fired, producing a two-shell
  aggregate cost for this two-projectile action.
- `src/dfbeing.pas:1477-1514` resolves at least one projectile, computes the
  aggregate cost, and debits ammunition before emitting the ordinary volley.

## DRL-Rust boundary

The immutable `drl_core::behavior::SUPER_SHOTGUN_BEHAVIOR` profile records
ordered `AttackEffect::ProjectileCount(2)` and
`ResourceCost::Ammo { ammo_type: Shells, amount: 1 }` fragments. The existing
ranged command path remains execution authority for target/LOS/range and
death-drop preflight, damage RNG, event ordering, and transactional clip
consumption. Direct integration tests verify two attack events, two-shell
consumption, atomic below-cost rejection, and deterministic replay.

The legacy `IF_DUALSHOTGUN` presentation/spread behavior, exact timing and
accuracy, controlled runtime comparison, and audiovisual parity remain
deferred; source similarity alone is not parity proof.
