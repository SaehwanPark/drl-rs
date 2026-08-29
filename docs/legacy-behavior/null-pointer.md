# Charch's Null Pointer on-hit evidence

Status: delivered typed target-score branch and actor-only radius-1 splash in
`0.2.256`; exact delayed timing/geometry, splash immunity, terrain/item
destruction, controlled runtime comparison, and audiovisual parity remain
`NOT_RUN`.

## Pinned source

- Legacy checkout: `/Users/saehwan/repos/doom-the-roughlike-original`
- Revision: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`
- Behavior: `bin/data/drl/items/uitems.lua:62-112`

The `perk_unullpointer_hit` callback runs when the weapon hits a target. It
reduces target score count by `1000` for a boss or `2000` otherwise, with a
floor of `1000`, then schedules a range-1, delay-50, `10d1` splash-plasma
explosion at the target position. The weapon itself is a unique ranged plasma
item with zero direct damage, 60-cell clip capacity, accuracy 6, and shot cost
10. The callback returns `false`, suppressing direct damage; the pinned
explosion loop then checks blast cells and applies one damage result per
eligible actor after deduplication (`src/dfbeing.pas:2522-2549`,
`src/dflevel.pas:1004-1079`).

## Rust decisions

- `NullPointerHitTransition` owns only the deterministic target score branch;
  it is a pure transition rather than a callback registry.
- A typed `NullPointerExplosionScheduled` event records the evidence-backed
  delay/radius/damage payload. The bounded Rust resolver immediately applies
  fixed `10d1` Plasma environment damage to living actors on the center-plus-
  eight-neighbor clear cells in stable order; the delay remains presentation
  metadata rather than a pending simulation queue.
- Boss identity is an explicit core actor property, defaulting to false; it
  is not inferred from display names or monster kind.
- The source's `shotcost = 10` is retained as evidence only; the current
  ranged command uses the existing Rust action-cost policy and does not claim
  scalar timing/score-cost parity for that field.
- The transition consumes no RNG beyond the ordinary ranged hit roll, and
  rejected commands retain the existing transactional game snapshot. Death
  drops for every possible splash victim are preflighted before clip/RNG
  mutation.

Direct-core, ScenarioRunner/replay, generic MCP JSON, and BrowserSession paths
preserve the same splash events, observations, effects, scenes, and final
state. Terrain/cell and ground-item destruction, splash immunity, exact
delayed timing, callback state, controlled runtime, and audiovisual parity
remain deferred; source similarity alone is not parity proof.
