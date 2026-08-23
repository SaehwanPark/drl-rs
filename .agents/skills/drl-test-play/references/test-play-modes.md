# Test-Play Modes

Select a mode from implemented repository capability, not milestone intent.
When a requested gate is closed, return `NOT_RUN` for that mode.

## Mode 0: Scaffold and legacy evidence

Activation:

- the Rust scaffold or relevant legacy checkout and setup exist.

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
- seed and RNG-semantics version, or scripted focused outcomes;
- ordered semantic commands;
- command and simulation-turn limits;
- expected outcomes and invariants.

Required evidence:

- actual commands and semantic outcomes;
- final state or stable semantic digest where supported;
- repeat execution from identical inputs;
- diagnostic turn or command context for failure.

When testing a rejected command, compare the complete `Game` pre/post state,
including RNG, energy, counters, inventory/equipment, turn, and terminal state.

## Mode 2: Replay validation

Activation:

- a versioned replay schema exists;
- build, gameplay/ruleset semantics, seed, initial configuration, and command
  stream are recorded;
- replay validation reports mismatch context.

Required evidence:

- replay wire/schema version;
- engine/gameplay semantics version;
- ruleset/content semantics identifier and generator semantics where separate;
- original and replayed outcomes;
- first mismatch with turn and command context;
- stable reproduction on the relevant build.

A replay mismatch is `FAIL`. An unsupported declared semantics version is an
expected compatibility rejection when the contract says to reject it; do not
label current-build repeatability as cross-version compatibility.

## Mode 3: Scripted-bot cohort

Activation:

- bots consume ordinary player `Observation`;
- bots submit ordinary semantic `Command`;
- episode limits and failure artifact capture exist.

Required inputs:

- fixed seed cohort;
- RNG/gameplay semantics identifiers;
- bot policy and version;
- character and difficulty configuration;
- command and turn limits;
- predeclared metrics and invariants.

Required evidence:

- outcome for every seed, including crash and timeout;
- pathological seeds and replay candidates;
- aggregate summary that does not hide individual hard failures.

Do not give a bot omniscient state for an ordinary-player result.

## Mode 4: MCP parity and exploratory play

Activation:

- MCP lifecycle, observation, legal-action, and command interfaces exist;
- resource and episode limits are enforced;
- direct simulation behavior can be compared with MCP behavior.

Required evidence:

- MCP schema/capability version;
- semantic requests and responses;
- corresponding direct-simulation result for parity checks;
- ordinary-player observation boundary;
- replay or deterministic scenario for actionable failures.

Keep model reasoning and transcripts separate from authoritative test results.
A model-found issue remains exploratory until deterministic reduction, except
for directly captured crash, corruption, or information-leak evidence.

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
  explicit tolerances; missing rights/capture metadata is `INCONCLUSIVE`;
- exercise unsupported WebGPU, GPU loss, blocked/suspended audio,
  background-tab visibility, resize/DPR, and page-scroll recovery; none may
  change simulation state;
- keep Firefox/Safari/WebGL2 fallback, mobile/touch, controllers, and native
  desktop packaging post-1.0 until their acceptance gates are implemented.

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
- build, replay wire, RNG, gameplay, and content semantics versions;
- fixed seed cohort;
- player archetype and bot policy;
- metric, sample size, tolerance, and decision rule;
- crash and timeout treatment.

Record every outcome. Do not replace failed seeds, rerun with fresh cohorts
until a threshold passes, or treat every distribution shift as a defect.

## Artifact Naming

Before a committed scenario schema exists, use:

```text
m{NN}-{domain}-{behavior}-v{NNN}
```

Keep bulk run artifacts under:

```text
target/test-play/{suite-id}/{run-key}/
```

Derive `{run-key}` from immutable inputs where practical. Exclude wall-clock
time, absolute paths, presentation timing, map iteration order, and ephemeral
identifiers from canonical comparisons.

## Failure Promotion

- Reproduce a deterministic failure twice from the same recorded inputs.
- If outcomes differ, classify a nondeterminism defect.
- Minimize setup and command stream while preserving failure.
- Promote only diagnostic artifacts into the repository.
- Update an expected result only with an intentional behavior decision,
  specification reconciliation, semantic diff review, and independent approval.
