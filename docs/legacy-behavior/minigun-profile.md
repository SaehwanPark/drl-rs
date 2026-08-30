# Minigun typed behavior-profile evidence

Status: delivered typed ordinary-fire eight-shot profile plus first- and
second-level chainfire execution for `0.2.277`; third-level chainfire is the
active `0.2.278` slice; fourth-and-later chainfire,
spread/routing, controlled legacy runtime comparison, and audiovisual parity
remain `NOT_RUN`.

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
- `src/dfbeing.pas:1484-1488` reduces a first-level alternate burst by
  `Shots div 3`, yielding six projectiles for the eight-shot Minigun. The
  same routine leaves the level-one warm-up at the full eight-shot count
  (`FChainFire = 1`); at level two it adds `Shots div 2`, yielding twelve
  projectiles. Fourth-and-later levels remain outside this bounded execution
  slice.
- `src/dfbeing.pas:1496-1514` checks and debits the aggregate ammunition cost
  before emitting the ordered projectile loop at `:498-510`.

## DRL-Rust boundary

The immutable `drl_core::behavior::MINIGUN_BEHAVIOR` profile records ordered
`ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 1 }`, and typed
`AlternateAction::Chainfire`/`ChainfireLevel` fragments for six, eight, and
twelve projectiles. The existing `AttackRangedChainfire` command accepts
Minigun at warm-up level zero, one, or two, preflights and consumes six, eight,
or twelve loaded rounds, emits the corresponding ordered outcomes
(deterministic no-op misses fill post-lethal slots), and advances the shared
warm-up state only after acceptance. Direct-core, replay, MCP, and
BrowserSession tests verify the boundary and atomic rejection.

Fourth-and-later chainfire levels and the legacy alternate perk's exact
routing, timing, and accuracy remain deferred; source similarity alone is not
parity proof.
