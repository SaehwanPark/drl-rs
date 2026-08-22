# DRL Delivery Team Specification

## Goal

Coordinate bounded DRL-Rust development and test play without creating a
second source of truth or allowing exploratory results to become unsupported
completion claims.

The roadmap remains canonical. `SPEC.md` defines the one active slice. This
team specification defines only role selection, ownership, handoffs, and
failure behavior.

Project versioning is a delivery invariant. `VERSION` is the canonical
`x.y.z` value; a code-path change requires exactly one allowed component
increment with lower digits reset, while documentation-only and setting-only
changes keep the current version. The milestone owner runs
`scripts/check-version.sh`, and CI provides `DRL_VERSION_BASE` so the harness
can compare the candidate with its base commit. Automatic carry-over is never
performed.

## Architecture

The outer workflow is a pipeline:

```text
scope
  -> evidence and specification
  -> implementation
  -> test-play planning and focused execution
  -> determinism review
  -> final verification and canonical-document reconciliation
```

An expert pool is used only inside stages where specialization or context
isolation has concrete value. The milestone owner selects specialists rather
than invoking every role for every change. Determinism review forms a bounded
producer-reviewer edge before final acceptance.

## Roles

### Milestone owner

- Uses `.agents/skills/drl-milestone-delivery/SKILL.md`.
- Owns scope, synthesis, implementation integration, final acceptance, and the
  final handoff.
- Is the only role allowed to reconcile `SPEC.md`, `ARCHITECTURE.md`,
  `CHANGELOG.md`, and the roadmap.
- May perform a small slice directly when delegation would add no useful
  separation.

### Legacy archaeologist

- Uses `.agents/skills/drl-legacy-archaeology/SKILL.md`.
- Reads legacy Pascal, Lua, manuals, data, and runtime evidence.
- Classifies findings without choosing silently between contradictory sources.
- Writes evidence, not Rust implementation or milestone status.

### Test-play operator

- Uses `.agents/skills/drl-test-play/SKILL.md`.
- Selects only a play mode whose activation gate is satisfied.
- Records inputs, limits, observations, and reproducibility evidence.
- Does not treat a bot, model, or human interpretation as an authoritative
  simulation result.

### Determinism reviewer

- Uses `.agents/skills/drl-determinism-review/SKILL.md`.
- Independently compares the requested behavior, specification,
  implementation, tests, and produced evidence.
- Remains read-focused and reports `pass`, `fix`, or `blocked`.
- Does not revise the producer's implementation during the review pass.

## Delegation Gate

Delegate only when at least one condition holds:

- legacy or repository research is broad enough to consume the owner's
  implementation context;
- an independent review would materially reduce false confidence;
- tests can run in isolation without competing for mutable resources;
- disjoint implementation surfaces can be assigned precisely or isolated in
  separate checkouts.

Keep work direct when it is small, tightly coupled, or cheaper to verify in one
context. Delegation depth is one: specialists do not create subordinate teams.
The milestone owner remains the synthesis and acceptance owner.

Delegated workers are read-only by default. Parallel writes in one checkout
are allowed only for explicitly disjoint paths with a declared write manifest.
Prefer isolated checkouts. A shared-checkout worker must not run repository-wide
formatters, dependency or lockfile operations, generators with cross-cutting
outputs, or git state-changing commands. Writes to canonical project documents
are always serialized through the milestone owner.

## Phase Contract

### 1. Scope

- Select one milestone and its smallest coherent slice.
- Record observable outcomes, exclusions, required evidence, and acceptance
  checks.
- Decide whether direct work or a team workflow is justified.

### 2. Evidence and specification

- Inspect current implementation, tests, canonical documents, and relevant
  legacy evidence.
- Invoke the legacy archaeologist only when behavior or provenance depends on
  the reference implementation.
- Update the active `SPEC.md` slice before implementation.

### 3. Implementation

- Assign one integration owner.
- Keep randomness explicit and preserve the headless simulation boundary.
- Add the smallest tests that express the selected behavioral contract.

### 4. Test-play planning and focused execution

- Invoke only modes enabled by the repository's implemented capabilities.
- Record unsupported requested modes as `NOT_RUN`; do not substitute a
  different activity and call it equivalent.
- Record the plan and provisional evidence needed by the independent review.
- Promote exploratory findings to completion evidence only after they are
  reduced to a deterministic test, scenario, or replay.

### 5. Determinism review

- Compare both sides of consequential boundaries: specification and tests,
  commands and state transitions, observations and hidden state, events and
  presentation, seeded inputs and reproduced outputs.
