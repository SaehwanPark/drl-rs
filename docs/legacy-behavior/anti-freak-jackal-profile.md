# Anti-Freak Jackal typed aimed-fire evidence

Status: delivered typed Anti-Freak Jackal aimed-fire support in `0.2.250`;
the legacy delayed explosion callback remains `NOT_RUN`. Generic ranged
execution and the shared aimed policy remain the Rust authorities.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/uitems.lua:321-357` defines `ujackal` with a six-round
  clip, Ammo9mm, no explicit `shots` field, `damage 5d3`, radius 1, red color,
  and the `perk_altfire_aimed` hook. The absent shot count is resolved as one
  projectile by the cited Pascal fallback below.
- `src/dfbeing.pas:1477-1515` falls back to one projectile when a weapon does
  not provide an explicit shot count and resolves ammunition before the fire
  loop. The Rust contract intentionally rejects an under-supplied clip
  atomically rather than reproducing legacy partial-ammo reduction.
- `src/dfitem.pas:627-634` clamps an absent zero `ShotCost` to one round before
  callback multipliers are applied.
- `bin/data/drl/perks.lua:128-169` supplies the aimed +3 accuracy and doubled fire-cost
  policy. The callback's delayed explosion state is outside this slice.

## DRL-Rust boundary

`ANTI_FREAK_JACKAL_BEHAVIOR` records one ordered projectile, one Ammo9mm
round, and the shared aimed-fire fragment (+3 accuracy and 2× action cost).
Generic ranged execution remains authoritative for target legality, line of
sight, range, damage RNG, event ordering, and transactional clip consumption.
An accepted aimed command consumes one round and pays `ActionCost(2_000)`.

Direct-core, ScenarioRunner/replay, MCP action-catalog/JSON, and
`BrowserSession` tests verify deterministic events, observations, effects,
scene state, and replay parity. Empty-clip rejection is state-identical.

The legacy explosion delay 40, radius 1, red presentation, callback state and
timing, controlled runtime, browser capture, and audiovisual parity remain
deferred; source similarity alone is not parity proof.
