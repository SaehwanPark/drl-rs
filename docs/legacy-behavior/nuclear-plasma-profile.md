# Nuclear Plasma Rifle typed behavior-profile evidence

Status: delivered typed behavior profile and first-, second-, third-, fourth-,
fifth-, sixth-, and seventh-level chainfire execution in `0.2.306`; eighth-and-later
chainfire levels, controlled legacy runtime comparison, and audiovisual parity
remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. Its unrelated local
audio metadata changes and untracked `fpcvalkyrie/` directory are outside this
evidence.

- `bin/data/drl/items/eitems.lua:436-472` defines `unplasma`, attaches the
  `perk_altfire_chainfire`, `perk_altreload_nuke`, and `perk_weapon_recharge`
  perks, and sets the recharge delay `40` and cadence `2`.
- `bin/data/drl/perks.lua:223-249` defines the confirmed full-clip nuclear
  overload transition; `bin/data/drl/perks.lua:350-386` defines the periodic
  recharge callback.
- `src/drlinventory.pas:238-244` ticks equipped inventory slots, while
  `src/dfbeing.pas:1619-1629` and `src/dflevel.pas:1378-1388` establish the
  actor/level tick ownership.

## DRL-Rust boundary

The immutable `NUCLEAR_PLASMA_BEHAVIOR` profile records the typed direct Plasma
target path alongside the six-projectile
ordinary volley, one-cell cost, first-level
`AlternateAction::Chainfire { shot_count: 4, ammo_cost: 4 }`, second-level
`AlternateAction::ChainfireLevel { level: 1, shot_count: 6, ammo_cost: 6 }`,
third-level `AlternateAction::ChainfireLevel { level: 2, shot_count: 9,
ammo_cost: 9 }`, fourth-level `AlternateAction::ChainfireLevel { level: 3,
shot_count: 9, ammo_cost: 9 }`, fifth-level `AlternateAction::ChainfireLevel {
level: 4, shot_count: 9, ammo_cost: 9 }`, and sixth-level
`AlternateAction::ChainfireLevel { level: 5, shot_count: 9, ammo_cost: 9 }`,
seventh-level `AlternateAction::ChainfireLevel { level: 6, shot_count: 9,
ammo_cost: 9 }`,
typed `AlternateAction::Overload`, and
`PeriodicEffect::Recharge` fragments
(delay `40`, cadence `2`, amount `1`). Generic ranged execution owns the
bounded chainfire command while dedicated `nuclear_overload` and
`WeaponRechargeState` transitions remain the execution authorities for their
existing actions; no new command, callback registry, replay-wire field, or
recharge timing is added.

The profile intentionally does not claim eighth-and-later chainfire levels,
dynamic target rotation, or other deferred weapon behavior. Those remain
separate evidence-backed slices.
