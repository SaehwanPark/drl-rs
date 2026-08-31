# Specification

Last reviewed: 2026-08-31
Current project version: `0.2.321`
Audited starting checkpoint: `main` at
`447176b` (merged PR #429; M9 candidate is the working-tree slice)
Delivery checkpoint: `main` at
`180f7dd2d350b11c114ae4f5fdbc27ba12d32829` (merged PR #430; M9 delivered)

The [Roadmap](docs/DRL-RS_Project_Roadmap.md) owns milestone scope, ordering,
and progress. [`docs/steering/current-priorities.md`](docs/steering/current-priorities.md)
constrains slice selection while its stop gates remain open. This file expands
**exactly one active implementation slice**. Delivered history belongs in the
roadmap, changelog, evidence notes, and Git rather than accumulating here.

## 1. Status vocabulary

- `[x]` — **Delivered and verified**: supported by checked implementation and
  evidence.
- `[ ]` — **Open**: required by the active slice and not yet delivered.
- `NOT_RUN` — **Environment unavailable**: prerequisites were unavailable; no
  pass or failure is inferred.
- `INCONCLUSIVE` — **Evidence unresolved**: available evidence cannot support
  the claim.

## 2. Active implementation slice: M9 — Whole-rule chainfire state model

### 2.1 Objective

Replace the current per-level chainfire plateau tables with one typed,
deterministic state model for the six supported alternate-fire weapon families.
The model must make initial, warming, sustained, and saturated states explicit,
derive volley size and aggregate ammunition cost from one integer formula,
preserve transactional rejection, and name the current target and trait
boundaries without importing legacy callback machinery.

This is a Gate B fidelity slice. It advances gameplay semantics while keeping
the replay, MCP, browser, and presentation boundaries deterministic.

### 2.2 Audited starting point

At audited starting revision `447176b` (version `0.2.320`):

- `drl-core::behavior` exposes separate per-family profile functions and
  constants for bounded levels, with sustained plateaus repeated as individual
  entries;
- `Game::execute_player_ranged_attack` selects one family profile and rejects
  levels outside that table before consuming clip state or combat RNG;
- `chainfire_level` is a single `u8`, advanced after accepted chainfire and
  reset by ordinary fire;
- chainfire reserves a complete burst, emits deterministic no-op continuation
  misses after a lethal target, and rejects under-supply atomically;
- replay V2 and the semantics-bound browser snapshot V3 already identify the
  gameplay semantics version, so this slice must make one explicit replay
  version decision rather than silently changing history meaning.

The legacy source at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`
uses the same integer state byte for all `2..255` chainfire levels: level zero
subtracts one third of the ordinary shot count, level one is unchanged, and
levels two through 255 add half the ordinary shot count. Its partial-ammo
fallback, rotational target routing, and Lua trait hooks are separately
classified in the evidence artifact and are not copied by assumption.

### 2.3 Scope and ownership

- **Steering gate:** Gate B — fidelity slices close semantic branches, not
  counters.
- **Primary owner:** `drl-core::chainfire` owns the pure state model; `Game`
  remains the execution and transaction authority; replay/MCP/browser remain
  projections of the core result.
- **Identity source:** each model maps a protocol `ItemArchetype` to one
  ordinary projectile count and per-projectile ammunition cost; no boundary
  duplicates the formulas.
- **Project version:** implementation advances `VERSION` from `0.2.320` to
  `0.2.321`.
- **Gameplay/replay semantics:** gameplay semantics advances from `127` to
  `128`; replay wire/schema, RNG-sampling, generator, ruleset, and snapshot V3
  grammars do not change. V3 snapshots carrying `127` become incompatible by
  the existing identity policy.

### 2.4 Typed model contract

For ordinary projectile count `n`, per-projectile ammunition cost `c`, and
chainfire level `l`, the model exposes:

| State | Level | Projectile count | Aggregate cost |
| --- | --- | --- | --- |
| Initial | `0` | `n - (n div 3)` | count × `c` |
| Warming | `1` | `n` | count × `c` |
| Sustained | `2..=254` | `n + (n div 2)` | count × `c` |
| Saturated | `255` | `n + (n div 2)` | count × `c` |

The model uses saturating arithmetic for the `u8` state transition. Saturated
advancement remains `255`; ordinary fire resets to `0`. The current DRL-Rust
resource decision is full-burst-only: if the aggregate cost is unavailable,
the command rejects before RNG or mutation. Target routing remains an explicit
fixed-requested-target policy with deterministic no-op continuation events
after a lethal target. The core has no player-trait state, so legacy Ammochain
and Entrenchment hooks are recorded as a future typed trait policy rather than
silently simulated.

### 2.5 Transaction and boundary contract

- All six supported families (BFG 10K, Chaingun, Minigun, Plasma Rifle, Laser
  Rifle, and Nuclear Plasma Rifle) use the same pure model and accept saturated
  sustained levels.
- A chainfire command validates target, weapon, model, and full aggregate cost
  before consuming clip state or combat RNG; failed commands preserve exact
  game/RNG identity and chainfire level.
- Accepted bursts emit the model's ordered projectile count, advance the state
  once, and retain the existing deterministic post-lethal no-op continuation.
- Ordinary fire and non-chainfire actions reset the state. Reload does not
  invent a new chainfire rule; it only restores ammunition as currently
  implemented.
- Direct core, replay, MCP, and BrowserSession projections must report the same
  state, events, observations, and error identity for representative sustained
  and saturated commands.

### 2.6 Acceptance criteria

- [x] A pure typed model exposes all four state classes, formulas, costs,
  saturation, and reset/advance transitions with unit tests.
- [x] All six weapon families use the model; family-specific higher-level
  rejection paths and duplicated runtime formulas are removed.
- [x] Existing initial, warming, and sustained vectors remain unchanged; each
  family has a saturated-level vector.
- [x] Full-burst under-supply remains an atomic rejection with no RNG or state
  advance; ordinary fire resets chainfire.
- [x] Fixed-target routing and post-lethal continuation are named and covered;
  legacy rotational spread remains outside this slice.
- [x] The absent current-core trait model is documented as an explicit future
  policy; no Ammochain/Entrenchment behavior is inferred from names alone.
- [x] Direct core, replay, MCP, and BrowserSession boundaries remain
  deterministic for representative sustained and saturated commands.
- [x] Gameplay semantics advances exactly once; replay wire/schema and snapshot
  V3 parsing remain otherwise unchanged.
- [x] `sh scripts/check-repository.sh`, `scripts/check-version.sh`, and the
  relevant hosted repository/WASM checks pass for reviewed merge revision
  `180f7dd` (PR #430).
- [x] Independent determinism review returns `pass` with evidence classes,
  exact rejection identity, and persistent-history impact recorded.
- [x] Roadmap, README, architecture, changelog, browser guide, and steering
  records are reconciled from verified evidence; unavailable runtime,
  audiovisual, human, performance, and legacy captures remain `NOT_RUN`.

### 2.7 Non-goals

- No projectile accuracy, timing, scatter, delayed-explosion, splash, or
  audiovisual changes.
- No partial-ammo execution, best-effort continuation, or new trait subsystem.
- No Lua runtime, callback registry, or mechanical translation of legacy
  architecture.
- No claim of controlled legacy runtime parity where the required environment
  is unavailable.

### 2.8 Evidence boundary

The legacy source establishes the integer state formulas and transition
conditions, but not current-Rust RNG or event traces. Current Rust tests prove
the typed model and boundary behavior. Controlled legacy runtime, target
rotation/spread parity, trait hooks, human play, broad browser coverage,
audiovisual parity, and performance remain `NOT_RUN`/open unless separately
recorded.

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
6. presentation timing and storage side effects do not advance gameplay;
7. no runtime Lua or generic callback recreation;
8. current-Rust, cross-version, legacy, browser, audiovisual, and performance
   evidence remain separately labeled.
