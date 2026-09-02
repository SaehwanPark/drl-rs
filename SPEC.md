# Specification

Last reviewed: 2026-09-02
Current project version: `0.2.340`
Audited starting checkpoint: `main` at `2a089c6` (Laser Rifle direct-Plasma
delivery and canonical documentation reconciliation)
Delivery checkpoint: `main` merge commit `d855725` (PR #456, merged)

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

## 2. Active implementation slice: M9 — Blaster direct Plasma classification

Slice status: **delivered and verified** at implementation head `8e05aa5`;
merge checkpoint is `d855725` (PR #456, merged).

### 2.1 Objective

Complete the Blaster direct-hit damage-family branch. The pinned legacy record
declares `2d4 DAMAGE_PLASMA`; after each successful ordinary or aimed hit, route
only the Blaster target damage through the existing typed Plasma path so
catalog-defined Blue Armor resistance applies before flat protection. Preserve
the already-delivered one-projectile shot, aimed-fire action cost, recharge
timer, event ordering, death/drop handling, and transaction boundary.

This is a bounded vertical fidelity slice. It changes the direct
`DamageApplied` event from unclassified to `Some(DamageType::Plasma)` only for
Blaster hits and leaves every other direct weapon path unchanged.

### 2.2 Audited starting point

At audited starting revision `2a089c6` (version `0.2.339`):

- The Blaster record (`eitems.lua:135-169`) declares a one-projectile `2d4
  DAMAGE_PLASMA` weapon, while `dfbeing.pas:2536-2549` passes the item damage
  type into direct `ApplyDamage` and `dfbeing.pas:2162-2182` maps Plasma
  families to plasma resistance.
- Rust already has typed Plasma actor damage, catalog-defined Blue Armor 20%
  Plasma resistance, and a typed `DamageApplied` event field. The Blaster
  one-projectile ordinary shot, aimed-fire command, recharge transition, and
  no-reload rejection are already implemented and projected through core,
  replay, MCP, and browser boundaries.
- The shared direct ranged path called untyped `World::apply_damage` and
  emitted `damage_type: None` for each Blaster direct hit.

### 2.3 Scope and ownership

- **Roadmap:** M9 vertical canonical-fidelity completion of the Blaster
  damage-family branch.
- **Primary owner:** `Game`'s typed ranged execution boundary selects Plasma for
  the Blaster direct target; `World` and `Actor` retain the existing
  resistance and flat-protection formulas. Boundary crates remain projections.
- **Content registration:** the existing Blaster catalog definition remains
  the single source of its identity and damage range; no duplicate weapon table
  or callback registry is introduced.
- **Project version:** implementation advances `VERSION` from `0.2.339` to
  `0.2.340`.
- **Replay/RNG:** gameplay semantics advance from `141` to `142`; replay wire,
  RNG sampling (`1`), generator semantics (`2`), and ruleset identity
  (`drl-rs-ruleset-v1`) remain unchanged. Typed mitigation consumes no RNG.
- **Protocol/boundaries:** no new wire event or schema is needed; the existing
  optional `DamageApplied.damage_type` projection carries `Plasma`.

### 2.4 Review and branch contract

- Every successful Blaster ordinary or aimed target hit emits
  `DamageApplied` with
  `damage_type: Some(DamageType::Plasma)` and applies the existing Plasma
  resistance before flat protection.
- The raw `AttackOutcome` results and one-roll-per-projectile RNG stream remain
  unchanged; `AttackOutcome::is_lethal` retains its existing raw-damage
  contract, while actual actor death remains authoritative in `World`.
- Blaster's one-projectile ordinary/aimed shot, one-cell cost, recharge timer,
  no-reload policy, event ordering, and final RNG state remain unchanged. Other
  direct weapons and BFG/rocket fanouts retain their existing typed or untyped
  paths.
- Other direct weapons retain their untyped damage events and prior mitigation.
- The existing core transaction guard still owns command rejection and exact
  state/RNG restoration; this slice adds no queue, callback, or new clone.

### 2.5 Acceptance criteria

- [x] Blaster direct ordinary and aimed hits emit typed Plasma damage
  and Blue Armor reduces the applied amounts using the existing deterministic
  resistance-before-flat protection formula.
- [x] An unarmored control preserves all successful raw direct damage amounts,
  while
  the same seed keeps the raw rolls and final RNG state identical between
  armored and unarmored targets.
- [x] Direct replay and repeated replay runs produce identical game state,
  event stream, and typed direct-shot projection; stale semantics `141` is
  rejected before execution.
- [x] Existing Blaster aimed/recharge/no-reload rejection/rollback, replay, MCP,
  metrics/audio/render, and BrowserSession parity tests pass without new wire
  or RNG behavior.
- [x] Formatting, clippy, `sh scripts/check-repository.sh`, version transition,
  hosted checks, and an attributable independent determinism review pass on the
  final implementation commit.

### 2.6 Non-goals

- No direct Plasma classification for other weapons, generic resistance
  aggregation, legacy armor durability degradation, or SPLASMA divisors.
- No Blaster recharge timing/state changes, aimed accuracy/action-cost changes,
  manual-reload policy changes, spread/routing, delayed queue, terrain/content
  mutation, callback recreation, or runtime/audiovisual parity changes beyond
  the direct target classification.
- No change to the replay wire schema, RNG sampling algorithm, generator
  semantics, ruleset identity, or unrelated event projections.

### 2.7 Evidence boundary

This slice proves the current-Rust Blaster direct ordinary/aimed Plasma
events and Blue Armor mitigation policy, plus replay determinism and stable
state/RNG behavior. It does not prove controlled legacy runtime, exact recharge
cadence or accuracy, manual callback state, spread/routing, terrain/content
behavior, broader resistance aggregation, durability, balance, audiovisual
parity, or human play.

### 2.8 Delivery evidence

Delivery evidence: implementation head `8e05aa5` merged in PR #456 as
`d855725`. Focused Blaster direct-Plasma tests (2/2), existing Blaster
special-item tests (152/152), the full workspace suite, strict Clippy,
version/repository/diff gates, and hosted Repository and WASM checks pass. The
independent determinism review returned PASS. The hosted Review-policy check
remains the documented solo-maintainer exception; the optional
reference-capture preflight is `NOT_RUN` because its local manifest is
unavailable. Controlled legacy runtime, audiovisual, balance, and human-play
surfaces remain outside this slice's evidence boundary.

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
