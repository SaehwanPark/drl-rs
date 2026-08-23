# ADR 0004 — Explicit Deterministic RNG

**Status:** Accepted

**Date:** 2026-08-18

---

## Context

DRL-Rust's core value proposition as a testing platform depends on
reproducibility: under the same declared simulation semantics, the same initial
state/seed and command sequence must produce bit-identical state transitions.

This guarantee is impossible if any simulation path uses ambient, thread-local,
or OS-seeded randomness. It also depends on the sampling algorithms layered on
top of the raw PRNG: changing bounded-range reduction or probability conversion
changes deterministic histories even if the underlying generator and seed stay
the same.

---

## Decision

All gameplay randomness in DRL-Rust flows through a single explicit `GameRng`
value that is:

1. **Owned explicitly** — `GameRng` is simulation state and is never global,
   thread-local, static, or presentation-owned.

2. **Seeded from a known initial seed** — sessions are initialized with an
   explicit `u64` seed.

3. **Based on documented algorithms** — `GameRng` uses SplitMix64 for seed
   mixing and Xoshiro256++ for raw output.

4. **Sampled through documented deterministic contracts** — bounded integers
   use an unbiased algorithm rather than modulo reduction; boolean/probability
   conversion is explicitly defined. Raw output and representative derived
   samples are pinned by golden vectors.

5. **Isolated from presentation** — rendering frame timing, audio callbacks,
   UI input handling, and network I/O must not consume from or influence
   `GameRng`.

6. **Prohibited from ambient sources** — `rand::thread_rng()`, `OsRng`,
   `SystemTime`-based seeding, and other non-explicit randomness sources are
   banned from simulation authority.

7. **Versioned when replay-visible semantics change** — an intentional change
   to PRNG or sampling semantics must advance the appropriate gameplay/RNG
   semantics contract rather than silently reinterpret historical replay data.

Runtime Lua is not part of the architecture; ADR 0008 supersedes the earlier
transitional scripting plan.

The near-term migration contract is detailed in
[`docs/steering/decisions/replay-semantics-and-rng-stability.md`](../steering/decisions/replay-semantics-and-rng-stability.md).

---

## Consequences

- A failing seed can be reproduced exactly under the same declared semantics.
- Batch simulation, scenarios, and replay regression suites have an explicit
  RNG contract rather than relying on implementation accident.
- New gameplay features that require randomness must use `GameRng` and preserve
  command-rejection atomicity; a rejected action must not consume RNG.
- Saving/loading game state must include current RNG state.
- Cross-version reproducibility is an explicit compatibility property, not an
  automatic consequence of using the same seed and command stream.
