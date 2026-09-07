# Specification

Last reviewed: 2026-09-07
Current project version: `0.2.349`

The roadmap owns milestone scope and ordering. This file expands exactly one
active implementation slice; delivered history belongs in the roadmap,
changelog, evidence notes, and Git.

## 1. Status vocabulary

- `[x]` — delivered and verified by checked implementation and evidence.
- `[ ]` — open and required by the active slice.
- `NOT_RUN` — prerequisites unavailable; no pass or failure is inferred.
- `INCONCLUSIVE` — available evidence cannot support the claim.

## 2. Active implementation slice: fair ground-item observations (audit F1)

Slice status: **delivered and verified locally**. This is the first remediation
slice recommended by `docs/project-audit-2026-09-07.md` and is bounded to the
observation boundary.

### 2.1 Objective

Ensure player observations and their MCP projection never disclose live ground
items outside the player's current field of view. Explored terrain remains
remembered, but hidden item state is not memory. When visibility returns, the
current items are observable again.

### 2.2 Scope and acceptance

- [x] Filter player-observation ground items by current visibility, not merely
  `explored_tiles`.
- [x] Add regression coverage for hidden item addition, removal, and count
  changes; two worlds with identical visible state and observation memory but
  different hidden items produce identical fair observations.
- [x] Verify legitimate reveal after the player regains visibility and preserve
  omniscient/debug observations.
- [x] Confirm MCP JSON is identical for identical fair observations and contains
  no hidden item entries.
- [x] Preserve command atomicity, deterministic RNG/replay behavior, and the
  existing simulation/protocol/presentation boundaries.

### 2.3 Explicit non-goals

No remembered item snapshot, protocol schema change, MCP framing change, content
migration, renderer redesign, or claim of visual/legacy parity is part of this
slice. F2 (bounded MCP framing) and F3 (`.mjs` version classification) remain
separate subsequent slices.

### 2.4 Delivery evidence

- `cargo test --locked -p drl-core --lib`: 197 passed, including hidden item
  addition, removal, count-change, two-world, omniscient, and reveal coverage.
- `cargo test --locked -p drl-mcp --lib`: 84 passed, including JSON parity and
  an empty serialized `ground_items` assertion for hidden state.
- `sh scripts/check-web.sh`: PASS (browser/library and headless Chrome checks).
- `cargo fmt --all -- --check`, `sh scripts/check-version.sh`,
  `sh scripts/check-spec-structure.sh`, and `git diff --check`: PASS.
- `sh scripts/check-repository.sh`: INCOMPLETE; it reached the workspace
  integration suite after all preceding script contracts passed, but the local
  run was aborted before completion. No full-suite pass is claimed.
- Read-focused determinism review: PASS by owner inspection of the world/MCP
  producer and consumer boundary; no command, RNG, replay, or schema path was
  changed. A separate reviewer was unavailable because the checkout was dirty.
- Unavailable native, browser-capture, controlled-legacy, audiovisual, and
  human acceptance remain `NOT_RUN`.

## 3. Enduring invariants

The active slice must preserve:

1. no ambient state, platform APIs, filesystem, browser, or presentation policy
   in `drl-core`;
2. identical declared seed, commands, and semantics produce identical current
   simulation results;
3. incompatible histories fail explicitly before simulation;
4. rejected commands and rejected restores do not partially mutate authoritative
   simulation state;
5. renderers, browser code, MCP, and bots consume fair observations/events and
   do not inspect hidden core state;
6. presentation timing, resize, scale factor, surface loss, and storage side
   effects do not advance gameplay;
7. no runtime Lua or generic callback recreation;
8. current-Rust, cross-version, legacy, browser, audiovisual, and performance
   evidence remain separately labeled.
