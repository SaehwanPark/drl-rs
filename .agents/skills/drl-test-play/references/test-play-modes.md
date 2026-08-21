# Test-Play Modes

Select a mode from implemented repository capability, not milestone intent.
When a requested gate is closed, return `NOT_RUN` for that mode.

## Mode 0: Scaffold and legacy evidence

Activation:

- the Rust scaffold or relevant legacy checkout and setup exist.

Scaffold checks are the only DRL-Rust execution available during Milestone 0.
Legacy evidence probes remain available in later milestones whenever a bounded
behavior question requires them.

Allowed activities:

- launch and repository-check smoke tests for the Rust scaffold;
- focused legacy source or manual-runtime probes;
- review of proposed scenario inputs and acceptance criteria.

Required evidence:

- Rust or legacy revision and dirty state;
- exact command or probe steps;
- configuration, frontend, and developer controls used;
- observations and uncertainty.

Limits:

- scaffold execution is not gameplay;
- legacy behavior is not a deterministic DRL-Rust trace oracle;
- no playability, replay, bot, MCP, or fidelity completion claim is available.

## Mode 1: Seeded headless scenario

Activation:

- the DRL-Rust simulation has an explicit initial state;
- semantic commands enter through an implemented command boundary;
- gameplay randomness is injected explicitly;
- semantic outcomes can be inspected without a renderer.

Required inputs:

- scenario identifier;
- build or revision;
- canonical initial state or fixture;
- seed or scripted focused outcomes;
- ordered semantic commands;
- command and simulation-turn limits;
- expected outcomes and invariants.

Required evidence:

- actual commands and semantic outcomes;
- final state or stable semantic digest where supported;
- repeat execution from identical inputs;
- diagnostic turn or command context for failure.

Do not freeze an external fixture schema before its active milestone or ADR
selects one. Rust constructors are valid early scenario inputs.

## Mode 2: Replay validation

Activation:

- a versioned replay schema exists;
- build/content version, seed, initial configuration, and command stream are
  recorded;
- replay validation reports mismatch context.

Required evidence:

- replay schema and content version;
- original and replayed outcomes;
- first mismatch with turn and command context;
- stable reproduction on the relevant build.

A replay mismatch is `FAIL`. A replay that cannot be parsed because its
declared schema is unsupported is `INCONCLUSIVE` unless compatibility behavior
defines a different expected result.

## Mode 3: Scripted-bot cohort

Activation:

- bots consume ordinary player `Observation`;
- bots submit ordinary semantic `Command`;
- episode limits and failure artifact capture exist.

Required inputs:

- fixed seed cohort;
- bot policy and version;
- character and difficulty configuration;
- command and turn limits;
- predeclared metrics and invariants.

Required evidence:

- outcome for every seed, including crash and timeout;
- pathological seeds and replay candidates;
- aggregate summary that does not hide individual hard failures.

Do not give a bot omniscient state for an ordinary-player result. Developer
instrumentation must be separate and labeled.

## Mode 4: MCP parity and exploratory play

Activation:

- the MCP lifecycle, observation, legal-action, and command interfaces exist;
- resource and episode limits are enforced;
- direct simulation behavior can be compared with MCP behavior.

Required evidence:

- MCP schema or capability version;
- semantic requests and responses;
- corresponding direct-simulation result for parity checks;
- ordinary-player observation boundary;
- replay or deterministic scenario for actionable failures.

Keep model reasoning and conversation transcripts separate from authoritative
test results. A model-found issue remains exploratory until deterministic
reduction, except for directly captured crash, corruption, or information-leak
evidence.

## Mode 5: Human frontend play

Activation:

- the tested browser frontend and interaction path are implemented;
- the task protocol and build are fixed;
- simulation outcomes can be separated from presentation timing.

Browser-first requirements:

- test desktop Chrome/Edge with WebGPU first and record browser version, OS,
  adapter/backend, viewport, DPR, build revision, focus ownership, and audio
  unlock/mute state;
- compare named scenes against a revisioned reference-capture manifest with
  explicit tolerances; missing rights or capture metadata is `INCONCLUSIVE`;
- exercise unsupported WebGPU, GPU loss, blocked/suspended audio,
  background-tab visibility, resize/DPR, and page-scroll recovery; none may
  change simulation state;
- keep Firefox/Safari/WebGL2 fallback, mobile/touch, controllers, and native
  desktop packaging post-1.0 until their acceptance gates are implemented.

Use structured tasks for:

- discoverability and input ergonomics;
- feedback, readability, pacing, audio, and game feel;
- frontend-to-command mapping;
- recovery from invalid input or interrupted flow.

Record task, participant context relevant to interpretation, build,
configuration, actions, observations, and subjective judgments separately.
Human feedback may motivate a change, but deterministic mechanics claims still
require simulation evidence.

## Mode 6: Statistical balance study

Activation:

- batch execution and machine-readable metrics exist;
- fixed seed cohorts and configuration are recordable;
- the regression policy defines meaningful metrics and tolerances.

Declare before execution:

- hypothesis;
- immutable evaluation surface;
- build and content versions;
- fixed seed cohort;
- player archetype and bot policy;
- metric, sample size, tolerance, and decision rule;
- crash and timeout treatment.

Record every outcome. Do not replace failed seeds, rerun with fresh cohorts
until a threshold passes, or treat every distribution shift as a defect.

## Artifact Naming

Before a committed scenario schema exists, use descriptive working identifiers:

```text
m{NN}-{domain}-{behavior}-v{NNN}
```

Example:

```text
m01-movement-blocked-boundary-v001
```

Until a milestone selects a committed schema, keep bulk run artifacts under:

```text
target/test-play/{suite-id}/{run-key}/
```

Derive `{run-key}` from immutable inputs where practical, for example:

```text
build-{git12}__inputs-{manifest-sha12}
```

Exclude wall-clock time, absolute paths, presentation timing, map iteration
order, and ephemeral identifiers from canonical comparisons.

The active scenario or replay milestone may replace the working identifier
shape. When replay and fixture formats are explicitly selected, prefer these
durable locations:

```text
tests/fixtures/{milestone}/
tests/scenarios/{milestone}/
tests/replays/regressions/
```

Do not create those directories merely to satisfy this reference; their active
milestone must define the format and ownership first.

## Failure Promotion

- Reproduce a deterministic failure twice from the same recorded inputs.
- If the outcomes differ, classify a nondeterminism defect.
- Minimize the setup and command stream while preserving the failure.
- Promote only diagnostic artifacts into the repository.
- Update an expected result only with an intentional behavior decision,
  specification reconciliation, semantic diff review, and independent
  approval.
