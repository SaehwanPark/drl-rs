# Blaster typed behavior-profile evidence

Status: delivered typed behavior profile for `0.2.205`; aimed fire, controlled
legacy runtime comparison, and audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. Its unrelated local
audio metadata changes and untracked `fpcvalkyrie/` directory are outside this
evidence.

- `bin/data/drl/items/eitems.lua:135-169` defines `ublaster` with a ten-cell
  clip, `IF_NORELOAD`, and `perk_weapon_recharge`; its creation callback sets
  recharge delay `30` and amount `1`.
- `bin/data/drl/perks.lua:350-386` increments the equipped recharge timer per
  item tick, restores one cell at delay plus cadence, clamps at capacity, and
  resets the timer after firing.
- `src/drlinventory.pas:238-244` limits item ticks to equipped inventory slots;
  `src/dfbeing.pas:1619-1629` and `src/dflevel.pas:1378-1388` establish actor
  and level tick ownership.

## DRL-Rust boundary

The immutable `BLASTER_BEHAVIOR` profile records the existing typed periodic
recharge fragment (delay `30`, cadence `10`, amount `1`). The dedicated
`WeaponRechargeState` transition remains the execution authority; no new
command, callback registry, replay-wire field, or gameplay timing is added.

The profile intentionally does not claim the legacy aimed-fire callback,
manual-reload policy, or other deferred weapon behavior. Those remain separate
evidence-backed slices.
