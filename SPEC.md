# Specification

Last reviewed: 2026-09-02
Current project version: `0.2.332`
Audited starting checkpoint: `main` at `3806611` (PR #447 Null Pointer merge
and canonical docs reconciliation)
Delivery checkpoint: `main` merge commit `e902d71` (PR #448, merged)

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

## 2. Active implementation slice: M9 — Rocket Launcher ground-item destruction

Slice status: **delivered and verified** at the delivery checkpoint above.

### 2.1 Objective

Complete the existing Rocket Launcher radius-4 explosion branch for the one
legacy ground-item effect represented by the current Rust world model. After
each clear blast cell's post-falloff `6d6` Fire damage, remove at most the
lowest-ID ordinary ground item when damage is strictly greater than `10`, and
emit the existing `GroundItemDestroyed` event after actor processing. No new
legacy feature marker is invented; every represented ground item is an
ordinary item under the current Rust policy.

This is a bounded vertical fidelity slice. It preserves the existing radius-4
geometry, one `6d6` roll per clear cell, integer distance falloff, stable actor
deduplication, typed Fire mitigation, event ordering, death/drop handling, and
transaction boundaries while adding the missing item effect. Terrain/content
mutation, feature or indestructible-item markers, chained explosions, delayed
queues, projectile routing, and rocket-jump behavior remain separate work.

### 2.2 Audited starting point

At audited starting revision `3806611` (version `0.2.331`):

- The Rocket Launcher record (`items.lua:657-705`) declares a one-shot `6d6`
  `DAMAGE_FIRE` radius-4 explosion with delay metadata.
- The pinned explosion loop (`dflevel.pas:1039-1085`) rolls one damage value per
  clear cell, applies integer distance falloff, processes the actor first, and
  destroys the cell's item when the resulting damage is greater than `10`.
- Rust already has deterministic radius-four geometry, typed Fire actor damage,
  post-falloff damage, `World::destroy_ground_item_at`, and the ordered
  `GroundItemDestroyed` event. The Rocket policy currently selects no item
  effect, so the rule is the remaining bounded gap.

### 2.3 Scope and ownership

- **Roadmap:** M9 vertical canonical-fidelity completion for the Rocket Launcher
  ground-item effect.
- **Primary owners:** the Rocket resolver owns blast geometry, roll/falloff, and
  actor ordering; the shared splash policy owns the thresholded item effect;
  `World` owns deterministic lowest-ID removal; boundary crates remain
  projections only.
- **Content registration:** the current ground-item vocabulary is the single
  source for representable ordinary items; no feature marker or duplicate item
  destruction table is introduced.
- **Project version:** implementation advances `VERSION` from `0.2.331` to
  `0.2.332`.
- **Replay/RNG:** gameplay semantics advance from `133` to `134`; replay wire,
  RNG sampling (`1`), generator semantics (`2`), and ruleset identity
  (`drl-rs-ruleset-v1`) remain unchanged. Item destruction consumes no RNG;
  accepted and rejected command transaction guarantees remain unchanged.
- **Protocol/boundaries:** no new wire event is needed. Existing typed
  `DamageApplied` projections remain stable while the core applies the policy.

### 2.4 Review and branch contract

- A post-falloff damage result of exactly `10` does not destroy an item; `11` or
  greater removes at most one lowest-ID ordinary item at that blast cell.
- The Rocket actor splash keeps its typed Fire path and applies actor damage
  before `GroundItemDestroyed`, with lethal death/drop follow-up afterward.
- Fixed damage rolls, geometry, distance falloff, deduplication, event ordering,
  and final RNG state remain unchanged.
- The existing core transaction guard still owns command rejection and exact
  state/RNG restoration; this slice adds no new mutable queue or callback.
- The core remains independent of filesystem, browser, audio, and MCP IO.

### 2.5 Acceptance criteria

- [x] Rocket Launcher blast cells remove the lowest-ID ordinary ground item only
  for post-falloff damage greater than `10`, preserve non-destructive cells, and
  emit the event after actor processing.
- [x] Same-seed direct/replay coverage proves item selection and event output
  are repeatable while the final RNG state matches the expected per-cell roll
  stream.
- [x] Existing Rocket Launcher rejection/rollback, replay, MCP, metrics/audio/
  render, and BrowserSession parity tests pass without new wire or RNG behavior.
- [x] Formatting, clippy, `sh scripts/check-repository.sh`, version transition,
  hosted checks, and an attributable independent determinism review pass on the
  final implementation commit.

### 2.6 Non-goals

- No terrain/content mutation, feature or indestructible-item model, chained
  explosions, delayed queue, projectile routing, or rocket-jump behavior.
- No direct damage-type classification changes, resistance changes, or new
  replay/MCP/browser schema.
- No claim of controlled legacy runtime, audiovisual, balance, browser, or
  performance parity beyond the current-Rust tests and hosted browser gate.

### 2.7 Evidence boundary

This slice proves the current-Rust Rocket Launcher ground-item rule, its
post-falloff threshold, deterministic selection, event ordering, replay
determinism, and stable boundary projections. It will not prove legacy feature
markers, terrain/content callbacks, delayed timing, projectile routing,
rocket-jump, controlled legacy runtime, audiovisual parity, balance, or human
play; those surfaces remain open or `NOT_RUN` in the roadmap.

### 2.8 Delivery evidence

- Independent determinism review of final head
  `49f1451f59ce883a84cd10d0e01d8e3793540572`: **PASS** by
  `/root/red_armor_review`; no severity-ranked defects or focused fix were
  required.
- Local workspace tests, clippy, formatting, version check, repository checks,
  and native/headless browser checks: **PASS**. The optional reference-capture
  preflight is `NOT_RUN` because its local manifest is unavailable.
- PR #448 hosted Repository checks and WASM browser checks: **PASS** in run
  `33653780858`. The protected-path Review policy check failed closed under the
  documented solo-maintainer `enforce_admins=false` exception; administrator
  merge was used after the review receipt was recorded.
- Merge checkpoint: `e902d71`; the temporary implementation branch was removed
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
