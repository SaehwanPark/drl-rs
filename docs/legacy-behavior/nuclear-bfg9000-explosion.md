# Nuclear BFG 9000 delayed explosion evidence

Status: delivered bounded typed schedule metadata and immediate actor-only
radius-8 fanout for `0.2.268`; EFCHAIN secondary explosions, terrain/content
and ground-item effects, delayed timing/state-machine parity, controlled runtime
comparison, and audiovisual parity remain open or `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. The checkout was dirty
when inspected: unrelated modifications were present under
`bin/data/drlhq/{audio.lua,meta.lua}`, `bin/data/drllq/{audio.lua,meta.lua}`,
and an untracked `fpcvalkyrie/` directory; none overlap the sources below.

- `bin/data/drl/items/eitems.lua:474-518` defines `unbfg9000` with item
  `damage = "8d6"`, `damagetype = DAMAGE_SPLASMA`, `radius = 8`, an explosion
  payload carrying `delay = 33`, `knockback = 16`, and the
  `EFSELFSAFE`/`EFAFTERBLINK`/`EFCHAIN`/`EFNODISTANCEDROP` flags, plus separate
  recharge/alternate-overload perks.
- `src/dfbeing.pas:2636-2644` copies the item explosion payload, assigns the
  item radius to the explosion range, and schedules it after the ranged path.
- `src/dfdata.pas:896-965` shows that an omitted explosion `range` defaults to
  zero; the ranged-path assignment is therefore material to the observed
  radius-8 schedule. `src/dflevel.pas:991-1095` rolls the assigned damage
  independently per clear cell, suppresses distance falloff for
  `EFNODISTANCEDROP`, skips the active firing actor for `EFSELFSAFE`, and
  applies knockback before actor damage.

## DRL-Rust boundary

Gameplay semantics `77` (project version `0.2.268`) records one typed
`NuclearBfg9000ExplosionScheduled` event after the Nuclear BFG 9000
direct-target hit, carrying delay `33`, radius `8`, and knockback `16`, then
resolves an immediate deterministic actor-only radius-8 fanout. Each clear
cell consumes one `8d6` Plasma roll without distance falloff; the firing actor
is splash-safe, other living actors are de-duplicated and receive radial
integer `damage / 16` knockback before environmental damage, and lethal
death/drop/game-over follow-up remains ordered. EFCHAIN secondary explosions,
terrain/content and ground-item effects, delayed timing, projectile routing,
NukeRun, controlled legacy runtime, and audiovisual comparison remain separate
slices or `NOT_RUN`.
