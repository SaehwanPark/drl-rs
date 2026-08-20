# ADR 0005 — Lua Transitional Strategy

**Status:** Accepted (implementation pending — Milestone 3)

**Date:** 2026-08-18

---

## Context

The legacy DRL implementation uses Lua extensively for content definition:
actor prototypes, item tables, AI behavior policies, level generation scripts,
and more. A large body of existing Lua content encodes game rules that DRL-Rust
must preserve semantically.

Rewriting all Lua content in pure Rust immediately would be high-risk: it would
require deep understanding of every script before the Rust simulation is mature
enough to validate the results. At the same time, simply embedding a Lua
runtime with unrestricted access to Rust internals would recreate the
coupling problems of the legacy architecture.

A principled boundary design is needed before any Lua integration work begins.

---

## Decision

Lua is a **transitional content and behavior layer**, not a core engine
mechanism. The integration strategy is:

### 1. Rust owns all simulation invariants

Rust code in `drl-core` defines and enforces all game rules that affect
correctness: damage calculation bounds, inventory capacity, actor liveness,
map validity, command legality, and replay determinism. Lua cannot violate
these invariants.

### 2. Lua operates through a narrow typed boundary

Lua may only interact with the simulation through a defined, explicitly typed
API surface — a set of allowed queries (read operations) and allowed commands
(actions submitted through the standard `Command` model). Lua does not receive
raw mutable references to `World`, `Actor`, `Map`, or `GameRng`.

### 3. Lua errors are isolated

A Lua script error must not corrupt simulation state or crash the process. Lua
execution happens inside a defined scope with controlled error propagation. The
simulation falls back to a safe default behavior on Lua failure.

### 4. Determinism is preserved

Any Lua behavior that uses randomness must use a generator derived from the
simulation's explicit `GameRng` seed. Lua scripts must not independently seed
their own generators.

### 5. The boundary is explicit and stable

The Lua API surface is documented. Lua scripts must not rely on internal Rust
struct field names, memory layout, or implementation details. The boundary is
designed to be stable enough that content scripts do not need to change when
internal Rust implementation evolves.

### 6. Long-term direction: optional, not foundational

As DRL-Rust matures, gameplay systems may migrate from Lua into native Rust
with no behavioral loss. Lua should become an optional extension mechanism
rather than a required runtime for core gameplay.

---

## Consequences

- `drl-script` remains a placeholder crate until Milestone 3 begins.
- When Lua integration is implemented, `drl-core` must not gain a direct
  dependency on any Lua runtime crate. The integration layer lives in
  `drl-script` or a dedicated boundary crate.
- Legacy Lua content is used as a behavioral reference but not copied as-is.
  Scripts are adapted to the DRL-Rust Lua API contract.
- Content authors writing Lua for DRL-Rust target the documented DRL-Rust
  API, not the legacy Pascal-internal Lua globals.
