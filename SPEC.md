# Specification

Last reviewed: 2026-09-02
Current project version: `0.2.333`
Audited starting checkpoint: `main` at `c07a256` (Rocket ground-item slice
and canonical documentation reconciliation)
Delivery checkpoint: `main` merge commit `5242e3c` (PR #449, merged)

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

## 2. Active implementation slice: M9 — Rocket Launcher direct Fire classification

Slice status: **delivered and verified** at the delivery checkpoint above.

### 2.1 Objective

Complete the Rocket Launcher direct-hit damage-family branch. The pinned legacy
record declares `6d6 DAMAGE_FIRE`; after a successful direct hit, route only the
Rocket Launcher target damage through the existing typed Fire path so
catalog-defined Red Armor resistance applies before flat protection. Preserve
the already-delivered radius-4 splash, item destruction, event ordering,
death/drop handling, and transaction boundary.

This is a bounded vertical fidelity slice. It changes the direct `DamageApplied`
event from unclassified to `Some(DamageType::Fire)` only for Rocket Launcher
hits and leaves every other direct weapon path unchanged.

### 2.2 Audited starting point

At audited starting revision `c07a256` (version `0.2.332`):

- The Rocket Launcher record (`items.lua:657-688`) declares a one-shot `6d6`
  `DAMAGE_FIRE` weapon, while `dfbeing.pas:2636-2644` propagates the weapon
  damage family into direct damage and explosion resolution.
- Rust already has typed Fire actor damage, catalog-defined Red Armor 25%
  Fire resistance, and a typed `DamageApplied` event field.
- The generic direct ranged path still calls untyped `World::apply_damage` and
  emits `damage_type: None`; the Rocket splash path is already typed Fire.

### 2.3 Scope and ownership

- **Roadmap:** M9 vertical canonical-fidelity completion of the Rocket Launcher
  direct damage-family branch.
- **Primary owner:** `Game`'s typed ranged execution boundary selects Fire for
  the Rocket Launcher direct target; `World` and `Actor` retain the existing
  resistance and flat-protection formulas. Boundary crates remain projections.
- **Content registration:** the existing Rocket Launcher catalog definition is
  the single source of its identity and damage range; no duplicate weapon table
  or callback registry is introduced.
- **Project version:** implementation advances `VERSION` from `0.2.332` to
  `0.2.333`.
- **Replay/RNG:** gameplay semantics advance from `134` to `135`; replay wire,
  RNG sampling (`1`), generator semantics (`2`), and ruleset identity
  (`drl-rs-ruleset-v1`) remain unchanged. Typed mitigation consumes no RNG.
- **Protocol/boundaries:** no new wire event or schema is needed; the existing
  optional `DamageApplied.damage_type` projection carries `Fire`.

### 2.4 Review and branch contract

- A successful Rocket Launcher direct hit emits `DamageApplied` with
  `damage_type: Some(DamageType::Fire)` and applies the existing Fire resistance
  before flat protection.
- The raw `AttackOutcome::Hit` damage and one-roll RNG stream remain unchanged;
  `AttackOutcome::is_lethal` retains its existing raw-damage contract, while
  actual actor death remains authoritative in `World`.
- Rocket splash cells retain typed Fire mitigation, thresholded ground-item
  destruction, geometry, falloff, deduplication, event ordering, and final RNG
  state.
- Other direct weapons retain their untyped damage events and prior mitigation.
- The existing core transaction guard still owns command rejection and exact
  state/RNG restoration; this slice adds no queue, callback, or new clone.

### 2.5 Acceptance criteria

- [x] Rocket Launcher direct hits emit typed Fire damage and Red Armor reduces
  the applied amount using the existing deterministic resistance-before-flat
  protection formula.
- [x] An unarmored control preserves the raw direct damage amount, while the
  same seed keeps the raw roll and final RNG state identical between armored
  and unarmored targets.
- [x] Direct replay and repeated replay runs produce identical game state,
  event stream, and typed direct-hit projection; stale semantics `134` is
  rejected before execution.
- [x] Existing Rocket splash/ground-item, rejection/rollback, replay, MCP,
  metrics/audio/render, and BrowserSession parity tests pass without new wire
  or RNG behavior.
- [x] Formatting, clippy, `sh scripts/check-repository.sh`, version transition,
  hosted checks, and an attributable independent determinism review pass on the
  final implementation commit.

### 2.6 Non-goals

- No direct Fire classification for other weapons, generic resistance
  aggregation, legacy armor durability degradation, or SPLASMA divisors.
- No projectile routing, delayed queue, terrain/content mutation, feature-item
  behavior, rocket-jump, callback recreation, or runtime/audiovisual parity.
- No change to the replay wire schema, RNG sampling algorithm, generator
  semantics, ruleset identity, or unrelated event projections.

### 2.7 Evidence boundary

This slice proves the current-Rust Rocket Launcher direct-hit Fire event and
Red Armor mitigation policy, plus replay determinism and stable boundary
projection. It does not prove controlled legacy runtime, exact timing or
accuracy, projectile routing, terrain/content behavior, broader resistance
aggregation, durability, balance, audiovisual parity, or human play.

### 2.8 Delivery evidence

- Independent determinism review of final head `a0a0fcd`: **PASS** by
  `/root/rocket_review`; no actionable findings were identified.
- Local workspace tests, clippy, formatting, version check, repository checks,
  native/headless browser checks, and `git diff --check`: **PASS**. The optional
  reference-capture preflight is `NOT_RUN` because its local manifest is
  unavailable.
- PR #449 hosted Repository and WASM browser checks: **PASS** in run
  `33656765327`. The protected-path Review policy check failed closed in run
  `33656765373` because the sole maintainer cannot create a non-self approval;
  the documented live `enforce_admins=false` exception was used after the
  independent review receipt was recorded.
- Merge checkpoint: `5242e3c`; the temporary implementation branch was removed
  locally and remotely. No controlled legacy runtime, audiovisual, balance, or
  human-play claim is inferred from these checks.

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
