# Steering Decision — Atomic Command Transactions

**Status:** Active steering constraint; candidate for future ADR consolidation

**Date:** 2026-08-23

---

## Context

ADR 0001 defines `Game::step` as the single auditable path for explicit state
transitions. Project architecture also states that illegal or rejected commands
do not mutate world state or advance the simulation.

Current command implementations can perform mutation before all expected
validation has completed. Examples include consuming ammunition before an
out-of-range ranged attack is rejected and removing an inventory item before a
failed equipment-slot validation. Without an executable transaction invariant,
future command families can repeat the same class of defect.

A deterministic simulation must make failed actions particularly strict:
rejection cannot consume RNG, energy, inventory, counters, or hidden state even
when turn counters themselves remain unchanged.

---

## Decision

### 1. Rejection is state identity

For every command `C` and valid pre-state `G`:

```text
if G.step(C) returns Err(_)
then post_game == pre_game
```

Equality includes every simulation-owned field represented by `Game` equality,
including RNG state.

### 2. Prefer validate/prepare/commit

Command handling should be structured so expected user-facing errors occur
before mutation.

A preferred pattern is:

```text
Command
  -> validate and prepare against immutable state
  -> PreparedAction
  -> commit mutation and emit events
```

`PreparedAction` is an internal typed representation. It should contain the
resolved IDs, positions, costs, and policy choices required for execution so
commit has few or no expected rejection paths.

### 3. Interim rollback is permitted but not the target design

A bounded snapshot/rollback guard may be used as a short-term backstop while
individual command paths are refactored. Permanent unconditional full-state
cloning per command is discouraged because headless batch/cohort workloads make
that cost unnecessary when preparation can establish legality without mutation.

### 4. RNG is part of the transaction

Validation and rejected preparation must not consume `GameRng`. Random choices
needed by an accepted action occur at commit time or through a prepared
operation whose RNG mutation is committed atomically with the action.

### 5. Test the invariant generically

The core test suite must provide a reusable helper that clones a game, submits a
command expected to fail, and asserts exact game equality afterwards.

Every command family requires representative rejection tests, including paths
that occur after nontrivial lookup or validation.

---

## Consequences

- Multi-step inventory/equipment/attack actions require validation ordering to
  be explicit.
- `Game::step` and helper functions may become two-phase internally.
- Errors become trustworthy inputs to MCP legal-action probing and other cloned
  simulation checks.
- Replay and scenario diagnostics no longer risk observing silent state drift
  after a rejected command.
- New command-family PRs are incomplete until both success behavior and
  rejection atomicity are tested.
