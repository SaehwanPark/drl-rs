# Red Armor profile

This note pins the bounded Red Armor Fire-resistance rule used by the M9
vertical slice. It does not claim the complete legacy resistance stack.

## Legacy evidence

- Checkout revision: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c` in
  `/Users/saehwan/repos/doom-the-roughlike-original`.
- `bin/data/drl/items/items.lua:74-90` defines `rarmor` as armor value `4`
  with `resist = { fire = 25 }`.
- `src/dfbeing.pas:2078-2182` selects Fire resistance from the typed damage
  family, applies percentage reduction before flat protection, rounds positive
  values, preserves one point for nonzero resisted damage, and makes 100%
  resistance zero.
- The legacy checkout is dirty in unrelated audio/meta files and contains an
  untracked `fpcvalkyrie/` directory; the cited item and damage files are not
  among those changes.

## Current Rust boundary

`ItemDefinitionKind::Armor` is the authoritative compile-time source for
`fire_resistance`; `Item::from_spawn_kind` carries it into `ArmorProperties`.
`Actor::take_damage_typed` applies the existing integer helper before flat
protection, and the existing typed Rocket Launcher actor-splash route supplies
`DamageType::Fire` through `World`.

Blue Armor's delivered Plasma field remains independent. This slice does not
implement weapon/body-zone/hook aggregation, direct Fire classification,
difficulty caps, durability side effects, or runtime/audiovisual parity.
