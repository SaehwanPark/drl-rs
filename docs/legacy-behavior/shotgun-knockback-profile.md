# Standard Shotgun typed behavior-profile evidence

Status: delivered typed profile for `0.2.217`; exact legacy force/timing,
spread/falloff, controlled runtime comparison, and audiovisual parity remain
`NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/items.lua` declares `shotgun` with shell ammunition,
  one-shell capacity, and legacy `knockback = 8`.
- `docs/legacy-behavior/combat.md` records the delivered DRL-Rust contract of
  one-cell kinetic displacement for a surviving Shotgun hit.

## DRL-Rust boundary

The immutable `drl_core::behavior::SHOTGUN_BEHAVIOR` profile records ordered
`HitEffect::Knockback { distance: 1 }` and
`ResourceCost::Ammo { ammo_type: Shells, amount: 1 }` fragments. Generic ranged
execution remains authoritative for target/LOS/range/death-drop preflight,
damage RNG, collision-aware displacement, event ordering, and transactional
rejection behavior. The profile's one-cell distance is the current Rust grid
contract; it does not reinterpret the legacy scalar force of eight.

Exact legacy force/timing, spread/falloff, controlled runtime comparison, and
audiovisual parity remain deferred and are not inferred from source similarity
alone.
