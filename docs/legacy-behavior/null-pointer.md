# Charch's Null Pointer on-hit evidence

Status: bounded typed behavior target for `0.2.131`; exact delayed explosion
geometry/damage, controlled runtime comparison, and audiovisual parity remain
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
10.

## Rust decisions

- `NullPointerHitTransition` owns only the deterministic target score branch;
  it is a pure transition rather than a callback registry.
- A typed `NullPointerExplosionScheduled` event records the evidence-backed
  delay/radius/damage payload. Applying area damage and exact target ordering is
  intentionally deferred until a geometry/effects slice has executable
  evidence.
- Boss identity is an explicit core actor property, defaulting to false; it
  is not inferred from display names or monster kind.
- The source's `shotcost = 10` is retained as evidence only; the current
  ranged command uses the existing Rust action-cost policy and does not claim
  scalar timing/score-cost parity for that field.
- The transition consumes no RNG beyond the ordinary ranged hit roll, and
  rejected commands retain the existing transactional game snapshot.
