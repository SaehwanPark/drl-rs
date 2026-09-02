# Specification

Last reviewed: 2026-09-02
Current project version: `0.2.335`
Audited starting checkpoint: `main` at `3f059d3` (Standard BFG direct-Plasma
slice and canonical documentation reconciliation)
Delivery checkpoint: `main` merge commit `c58abc9` (PR #451, merged)

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

## 2. Active implementation slice: M9 — Nuclear BFG 9000 direct Plasma classification

Slice status: **delivered and verified** on `main` in merge commit `c58abc9`.

### 2.1 Objective

Complete the Nuclear BFG 9000 direct-hit damage-family branch. The pinned
legacy record declares `8d6 DAMAGE_SPLASMA`; after a successful direct hit,
route only the Nuclear BFG 9000 target damage through the existing typed Plasma
path so catalog-defined Blue Armor resistance applies before flat protection.
Preserve the already-delivered radius-8 splash, ground-item effect, recharge,
overload, event ordering, death/drop handling, and transaction boundary.

This is a bounded vertical fidelity slice. It changes the direct
`DamageApplied` event from unclassified to `Some(DamageType::Plasma)` only for
Nuclear BFG 9000 hits and leaves every other direct weapon path unchanged.

### 2.2 Audited starting point

At audited starting revision `3f059d3` (version `0.2.334`):

- The Nuclear BFG 9000 record (`eitems.lua:474-518`) declares an `8d6`
  `DAMAGE_SPLASMA` weapon, while `dfbeing.pas:2636-2644` propagates the weapon
  damage family into direct damage and explosion resolution.
- Rust already has typed Plasma actor damage, catalog-defined Blue Armor 20%
  Plasma resistance, and a typed `DamageApplied` event field. The Nuclear BFG
  radius-8 splash is already typed Plasma, as are its ground-item and overload
  boundaries where applicable.
- The shared direct ranged path still calls untyped `World::apply_damage` and
  emits `damage_type: None` for the Nuclear BFG 9000 direct hit.

### 2.3 Scope and ownership

- **Roadmap:** M9 vertical canonical-fidelity completion of the Nuclear BFG
  9000 direct damage-family branch.
- **Primary owner:** `Game`'s typed ranged execution boundary selects Plasma for
  the Nuclear BFG 9000 direct target; `World` and `Actor` retain the existing
  resistance and flat-protection formulas. Boundary crates remain projections.
- **Content registration:** the existing Nuclear BFG 9000 catalog definition
  remains the single source of its identity and damage range; no duplicate
  weapon table or callback registry is introduced.
- **Project version:** implementation advances `VERSION` from `0.2.334` to
  `0.2.335`.
- **Replay/RNG:** gameplay semantics advance from `136` to `137`; replay wire,
  RNG sampling (`1`), generator semantics (`2`), and ruleset identity
  (`drl-rs-ruleset-v1`) remain unchanged. Typed mitigation consumes no RNG.
- **Protocol/boundaries:** no new wire event or schema is needed; the existing
  optional `DamageApplied.damage_type` projection carries `Plasma`.

### 2.4 Review and branch contract

- A successful Nuclear BFG 9000 direct hit emits `DamageApplied` with
  `damage_type: Some(DamageType::Plasma)` and applies the existing Plasma
  resistance before flat protection.
- The raw `AttackOutcome::Hit` damage and one-roll RNG stream remain unchanged;
  `AttackOutcome::is_lethal` retains its existing raw-damage contract, while
  actual actor death remains authoritative in `World`.
- Nuclear BFG splash cells retain typed Plasma mitigation, thresholded
  ground-item destruction, self-safe geometry, deduplication, event ordering,
  and final RNG state. Recharge, alternate overload, and Rocket/Standard BFG
  behavior remain unchanged.
- Other direct weapons retain their untyped damage events and prior mitigation.
- The existing core transaction guard still owns command rejection and exact
  state/RNG restoration; this slice adds no queue, callback, or new clone.

### 2.5 Acceptance criteria

- [x] Nuclear BFG 9000 direct hits emit typed Plasma damage and Blue Armor
  reduces the applied amount using the existing deterministic
  resistance-before-flat protection formula.
- [x] An unarmored control preserves the raw direct damage amount, while the
  same seed keeps the raw roll and final RNG state identical between armored
  and unarmored targets.
- [x] Direct replay and repeated replay runs produce identical game state,
  event stream, and typed direct-hit projection; stale semantics `136` is
  rejected before execution.
- [x] Existing Nuclear BFG splash/ground-item, recharge/overload, rejection/
  rollback, replay, MCP,
  metrics/audio/render, and BrowserSession parity tests pass without new wire
  or RNG behavior.
- [x] Formatting, clippy, `sh scripts/check-repository.sh`, version transition,
  hosted checks, and an attributable independent determinism review pass on the
  final implementation commit.

### 2.6 Non-goals

- No direct Plasma classification for other weapons, generic resistance
  aggregation, legacy armor durability degradation, or SPLASMA divisors.
- No Nuclear BFG splash, recharge, overload, projectile routing, delayed queue,
  terrain/content mutation, callback recreation, or runtime/audiovisual parity
  changes beyond the direct target classification.
- No change to the replay wire schema, RNG sampling algorithm, generator
  semantics, ruleset identity, or unrelated event projections.

### 2.7 Evidence boundary

This slice proves the current-Rust Nuclear BFG 9000 direct-hit Plasma event and
Blue Armor mitigation policy, plus replay determinism and stable state/RNG
behavior. It does not prove controlled legacy runtime, exact timing or
accuracy, projectile routing, terrain/content behavior, broader resistance
aggregation, durability, balance, audiovisual parity, or human play.

### 2.8 Delivery evidence

Implementation commit `d2f1236` merged through PR #451 as `c58abc9`. Focused
and workspace tests, formatting, clippy, version/repository/web checks, hosted
Repository/WASM checks, and the independent `/root/rocket_review` determinism
review all passed. The hosted Review-policy check remains the documented
solo-maintainer exception (`enforce_admins=false`). The optional
reference-capture preflight remains `NOT_RUN` because its local manifest is
unavailable; controlled legacy runtime, audiovisual, and human-play surfaces
remain outside this slice's evidence boundary.

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
