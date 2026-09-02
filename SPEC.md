# Specification

Last reviewed: 2026-09-02
Current project version: `0.2.331`
Audited starting checkpoint: `main` at `ca31143` (PR #446 Anti-Freak merge
and canonical docs reconciliation)
Delivery checkpoint: pending for `codex/null-pointer-plasma-resistance`

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

## 2. Active implementation slice: M9 — Null Pointer Plasma mitigation

Slice status: **in progress** on the bounded implementation branch above.

### 2.1 Objective

Route the existing Null Pointer actor-only radius-1 explosion through the typed
Plasma damage path. Blue Armor's catalog-defined `20%` Plasma resistance is
applied before the existing flat armor protection with deterministic integer
rounding and the legacy minimum-one rule. The item catalog remains the single
source for the value; no broad legacy resistance stack is recreated.

This is a bounded vertical fidelity slice. It preserves the existing
center-plus-eight-neighbor fanout, fixed `10d1` damage, stable actor
deduplication, event ordering, death/drop handling, and transaction boundaries
while adding the missing typed mitigation. Direct Plasma weapon classification,
the legacy SPLASMA armor divisor, armor durability/body zones, equipment slots,
hooks, difficulty modifiers, and broader resistance aggregation remain separate
work.

### 2.2 Audited starting point

At audited starting revision `ca31143` (version `0.2.330`):

- Null Pointer's custom splash resolver emits a typed Plasma event but calls the
  untyped `World::apply_damage`, so Blue Armor does not mitigate its blast.
- The Null Pointer item record (`uitems.lua:63-112`) schedules a range-1 `10d1`
  `DAMAGE_SPLASMA` explosion; the pinned explosion loop
  (`dflevel.pas:1039-1080`) passes its damage type into `ApplyDamage` for each
  actor.
- Blue Armor is already definition-backed with protection `2`, durability
  `100`, and `20%` Plasma resistance. The typed actor/world path applies that
  percentage before flat protection using the verified integer rounding and
  minimum-one policy.

### 2.3 Scope and ownership

- **Roadmap:** M9 vertical canonical-fidelity completion for the Null Pointer
  Plasma mitigation branch.
- **Primary owners:** the Null Pointer behavior resolver owns blast geometry,
  fixed damage, and event ordering; `World` routes typed damage; the armor catalog and
  `ArmorProperties` own the resistance value and lookup; boundary crates remain
  projections only.
- **Content registration:** Blue Armor remains the single catalog source for
  `20` Plasma resistance; no archetype-name match or duplicate resistance table
  is introduced.
- **Project version:** implementation advances `VERSION` from `0.2.330` to
  `0.2.331`.
- **Replay/RNG:** gameplay semantics advance from `132` to `133`; replay wire,
  RNG sampling (`1`), generator semantics (`2`), and ruleset identity
  (`drl-rs-ruleset-v1`) remain unchanged. Resistance consumes no RNG; accepted
  and rejected command transaction guarantees remain unchanged.
- **Protocol/boundaries:** no new wire event is needed. Existing typed
  `DamageApplied` projections remain stable while the core applies the policy.

### 2.4 Review and branch contract

- Percentage mitigation is computed with integer arithmetic, rounds to nearest
  for positive values, and clamps a nonzero resisted amount to one before flat
  protection. Zero damage remains zero.
- The Null Pointer actor splash now uses the typed Plasma route; the delivered
  BFG Plasma and Anti-Freak/Rocket Fire paths retain their behavior, while
  untyped direct hits and unrelated environment damage preserve their prior
  behavior.
- Fixed splash damage, geometry, deduplication, event/death ordering, and RNG
  state remain unchanged.
- The existing core transaction guard still owns command rejection and exact
  state/RNG restoration; this slice adds no new mutable queue or callback.
- The core remains independent of filesystem, browser, audio, and MCP IO.

### 2.5 Acceptance criteria

- [ ] The Null Pointer actor splash routes through typed Plasma mitigation and
  retains the existing event, fanout, and death/drop ordering.
- [ ] A same-seed replay pair proves that Blue Armor reduces the player splash
  amount by the catalog-defined resistance plus flat protection while the
  unarmored amount remains the fixed raw damage.
- [ ] Existing Null Pointer rejection/rollback, replay, MCP, metrics/audio/
  render, and BrowserSession parity tests pass without new wire or RNG behavior.
- [ ] Formatting, clippy, `sh scripts/check-repository.sh`, version transition,
  hosted checks, and an attributable independent determinism review pass on the
  final implementation commit.

### 2.6 Non-goals

- No full legacy resistance aggregation across weapons, body zones, hooks,
  difficulty, or durability; no new equipment-slot model.
- No direct Plasma weapon classification, legacy SPLASMA armor divisor,
  terrain/cell destruction, or audiovisual/balance work.
- No claim of controlled legacy runtime, audiovisual, balance, browser, or
  performance parity beyond the current-Rust tests and hosted browser gate.

### 2.7 Evidence boundary

This slice proves the current-Rust Null Pointer Plasma actor-splash route, Blue
Armor mitigation, replay determinism, and stable boundary projections. It will
not prove legacy body-zone aggregation, direct Plasma classification, the
legacy SPLASMA armor divisor, other resistance families, controlled legacy
runtime, audiovisual parity, balance, or human play; those surfaces remain open
or `NOT_RUN` in the roadmap.

### 2.8 Delivery evidence

- Delivery evidence is pending. The implementation and review handoff will
  record the exact final head, hosted run, independent review receipt, and merge
  checkpoint here after they exist.

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
