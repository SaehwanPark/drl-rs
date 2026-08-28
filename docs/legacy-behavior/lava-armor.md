# Lava Armor recharge evidence

Status: delivered typed transition and behavior profile through `0.2.207`;
hazard damage, resistance equations, controlled runtime comparison, and exact
presentation parity remain `NOT_RUN`.

## Pinned source

- Legacy checkout: `/Users/saehwan/repos/doom-the-roughlike-original`
- Revision: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`
- Behavior: `bin/data/drl/items/uitems.lua:924-967`
- Related hazard definition (out of scope here):
  `bin/data/drl/cells.lua:445-477`

The `perk_ulavaarmor` callback initializes `pp_lavarecharge` to zero. On each
item tick, it runs only while durability is below maximum, increments the
timer, and when the timer is greater than four checks the owner's current map
cell. Lava restores `min(durability + 3, maxdurability)` and resets the timer;
the fifth tick on any other cell also resets the timer without restoring
durability. The source definition identifies Lava Armor as unique,
non-destroyable, non-repairable armor with fire/plasma resistance and armor,
movement, and knockback scalar fields.

## Rust decisions

- `LavaRechargeState` is explicit core state, not a callback registry or Lua
  runtime bridge.
- `Tile::Lava`/`TileKind::Lava` is walkable and transparent for the bounded
  terrain contract; the current browser/minimap presentation uses a semantic
  tint over existing geometry.
- The transition ticks only after accepted player commands and emits
  `GameEvent::LavaArmorRecharged` only when durability increases.
- Rejected commands roll back the complete game snapshot through the existing
  transactional `Game::step` guard; recharge consumes no RNG.
- Replay/scenario fixtures may seed equipped armor below maximum so the actual
  recharge event, rather than only the full-armor no-op path, is deterministic
  and directly asserted.
