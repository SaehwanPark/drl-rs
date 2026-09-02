# Nuclear Plasma Rifle typed ordinary-volley evidence

Status: delivered typed Nuclear Plasma Rifle ordinary-fire and first- through
seventh-level chainfire support in `0.2.306`;
`0.2.338` now classifies successful ordinary and already implemented chainfire
direct target hits as typed Plasma;
eighth-and-later chainfire levels,
controlled legacy runtime comparison, and audiovisual parity remain `NOT_RUN`.
Existing overload and periodic recharge evidence is retained in
`nuclear-plasma-profile.md` and `nuclear-plasma.md`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. Its unrelated local
audio metadata changes and untracked `fpcvalkyrie/` directory are outside this
evidence.

- `bin/data/drl/items/eitems.lua:436-472` defines `unplasma` with a 24-cell
  clip, `shots = 6`, and Cell ammunition; chainfire, overload, and recharge
  perks are attached separately.
- `src/dfitem.pas:249-255` defaults absent `shotcost` to zero, while
  `src/dfitem.pas:627-634` clamps the effective per-projectile cost to at
  least one and applies the aggregate shot count before firing.
- `src/dfbeing.pas:1477-1515` resolves the six-shot count and deducts the
  aggregate cost before the ordered fire loop. Its chainfire adjustment uses
  `shots - (shots div 3)` at level zero, the unchanged six-shot count at level
  one, and `shots + (shots div 2)` at level two and later; the first seven
  bounded levels therefore emit four, six, nine, nine, nine, nine, and nine
  projectiles.
  The legacy path may partially reduce a clip when ammunition is insufficient,
  which is intentionally outside the Rust atomic below-volley contract.

## DRL-Rust boundary

`NUCLEAR_PLASMA_BEHAVIOR` now records six ordered projectiles and one Cell per
projectile alongside its four-projectile/four-cell first-level,
  six-projectile/six-cell second-level, and nine-projectile/nine-cell third-,
  fourth-, fifth-, sixth-, and seventh-level chainfire, existing typed overload, and
delay-40/cadence-2/amount-1
recharge fragments. Generic ranged execution remains authoritative for target,
line-of-sight, range, damage RNG, event ordering, and transactional clip
consumption; one accepted ordinary command consumes six cells, while first-,
second-, third-, fourth-, fifth-, sixth-, and seventh-level chainfire consume
four, six, nine, nine, nine, nine, and nine cells respectively and emit matching ordered ranged
events.

ScenarioRunner/replay, MCP JSON/catalog, and `BrowserSession` tests verify
deterministic parity for both ordinary and chainfire commands. Clips below six
ordinary/second-level or four first-level or nine third-/fourth-/fifth-/sixth-level
chainfire cells are rejected before
mutation, and the existing overload and periodic-recharge ownership are
unchanged.

Gameplay semantics `140` (project `0.2.338`) classifies each successful Nuclear
Plasma Rifle ordinary-fire and already implemented chainfire direct target hit
as `DamageType::Plasma`. The existing typed damage path therefore applies Blue
Armor's catalog-defined 20% Plasma resistance before flat protection without
changing raw attack rolls, clip costs, warm-up state, periodic recharge,
alternate overload, event ordering, or replay/RNG identities. The legacy source
declares `DAMAGE_PLASMA` and carries the item damage family into direct
`ApplyDamage`; this slice records the current Rust boundary only and does not
claim exact legacy timing, accuracy, higher-level chainfire, overload/nuke
effects, or audiovisual parity.

Eighth-and-later chainfire callback state, alternate target routing, exact
timing, controlled runtime, browser capture, and audiovisual parity remain
deferred; source similarity alone is not parity proof.
