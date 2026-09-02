# Specification

Last reviewed: 2026-09-02
Current project version: `0.2.330`
Audited starting checkpoint: `main` at `402ea05` (PR #445 Red Armor merge
and steering baseline reconciled)
Delivery checkpoint: **in progress** on temporary branch
`codex/anti-freak-fire-resistance`

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

## 2. Active implementation slice: M9 — Anti-Freak Jackal Fire mitigation

Slice status: **in progress** on the temporary branch above.

### 2.1 Objective

Route the existing Anti-Freak Jackal radius-1 explosion through the typed Fire
damage path. Red Armor's catalog-defined `25%` Fire resistance is applied before
the existing flat armor protection with deterministic integer rounding and the
legacy minimum-one rule. The item catalog remains the single source for the
value; no broad legacy resistance stack is recreated.

This is a bounded vertical fidelity slice. It preserves the existing
center-plus-eight-neighbor fanout, one `5d3` roll per blast cell, radial
knockback, raw-damage ground-ammo threshold, event ordering, and transaction
boundaries while adding the missing typed mitigation. Direct Fire weapon
classification, armor durability/body zones, equipment slots, hooks, difficulty
modifiers, and the remaining Plasma/Acid resistance families remain separate
work.

### 2.2 Audited starting point

At audited starting revision `402ea05` (version `0.2.329`):

- Anti-Freak Jackal's custom splash resolver emits a typed Fire event but calls
  the untyped `World::apply_damage`, so Red Armor does not mitigate its blast.
- The Anti-Freak item record (`uitems.lua:321-357`) documents `5d3` Fire damage
  and radius `1`; the pinned explosion loop (`dflevel.pas:1039-1080`) passes its
  damage type into `ApplyDamage` for each actor.
- Red Armor is already definition-backed with protection `4`, durability `100`,
  and `25%` Fire resistance. The typed actor/world path applies that
  percentage before flat protection using the verified integer rounding and
  minimum-one policy.

### 2.3 Scope and ownership

- **Roadmap:** M9 vertical canonical-fidelity completion for the Anti-Freak
  Jackal Fire mitigation branch.
- **Primary owners:** the Anti-Freak behavior resolver owns blast geometry,
  rolls, and event ordering; `World` routes typed damage; the armor catalog and
  `ArmorProperties` own the resistance value and lookup; boundary crates remain
  projections only.
- **Content registration:** Red Armor remains the single catalog source for
  `25` Fire resistance; no archetype-name match or duplicate resistance table
  is introduced.
- **Project version:** implementation advances `VERSION` from `0.2.329` to
  `0.2.330`.
- **Replay/RNG:** gameplay semantics advance from `131` to `132`; replay wire,
  RNG sampling (`1`), generator semantics (`2`), and ruleset identity
  (`drl-rs-ruleset-v1`) remain unchanged. Resistance consumes no RNG; accepted
  and rejected command transaction guarantees remain unchanged.
- **Protocol/boundaries:** no new wire event is needed. Existing typed
  `DamageApplied` projections remain stable while the core applies the policy.

### 2.4 Review and branch contract

- Percentage mitigation is computed with integer arithmetic, rounds to nearest
  for positive values, and clamps a nonzero resisted amount to one before flat
  protection. Zero damage remains zero.
- The Anti-Freak actor splash now uses the typed Fire route; the delivered
  Rocket Fire and BFG Plasma paths retain their behavior, while untyped direct
  hits and unrelated environment damage preserve their prior behavior.
- Blast-cell RNG sampling, geometry, knockback, raw ground-ammo threshold, and
  event/death ordering remain unchanged.
- The existing core transaction guard still owns command rejection and exact
  state/RNG restoration; this slice adds no new mutable queue or callback.
- The core remains independent of filesystem, browser, audio, and MCP IO.

### 2.5 Acceptance criteria

- [x] The Anti-Freak actor splash routes through typed Fire mitigation and
  retains the existing event, knockback, ground-ammo, and death ordering.
- [x] A same-seed replay pair proves that Red Armor reduces the player splash
  amount by the catalog-defined resistance plus flat protection while the
  unarmored amount remains the raw roll.
- [x] Existing Anti-Freak rejection/rollback, replay, MCP, metrics/audio/render,
  and BrowserSession parity tests pass without new wire or RNG behavior.
- [ ] Formatting, clippy, `sh scripts/check-repository.sh`, version transition,
  hosted checks, and an attributable independent determinism review pass on the
  final implementation commit.

### 2.6 Non-goals

- No full legacy resistance aggregation across weapons, body zones, hooks,
  difficulty, or durability; no new equipment-slot model.
- No direct Fire weapon classification, Plasma/Acid resistance migration,
  terrain/cell destruction, or audiovisual/balance work.
- No claim of controlled legacy runtime, audiovisual, balance, browser, or
  performance parity beyond the current-Rust tests and hosted browser gate.

### 2.7 Evidence boundary

This slice proves the current-Rust Anti-Freak Jackal Fire actor-splash route,
Red Armor mitigation, replay determinism, and stable boundary projections. It
will not prove legacy body-zone aggregation, direct Fire classification, other
resistance families, controlled legacy runtime, audiovisual parity, balance, or
human play; those surfaces remain open or `NOT_RUN` in the roadmap.

### 2.8 Delivery evidence

- Implementation and delivery evidence will be recorded here after the
  independent review, hosted checks, merge, and post-merge reconciliation.

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
