# Steering Decision — Replay Semantics and RNG Stability

**Status:** Active steering constraint; candidate for future ADR consolidation

**Date:** 2026-08-23

---

## Context

DRL-Rust treats determinism and replayability as core product capabilities.
`GameRng` is explicit and replay execution can verify that the same current
implementation produces the same result twice.

Two separate compatibility concerns remain:

1. **RNG sampling semantics.** A change to bounded sampling or probability
   conversion changes downstream deterministic histories even when the PRNG
   algorithm and initial seed are unchanged.
2. **Replay interpretation semantics.** A replay wire envelope can remain
   syntactically V1 while current item definitions, combat rules, generation,
   or other gameplay policies change. Reconstructing old spawns through current
   definitions can therefore reinterpret an old replay silently.

The current modulo-based bounded integer sampler also has modulo bias when the
requested span does not divide the generator's integer domain evenly.

---

## Decision

### 1. Use unbiased bounded integer sampling

`GameRng` shall use a documented unbiased bounded-integer algorithm, such as
rejection sampling or an equivalent well-understood method.

The chosen algorithm is part of deterministic simulation semantics.

### 2. Probability sampling is explicit

Boolean/probability sampling shall use a documented integer-domain contract.
Floating-point input may be accepted at an outer API if useful, but conversion
to deterministic integer thresholds must be explicit, bounded, and tested.
Core game rules should prefer rational/integer probabilities when practical.

### 3. Golden RNG vectors define the supported stream

Tests shall pin:

- raw PRNG output vectors;
- representative bounded-range samples;
- representative boolean/probability samples;
- shuffle behavior where shuffle order is simulation-visible.

Intentional changes require an explicit semantics-version decision.

### 4. Replay schema and gameplay semantics are separately versioned

A replay shall distinguish at least:

- wire/schema version;
- engine/gameplay semantics version;
- ruleset/content semantics identifier.

Procedural-generation compatibility must either be included in the ruleset
identifier or recorded separately as a generator-semantics version.

### 5. Incompatible replays fail explicitly

Until a migration mechanism exists, a replay whose semantics identifier is not
supported by the running engine is rejected with a diagnostic error.

The engine must not silently load an old replay and reinterpret its item spawns,
monster defaults, or generator behavior through current definitions.

### 6. Determinism claims use precise language

- **repeatable-current**: same revision/semantics + same replay/seed produces the
  same result;
- **cross-version-compatible**: explicitly supported semantics compatibility or
  migration exists;
- **legacy-parity**: controlled comparison with the canonical legacy behavior
  exists.

These claims are independent.

---

## Consequences

- Correcting bounded sampling may intentionally invalidate current development
  replay streams; doing so before 1.0 is preferred to freezing known bias.
- Replay metadata becomes slightly larger and validation stricter.
- Content/balance changes can declare whether they preserve or advance gameplay
  semantics instead of accidentally changing historical interpretation.
- Cohort and agent evaluations gain a stronger reproducibility contract.
