# Mega Buster typed behavior-profile evidence

Status: delivered bounded typed post-kill morph and ordinary-fire volley/cost
profile for `0.2.357`; exact legacy dice/splash/timing behavior, controlled
legacy runtime comparison, and audiovisual parity remain `NOT_RUN`.

## Pinned source

Evidence is pinned to revision
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c` of
`doom-the-roughlike-original`:

- `bin/data/drl/items/uitems.lua:437-469` defines `umega` as a ranged weapon
  using the `ammo` family, with `shots = 3`, `shotcost = 3`, and the
  `perk_umega_kill` callback.
- `src/dfitem.pas:247-255` loads the declared `Shots` and `ShotCost` fields.
- `src/dfbeing.pas:1477-1481` resolves the ordinary shot count from the item
  field, preserving three projectiles for Mega Buster fire.
- `src/dfbeing.pas:1496-1514` computes the aggregate ammunition cost from the
  resolved shot count and per-shot cost before firing.

The legacy `ammo` family is mapped to Rust `Ammo9mm` by the existing typed item
catalog. The kill callback at `uitems.lua:359-435` reads the defeated target's
equipped weapon damage family (defaulting to Bullet), maps it to Bullet/Fire/
Acid/Plasma, and mutates the Mega Buster only when the resulting mode changes.
`src/dfbeing.pas:1785-1824` pins the callback after lethal damage marks the
victim dying and before inventory drops.

## DRL-Rust boundary

The immutable `drl_core::behavior::MEGA_BUSTER_BEHAVIOR` profile records the
ordered `AttackEffect::ProjectileCount(3)` and
`ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 3 }` fragments. Generic ranged
execution remains the authority for target/LOS/range validation, damage RNG,
event ordering, and transactional nine-round clip consumption. The profile
does not add a command, replay wire field, RNG algorithm, or callback registry.

The bounded direct-target path now stores a typed `MegaBusterMorphMode` on the
Mega Buster and emits `GameEvent::MegaBusterMorphed` after `ActorDied` and before
`ItemDropped`. Optional target equipment is represented by
`MonsterSpawnSpec.equipped_weapon` and reconstructed with deterministic item
IDs in Scenario/ReplayEngine/MCP replay JSON. Bullet/Physical, Fire, Acid, and
Plasma profiles retain legacy `1d8`, `4d2`, `4d2`, and `1d10` provenance while
using current Rust ranges `(1,8)`, `(4,8)`, `(4,8)`, and `(1,10)`.

Fire/Acid radius-one execution, exact dice distribution, legacy
accuracy/miss/timing/presentation, same-volley mutation, controlled runtime,
and audiovisual parity remain explicit follow-up or `NOT_RUN` work.
