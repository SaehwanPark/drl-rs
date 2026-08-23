---
name: drl-determinism-review
description: Independently review a bounded DRL-Rust change for reproducibility, transactional safety, replay semantics, boundary integrity, evidence coherence, and claims that exceed implemented capability.
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
- `docs/steering/README.md`, `docs/steering/current-priorities.md`, and
  applicable steering decisions;
- relevant roadmap outcomes and architecture invariants;
- the produced diff, implementation, and tests;
- legacy evidence when behavior fidelity is claimed;
- test-play plans, run artifacts, and verification results when present.

Review only implemented surfaces. A planned feature is not a defect unless the
active slice requires it.

## Review Boundaries

Compare both sides of every applicable boundary:

- specification outcomes against test assertions;
- semantic commands against actual state transitions;
- rejected commands against exact pre/post `Game` identity, including RNG;
- player observations against hidden world state;
- semantic events against presentation or platform effects;
- initial state, seed, RNG semantics, command stream, and replay metadata
  against reproduced outcomes;
- replay wire/schema version against gameplay/ruleset compatibility claims;
- legacy evidence against typed Rust behavior claims;
- content catalog identity against generated/derived protocol, replay,
  validation, and presentation projections;
- direct simulation calls against replay, bot, frontend, or MCP clients;
- local and remote completion claims against exact verification evidence.

Also inspect for ambient randomness, modulo-biased bounded sampling, unstable
iteration, wall-clock decisions, filesystem/OS dependencies in simulation,
presentation timing that changes gameplay, unbounded episodes, unstable IDs in
canonical output, retries that conceal nondeterminism, mutation-before-error,
and hidden-state leakage.

## Workflow

1. Restate the slice's observable outcomes, non-goals, acceptance evidence, and
   steering-gate impact.
2. Identify consequential boundaries touched by the change.
3. Read producer and consumer sides of each boundary together.
4. Trace randomness, time, iteration order, I/O, and state mutation from input
   to semantic outcome.
5. For changed command paths, inspect both success and representative rejection
   paths; `Err` must not mutate world, inventory, energy, counters, turn, or RNG.
6. For replay-visible changes, distinguish current-version repeatability from
   cross-version compatibility and check semantics metadata explicitly.
7. For legacy behavior/content changes, verify that static-definition coverage
   is not reported as behavior-complete without the required typed behavior and
   evidence.
8. Compare each completion claim with the exact test, run, inspection, or
   remote result that supports it.
9. Attempt the smallest relevant reproduction or targeted check when permitted.
10. Report concrete findings with impact, evidence, and the smallest safe fix.
11. Return one disposition:
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
- predecessor artifact and revision;
- review disposition;
- steering gates inspected;
- boundaries inspected;
- blocking findings and non-blocking risks;
- checks or reproductions run;
- unverified surfaces;
- required fix or decision;
- evidence supporting a pass.

Order findings by impact. Distinguish confirmed defects from risks and
unverified areas. Do not fill a missing producer artifact with reviewer
assumption.

## Stop Conditions

- The active specification, roadmap, accepted architecture, and steering rules
  conflict.
- Required implementation, evidence, or run artifacts are unavailable.
- The diff contains unrelated work that prevents bounded review.
- A determinism claim depends on inputs or outputs that were not recorded.
- Resolving a finding requires a decision outside the active milestone slice.

Return `blocked` with the exact missing input or decision.

## Validation

- Both sides of each reported boundary were inspected.
- Findings cite concrete repository or run evidence.
- Rejection atomicity is reviewed for changed command paths.
- Replay-current repeatability is not conflated with cross-version compatibility.
- Static content coverage is not conflated with behavior or legacy parity.
- Current defects are separated from future planned capability.
- `pass` has affirmative evidence, not merely absence of observed failure.
- The review does not modify producer output or silently expand scope.

## Browser Boundary Checks

- Inspect `Command -> Game -> observation/events -> scene/cues -> WebGPU/Web
  Audio`; presentation callbacks, RAF timing, tab visibility, and DPR changes
  must not mutate the game.
- Reproduce an identical seed and semantic command stream through browser and
  direct core, comparing events, final observation, and replay semantics.
- Check GPU loss, blocked audio, unsupported WebGPU, and failed assets produce
  explicit status/error without hidden-state access or a simulation step.
