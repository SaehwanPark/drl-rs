# Near-Term Development Steering

Last reviewed: 2026-08-23
Baseline project version: `0.2.88`

## Purpose

This document captures the current priority order for DRL-Rust after the first
large wave of deterministic-engine, browser, tooling, and typed-content work.
It does not replace the roadmap or the single active slice in `SPEC.md`.
Instead, milestone owners use it to decide which candidate work is eligible to
become the next bounded slice.

## Current diagnosis

The project foundation is strong: deterministic simulation, semantic commands,
replay/scenario infrastructure, WebGPU/WASM presentation, MCP tooling, content
provenance, CI, and release-rights gates already exist.

The main project risk has shifted. Infrastructure and scalar content coverage
are now advancing faster than canonical DRL behavioral fidelity. The legacy
game expresses important mechanics through a broad Pascal/Lua hook system;
porting scalar fields without choosing an explicit Rust behavioral model would
accumulate migration debt and encourage a new form of callback/special-case
sprawl.

A second risk is correctness drift inside the deterministic core. A documented
transactional-command invariant is only valuable if every rejected command is
actually state-identical. That invariant must be executable, not aspirational.

## Priority order

Until the gates below are closed, select new work in this order:

1. **Simulation correctness invariants**
   - rejected-command atomicity;
   - no mutation or RNG consumption on errors;
   - property/invariant tests across the command surface.
2. **Deterministic semantics stability**
   - unbiased bounded RNG sampling;
   - golden vectors;
   - replay semantics/ruleset versioning.
3. **Content-model scalability**
   - one authoritative catalog for routine item/content registration;
   - reduced manual fan-out while retaining compiler exhaustiveness.
4. **Typed legacy behavior model**
   - explicit effects/actions/state machines instead of generic callback
     registries;
   - validated on difficult legacy examples.
5. **Protocol/domain boundary cleanup**
   - protocol owns stable contracts and IDs;
   - core/domain owns gameplay balance and implementation policy.
6. **Vertical canonical fidelity**
   - bounded end-to-end slices with mechanics, behavior, replay/scenario tests,
     and presentation.
7. **Resume content breadth**
   - only after the preceding architecture supports behavior-complete migration.
8. **Further platform/tooling expansion**
   - only when it directly enables a fidelity or release requirement.

## Development stop gates

### Gate A — Rejected commands are atomic

Do not add new simulation command families while known mutation-before-error
paths remain. `Err` must imply exact pre/post game equality.

### Gate B — Determinism semantics are explicit

Do not declare replay stability across releases until wire schema, gameplay
semantics, content/ruleset version, and RNG sampling behavior are explicitly
bound.

### Gate C — Content registration is not shotgun surgery

Do not normalize 10+ routine file edits for each ordinary item family as the
long-term workflow. Compiler exhaustiveness should remain, but routine
projections should come from one authoritative catalog.

### Gate D — Behavior model passes hard cases

Do not resume mass migration of uniques/exotics/special armor after merely
copying scalar fields. First prove the Rust behavior model against several
legacy callback-heavy cases.

### Gate E — Fidelity claims remain evidence-bounded

No source-level similarity, matching name, copied scalar, or current-Rust test
is sufficient to claim canonical runtime parity. Runtime/capture claims require
the controlled reference environment or remain `NOT_RUN`/`INCONCLUSIVE`.

## How to choose a slice

A good slice:

- closes one named gate or validates one architectural decision;
- is narrow enough to review independently;
- has observable acceptance criteria before implementation;
- uses pinned legacy source/runtime evidence when behavior depends on legacy;
- leaves canonical documents more truthful and simpler than before.

A poor slice at this stage:

- adds several scalar-only item families while their callbacks remain deferred;
- creates another cross-cutting registry to accommodate one new archetype;
- adds an MCP/browser/release feature unrelated to current fidelity goals;
- introduces a general plugin/event framework before stress cases demonstrate
  its need;
- claims parity from source inspection alone.

## Preferred architecture shape

```text
semantic Command
      |
      v
validate / prepare  -----> rejected: no state change
      |
      v
PreparedAction
      |
      v
commit deterministic mutation
      |
      +--> GameEvent stream
      +--> next Game state

legacy Pascal/Lua evidence
      |
      v
build-time evidence / catalog
      |
      +--> immutable scalar definitions
      +--> typed behavior specs
      +--> explicit unresolved gaps

no runtime Lua
```

The catalog should describe content. Behavior should remain explicit enough to
review and test. Special cases may use dedicated typed state machines where
composition would be less clear.

## Progress language

Use separate labels internally and in roadmap notes for:

- **definition-covered** — scalar/static metadata migrated;
- **behavior-covered** — relevant runtime mechanics implemented and tested;
- **legacy-compared** — controlled comparison against canonical runtime/evidence
  completed within stated tolerance;
- **presentation-compared** — controlled visual/audio comparison completed.

A definition-covered item is not behavior-complete by implication.

## Architectural hygiene

- Split large modules when they contain distinct reasons to change, not to meet
  arbitrary line counts.
- Prefer modules over additional crates unless an enforced dependency boundary
  is valuable.
- Keep `drl-core` free of ambient state and platform dependencies.
- Keep `drl-protocol` focused on stable semantic contracts rather than using it
  as the default home for gameplay balance.
- Remove or rename placeholder boundaries whose names conflict with accepted
  architecture, such as a runtime-sounding scripting crate if it remains only a
  build-time conversion boundary.

## Rights/provenance steering

The release-rights inventory should explicitly track legacy creative text and
other expression copied into Rust-owned definitions, separately from numeric
mechanics and graphics assets. This is an evidence/clearance question, not a
legal conclusion. Unknown status remains explicit and does not silently inherit
the repository's project-code license.
