# Standard BFG 9000 delayed explosion evidence

Status: delivered bounded typed schedule metadata, immediate actor fanout, and
thresholded ordinary ground-item destruction in `0.2.269`; delayed
timing/state-machine parity, controlled runtime comparison, browser capture,
and audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. The checkout was dirty
when inspected: unrelated modifications were present under
`bin/data/drlhq/{audio.lua,meta.lua}`, `bin/data/drllq/{audio.lua,meta.lua}`,
and an untracked `fpcvalkyrie/` directory; none overlap the sources below.

- `bin/data/drl/items/eitems.lua:84-120` defines the standard `bfg9000` with
  `damage = "10d6"`, `DAMAGE_SPLASMA`, item `radius = 8`, `shotcost = 40`,
  and an explosion payload carrying `delay = 33` and `knockback = 16`.
  The payload records `EFSELFSAFE`, `EFAFTERBLINK`, `EFCHAIN`, and
  `EFNODISTANCEDROP`; this slice implements only the self-safe actor-fanout
  portion, while the other flags remain explicit deferred work.
- `src/dfbeing.pas:2636-2644` copies the item explosion payload, assigns the
  item radius to the explosion range, and schedules it after the ranged path.
- `src/dfdata.pas:896-965` shows that an omitted explosion `range` defaults to
  zero; the `dfbeing.pas` assignment is therefore material to the observed
  radius-8 schedule.
- `src/dflevel.pas:991-1095` applies one damage roll per clear blast cell,
  suppresses distance falloff for `EFNODISTANCEDROP`, skips the active actor for
  `EFSELFSAFE`, and destroys the lowest-ID non-feature ground item when the
  rolled damage is greater than `10`.

## DRL-Rust boundary

Gameplay semantics `78` (project version `0.2.269`) records one typed
`Bfg9000ExplosionScheduled` event after the standard BFG 9000 direct-target hit,
then immediately resolves the in-bounds, line-of-sight-cleared radius-8 cells
in stable center-then-ring order. Each clear cell consumes one `10d6` Plasma
roll without distance falloff; `EFSELFSAFE` skips the firing actor, other living
actors are processed once with radial integer `damage / 16` knockback before
environmental damage, and lethal victims retain normal death/drop/game-over
ordering. After actor processing, a roll greater than `10` removes at most the
lowest-ID ordinary ground item on that blast cell and emits
`GroundItemDestroyed` before lethal follow-up. The delay remains presentation
metadata rather than a pending core queue. Secondary `EFCHAIN` explosions,
`EFAFTERBLINK` timing, terrain/content mutation, projectile routing, Nuclear
BFG ground-item behavior, controlled legacy runtime, browser capture, and
audiovisual comparison remain separate slices or `NOT_RUN`.
