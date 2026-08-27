# Nuclear BFG 9000 periodic recharge evidence

Status: source-backed behavior evidence; controlled legacy runtime comparison
is `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. It has unrelated local
audio metadata changes and an untracked `fpcvalkyrie/` directory; those files
are outside this evidence.

- `bin/data/drl/items/eitems.lua:474-518` defines `unbfg9000` as a 40-cell
  Nuclear BFG 9000, adds `perk_weapon_recharge`, and sets `delay = 0`,
  `tick = 5`, and `amount = 1` in `OnCreate`.
- `bin/data/drl/perks.lua:350-386` increments the equipped item's recharge
  timer by one on each item tick, restores `amount` when the timer reaches
  `delay + tick`, then subtracts `tick`; firing resets the timer to zero and
  the clip is clamped at capacity.
- `src/drlinventory.pas:238-244` ticks only equipped inventory slots, while
  `src/dfbeing.pas:1619-1629` and `src/dflevel.pas:1378-1388` establish the
  actor/level tick ownership.

## Bounded Rust contract

DRL-Rust uses an explicit `WeaponRechargePolicy` for the Nuclear BFG 9000:
delay `0`, cadence `5`, amount `1`, and clip capacity `40`. The headless core
advances this state once after each accepted player command, restoring one cell
at tick `5` and every five ticks while below capacity. Full clips leave the
timer unchanged; successful fire resets it; rejected commands roll back timer,
clip, reserve, and RNG through the existing transaction guard.

The existing `WeaponRecharged` event reports the restored amount, resulting
clip, maximum clip, and retained timer. ScenarioRunner/replay and
BrowserSession/direct-core tests establish deterministic boundary parity. No
new command or replay-wire field is required.

## Boundaries and open questions

This slice does not implement Nuclear BFG alternate nuke, exact-hit/explosion,
other rechargeable families, partial-reserve policy, controlled legacy runtime
comparison, or audiovisual parity. The accepted-command tick is the bounded
deterministic abstraction and is not a claim of exact legacy wall-clock
cadence.
