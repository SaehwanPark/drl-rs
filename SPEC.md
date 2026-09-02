# Specification

Last reviewed: 2026-09-02
Current project version: `0.2.338`
Audited starting checkpoint: `main` at `0f5c047` (Plasma Rifle direct-Plasma
slice and canonical documentation reconciliation)
Delivery checkpoint: `main` merge commit `6c6387c` (PR #454, merged)

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

## 2. Active implementation slice: M9 — Nuclear Plasma Rifle direct Plasma classification

Slice status: **delivered and verified** at implementation head `3f1601e`;
merge checkpoint is `6c6387c` (PR #454, merged).

### 2.1 Objective

Complete the Nuclear Plasma Rifle direct-hit damage-family branch. The pinned
legacy record declares `1d7 DAMAGE_PLASMA`; after each successful ordinary or
already implemented chainfire hit, route only the Nuclear Plasma Rifle target
damage through the existing typed Plasma path so catalog-defined Blue Armor
resistance applies before flat protection. Preserve the already-delivered
six-projectile volley, first-through-seventh chainfire progression, periodic
recharge, alternate overload, event ordering, death/drop handling, and
transaction boundary.

This is a bounded vertical fidelity slice. It changes the direct
`DamageApplied` event from unclassified to `Some(DamageType::Plasma)` only for
Nuclear Plasma Rifle hits and leaves every other direct weapon path unchanged.

### 2.2 Audited starting point

At audited starting revision `0f5c047` (version `0.2.337`):

- The Nuclear Plasma Rifle record (`eitems.lua:436-472`) declares a
  six-projectile `1d7 DAMAGE_PLASMA` weapon, while `dfbeing.pas:2536-2549` passes the item
  damage type into direct `ApplyDamage` and `dfbeing.pas:2162-2182` maps Plasma
  families to plasma resistance.
- Rust already has typed Plasma actor damage, catalog-defined Blue Armor 20%
  Plasma resistance, and a typed `DamageApplied` event field. The Nuclear
  Plasma Rifle six-projectile ordinary volley, first-through-seventh chainfire
  transitions, periodic recharge, and alternate overload are already
  implemented and projected through core, replay, MCP, and browser boundaries.
- The shared direct ranged path still calls untyped `World::apply_damage` and
  emits `damage_type: None` for each Nuclear Plasma Rifle direct volley hit.

### 2.3 Scope and ownership

- **Roadmap:** M9 vertical canonical-fidelity completion of the Nuclear Plasma
  damage-family branch.
- **Primary owner:** `Game`'s typed ranged execution boundary selects Plasma for
  the Nuclear Plasma Rifle direct target; `World` and `Actor` retain the existing
  resistance and flat-protection formulas. Boundary crates remain projections.
- **Content registration:** the existing Nuclear Plasma Rifle catalog definition remains
  the single source of its identity and damage range; no duplicate weapon table
  or callback registry is introduced.
- **Project version:** implementation advances `VERSION` from `0.2.337` to
  `0.2.338`.
- **Replay/RNG:** gameplay semantics advance from `139` to `140`; replay wire,
  RNG sampling (`1`), generator semantics (`2`), and ruleset identity
  (`drl-rs-ruleset-v1`) remain unchanged. Typed mitigation consumes no RNG.
- **Protocol/boundaries:** no new wire event or schema is needed; the existing
  optional `DamageApplied.damage_type` projection carries `Plasma`.

### 2.4 Review and branch contract

- Every successful Nuclear Plasma Rifle ordinary or chainfire target hit emits
  `DamageApplied` with
  `damage_type: Some(DamageType::Plasma)` and applies the existing Plasma
  resistance before flat protection.
- The raw `AttackOutcome` results and one-roll-per-projectile RNG stream remain
  unchanged; `AttackOutcome::is_lethal` retains its existing raw-damage
  contract, while actual actor death remains authoritative in `World`.
- Nuclear Plasma Rifle's six-projectile ordinary volley, first-through-seventh
  chainfire counts, clip costs, warm-up state, periodic recharge, alternate
  overload, event ordering, and final RNG state remain unchanged. Other direct
  weapons and BFG/rocket fanouts retain their existing typed or untyped paths.
- Other direct weapons retain their untyped damage events and prior mitigation.
- The existing core transaction guard still owns command rejection and exact
  state/RNG restoration; this slice adds no queue, callback, or new clone.

### 2.5 Acceptance criteria

- [x] Nuclear Plasma Rifle direct ordinary and chainfire hits emit typed Plasma damage
  and Blue Armor reduces the applied amounts using the existing deterministic
  resistance-before-flat protection formula.
- [x] An unarmored control preserves all successful raw direct damage amounts,
  while
  the same seed keeps the raw rolls and final RNG state identical between
  armored and unarmored targets.
- [x] Direct replay and repeated replay runs produce identical game state,
  event stream, and typed direct-volley projection; stale semantics `139` is
  rejected before execution.
- [x] Existing Nuclear Plasma Rifle chainfire, rejection/rollback, replay, MCP,
  metrics/audio/render, and BrowserSession parity tests pass without new wire
  or RNG behavior.
- [x] Formatting, clippy, `sh scripts/check-repository.sh`, version transition,
  hosted checks, and an attributable independent determinism review pass on the
  final implementation commit.

### 2.6 Non-goals

- No direct Plasma classification for other weapons, generic resistance
  aggregation, legacy armor durability degradation, or SPLASMA divisors.
- No Nuclear Plasma Rifle overcharge behavior changes, higher chainfire levels,
  spread/routing, delayed queue, terrain/content mutation, callback recreation,
  or runtime/audiovisual parity changes beyond the direct target classification.
- No change to the replay wire schema, RNG sampling algorithm, generator
  semantics, ruleset identity, or unrelated event projections.

### 2.7 Evidence boundary

This slice proves the current-Rust Nuclear Plasma Rifle direct
ordinary/chainfire Plasma events and Blue Armor mitigation policy, plus replay
determinism and stable state/RNG behavior. It does not prove controlled legacy
runtime, exact timing or accuracy, overload/nuke effects, higher chainfire
levels, spread/routing, recharge timing beyond the existing policy,
terrain/content behavior, broader resistance aggregation, durability, balance,
audiovisual parity, or human play.

### 2.8 Delivery evidence

Delivery evidence: implementation head `3f1601e` merged in PR #454 as
`6c6387c`. Focused Nuclear Plasma direct-Plasma tests (3/3), existing
chainfire tests (22/22), Nuclear Plasma browser-boundary tests (3/3), the full
workspace suite, strict Clippy, version/repository gates, and hosted Repository
and WASM checks pass. The independent determinism review returned PASS. The
hosted Review-policy check remains the documented solo-maintainer exception;
the optional reference-capture preflight is `NOT_RUN` because its local
manifest is unavailable. Controlled legacy runtime, audiovisual, balance, and
human-play surfaces remain outside this slice's evidence boundary.

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
