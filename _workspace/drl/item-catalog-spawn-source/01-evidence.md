predecessor: 00-scope.md
run_id: item-catalog-spawn-source-01c45a3
revision: 01c45a3b3918d0224456947fbd2af1b3512e26a0
status: PASS

# Evidence inspected

- `crates/drl-protocol/src/item.rs`: the current `ItemArchetype` declaration,
  ordered `ALL`, stable names, and loose-ammo shape projection.
- `crates/drl-protocol/src/replay.rs`: the duplicated `ItemSpawnKind` enum,
  normalized `ALL`, exhaustive `archetype` match, stack-count match, and
  inverse lookup.
- `crates/drl-core/src/item_definition.rs`: core definition lookup consumes
  `ItemSpawnKind::ALL` and remains a separate balance-owned catalog.
- `crates/drl-mcp/src/replay_json.rs` and `replay_json_decode.rs`: replay JSON
  consumes stable archetypes and explicit loose-ammo counts.
- `SPEC.md` sections 2.6, 2.7q, and 2.8: Gate C requires a routine catalog
  path while preserving explicit behavioral and count-sensitive boundaries.
- `docs/steering/current-priorities.md` and
  `docs/steering/decisions/item-registration-fanout-inventory.md`: catalog
  convergence is preferred before broad scalar-only content work.

## Current duplication

The same stable item families were listed in `item.rs` for archetype identity
and again in `replay.rs` for spawn variants, normalized order, and mapping.
Loose-ammo payload shape is the only intentional per-family payload
difference. The selected change centralizes routine identity and normalized
spawn data without moving gameplay definitions or presentation policy across
crate boundaries; count-sensitive handling stays explicit.

## Deferred evidence

Legacy runtime, browser, audio/visual, and external capture comparisons are
`NOT_RUN`; no behavior or parity claim is made.
