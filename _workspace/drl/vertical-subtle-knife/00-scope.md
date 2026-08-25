# Vertical Subtle Knife Encounter — Scope

## Objective

Exercise the already-delivered Subtle Knife transition as one bounded vertical
slice: declarative scenario construction, deterministic replay, core event
ordering, and the browser presentation boundary.

## In scope

- A fixed scenario with a Subtle Knife-equipped player, one visible target, and
  one occluded target.
- The existing `Command::Invoke` contract, deterministic target selection,
  internal damage, player cost, and accepted-turn ordering.
- ScenarioRunner and ReplayEngine evidence for the same command sequence.
- BrowserSession presentation parity against direct `Game::step`, including
  observations, events, effects, and rendered scene derivation.

## Explicit non-goals

- New Subtle Knife balance or gameplay semantics.
- New protocol fields, RNG behavior, AI policy, armor/resistance parity, or
  death-drop policy beyond the existing typed contract.
- Browser audio, WebGPU, legacy runtime, and audiovisual parity; these remain
  `NOT_RUN` unless an enabled environment supplies direct evidence.

## Version and semantics boundary

This slice adds end-to-end evidence and a test-only browser construction path;
it does not change the accepted simulation transition or gameplay semantics
version. The project version advances for the code/test delivery, while replay
gameplay semantics remain the current compatible value.
