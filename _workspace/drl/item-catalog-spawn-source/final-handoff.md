predecessor: 04-verification.md
run_id: item-catalog-spawn-source-01c45a3
owner: milestone owner (Codex)
role: Gate C content-catalog delivery
input_revision: 01c45a3b3918d0224456947fbd2af1b3512e26a0
output_revision: pending PR merge
status: PASS

# Handoff

This slice unifies `ItemArchetype` and normalized `ItemSpawnKind` routine
registration in one protocol compile-time declaration. Existing public paths,
catalog order, replay JSON behavior, inverse loose-ammo reconstruction, core
definition lookup, and presentation boundaries remain intact.

## Files changed

- `crates/drl-protocol/src/item.rs`: shared catalog-generated archetype/spawn
  variants, ordered views, stable names, normalized values, and mapping.
- `crates/drl-protocol/src/replay.rs`: preserved replay re-export and explicit
  count-sensitive stack/inverse reconstruction.
- `SPEC.md`, `ARCHITECTURE.md`, `README.md`, `CHANGELOG.md`, roadmap,
  versioning, and Gate C inventory: aligned to verified scope and `0.2.143`.
- `_workspace/drl/item-catalog-spawn-source/`: scope, evidence, plan, review,
  verification, and this handoff.

## Deferred work

Behavior vocabulary, legacy runtime parity, gameplay-balance catalog
convergence, presentation mappings, and external capture remain open or
`NOT_RUN`.

Next owner: milestone owner selects the next roadmap slice after PR review and
hosted checks.
