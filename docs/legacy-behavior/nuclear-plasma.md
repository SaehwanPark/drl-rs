# Nuclear Plasma Rifle periodic recharge evidence

Status: source-backed behavior evidence; controlled legacy runtime comparison
is `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. It has unrelated local
audio metadata changes and an untracked `fpcvalkyrie/` directory; those files
are outside this evidence.

- `bin/data/drl/items/eitems.lua:436-472` defines `unplasma` as a 24-cell
  Nuclear Plasma Rifle, adds `perk_weapon_recharge`, and sets `delay = 40` and
  `tick = 2` in `OnCreate`.
- `bin/data/drl/perks.lua:350-386` increments the equipped item's recharge
  timer by one on each item tick, restores `amount` when the timer reaches
  `delay + tick`, then subtracts `tick`; firing resets the timer to zero and
  the clip is clamped at capacity.
- `src/drlinventory.pas:238-244` ticks only equipped inventory slots, while
  `src/dfbeing.pas:1619-1629` and `src/dflevel.pas:1378-1388` establish the
  actor/level tick ownership.

## Bounded Rust contract

DRL-Rust uses an explicit `WeaponRechargePolicy` for the Nuclear Plasma Rifle:
delay `40`, cadence `2`, amount `1`, and clip capacity `24`. The headless core
advances this state once after each accepted player command, restoring one cell
at tick `42` and every two ticks while below capacity. Full clips leave the
timer unchanged; successful fire resets it; rejected commands roll back timer,
clip, reserve, and RNG through the existing transaction guard.

The existing `WeaponRecharged` event reports the restored amount, resulting
clip, maximum clip, and retained timer. ScenarioRunner/replay and
BrowserSession/direct-core tests establish deterministic boundary parity. No
new command or replay-wire field is required.

## Boundaries and open questions

This slice does not implement Nuclear Plasma alternate nuke/chainfire behavior,
other rechargeable families, partial-reserve policy, controlled legacy runtime
comparison, or audiovisual parity. The accepted-command tick is the bounded
deterministic abstraction and is not a claim of exact legacy wall-clock
cadence.
