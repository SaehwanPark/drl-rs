---
name: drl-test-play
description: Plan and run capability-gated DRL scenarios, replays, bots, MCP sessions, legacy probes, statistical studies, or human play without overstating exploratory evidence.
---

# DRL Test Play

## When to Use

- Use this skill to design, run, resume, or review a bounded DRL test-play
  activity.
- Use deterministic scenarios/replays to verify command atomicity, behavior
  interactions, replay semantics, and vertical fidelity where those surfaces
  are implemented.
- Do not use test play as a substitute for a smaller unit/property invariant
  when that invariant fully expresses the rule.

## Required Inputs

- the active milestone slice and requested play mode;
- verified repository capabilities at the current revision;
- applicable steering gate from `docs/steering/current-priorities.md`;
- a bounded task, scenario, or question;
- initial state/setup, ordered inputs, seed/RNG-semantics policy, and episode
  limits when applicable;
- predeclared assertions, metrics, or observation questions;
- output and failure-artifact location.

Read `references/test-play-modes.md` before selecting a mode. Planned roadmap
capabilities do not satisfy activation gates.

## Result Status

- `PASS`: requested mode ran and met predeclared criteria;
- `FAIL`: mode ran and violated a criterion or invariant;
- `INCONCLUSIVE`: activity ran but cannot resolve the question;
- `NOT_RUN`: requested mode or required setup was unavailable.

Do not substitute another mode and report it as a pass.

## Workflow

1. Confirm the mode's activation gate from implemented code/tests.
2. Write question, setup, inputs, limits, acceptance criteria, and expected
   evidence before execution.
3. Separate authoritative simulation inputs/outputs from operator, bot, model,
   or human reasoning.
4. For command-rejection tests, record complete state identity requirements,
   including RNG state.
5. For replay tests, record wire schema and gameplay/ruleset semantics; do not
   label same-build repeatability as cross-version compatibility.
6. Run the smallest bounded activity that can answer the question.
7. Capture build/source revision, configuration, seed, initial state, command
   sequence, limits, semantic outputs, and failure context.
8. Attempt reproduction from recorded inputs. Changing outcomes under identical
   deterministic inputs is a nondeterminism defect.
9. Classify with one status and state exactly which claims it supports.
10. Minimize actionable exploratory findings into a deterministic unit test,
    scenario, or replay before using them as regression/completion evidence.
11. Return artifacts to the milestone owner. Do not update golden expectations,
    specifications, steering gates, or roadmap status directly.

## Outputs

For coordinated work, create:

- `_workspace/drl/{milestone}-{slice}/02-test-plan.md` before execution;
- attributable execution results for the milestone owner to synthesize into
  `_workspace/drl/{milestone}-{slice}/04-verification.md`.

The test plan identifies run identifier, owner and role, input/output repository
state, predecessor artifact and revision, status, mode, activation evidence,
setup, inputs, limits, assertions/questions, artifacts, and stop conditions.
The milestone owner owns `04-verification.md` after determinism review and any
focused fix pass.

## Stop Conditions

- The requested mode's activation gate is not satisfied.
- Required revision, seed, initial state, commands, limits, RNG/replay semantics,
  or acceptance criteria cannot be recorded.
- The test would expose privileged world state to an ordinary player/agent.
- The evaluation surface changes during a fixed comparison/statistical run.
- Continuing would overwrite a golden result without an intentional behavior
  decision.
- The activity becomes unbounded or leaves the selected milestone slice.

## Validation

- The selected mode is implemented, not merely planned.
- Inputs and limits are sufficient to reproduce it where supported.
- Result status follows the defined meanings.
- Exploratory reasoning is separate from authoritative evidence.
- Rejected-command checks compare full simulation state where applicable.
- Replay results distinguish current semantics from cross-version claims.
- Statistical conclusions use predeclared cohort, metric, sample size, and
  tolerance.
- No playability, fidelity, replay, MCP, or remote claim exceeds the exact mode
  that ran.

## Browser Human-Play Mode

- Record browser/version, OS, GPU adapter/backend, viewport, DPR, build
  revision, seed, focus ownership, and audio state.
- Test WebGPU unavailable, GPU loss, resize/DPR, background-tab visibility,
  blocked/suspended audio, and page-scroll recovery; none may change simulation
  state or replay output.
- Compare named scenes against approved legacy captures with stated tolerances.
  Subjective impression without a capture manifest is `INCONCLUSIVE`.
