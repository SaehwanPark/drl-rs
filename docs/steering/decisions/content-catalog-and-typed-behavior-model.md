# Steering Decision — Authoritative Content Catalog and Typed Behavior Model

**Status:** Active steering constraint; candidate for future ADR consolidation

**Date:** 2026-08-23

---

## Context

DRL-Rust deliberately rejects a literal reproduction of the legacy Pascal/Lua
architecture. ADR 0008 further establishes that Lua is a build-time research
and conversion input, not a runtime dependency.

The current typed migration approach successfully provides compiler-checked
spawn/archetype coverage, but ordinary content additions increasingly require
manual edits across protocol enums, display strings, replay codecs, core
factories/definitions, validation, assets, documentation, and other exhaustive
registries.

Meanwhile, many important legacy mechanics are expressed through callback-heavy
Lua behavior: equip/unequip effects, hit/kill hooks, alternate fire/reload,
recharge, item sets, periodic effects, target-dependent behavior, and special
resource costs.

If scalar migration continues without a behavioral architecture, the project
risks replacing the old callback system with a large collection of Rust
`match` statements and one-off special cases distributed across subsystems.

---

## Decision

### 1. One authoritative catalog for routine content identity

Routine immutable content identity and metadata shall originate from one
compile-time authoritative catalog or an equivalent mechanically single-source
representation.

The catalog may generate or derive:

- stable archetype/spawn identifiers;
- canonical string representations;
- routine replay names;
- validation coverage lists;
- presentation/asset identity mappings where appropriate;
- static definitions and lookup tables.

The implementation may use ordinary Rust constants and `macro_rules!`; a
procedural macro or external code generator is not required unless simpler
approaches prove insufficient.

### 2. Exhaustiveness remains a feature

This decision does not replace strong types with string-keyed registries or
runtime reflection. The compiler should continue to detect missing cases at
meaningful semantic boundaries.

The goal is to remove accidental duplication, not semantic redundancy that
expresses a distinct invariant.

### 3. Behavior is typed and explicit

Legacy callback behavior shall be represented through a bounded Rust vocabulary
of typed concepts rather than a generic callback bus.

The model should support, as evidence requires:

- passive modifiers and resistances;
- equip/unequip effects;
- set membership and set activation;
- attack/hit/kill effects;
- alternate fire/reload/use actions;
- recharge/periodic effects;
- explicit HP/energy/ammo/status costs;
- deterministic target-selection policies;
- dedicated typed state machines for genuinely exceptional behavior.

### 4. Catalog data does not execute arbitrary code

The authoritative catalog describes identity, immutable data, and references to
known typed behavior. It must not become a dynamic script system hidden inside
Rust data.

Behavior execution remains owned by deterministic core systems and is covered
by ordinary Rust tests.

### 5. Validate the model on difficult legacy cases

The architecture is accepted only after it represents several callback-heavy
legacy examples with materially different mechanisms. Scalar-only items are not
sufficient proof.

### 6. Protocol and gameplay policy are separate concerns

Stable semantic IDs and wire/view contracts may live in `drl-protocol`.
Gameplay balance values and behavior definitions should live in `drl-core` or a
future domain/content boundary whose dependency direction preserves the pure
core.

A type is not protocol-owned merely because it is exposed through a protocol.

---

## Consequences

- Adding a conventional item should require one catalog entry plus genuinely
  unique behavior/presentation work, rather than synchronized manual registry
  edits across many files.
- Some current enums/lookups may become generated from a shared macro/catalog.
- Existing definition tables can migrate incrementally; no big-bang rewrite is
  required.
- Behavior-complete status becomes distinct from static-definition coverage.
- Runtime Lua remains absent.
- `drl-script` should be removed or renamed if it remains a placeholder whose
  name implies a runtime scripting role that the architecture no longer has.
