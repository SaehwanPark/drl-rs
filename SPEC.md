# Specification

Last reviewed: 2026-09-02
Current project version: `0.2.329`
Audited starting checkpoint: `main` at `2dc55b6` (PR #444 Blue Armor merge
and steering baseline reconciled)
Delivery checkpoint: `main` merge commit `530794c` (PR #445, merged)

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

## 2. Active implementation slice: M9 — Red Armor Fire mitigation

Slice status: **delivered and verified** at the delivery checkpoint above.

### 2.1 Objective

Apply the documented Red Armor `25%` Fire resistance to the existing typed
actor-splash damage path. Resistance is applied before the existing flat armor
protection and uses deterministic integer rounding with the legacy minimum-one
rule. The item catalog remains the single source for the value; no broad legacy
resistance stack is recreated.

This is a bounded vertical fidelity slice. It covers typed actor damage already
emitted by the Rocket Fire splash and leaves direct Fire weapon classification,
armor durability/body zones, equipment slots, hooks, difficulty modifiers, and
the remaining Plasma/Acid resistance families for separate work.

### 2.2 Audited starting point

At audited starting revision `2dc55b6` (version `0.2.328`):

- Red Armor is definition-covered with protection `4` and durability `100`,
  but has no typed Fire resistance field or runtime mitigation.
- `GameEvent::DamageApplied` and the Rocket actor-splash policy already carry
  `DamageType::Fire`; the typed world route currently has no Red Armor Fire
  resistance to apply.
- The legacy item record (`items.lua:74-90`) gives Red Armor `resist.fire =
  25`. Legacy `dfbeing.pas:2078-2182` applies typed percentage resistance
  before armor protection and clamps nonzero damage to at least one point.

### 2.3 Scope and ownership

- **Roadmap:** M9 vertical canonical-fidelity completion for the Red Armor
  Fire mitigation branch.
- **Primary owners:** the armor content definition owns the resistance value;
  `ArmorProperties` exposes typed lookup; `Actor` owns pure mitigation order;
  `World` and `Game` route typed splash damage; boundary crates remain
  projections only.
- **Content registration:** every armor catalog entry carries explicit typed
  resistance fields, with Red Armor set to `25` Fire, the delivered Blue Armor
  set to `20` Plasma, and other current values explicitly `0`; no
  archetype-name match or duplicate resistance table is introduced.
- **Project version:** implementation advances `VERSION` from `0.2.328` to
  `0.2.329`.
- **Replay/RNG:** gameplay semantics advance from `130` to `131`; replay wire,
  RNG sampling (`1`), generator semantics (`2`), and ruleset identity
  (`drl-rs-ruleset-v1`) remain unchanged. Resistance consumes no RNG; accepted
  and rejected command transaction guarantees remain unchanged.
- **Protocol/boundaries:** no new wire event is needed. Existing typed
  `DamageApplied` projections remain stable while the core applies the policy.

### 2.4 Review and branch contract

- Percentage mitigation is computed with integer arithmetic, rounds to nearest
  for positive values, and clamps a nonzero resisted amount to one before flat
  protection. Zero damage remains zero.
- The typed route is used by the existing Rocket Fire actor splash; BFG Plasma
  retains its delivered Blue Armor mitigation, while untyped direct hits and
  environment damage preserve their prior behavior.
- The existing core transaction guard still owns command rejection and exact
  state/RNG restoration; this slice adds no new mutable queue or callback.
- The core remains independent of filesystem, browser, audio, and MCP IO.

### 2.5 Acceptance criteria

- [x] Red Armor’s catalog resistance is carried into `ArmorProperties` and a
  pure typed mitigation helper covers rounding, minimum-one, and zero cases.
- [x] Rocket Fire actor splash routes through typed mitigation; Red Armor
  reduces the same deterministic hit while Blue Armor Plasma and non-Fire
  paths remain unchanged.
- [x] Existing rejection/rollback, replay, MCP, metrics/audio/render, and
  BrowserSession parity tests pass without new wire or RNG behavior.
- [x] Formatting, clippy, `sh scripts/check-repository.sh`, version transition,
  and an attributable independent determinism review pass on the final
  implementation commit.

### 2.6 Non-goals

- No full legacy resistance aggregation across weapons, body zones, hooks,
  difficulty, or durability; no new equipment-slot model.
- No direct Fire weapon classification, Plasma/Acid resistance migration,
  terrain/content mutation, or audiovisual/balance work.
- No claim of controlled legacy runtime, audiovisual, balance, browser, or
  performance parity beyond the current-Rust tests and hosted browser gate.

### 2.7 Evidence boundary

This slice proves the current-Rust Red Armor mitigation branch for typed
Fire actor splash and its stable boundary projections. It will not prove
legacy body-zone aggregation, direct Fire classification, other resistance
families, controlled legacy runtime, audiovisual parity, balance, or human
play; those surfaces remain open or `NOT_RUN` in the roadmap.

### 2.8 Delivery evidence

- Independent determinism review of implementation head
  `3370713fc66c1585fafa2a0d7fe8d6357902becf`: **PASS**. The focused
  follow-up review of docs-only steering reconciliation at final branch head
  `4688048a426d273c13b6ca9d4b3e842c37371d8b`: **PASS**.
- Local workspace tests, clippy, formatting, version check, repository checks,
  and native/headless browser checks: **PASS**. The optional reference-capture
  preflight is `NOT_RUN` because its local manifest is unavailable.
- PR #445 hosted Repository checks and WASM browser checks: **PASS** in run
  `33644720157`. The protected-path Review policy check failed closed under the
  documented solo-maintainer `enforce_admins=false` exception; administrator
  merge was used after the review receipt was recorded.
- Merge checkpoint: `530794c`; the temporary implementation branch was removed
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
