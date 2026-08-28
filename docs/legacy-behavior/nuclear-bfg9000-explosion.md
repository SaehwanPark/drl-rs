# Nuclear BFG 9000 delayed explosion evidence

Status: delivered bounded typed schedule metadata for `0.2.203`; explosion
damage/geometry, controlled runtime comparison, and audiovisual parity remain
`NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. The checkout was dirty
when inspected: unrelated modifications were present under
`bin/data/drlhq/{audio.lua,meta.lua}`, `bin/data/drllq/{audio.lua,meta.lua}`,
and an untracked `fpcvalkyrie/` directory; none overlap the sources below.

- `bin/data/drl/items/eitems.lua:474-518` defines `unbfg9000` with item
  `radius = 8`, an explosion payload carrying `delay = 33` and
  `knockback = 16`, and separate recharge/alternate-overload perks.
- `src/dfbeing.pas:2636-2644` copies the item explosion payload, assigns the
  item radius to the explosion range, and schedules it after the ranged path.
- `src/dfdata.pas:896-965` shows that an omitted explosion `range` defaults to
  zero; the ranged-path assignment is therefore material to the observed
  radius-8 schedule.

## DRL-Rust boundary

Gameplay semantics `41` (project version `0.2.203`) records one typed
`NuclearBfg9000ExplosionScheduled` event after the Nuclear BFG 9000
direct-target hit, carrying delay `33`, radius `8`, and knockback `16`. The
event is deterministic and replay-visible; recharge, alternate overload,
explosion geometry, splash damage, knockback application, projectile routing,
NukeRun, controlled legacy runtime, and audiovisual comparison remain separate
slices or `NOT_RUN`.
