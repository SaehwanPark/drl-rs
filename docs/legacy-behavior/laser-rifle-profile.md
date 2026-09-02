# Laser Rifle typed behavior-profile evidence

Status: delivered typed ordinary-fire five-shot profile and first-, second-,
third-, fourth-, fifth-, sixth-, and seventh-level chainfire execution in
`0.2.307`. Project `0.2.339` adds typed Plasma classification for successful
direct ordinary and bounded chainfire target hits; higher chainfire levels,
spread/routing, controlled legacy runtime comparison, and audiovisual parity
remain `NOT_RUN`.

## Pinned source

Evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`:

- `bin/data/drl/items/eitems.lua:289-326` defines `ulaser` as an exotic ranged
  weapon using the `cell` family, with a 40-cell capacity, `damage = "1d7"`,
  `damagetype = DAMAGE_PLASMA`, and `shots = 5`. No `shotcost` field is
  declared; only the alternate chainfire perk is attached during creation.
- `src/dfitem.pas:247-255` loads the declared `Shots` and `ShotCost` fields,
  defaulting absent values to zero.
- `src/dfitem.pas:627-634` clamps the effective per-projectile cost to at
  least one and multiplies it by the resolved shot count before firing.
- `src/dfbeing.pas:1477-1491` resolves ordinary fire to five projectiles and
  the chainfire warm-up formula: level zero uses `5 - (5 div 3) = 4`, level
  one keeps `5`, and level two and later add `5 div 2 = 2` to produce seven;
  chainfire adjustments require alternate fire.
- `src/dfbeing.pas:1496-1514` checks and debits the aggregate ammunition cost
  before emitting the ordered projectile loop at `:498-510`.

## DRL-Rust boundary

The immutable `drl_core::behavior::LASER_RIFLE_BEHAVIOR` profile records
ordered `AttackEffect::ProjectileCount(5)`, one-cell ordinary cost, and
`AlternateAction::Chainfire` fragments for four/four, five/five, and seven/seven
projectile/cell levels (the seven/seven profile covers bounded warm-up levels
two through six). The existing ranged command path remains execution authority
for target/LOS/range and death-drop preflight, damage RNG, event ordering, and
transactional clip consumption. In `0.2.339`, successful Laser Rifle direct
ordinary and first-through-seventh chainfire target hits select the existing
typed Plasma path, so Blue Armor's 20% resistance applies before flat
protection without changing raw rolls, clip costs, warm-up, or RNG order.
Direct integration tests verify five ordered
ordinary events, four-, five-, and seven-ordered chainfire events, four-, five-,
and seven-cell chainfire consumption, atomic below-cost rejection, warm-up
reset/advancement through level seven, and deterministic replay.

Higher chainfire levels, the legacy alternate perk's exact routing, timing,
and accuracy, controlled runtime comparison, and audiovisual parity remain
deferred; source similarity alone is not parity proof. Direct-core, replay,
MCP, and BrowserSession/physical-key tests verify the bounded execution
boundary.
