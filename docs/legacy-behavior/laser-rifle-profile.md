# Laser Rifle typed behavior-profile evidence

Status: delivered typed ordinary-fire five-shot profile and first-level
chainfire execution in `0.2.262`; higher chainfire levels, spread/routing,
controlled legacy runtime comparison, and audiovisual parity remain `NOT_RUN`.

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
- `src/dfbeing.pas:1477-1491` resolves ordinary fire to five projectiles and
  the first chainfire level's `5 - (5 div 3) = 4` projectiles; chainfire
  adjustments require alternate fire.
- `src/dfbeing.pas:1496-1514` checks and debits the aggregate ammunition cost
  before emitting the ordered projectile loop at `:498-510`.

## DRL-Rust boundary

The immutable `drl_core::behavior::LASER_RIFLE_BEHAVIOR` profile records
ordered `AttackEffect::ProjectileCount(5)`, one-cell ordinary cost, and
`AlternateAction::Chainfire { shot_count: 4, ammo_cost: 4 }` fragments. The
existing ranged command path remains execution authority for target/LOS/range
and death-drop preflight, damage RNG, event ordering, and transactional clip
consumption. Direct integration tests verify five ordered ordinary events,
four ordered chainfire events, four-cell chainfire consumption, atomic
below-cost rejection, warm-up reset/advancement, and deterministic replay.

Higher chainfire levels, the legacy alternate perk's exact routing, timing,
and accuracy, controlled runtime comparison, and audiovisual parity remain
deferred; source similarity alone is not parity proof. Direct-core, replay,
MCP, and BrowserSession/physical-key tests verify the bounded execution
boundary.
