# Mega Buster typed behavior-profile evidence

Status: delivered typed ordinary-fire volley/cost profile for `0.2.227`;
the kill morph callback, controlled legacy runtime comparison, and audiovisual
parity remain `NOT_RUN`.

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
catalog. The kill callback at `uitems.lua:359-435` mutates weapon properties
after a kill and is outside this ordinary-fire contract.

## DRL-Rust boundary

The immutable `drl_core::behavior::MEGA_BUSTER_BEHAVIOR` profile records the
ordered `AttackEffect::ProjectileCount(3)` and
`ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 3 }` fragments. Generic ranged
execution remains the authority for target/LOS/range validation, damage RNG,
event ordering, and transactional nine-round clip consumption. The profile
does not add a command, replay wire field, RNG algorithm, or callback registry.

The direct-target path intentionally leaves the kill morph callback, spread or
projectile routing, exact timing/accuracy, controlled legacy runtime capture,
and audiovisual parity outside this slice.
