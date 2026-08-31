# Specification

Last reviewed: 2026-08-31
Current project version: `0.2.322`
Audited starting checkpoint: `main` at
`34f6df38652efaf92f2463257802c0d53dc05094` (merged PR #431; M9 delivered)

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

## 2. Active implementation slice: M1/M11 — Gate C transaction baseline

### 2.1 Objective

Measure the cost of the existing command transaction boundary and remove one
verified redundant outer snapshot. The slice must produce a repeatable,
machine-readable accepted/rejected core benchmark with allocation counters,
make transaction ownership explicit at core and boundary layers, and retain the
core rollback backstop until a measured prepare/commit migration is justified.

This is a Gate C transaction-ownership slice. It does not change gameplay
semantics, replay identities, or the fair observation boundary.

### 2.2 Audited starting point

At audited starting revision `34f6df3` (version `0.2.321`):

- `Game::step` clones the complete `GameState` for every command and restores
  it on rejection; exact `Game` equality, including RNG state, is an enduring
  correctness invariant.
- `BrowserSession::submit` takes a second full `Game` snapshot even though the
  core already owns rollback; the browser also owns observation, presentation,
  and successful-command history bookkeeping.
- `McpSession::legal_actions` clones once per candidate to probe core legality
  from a fair observation. Those clones are admission probes, not rollback.
- Inventory insertion retains a separate local staging clone. Removing the
  core snapshot wholesale is unsafe while late fallible handlers remain.
- No benchmark target or allocation measurement exists yet.

### 2.3 Scope and ownership

- **Steering gate:** Gate C — the rollback backstop has an exit budget.
- **Primary owner:** `drl-core::Game::step` owns authoritative simulation
  rollback and RNG atomicity; boundary layers own only their respective
  bookkeeping or admission policy.
- **Benchmark owner:** `crates/drl-core/benches/transaction.rs` is a
  benchmark-only executable with a counting allocator; it must not affect
  normal library allocation behavior.
- **Project version:** implementation advances `VERSION` from `0.2.321` to
  `0.2.322`.
- **Gameplay/replay semantics:** gameplay, replay wire/schema, RNG-sampling,
  generator, ruleset, and snapshot V3 identities remain unchanged.

### 2.4 Benchmark contract

- The optimized benchmark uses fixed seed `42`, a `20×15` arena, setup outside
  the timed region, `std::hint::black_box`, and bounded defaults of 10,000
  warm-up operations plus five measured samples of 100,000 operations.
- It measures accepted `Command::Wait` and rejected blocked movement from
  `(1, 1)`, and reports per-sample and median elapsed time, operations/sec,
  allocation/deallocation calls, and allocated/deallocated bytes.
- Output is machine-readable and includes schema, revision, host/toolchain,
  profile, fixture, command label, iteration counts, and explicit ownership
  labels. Timings and allocation counts are same-host baseline evidence, not
  universal performance targets.

### 2.5 Transaction and boundary contract

- Every `Game::step` retains one complete `GameState` snapshot per command
  until a future prepare/commit slice proves equivalent rejection identity.
- `BrowserSession::submit` relies on core rollback and takes zero additional
  simulation snapshots; it still owns presentation observations, effects,
  error text, and successful-command history.
- `McpSession::legal_actions` retains one cloned core probe per candidate as
  fair-observation admission validation; `McpSession::step` owns metrics,
  replay, and terminal bookkeeping after the core accepts the command.
- Inventory staging clones remain local atomicity guards and are not counted as
  outer transaction ownership.

### 2.6 Acceptance criteria

- [ ] A benchmark-only target runs with fixed fixtures and emits repeatable,
  machine-readable accepted/rejected throughput and allocation measurements.
- [ ] Core rejected-command equality and RNG preservation remain covered by the
  existing command-atomicity matrix.
- [ ] Removing the BrowserSession outer snapshot preserves rejected-session
  equality and accepted history/presentation behavior.
- [ ] MCP candidate clones are documented as admission probes rather than
  rollback snapshots, with no duplicate legality policy introduced.
- [ ] The retained core snapshot has an explicit one-snapshot-per-command
  budget and a documented prepare/commit exit condition.
- [ ] Gameplay/replay/snapshot identities remain unchanged and no benchmark
  dependency is added to production crates.
- [ ] Local format, check, test, clippy, benchmark, repository, and version
  checks pass; relevant hosted checks pass for the reviewed merge revision.
- [ ] Roadmap, README, architecture, changelog, browser guide, and steering
  records are reconciled from verified evidence; unavailable GPU, human,
  audiovisual, performance, and legacy captures remain `NOT_RUN`.

### 2.7 Non-goals

- No wholesale removal of the core rollback snapshot or broad prepare/commit
  refactor.
- No MCP legal-action redesign, observation-policy duplication, or new gameplay
  semantics.
- No universal performance target or cross-host ranking from timing output.
- No browser/GPU, human, audiovisual, or controlled legacy-runtime parity claim.

### 2.8 Evidence boundary

The benchmark proves current-Rust transaction cost and allocation behavior only
for its declared fixed fixture and host. Exact rejection tests prove state
identity; the benchmark does not replace them. Browser presentation cost, MCP
probe cost, inventory staging, controlled runtime comparison, and broad
performance/generalization remain separately labeled `NOT_RUN` or open.

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
