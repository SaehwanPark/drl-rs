# ADR 0008: Build-time migration of legacy content

- Status: Accepted
- Date: 2026-08-21
- Supersedes: [ADR 0005](0005-lua-transitional-strategy.md)

## Context

Runtime Lua was previously considered as a transitional compatibility layer.
Shipping a Lua VM in WASM would expand the security, determinism, bundle, and
licensing surface while preserving legacy implementation machinery the project
is explicitly replacing.

Legacy content is not only scalar data. Callback-heavy items and levels encode
behavior through hooks, alternate actions, equipment effects, recharge, set
logic, target selection, and other state transitions. Build-time conversion
must therefore preserve explicit evidence gaps rather than treating copied
fields as behavior-complete migration.

## Decision

Lua and legacy data remain research and conversion inputs only. Conversion
tools may read a pinned legacy revision and emit typed Rust/content evidence,
but the browser bundle ships no Lua VM, Lua scripts, or runtime legacy object
model. `drl-core` owns gameplay authority and deterministic RNG.

Routine content identity should converge on one authoritative compile-time
catalog or equivalent single-source representation. Legacy callbacks are
translated into a bounded typed Rust behavior vocabulary or dedicated typed
state machines; they are not recreated as a generic string-keyed callback bus.
Unknown behavior remains an explicit migration gap.

Imported assets and copied creative expression require provenance and
redistribution status appropriate to their category. The graphics directory is
eligible under its recorded CC BY-SA 4.0 terms; audio, music, and fonts remain
excluded until separately cleared. Legacy-derived descriptive text is tracked
separately from numeric/factual mechanics rather than silently inheriting the
project-code license.

See the current steering decision:
[`docs/steering/decisions/content-catalog-and-typed-behavior-model.md`](../steering/decisions/content-catalog-and-typed-behavior-model.md).

## Consequences

- `drl-script` is not a runtime Lua dependency. If its lasting responsibility is
  only import/conversion, it should be removed until needed or renamed to make
  that responsibility explicit (for example, `drl-content-import`).
- M3 records evidence and stable semantic asset identifiers before presentation
  wiring.
- Lua behavior gaps are tracked as evidence/roadmap work and tested through
  Rust commands, observations, scenarios, and replays.
- Definition coverage, behavior coverage, legacy comparison, and presentation
  comparison remain distinct status claims.
- Broad scalar-only migration is not evidence that the typed behavior model is
  adequate; difficult callback-heavy cases must validate the model first.
