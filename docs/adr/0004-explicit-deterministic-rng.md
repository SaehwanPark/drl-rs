# ADR 0004 — Explicit Deterministic RNG

**Status:** Accepted

**Date:** 2026-08-18

---

## Context

DRL-Rust's core value proposition as a testing platform depends on
reproducibility: given the same seed and the same sequence of commands, two
independent runs must produce bit-identical state at every step.

This guarantee is impossible to maintain if any part of the simulation uses
ambient, thread-local, or OS-seeded randomness — sources that are invisible
in the game's state snapshot and cannot be replicated from a recorded seed.

The legacy Pascal implementation uses its own global RNG. DRL-Rust must not
reproduce that pattern, but must achieve the same net result: all randomness
in the simulation is fully determined by the initial seed.

---

## Decision

All gameplay randomness in DRL-Rust flows through a single explicit `GameRng`
value that is:

1. **Owned and threaded explicitly** — `GameRng` is passed as an explicit
   parameter to every function that needs randomness. It is never stored in a
   global, thread-local, or static variable.

2. **Seeded from a known initial seed** — sessions are initialized with an
   explicit `u64` seed. The same seed always produces the same `GameRng`
   state sequence, and therefore the same game outcomes for the same commands.

3. **Based on a documented algorithm** — `GameRng` wraps SplitMix64 (for
   seed mixing) and Xoshiro256++ (for output). Both algorithms are
   deterministic, fast, and have well-understood statistical properties.

4. **Isolated from presentation** — rendering frame timing, audio callbacks,
   UI input handling, and network I/O must not consume from or influence
   `GameRng`.

5. **Prohibited from ambient sources** — `rand::thread_rng()`, `OsRng`,
   `SystemTime`-based seeding, and any other non-explicit randomness source
   are banned from `drl-core` and `drl-protocol`. This is enforced by the
   crate dependency graph (those crates are not declared as dependencies).

### Lua consideration

When Lua scripting is introduced (Milestone 3), Lua-driven behavior that
requires randomness must receive a deterministic seed or generator derived
from `GameRng`. Lua scripts must not seed their own RNG independently.

---

## Consequences

- Any session is fully reproducible: seed + command stream → bit-identical
  outcomes, regardless of host machine, OS, Rust version (within the defined
  toolchain), or time of execution.
- Batch simulation, scenario testing, and replay regression suites are
  reliable by construction.
- Debugging probabilistic issues is tractable: a failing seed can be replayed
  exactly.
- New gameplay features that require randomness must thread `GameRng` through
  their call chain rather than creating local sources.
- Saving and loading game state must include the current `GameRng` state to
  preserve reproducibility from mid-session save points.
