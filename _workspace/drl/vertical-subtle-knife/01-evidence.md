# Vertical Subtle Knife Encounter — Evidence

## Existing source and implementation evidence

- `docs/legacy-behavior/subtle-knife.md` records the bounded source-derived
  costs, tired state, score cost, visible-target rule, and internal damage.
- `crates/drl-core/src/subtle_knife.rs` owns the pure player-cost transition.
- `crates/drl-core/src/game.rs` owns target selection, typed events, damage,
  and the accepted command pipeline.
- Existing `special_items` tests cover pure transitions, target visibility,
  rollback, lethal ordering, and replay determinism.

## New evidence target

The vertical slice will add a scenario-level assertion and a browser-boundary
parity assertion. Both must use the same stable item identity and command, and
must preserve the current target and event contracts without claiming legacy
runtime or full presentation parity.

## Boundary classification

- **Observed/verified:** Rust scenario, replay, event ordering, observations,
  effect timeline, and scene derivation.
- **Inferred:** none required for the implementation decision.
- **NOT_RUN:** controlled legacy execution, browser capture, audio, WebGPU,
  and broader monster/armor parity.
