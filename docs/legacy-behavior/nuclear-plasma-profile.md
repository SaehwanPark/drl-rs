# Nuclear Plasma Rifle typed behavior-profile evidence

Status: delivered typed behavior profile for `0.2.204`; alternate chainfire,
controlled legacy runtime comparison, and audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. Its unrelated local
audio metadata changes and untracked `fpcvalkyrie/` directory are outside this
evidence.

- `bin/data/drl/items/eitems.lua:436-472` defines `unplasma`, attaches the
  `perk_altreload_nuke` and `perk_weapon_recharge` perks, and sets the recharge
  delay `40` and cadence `2`.
- `bin/data/drl/perks.lua:223-249` defines the confirmed full-clip nuclear
  overload transition; `bin/data/drl/perks.lua:350-386` defines the periodic
  recharge callback.
- `src/drlinventory.pas:238-244` ticks equipped inventory slots, while
  `src/dfbeing.pas:1619-1629` and `src/dflevel.pas:1378-1388` establish the
  actor/level tick ownership.

## DRL-Rust boundary

The immutable `NUCLEAR_PLASMA_BEHAVIOR` profile records the already-delivered
typed `AlternateAction::Overload` and `PeriodicEffect::Recharge` fragments
(delay `40`, cadence `2`, amount `1`). Dedicated `nuclear_overload` and
`WeaponRechargeState` transitions remain the execution authorities; no new
command, callback registry, replay-wire field, or gameplay timing is added.

The profile intentionally does not claim the legacy `perk_altfire_chainfire`
callback, dynamic target rotation, or other deferred weapon behavior. Those
remain separate evidence-backed slices.
