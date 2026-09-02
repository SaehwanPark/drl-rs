# Specification

Last reviewed: 2026-09-02
Current project version: `0.2.336`
Audited starting checkpoint: `main` at `28f413c` (Nuclear BFG 9000 direct-Plasma
slice and canonical documentation reconciliation)
Delivery checkpoint: **pending** for the active branch

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

## 2. Active implementation slice: M9 — BFG 10K direct Plasma classification

Slice status: **implementation complete; handoff in progress** on temporary
branch `codex/bfg10k-direct-plasma-classification`.

### 2.1 Objective

Complete the BFG 10K direct-hit damage-family branch. The pinned legacy record
declares `6d4 DAMAGE_SPLASMA`; after each successful direct volley hit, route
only the BFG 10K target damage through the existing typed Plasma path so
catalog-defined Blue Armor resistance applies before flat protection. Preserve
the already-delivered five-projectile volley, chainfire progression, radius-2
splash, ground-item effect, event ordering, death/drop handling, and
transaction boundary.

This is a bounded vertical fidelity slice. It changes the direct
`DamageApplied` event from unclassified to `Some(DamageType::Plasma)` only for
BFG 10K hits and leaves every other direct weapon path unchanged.

### 2.2 Audited starting point

At audited starting revision `28f413c` (version `0.2.335`):

- The BFG 10K record (`uitems.lua:730-765`) declares a `6d4`
  `DAMAGE_SPLASMA` weapon with five configured projectiles, while
  `dfbeing.pas:476-510` carries the damage family through the scatter-weapon
  projectile path and `dfbeing.pas:2162-2182` maps Plasma families to plasma
  resistance.
- Rust already has typed Plasma actor damage, catalog-defined Blue Armor 20%
  Plasma resistance, and a typed `DamageApplied` event field. The BFG 10K
  radius-2 splash is already typed Plasma, as are its ground-item destruction
  and delayed schedule boundaries where applicable.
- The shared direct ranged path still calls untyped `World::apply_damage` and
  emits `damage_type: None` for each BFG 10K direct volley hit.

### 2.3 Scope and ownership

- **Roadmap:** M9 vertical canonical-fidelity completion of the BFG 10K direct
  damage-family branch.
- **Primary owner:** `Game`'s typed ranged execution boundary selects Plasma for
  the BFG 10K direct target; `World` and `Actor` retain the existing resistance
  and flat-protection formulas. Boundary crates remain projections.
- **Content registration:** the existing BFG 10K catalog definition remains
  the single source of its identity and damage range; no duplicate weapon table
  or callback registry is introduced.
- **Project version:** implementation advances `VERSION` from `0.2.335` to
  `0.2.336`.
- **Replay/RNG:** gameplay semantics advance from `137` to `138`; replay wire,
  RNG sampling (`1`), generator semantics (`2`), and ruleset identity
  (`drl-rs-ruleset-v1`) remain unchanged. Typed mitigation consumes no RNG.
- **Protocol/boundaries:** no new wire event or schema is needed; the existing
  optional `DamageApplied.damage_type` projection carries `Plasma`.

### 2.4 Review and branch contract

- Every successful BFG 10K direct volley hit emits `DamageApplied` with
  `damage_type: Some(DamageType::Plasma)` and applies the existing Plasma
  resistance before flat protection.
- The raw `AttackOutcome::Hit` damages and one-roll-per-projectile RNG stream
  remain unchanged; `AttackOutcome::is_lethal` retains its existing raw-damage
  contract, while actual actor death remains authoritative in `World`.
- BFG 10K splash cells retain typed Plasma mitigation, thresholded ground-item
  destruction, self-safe geometry, deduplication, event ordering, and final RNG
  state. Five-projectile volley cost, chainfire progression, delayed schedules,
  and Rocket/Standard/Nuclear BFG behavior remain unchanged.
- Other direct weapons retain their untyped damage events and prior mitigation.
- The existing core transaction guard still owns command rejection and exact
  state/RNG restoration; this slice adds no queue, callback, or new clone.

### 2.5 Acceptance criteria

- [x] BFG 10K direct volley hits emit typed Plasma damage and Blue Armor reduces
  the applied amounts using the existing deterministic
  resistance-before-flat protection formula.
- [x] An unarmored control preserves all five raw direct damage amounts, while
  the same seed keeps the raw rolls and final RNG state identical between
  armored and unarmored targets.
- [x] Direct replay and repeated replay runs produce identical game state,
  event stream, and typed direct-volley projection; stale semantics `137` is
  rejected before execution.
- [x] Existing BFG 10K splash/ground-item, chainfire, rejection/rollback,
  replay, MCP, metrics/audio/render, and BrowserSession parity tests pass
  without new wire or RNG behavior.
- [x] Formatting, clippy, `sh scripts/check-repository.sh`, version transition,
  hosted checks, and an attributable independent determinism review pass on the
  final implementation commit.

### 2.6 Non-goals

- No direct Plasma classification for other weapons, generic resistance
  aggregation, legacy armor durability degradation, or SPLASMA divisors.
- No BFG 10K chainfire, splash, ground-item, projectile routing, delayed queue,
  terrain/content mutation, callback recreation, or runtime/audiovisual parity
  changes beyond the direct target classification.
- No change to the replay wire schema, RNG sampling algorithm, generator
  semantics, ruleset identity, or unrelated event projections.

### 2.7 Evidence boundary

This slice will prove the current-Rust BFG 10K direct-volley Plasma events and
Blue Armor mitigation policy, plus replay determinism and stable state/RNG
behavior. It does not prove controlled legacy runtime, exact timing or
accuracy, scatter/projectile routing, terrain/content behavior, broader
resistance aggregation, durability, balance, audiovisual parity, or human play.

### 2.8 Delivery evidence

Delivery evidence is complete for implementation head `21c9df4`:

- Independent determinism review: **PASS** by `/root/rocket_review`; no
  actionable findings were identified.
- Focused and locked workspace tests, Clippy, formatting, version check,
  repository checks, native/headless browser checks, and `git diff --check`:
  **PASS**. The optional reference-capture preflight is `NOT_RUN` because its
  local manifest is unavailable.
- PR #452 hosted Repository and WASM browser checks: **PASS** in run
  `33664274355`. The protected-path Review policy check failed closed in run
  `33664274159` because the sole maintainer cannot create a non-self approval;
  the documented live `enforce_admins=false` exception remains in force after
  the independent review receipt was recorded.
- Merge remains pending; no controlled legacy runtime, audiovisual, balance,
  or human-play claim is inferred from these checks.

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
