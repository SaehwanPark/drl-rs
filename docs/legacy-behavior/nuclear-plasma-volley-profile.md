# Nuclear Plasma Rifle typed ordinary-volley evidence

Status: delivered typed Nuclear Plasma Rifle ordinary-fire support in
`0.2.249`; chainfire, controlled legacy runtime comparison, and audiovisual
parity remain `NOT_RUN`. Existing overload and periodic recharge evidence is
retained in `nuclear-plasma-profile.md` and `nuclear-plasma.md`.

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
  aggregate cost before the ordered fire loop; the legacy path may partially
  reduce a clip when ammunition is insufficient, which is intentionally outside
  the Rust atomic below-volley contract.

## DRL-Rust boundary

`NUCLEAR_PLASMA_BEHAVIOR` now records six ordered projectiles and one Cell per
projectile alongside the existing typed overload and delay-40/cadence-2/amount-1
recharge fragments. Generic ranged execution remains authoritative for target,
line-of-sight, range, damage RNG, event ordering, and transactional clip
consumption; one accepted command consumes six cells, emits six ordered ranged
events, and resets the typed recharge timer.

ScenarioRunner/replay, MCP JSON/catalog, and `BrowserSession` tests verify
deterministic parity. A clip below six cells is rejected before mutation, and
the existing overload, chainfire exclusion, and periodic-recharge ownership are
unchanged.

Chainfire callback state, alternate target routing, exact timing, controlled
runtime, browser capture, and audiovisual parity remain deferred; source
similarity alone is not parity proof.
