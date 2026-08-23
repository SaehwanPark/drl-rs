# ADR 0001 — Project Architecture Principles

**Status:** Accepted

**Date:** 2026-08-18

---

## Context

DRL-Rust is a ground-up Rust reimplementation of Doom the Roguelike. The
legacy Pascal/Lua codebase is the canonical source of intended game rules and
behavioral character, but its internal architecture — broad mutable global
state, cross-layer coupling, responsibility-heavy classes, string/byte-oriented
domain representation — is not a model to reproduce mechanically.

The project needs a coherent set of architecture principles that will guide
design decisions across many milestones, multiple contributors, and an
extended implementation timeline.

Current near-term applications of these principles are tracked in
[`docs/steering/`](../steering/README.md). Steering notes may constrain work but
do not silently supersede this accepted ADR.

---

## Decision

DRL-Rust adopts the following architectural principles, applied in order of
precedence:

### 1. Functional core, imperative shell

The deterministic simulation kernel (`drl-core`) is a pure function in the
architectural sense: given current simulation state and a command, it produces
the deterministic next state and ordered events. All I/O, rendering, audio,
persistence, and network communication live outside this boundary.

### 2. Typed domain model

Every significant concept in the game is represented by a named Rust type.
Primitive obsession (raw integers, bare strings, boolean flags used as
discriminants) is replaced by enums, newtypes, and structs that make invalid
states difficult or impossible to represent.

### 3. Algebraic data types over inheritance

Variant behavior is modeled with `enum` and explicit composition rather than
trait hierarchies or runtime callback polymorphism by default. This keeps
exhaustiveness checking, serialization, and determinism tractable.

### 4. Explicit state transitions

State changes are the result of processing commands through a single,
auditable kernel path (`Game::step`). Implicit mutations, side-channeled state,
and action-at-a-distance are avoided. Expected command rejection is atomic:
failed commands must not leave partial simulation mutation.

### 5. No ambient or global state

No global mutable state. No thread-local RNG. No ambient singletons. All
inputs to a computation are explicit or owned by the explicit simulation state.

### 6. Clean architectural boundaries

Crate dependencies enforce layering. `drl-protocol` owns stable semantic
contracts that must cross boundaries: commands, observations, events, stable
IDs/newtypes, and replay/wire representations. `drl-core` owns gameplay rules,
balance, behavior definitions, and simulation policy. Presentation and
transport crates depend outward from those contracts.

A gameplay concept does not become protocol-owned merely because one of its
stable identifiers or views crosses a protocol boundary.

### 7. Testability as first-class design

Every domain rule should be independently testable. The headless simulation
core is designed to run scenarios, bots, and replays without rendering or I/O.
Randomness is explicit and seedable. Consequential invariants should be
executable tests rather than documentation-only promises.

### 8. Avoid premature abstraction

Do not introduce ECS frameworks, trait towers, generic callback buses, or
plugin architectures until the gameplay domain is well understood and the
abstraction provides clear, concrete value. Start concrete; generalize from
verified evidence and difficult representative cases.

---

## Consequences

- `drl-core` may not depend on `drl-mcp`, `drl-render`, `drl-audio`, `drl-script`,
  or any crate with I/O or rendering capability.
- All simulation results are expressed as `GameEvent` streams; presentation
  reacts to events rather than polling or mutating simulation state directly.
- Stable concepts needed across crate/client boundaries are introduced in
  `drl-protocol`; gameplay balance and behavior policy remain in `drl-core` or
  a future domain/content boundary with the same dependency direction.
- Rejected-command state identity, including RNG state, is part of the explicit
  state-transition principle. See
  [`docs/steering/decisions/atomic-command-transactions.md`](../steering/decisions/atomic-command-transactions.md).
- Content abstraction work should preserve compiler exhaustiveness while
  avoiding duplicated manual registries. See
  [`docs/steering/decisions/content-catalog-and-typed-behavior-model.md`](../steering/decisions/content-catalog-and-typed-behavior-model.md).
- Architecture changes require `ARCHITECTURE.md` to be updated from verified
  evidence, not speculation.
