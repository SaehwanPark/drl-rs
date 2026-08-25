predecessor: none
run_id: item-catalog-spawn-source-01c45a3
starting_revision: 01c45a3b3918d0224456947fbd2af1b3512e26a0
owner: milestone owner (Codex)
role: Gate C content-catalog delivery
status: IN_PROGRESS

# Scope

The prior Gate C slices derive stable `ItemArchetype` identity from one
declaration and verify that `ItemSpawnKind::ALL` stays in the same order. This
slice removes the remaining routine declaration/mapping duplication by making
that same protocol catalog generate `ItemSpawnKind`, its normalized `ALL`
projection, and archetype mapping. Count-sensitive stack handling and inverse
reconstruction remain explicit.

## Observable outcomes

- One compile-time declaration owns every stable item identity and its spawn
  shape (`none`, unit, or counted loose ammo).
- `ItemArchetype` and replay-visible `ItemSpawnKind` preserve their public
  variant names, stable wire names, order, and count-sensitive behavior.
- Existing replay, scenario, MCP, core-definition, and presentation consumers
  retain their current contracts.
- Focused protocol tests prove order, stable-name round trips, spawn-family
  round trips, and explicit loose-ammo count handling.

## Gate and boundary decisions

- Steering gate: Gate C, routine content registration fan-out.
- Replay/gameplay semantics: unchanged; this is a compile-time ownership
  refactor and does not change commands, RNG, balance, or replay metadata.
- Protocol ownership: stable identity and normalized spawn contracts remain in
  `drl-protocol`; count-sensitive reconstruction and core gameplay definitions
  remain explicitly owned.
- Legacy evidence: not required for this registration-only slice.
- Non-goals: new item families, gameplay behavior, presentation mappings,
  runtime legacy parity, and broad behavior-vocabulary work.
