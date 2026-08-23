# ADR 0003 — Semantic Command Model

**Status:** Accepted

**Date:** 2026-08-18

---

## Context

DRL-Rust is designed to be operated by multiple kinds of clients: a human
player using keyboard input, scripted bots running headless tests, MCP-driven
AI agents, replay engines re-executing recorded sessions, and future
frontends that have not yet been designed.

Without a principled decision, each client type risks acquiring its own
privileged path to mutate simulation state — bypassing validation, leaking
game state it should not see, or producing non-reproducible outcomes.

---

## Decision

All clients — human input, scripted bots, MCP agents, replay engines, and
future frontends — interact with the simulation exclusively through the
`drl_protocol::Command` type submitted to the standard simulation API.

Concretely:

- `Command` is defined in `drl-protocol` and is the only authorized client
  mutation interface to `drl-core`'s `Game`.
- `Game::step(command)` is the single entry point for command-driven simulation
  advancement; simulation-owned RNG remains part of the explicit `Game` state.
- No client may modify `World`, `Actor`, `Map`, or other simulation state
  directly.
- MCP agents, bots, and replay execution all produce `Command` values and
  submit them through the same command path, making their gameplay authority
  equivalent to a human player's.
- Command validation (`CommandError`) is enforced uniformly for all callers.
- A rejected command is atomic: if `Game::step` returns `Err`, the complete
  simulation state, including RNG state, must equal its pre-command state.
- Replay execution reproduces a session only under explicitly compatible RNG,
  gameplay, content/ruleset, and generator semantics; a matching seed and
  command stream alone is not a cross-version compatibility promise.

The `Command` type is defined in `drl-protocol` so that it can be shared across
crates without creating a dependency on `drl-core`.

See the active steering notes on
[atomic command transactions](../steering/decisions/atomic-command-transactions.md)
and [replay semantics](../steering/decisions/replay-semantics-and-rng-stability.md).

---

## Consequences

- Current-semantics replay determinism has one auditable command path, but
  cross-version replay compatibility must be versioned and tested separately.
- MCP agents cannot bypass gameplay rules; they are subject to the same
  `CommandError` rejection paths and atomicity requirements as human input.
- Adding a new client type requires only a translation layer from the client's
  representation to `Command`, not a new simulation mutation interface.
- Omniscient debug access (`OmniscientObservation`, `game_get_dev_state`) is
  read-only and explicitly gated behind developer mode; it does not provide a
  mutation path.
- Command variants remain semantically meaningful player-facing actions rather
  than low-level implementation hooks.
