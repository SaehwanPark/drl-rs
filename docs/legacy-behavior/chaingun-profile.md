# Chaingun typed behavior-profile evidence

Status: delivered typed ordinary-fire four-shot profile and bounded first- and
second-level chainfire for `0.2.275`; third-and-later chainfire levels,
spread/routing, controlled legacy runtime comparison, and audiovisual parity
remain `NOT_RUN`.

## Pinned source

Evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`:

- `bin/data/drl/items/items.lua:694-725` defines `chaingun` as a ranged weapon
  using the `ammo` family, with a 40-round capacity and `shots = 4`. No
  `shotcost` field is declared; only the alternate chainfire perk is attached
  during creation.
- `src/dfitem.pas:247-255` loads the declared `Shots` and `ShotCost` fields,
  defaulting absent values to zero.
- `src/dfitem.pas:627-634` clamps the effective per-projectile cost to at
  least one and multiplies it by the resolved shot count before firing.
- `src/dfbeing.pas:1477-1481` resolves ordinary fire to four projectiles;
- `src/dfbeing.pas:1477-1491` resolves ordinary fire to four projectiles,
  first-level alternate chainfire to three (`4 - (4 div 3)`), and the
  second-level continuation to four; later levels are intentionally outside
  this bounded slice.
- `src/dfbeing.pas:1496-1514` checks and debits the aggregate ammunition cost
  before emitting the ordered projectile loop at `:498-510`.
- `src/dfbeing.pas:900-950` resets chain state for each fire attempt and clears
  continuation after ordinary fire; the typed Rust state follows this reset
  boundary for accepted commands without claiming legacy callback timing.

## DRL-Rust boundary

The immutable `drl_core::behavior::CHAINGUN_BEHAVIOR` profile records ordered
`AttackEffect::ProjectileCount(4)`,
`ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 1 }`, and typed first- and
second-level chainfire fragments. The existing ranged command path remains
execution authority for target/LOS/range and death-drop preflight, damage RNG,
event ordering, and transactional clip consumption. The typed
`AttackRangedChainfire` command preflights three rounds at level zero and four
rounds at level one atomically, emits the corresponding ordered attack events
(remaining slots become deterministic no-op misses if an earlier projectile
lethally ends the target), and advances observable chain state only after
acceptance; ordinary fire resets that state. Direct integration, replay, MCP
catalog/JSON, snapshot, and BrowserSession tests verify these boundaries.

Third-and-later chainfire levels, legacy target rotation/spread, exact
timing/accuracy, controlled runtime comparison, and audiovisual parity remain
deferred; source similarity alone is not parity proof.
