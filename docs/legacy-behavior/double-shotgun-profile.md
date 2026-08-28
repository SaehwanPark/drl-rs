# Double Shotgun dual-shot behavior evidence

Status: delivered deterministic dual-shot behavior and typed profile through
`0.2.216`; spread/falloff, exact legacy timing, controlled runtime comparison,
and audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`.

- `bin/data/drl/items/items.lua` declares `dshotgun` with `IF_DUALSHOTGUN`, a
  two-shell capacity, `shots = 2`, and one shell per emitted projectile.
- `src/dfitem.pas` reads the source `shots` field into ranged item properties;
  the Rust path preserves that count as two ordered projectiles.

## DRL-Rust boundary

The immutable `drl_core::behavior::DOUBLE_SHOTGUN_BEHAVIOR` profile records
ordered `AttackEffect::ProjectileCount(2)` and
`ResourceCost::Ammo { ammo_type: Shells, amount: 2 }` fragments. The existing
ranged command path remains execution authority for target/LOS/range/death-drop
preflight, damage RNG, event ordering, lethal handling, and transactional clip
consumption. Scenario, replay, MCP, and BrowserSession/direct-core parity tests
cover the two attack outcomes and two-shell cost.

Spread/falloff, exact legacy timing, controlled runtime comparison, and
audiovisual parity remain deferred and are not inferred from source similarity
alone.
