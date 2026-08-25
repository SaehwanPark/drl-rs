# Monster AI — Behavioral Specification

**Domain:** monster movement and tactical decisions
**Milestone relevance:** M2, M9 vertical fidelity
**Last updated:** 2026-08-25
**Status:** Source-informed, runtime comparison pending

## Evidence Sources

- **DRL-Rust implementation and tests** — `crates/drl-core/src/ai.rs` and
  `crates/drl-core/tests/monsters_ai.rs`.
- **Legacy Pascal source** — pinned revision
  `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`, especially
  `TBeing.MoveTowards` and `TBeing.TryMove`. Immutable source inspection and
  checkout state are recorded in
  `_workspace/drl/ai-movement-fallback/01-evidence.md`.

## Verified Behaviors

- **Preferred step** — `MonsterAi` first tries the smoothed one-step direction
  toward the player when the destination is in bounds and open. The strongly
  skewed smoothing ratio is supporting evidence from the dirty untracked
  `fpcvalkyrie` helper, not a clean pinned-source fact.
- **Candidate order** — when that preferred step is blocked, the AI retries the
  raw sign direction, then tries horizontal and vertical cardinal components
  in that order. A strongly skewed preferred direction may therefore be
  cardinal while the raw retry is diagonal.
- **Blocked candidates** — when all preferred, raw, horizontal, and vertical
  candidates are blocked, the AI waits rather than searching all remaining
  neighbors.
- **Determinism** — the decision is pure and consumes no RNG. Scheduled turns
  emit the normal movement/wait and action-cost events.
- **Direct-player separation** — player input uses its own destination-only
  movement path; AI fallback is not applied to player commands.

## Legacy Source Findings

The pinned `MoveTowards` implementation calls `TryMove` for the preferred
smoothed step, recomputes the raw normalized direction after a block, then
tries horizontal and vertical components in that order. The `CreateSmooth`
ratio threshold is present in the dirty untracked `fpcvalkyrie` helper and is
recorded as supporting implementation evidence. The routine does not perform
a general pathfinding search. This is source evidence, not a claim of
controlled runtime parity.

## Deliberate DRL-Rust Decisions

Keep the fallback policy explicit and bounded. Do not add a generic pathfinder
or allow direct player movement to inherit AI fallback behavior. A controlled
legacy runtime/capture comparison remains `NOT_RUN` until the reference
environment is available.

## Open Questions

- Exact legacy target-selection behavior when the player is not in direct line
  of travel remains open beyond this one-step fallback slice.
- Terrain movement modifiers, monster-specific callbacks, and full tactical
  behavior remain migration work.