- Request focused fixes when the slice is recoverable.
- Return `blocked` when evidence is missing or contradictory.

### 6. Final verification and reconciliation

- Run or rerun repository, slice-specific, and supported test-play checks after
  any focused fix pass.
- The milestone owner owns `04-verification.md` and writes or replaces it from
  attributable operator results after review and any focused fix pass.
- Update architecture only for verified structure or invariants.
- Update changelog and roadmap only for delivered results supported by
  evidence.
- Run the version contract after implementation. A code change without the
  required version transition is a failed delivery check; a documentation-only
  or setting-only diff must not receive a version bump.
- Keep remote-only criteria incomplete until the named remote check passes.

## Handoff Contract

Normal direct work returns a bounded in-thread handoff and creates no runtime
files. Use the ignored workspace only when work crosses an agent boundary,
must survive interruption, or needs an inspectable audit trail:

```text
_workspace/drl/{milestone}-{slice}/
  00-scope.md
  01-evidence.md
  02-test-plan.md
  03-review.md
  04-verification.md
  final-handoff.md
```

Use lowercase hyphenated `{milestone}-{slice}`, for example
`m00-legacy-movement-notes`. Omit an inapplicable intermediate file rather
than creating an empty placeholder.

Every handoff must identify:

- slice and milestone;
- run identifier;
- owner and role;
- input revision or repository state and, when it changed, output revision;
- predecessor artifact and revision;
- status;
- evidence inspected or produced;
- claims supported by that evidence;
- checks run and their exact outcomes;
- unresolved uncertainty, skipped work, and next owner.

Assign one run identifier when scope is selected, derived from the slice and
starting revision with a local rerun suffix when needed. Before consuming an
artifact, verify that its run identifier matches and that its predecessor
revision belongs to the recorded revision lineage. `00-scope.md` starts the
lineage with `predecessor: none` and the starting revision. Each later artifact
names the nearest prior artifact that exists; omitted optional phases do not
require placeholders. Artifacts may reference successive revisions as
implementation and focused fixes progress. Reject artifacts from another run
or a broken lineage. Before reusing the same workspace path, the milestone
owner must replace the old run files or archive them under a distinct local
path.

Allowed execution statuses are `PASS`, `FAIL`, `INCONCLUSIVE`, and `NOT_RUN`.
The determinism review uses `pass`, `fix`, or `blocked` so a review disposition
cannot be confused with a test result.

## Failure Policy

- Missing or contradictory required evidence yields `INCONCLUSIVE` and blocks
  affected behavior claims.
- An unavailable capability yields `NOT_RUN`; it is not a failure and is not a
  pass.
- A failed delegated branch is disclosed. Synthesis may continue only when the
  missing result is not required for acceptance.
- Conflicting specialist findings return to the milestone owner with both
  evidence trails preserved. The owner must resolve the conflict from source
  evidence or stop.
- Failed local checks leave completion claims and roadmap boxes unchanged.
- A remote check may be reported only from direct evidence for the relevant
  revision.
- Review revision is bounded to one focused fix pass. A second blocking result
  returns control to the milestone owner for re-scoping.

## Test-Play Activation

The detailed mode contract lives in
`.agents/skills/drl-test-play/references/test-play-modes.md`.

- Milestone 0 can run scaffold smoke checks and structured legacy probes.
- Structured legacy probes remain available in later milestones when a bounded
  behavior question requires them.
- Seeded DRL-Rust headless scenario play begins only after the Milestone 1
  command, state, and explicit-RNG boundaries exist.
- Replay and scripted-bot play require the corresponding Milestone 5
  infrastructure.
- MCP play requires the Milestone 6 semantic interface.
- Browser human play requires the implemented WASM frontend; game-feel or
  audiovisual conclusions require the relevant M7/M8 surface and reference
  captures. Native desktop play is post-1.0.
- Statistical play activates when the selected slice has batch execution,
  declared metrics, fixed cohorts, sample size, and tolerance policy.
  Milestone 11 broadens this into systematic balance analysis.

Browser test-play additionally records browser/version, OS, adapter/backend,
viewport, DPR, build revision, input focus, and audio state. Unsupported
WebGPU, blocked audio, GPU loss, background-tab timing, or resize behavior is
reported as `NOT_RUN`/`INCONCLUSIVE` for the affected claim rather than
silently substituted with a headless result.

## Validation

Use `docs/harness/drl-delivery/validation-scenarios.md` when changing role
boundaries, handoff names, or failure policy. Repository checks validate skill
structure, required harness paths, and required handoff/status vocabulary;
scenario review validates semantic agreement.
