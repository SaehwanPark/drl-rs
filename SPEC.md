# Specification

Last reviewed: 2026-09-02
Current project version: `0.2.327`
Audited starting checkpoint: `main` at
`d8bf55c` (merged PR #442; replay verification documentation reconciled)
Delivery checkpoint: `main` merge commit `41de1e9` (PR #443)

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

## 2. Active implementation slice: M9 — Rocket Launcher direct-hit actor splash

Slice status: **delivered and verified** from the audited starting checkpoint
above.

### 2.1 Objective

Complete the ordinary Rocket Launcher direct-hit branch with the observed
legacy explosion payload: one successful hit schedules a typed delay-40,
radius-4, knockback-8 explosion and immediately resolves a bounded actor-only
fanout. The fanout uses one deterministic `6d6` Fire roll per clear blast cell,
applies the legacy distance falloff, de-duplicates actors, applies radial
integer `damage / 8` knockback before damage, and preserves normal death/drop
ordering.

This is a bounded vertical fidelity slice. The delay remains presentation
metadata; no pending effect queue is introduced. Terrain/cell mutation,
ground-item destruction, splash immunity, wall-impact projectile routing,
chain explosions, rocket-jump alternate fire, and audiovisual parity remain
separate work.

### 2.2 Audited starting point

At audited starting revision `d8bf55c` (version `0.2.326`):

- `ROCKET_LAUNCHER_BEHAVIOR` records one Rocket projectile and one Rocket ammo
  unit, while generic ranged execution owns legality, direct-hit damage, and
  clip accounting.
- `GameEvent` already carries typed delayed-explosion schedule variants and
  the core already has deterministic radius geometry, actor de-duplication,
  knockback, death-drop, and replay/browser/MCP projections for related
  weapons.
- The legacy item record (`items.lua:657-688`) supplies `6d6`, Fire, radius 4,
  and delay 40; the explosion loop (`dflevel.pas:991-1095`) supplies one roll
  per clear cell, distance falloff, actor de-duplication, and default
  knockback 8. The current Rust Rocket Launcher only performs direct damage.

### 2.3 Scope and ownership

- **Roadmap:** M9 vertical canonical-fidelity completion for the Rocket
  Launcher ordinary-fire explosion branch.
- **Primary owners:** `crates/drl-core/src/rocket_launcher.rs` owns the typed
  geometry, `6d6` roll, falloff, and knockback constants; `Game` owns the
  transactional fanout and event ordering; boundary crates only project the
  new schedule event.
- **Content registration:** the existing Rocket Launcher catalog entry and
  behavior profile remain the single source for item identity, projectile
  count, and ammo cost; no duplicate weapon table is introduced.
- **Project version:** implementation advances `VERSION` from `0.2.326` to
  `0.2.327`.
- **Replay/RNG:** gameplay semantics advance from `128` to `129`; RNG sampling
  remains version `1`, generator semantics remain version `2`, and the ruleset
  identity remains `drl-rs-ruleset-v1`. Accepted hits consume one ordered roll
  per eligible blast cell; rejected commands consume no RNG and preserve the
  exact `Game` snapshot.
- **Protocol/boundaries:** add one typed
  `RocketLauncherExplosionScheduled` event and update metrics, audio, render,
  MCP JSON, and browser projections without moving gameplay policy out of the
  core.

### 2.4 Review and branch contract

- The schedule event follows the direct `DamageApplied` event for each
  successful direct projectile and precedes all fanout events. Every eligible
  actor is processed once in center-then-clockwise-ring order; the active
  player is not self-safe in this bounded policy.
- The explosion considers only in-bounds cells with a clear ray from the
  impact center. It does not destroy ground items or mutate terrain/content.
- Before clip mutation or combat/splash RNG, validate every possible
  death-drop destination in the radius-4 fanout. A late validation error
  restores world, turn, and RNG through the existing core transaction guard.
- The core remains independent of filesystem, browser, audio, and MCP IO.

### 2.5 Acceptance criteria

- [x] `rocket_launcher.rs` exposes tested radius-4 geometry, `6d6` bounds,
  strict distance-falloff math, and integer `damage / 8` knockback.
- [x] A successful direct Rocket Launcher hit emits the typed schedule event,
  consumes the documented per-cell RNG sequence, fans out to each actor once,
  and preserves death/drop/game-over ordering.
- [x] Empty-clip, blocked-target, and impossible death-drop rejections are
  state-identical, including RNG; the fanout does not mutate ground items or
  terrain.
- [x] Core, replay, scenario, MCP JSON, audio/metrics, render, and
  BrowserSession parity tests pass, including a replay double-run and a
  browser vertical encounter.
- [x] Formatting, clippy, `sh scripts/check-repository.sh`, version transition,
  and an attributable independent determinism review pass on the final
  implementation commit.

### 2.6 Non-goals

- No rocket-jump command, homing/wall-impact projectile routing, delayed core
  queue, `EFCHAIN`, terrain/content mutation, ground-item destruction, or
  splash-immunity implementation.
- No broad legacy explosion parity claim; the bounded actor-only policy is an
  explicit Rust decision where unsupported projectile/cell state remains.
- No claim of controlled legacy runtime, audiovisual, balance, or human-play
  parity.

### 2.7 Evidence boundary

This slice proves the current-Rust actor-only Rocket Launcher explosion branch
and its replay/boundary projections. It does not prove projectile routing,
terrain/content behavior, delayed timing, controlled legacy runtime,
audiovisual parity, balance, or human play; those surfaces remain open or
`NOT_RUN` in the roadmap. The verified implementation commit is `5dfb210`, with
semantics-bound browser fixture correction `16a9836`; PR #443 merged them as
`41de1e9`. The independent read-only determinism review covers exact branch
head `16a9836` and returned `drl-determinism-review: PASS`. Local repository and
web checks pass, as do hosted `Repository checks` and `WASM browser checks`.
Hosted `Review policy` failed closed because the sole maintainer cannot create a
non-self approval; the documented live `enforce_admins=false` exception was
used for the merge.

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
