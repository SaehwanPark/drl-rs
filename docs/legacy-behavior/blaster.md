# Blaster periodic recharge evidence

Status: source-backed behavior evidence; controlled legacy runtime comparison
is `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. That checkout also has
unrelated local changes in `drlhq`/`drllq` audio metadata and an untracked
`fpcvalkyrie/` directory; those files are outside this evidence.

- `bin/data/drl/items/eitems.lua:135-169` defines `ublaster` with cell ammo,
  a ten-cell maximum clip, `IF_NORELOAD`, and the recharge perk. The item
  creation callback sets delay `30` and amount `1`; the Nuclear Plasma Rifle
  uses the same perk with delay `40` and cadence tick `2` (see
  `nuclear-plasma.md`).
- `bin/data/drl/perks.lua:350-386` starts the recharge timer at zero, increments
  it by one per item tick while the equipped item is below capacity, restores
  one cell when the timer reaches delay plus tick (`40`), subtracts the tick
  cadence (`10`), clamps at capacity, and resets the timer on fire.
- `src/drlinventory.pas:238-244` limits inventory item ticks to equipped slots;
  `src/dfbeing.pas:1619-1629` and `src/dflevel.pas:1378-1388` establish the
  actor/scheduled tick ownership used by the source runtime.

## Bounded Rust contract

DRL-Rust models this callback as a typed `WeaponRechargeState` with explicit
policies for the `Blaster` and Nuclear Plasma Rifle archetypes. The deterministic
headless abstraction advances it once after each accepted player command. The
Blaster restores one cell at tick `40`, then every `10` ticks; Nuclear Plasma
restores one cell at tick `42`, then every `2` ticks. Both leave the timer
unchanged when full and reset it after a successful ranged shot. No reserve
ammunition, random numbers, or extra action cost are involved. The existing
full-game transaction guard restores timer and clip state on rejected commands.

Each restored cell emits one `GameEvent::WeaponRecharged` carrying entity,
item, restored amount, current/max clip, and retained timer values. The event is
projected through MCP JSON and is intentionally a no-op for current render/audio
effects; a future presentation slice may add a cue or visible timer.

## Boundaries and open questions

The accepted-command tick is an explicit deterministic abstraction, not a claim
that the exact scheduled legacy runtime cadence has been reproduced. Manual
reload denial for the other `IF_NORELOAD` families, aimed fire, other
rechargeable weapons, mods, partial-reserve behavior, replay-file migrations,
and audiovisual parity remain separate work.
