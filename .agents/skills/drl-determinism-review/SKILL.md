---
name: drl-determinism-review
description: Independently review a bounded DRL-Rust change for reproducibility, semantic-boundary integrity, evidence coherence, and claims that exceed implemented capability.
---

# DRL Determinism Review

## When to Use

- Use this skill after a consequential simulation, behavior, replay,
  test-play, observation, content, or platform-boundary change.
- Use it before final acceptance when an independent read-focused pass can
  reduce false confidence.
- Do not require it for trivial prose or mechanical changes with no
  consequential boundary.

## Required Inputs

- the original request and active `SPEC.md` slice;
- the relevant roadmap outcomes and architecture invariants;
- the produced diff, implementation, and tests;
- legacy evidence when behavior fidelity is claimed;
- test-play plans, run artifacts, and verification results when present.

Review only implemented surfaces. A planned feature is not a defect unless the
active slice requires it.

## Review Boundaries

Compare both sides of every applicable boundary:

- specification outcomes against test assertions;
- semantic commands against actual state transitions;
- player observations against hidden world state;
- semantic events against presentation or platform effects;
- initial state, seed, and command stream against reproduced outcomes;
- Lua or content requests against Rust-owned world authority;
- direct simulation calls against replay, bot, frontend, or MCP clients;
- local and remote completion claims against exact verification evidence.

Also inspect for ambient randomness, unstable iteration, wall-clock decisions,
filesystem or operating-system dependencies in the simulation, presentation
timing that changes gameplay, unbounded episodes, unstable identifiers in
canonical output, and retries that conceal nondeterminism.

## Workflow

1. Restate the slice's observable outcomes, non-goals, and acceptance evidence.
2. Identify the consequential boundaries touched by the change.
3. Read producer and consumer sides of each boundary together.
4. Trace randomness, time, iteration order, I/O, and state mutation from input
   to semantic outcome.
5. Compare each completion claim with the exact test, run, inspection, or
   remote result that supports it.
6. Attempt the smallest relevant reproduction or targeted check when the
   review environment permits it.
7. Report concrete findings with impact, evidence, and the smallest safe fix
   path.
8. Return one disposition:
   - `pass`: no blocking mismatch remains;
   - `fix`: bounded corrections can satisfy the existing slice;
   - `blocked`: missing evidence, contradictory sources, scope conflict, or an
     architectural decision prevents safe acceptance.

Remain read-focused during the review pass. The milestone owner owns fixes and
may request one focused re-review.

## Outputs

For coordinated work, write or return content shaped as
`_workspace/drl/{milestone}-{slice}/03-review.md` with:

- run identifier, owner and role;
- input and output repository state;
- predecessor artifact and revision, normally `02-test-plan.md` or the nearest
  prior artifact;
- review disposition;
- slice and input revision;
- boundaries inspected;
- blocking findings;
- non-blocking risks;
- checks or reproductions run;
- unverified surfaces;
- required fix or decision;
- evidence supporting a pass.

Order findings by impact. Distinguish confirmed defects from risks and
unverified areas. Do not fill a missing producer artifact with reviewer
assumption.

## Stop Conditions

- The active specification and roadmap conflict.
- Required implementation, evidence, or run artifacts are unavailable.
- The diff contains unrelated work that prevents bounded review.
- A determinism claim depends on inputs or outputs that were not recorded.
- Resolving a finding requires a decision outside the active milestone slice.

Return `blocked` with the exact missing input or decision.

## Validation

- Both sides of each reported boundary were inspected.
- Findings cite concrete repository or run evidence.
- Current defects are separated from future planned capability.
- `pass` has affirmative evidence, not merely an absence of observed failure.
- Local, remote, playability, replay, fidelity, and MCP claims are evaluated
  independently.
- The review does not modify producer output or silently expand scope.

## Browser Boundary Checks

- Inspect the full path `Command -> Game -> observation/events -> scene/cues ->
  WebGPU/Web Audio`; presentation callbacks, RAF timing, tab visibility, and
  device-pixel-ratio changes must not mutate the game.
- Reproduce an identical seed and semantic command stream through the browser
  session and direct core, comparing events, final observation, and replay.
- Check that GPU loss, blocked audio, unsupported WebGPU, and failed asset
  loads produce an explicit status/error without hidden-state access or a
  simulation step.
- Treat a remote web-CI result and a local browser playthrough as separate
  claims; do not promote either from `NOT_RUN` or `INCONCLUSIVE` by inference.
