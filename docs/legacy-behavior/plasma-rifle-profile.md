# Plasma Rifle typed behavior-profile evidence

Status: delivered typed six-projectile ordinary-fire volley and first- and
second-level chainfire execution in `0.2.279`; higher chainfire levels,
overcharge, exact legacy timing, controlled runtime comparison, and
audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. Its working tree has
unrelated audio/meta edits and an untracked `fpcvalkyrie/` directory; all
claims below come from the pinned revision.

- `bin/data/drl/items/items.lua:727-760` declares the Plasma Rifle with Cell
  ammunition, a six-projectile `shots` value, and no `shotcost` override. Its
  creation callback attaches chainfire and overcharge perks; this slice pins the
  first and second chainfire levels while leaving overcharge and later levels
  outside scope.
- `src/dfitem.pas:249-255` loads the `shots` and `shotcost` fields, while
  `src/dfbeing.pas:1477-1515` resolves six ordinary projectiles and the
  chainfire warm-up formula: level zero uses `6 - (6 div 3) = 4`, level one
  keeps `6`, and level two and later add `6 div 2`. Aggregate ammunition is
  preflighted before the ordered fire loop.
- `src/dfitem.pas:627-634` clamps the absent `shotcost` to one cell per
  projectile, so a complete six-projectile ordinary volley costs six cells.

## DRL-Rust boundary

The immutable `PLASMA_RIFLE_BEHAVIOR` profile records an ordered
`AttackEffect::ProjectileCount(6)`, one-cell `ResourceCost` per ordinary
projectile, and `AlternateAction::Chainfire` fragments for four/four and
six/six projectile/cell levels. Generic ranged execution preflights the
aggregate ordinary or chainfire clip cost before consuming clip state or
combat RNG. Each accepted chainfire command emits its fixed ordered outcomes,
fills post-lethal slots with deterministic no-op misses, and advances the
shared warm-up state only after acceptance.

Scenario/replay, MCP, and `BrowserSession` tests verify first-level four- and
second-level six-ordered-attack events, deterministic state/effect parity,
reload-backed clip restoration, and exact below-cost rejection.
The existing reload path remains authoritative after the volley.

Higher chainfire levels, overcharge, spread/routing, exact timing, controlled
legacy runtime, browser capture, and audiovisual parity remain deferred and are
not inferred from source similarity alone. Direct-core, replay, MCP, and
BrowserSession/physical-key tests verify the bounded execution boundary.
