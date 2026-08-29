# Anti-Freak Jackal typed aimed-fire evidence

Status: delivered typed Anti-Freak Jackal aimed-fire, delayed-explosion
schedule, bounded radius-1 splash fanout, and radial knockback support in
`0.2.253`; terrain/item destruction and the legacy callback remain `NOT_RUN`.
Generic ranged execution and the shared aimed policy remain the Rust
authorities.

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
- `src/dfdata.pas:212` defines the default explosion knockback as `8`, while
  `src/dfdata.pas:896-905` parses the explosion delay, range, knockback, and
  damage fields (with that default) into the item payload.
- `src/dfbeing.pas:2629-2644` copies the item's radius into the scheduled
  explosion range and carries the rolled damage/type into the delayed payload.
- `src/dflevel.pas:1039-1080` iterates clamped explosion cells, filters them
  through `ShotContact`, rolls damage for each eligible actor, and applies the
  damage type. The exact distance-helper source is not present in the pinned
  checkout, so this slice records the eight-neighbor radius-1 shape as an
  explicit Rust boundary decision rather than claiming runtime parity.
- The same pinned explosion loop computes knockback before `ApplyDamage`: its
  strength is the integer damage ratio against the payload knockback value,
  and a center actor receives the caller-supplied direction rather than an
  inferred radial direction. DRL-Rust makes the radial direction and center
  no-movement rule explicit for this bounded typed slice.
- `bin/data/drl/perks.lua:128-169` supplies the aimed +3 accuracy and doubled fire-cost
  policy. The callback's delayed explosion state is outside this slice.

## DRL-Rust boundary

`ANTI_FREAK_JACKAL_BEHAVIOR` records one ordered projectile, one Ammo9mm
round, the shared aimed-fire fragment (+3 accuracy and 2× action cost), and the
delayed-explosion schedule (delay 40, radius 1, default knockback 8). Generic
ranged execution remains authoritative for target legality, line of sight,
range, direct-hit damage RNG, event ordering, and transactional clip
consumption. An accepted aimed command consumes one round and pays
`ActionCost(2_000)`; a successful hit projects the typed schedule event and
then resolves a deterministic `5d3` fire-damage fanout across the bounded
center-plus-eight-neighbor cells. The resolver derives an integer `damage / 8`
radial displacement for non-center actors before applying each damage result;
blocked destinations stop the movement.

Direct-core, ScenarioRunner/replay, MCP schedule/action-catalog/JSON, and
`BrowserSession` tests verify deterministic events, observations, effects,
scene state, schedule projection, and replay parity. The generic MCP
`DamageApplied` JSON serializer is covered by the environment-damage contract;
an Anti-Freak-specific splash JSON fixture remains outside this slice. Empty-
clip rejection is state-identical.

Terrain/item destruction, red presentation, callback state and timing,
controlled runtime, browser capture, and audiovisual parity remain deferred;
source similarity alone is not parity proof.
