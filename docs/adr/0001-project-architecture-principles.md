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

---

## Decision

DRL-Rust adopts the following architectural principles, applied in order of
precedence:

### 1. Functional core, imperative shell

The deterministic simulation kernel (`drl-core`) is a pure function: given
current state and a command, it produces new state and a list of events. All
I/O, rendering, audio, persistence, and network communication live outside
this boundary.

### 2. Typed domain model

Every significant concept in the game is represented by a named Rust type.
Primitive obsession (raw integers, bare strings, boolean flags used as
discriminants) is replaced by enums, newtypes, and structs that make invalid
states difficult or impossible to represent.

### 3. Algebraic data types over inheritance

Variant behavior is modeled with `enum`, not trait hierarchies or runtime
polymorphism. This keeps exhaustiveness checking, serialization, and
determinism tractable.

### 4. Explicit state transitions

State changes are the result of processing commands through a single,
auditable kernel path (`Game::step`). Implicit mutations, side-channeled
state, and action-at-a-distance are avoided.

### 5. No ambient or global state

No global mutable state. No thread-local RNG. No ambient singletons. All
inputs to a computation are explicit parameters.

### 6. Clean architectural boundaries

Crate dependencies enforce the layering: `drl-protocol` (types only) →
`drl-core` (simulation) → `drl-mcp` / `drl-app` (presentation and transport).
Boundary violations are caught by automated tests.

### 7. Testability as first-class design

Every domain rule should be independently testable. The headless simulation
core is designed to run scenarios, bots, and replays without any rendering or
I/O. Randomness is explicit and seedable.

### 8. Avoid premature abstraction

Do not introduce ECS frameworks, trait towers, or plugin architectures until
the gameplay domain is well understood and the abstraction provides clear,
concrete value. Start concrete; generalize from verified evidence.

---

## Consequences

- `drl-core` may not depend on `drl-mcp`, `drl-render`, `drl-audio`, `drl-script`,
  or any crate with I/O or rendering capability.
- All simulation results are expressed as `GameEvent` streams; presentation
  reacts to events rather than polling or mutating simulation state directly.
- Contributors must introduce new domain concepts as types in `drl-protocol`
  before implementing behavior in `drl-core`.
- Architecture changes require `ARCHITECTURE.md` to be updated from verified
  evidence, not speculation.
