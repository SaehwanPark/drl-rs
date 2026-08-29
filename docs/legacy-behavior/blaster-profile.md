# Blaster typed behavior-profile evidence

Status: delivered typed behavior profile and `0.2.243` direct-core/
`BrowserSession` ordinary-fire boundary plus the `0.2.246` aimed-fire vertical
slice; exact legacy callback state/timing, controlled runtime comparison, and
audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. Its unrelated local
audio metadata changes and untracked `fpcvalkyrie/` directory are outside this
evidence.

- `bin/data/drl/items/eitems.lua:135-169` defines `ublaster` with a ten-cell
  clip, `IF_NORELOAD`, and `perk_weapon_recharge`; its creation callback sets
  recharge delay `30` and amount `1`.
- The same declaration attaches the shared `perk_altfire_aimed`; the shared
  perk at `bin/data/drl/perks.lua:128-169` applies +3 accuracy and doubles fire
  time, then clears `pp_aimed` after firing at `bin/data/drl/perks.lua:151-154`.
  Exact callback-state parity is not claimed by the Rust command boundary.
- The same definition has no `shots` or `shotcost` field. `src/dfitem.pas:249-252`
  therefore supplies zero defaults, `src/dfbeing.pas:1477-1481` resolves the
  ordinary path to one projectile, and `src/dfitem.pas:627-634` clamps its
  per-projectile cost to one cell before callback multipliers.
- `bin/data/drl/perks.lua:350-386` increments the equipped recharge timer per
  item tick, restores one cell at delay plus cadence, clamps at capacity, and
  resets the timer after firing.
- `src/drlinventory.pas:238-244` limits item ticks to equipped inventory slots;
  `src/dfbeing.pas:1619-1629` and `src/dflevel.pas:1378-1388` establish actor
  and level tick ownership.

## DRL-Rust boundary

The immutable `BLASTER_BEHAVIOR` profile records the current one-projectile,
one-cell ordinary-fire fragments, the shared typed aimed-fire fragment (+3
accuracy, doubled action cost), and the typed periodic recharge fragment (delay
`30`, cadence `10`, amount `1`). Generic ranged execution and the dedicated
`WeaponRechargeState` transition remain the execution authorities; replay/MCP
JSON/catalog and `BrowserSession` tests verify the aimed boundary without a new
callback registry or replay-wire field.

The profile intentionally does not claim the legacy aimed-fire callback state,
manual-reload policy beyond the existing `IF_NORELOAD` boundary, or other
deferred weapon behavior. Those remain separate evidence-backed slices.
