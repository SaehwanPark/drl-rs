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

- `Command` is defined in `drl-protocol` and is the only authorized mutation
  interface to `drl-core`'s `Game`.
- `Game::step(command, rng)` is the single entry point for advancing
  simulation state.
- No client may modify `World`, `Actor`, `Map`, or any simulation state
  directly.
- MCP agents, bots, and the replay engine all produce `Command` values and
  submit them through `Game::step`, making their execution semantically
  identical to a human player's.
- Command validation (`CommandError`) is enforced uniformly for all callers.
- The replay engine can reproduce any session by re-feeding the recorded
  `Command` stream with the same seed.

The `Command` type is defined in `drl-protocol` so that it can be shared
across crates without creating a dependency on `drl-core`.

---

## Consequences

- Replay determinism is structurally guaranteed: any session reproducible from
  its seed + command stream, regardless of who submitted the commands.
- MCP agents cannot cheat or bypass rules — they are subject to the same
  `CommandError` rejection paths as human input.
- Adding a new client type (e.g., a web frontend, a new bot policy) requires
  no changes to the simulation kernel — only a translation layer from the
  client's representation to `Command`.
- Omniscient debug access (`OmniscientObservation`, `game_get_dev_state`)
  is read-only and explicitly gated behind developer mode; it does not provide
  a mutation path.
- Command variants must be kept semantically meaningful (player-facing actions)
  rather than low-level implementation hooks.
