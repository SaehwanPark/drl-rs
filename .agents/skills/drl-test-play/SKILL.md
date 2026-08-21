---
name: drl-test-play
description: Plan and run capability-gated DRL scaffold checks, legacy probes, deterministic scenarios, replays, bots, MCP sessions, or human play without overstating exploratory evidence.
---

# DRL Test Play

## When to Use

- Use this skill to design, run, resume, or review a bounded DRL test-play
  activity.
- Use it for scaffold smoke checks and structured legacy probes now, then for
  headless scenarios, replays, bots, MCP, statistical studies, or human play
  only after their activation gates are implemented.
- Do not use it as a substitute for unit or property tests when a smaller
  deterministic test expresses the rule completely.

## Required Inputs

- the active milestone slice and requested play mode;
- verified repository capabilities at the current revision;
- a bounded task, scenario, or question;
- initial state or setup, ordered inputs, seed or randomness policy, and
  episode limits when applicable;
- predeclared assertions, metrics, or observation questions;
- the output and failure-artifact location.

Read `references/test-play-modes.md` before selecting a mode. Planned roadmap
capabilities do not satisfy activation gates.

## Result Status

- `PASS`: the requested mode ran and met its predeclared acceptance criteria;
- `FAIL`: the requested mode ran and violated a criterion or invariant;
- `INCONCLUSIVE`: the activity ran, but its evidence cannot resolve the
  question;
- `NOT_RUN`: the requested mode or required setup was unavailable.

Do not replace an unavailable mode with another activity and report the
substitute as a pass.

## Workflow

1. Confirm the mode's activation gate from implemented code and tests.
2. Write the question, setup, inputs, limits, acceptance criteria, and expected
   evidence before execution.
3. Separate authoritative simulation inputs and outputs from operator, bot,
   model, or human reasoning.
4. Run the smallest bounded activity that can answer the question.
5. Capture the exact build or source revision, configuration, seed, initial
   state, command sequence, limits, semantic outputs, and failure context that
   exist for the selected mode.
6. Attempt reproduction from the recorded inputs. If an ostensibly
   deterministic failure changes across repetitions, report a nondeterminism
   defect rather than retrying until it passes.
7. Classify the result with one status and state exactly which claims it
   supports.
8. Minimize actionable exploratory findings into a deterministic unit test,
   scenario, or replay before using them as regression or completion evidence.
9. Return artifacts to the milestone owner. Do not update golden expectations,
   specifications, or roadmap status directly.

## Outputs

For coordinated work, create:

- `_workspace/drl/{milestone}-{slice}/02-test-plan.md` before execution;
- attributable execution results for the milestone owner to synthesize into
  `_workspace/drl/{milestone}-{slice}/04-verification.md`.

The test plan identifies run identifier, owner and role, input and output
repository state, predecessor artifact and revision, status, mode, activation
evidence, setup, inputs, limits, assertions or questions, artifacts, and stop
conditions. Its predecessor is the nearest prior artifact, normally
`01-evidence.md` or `00-scope.md`.

The milestone owner owns and replaces `04-verification.md` after determinism
review and any focused fix pass. The operator must not write that shared file
unless the operator is also the milestone owner. The verification content
identifies the same run and revision-lineage fields, with `03-review.md` as its
normal predecessor, plus result status, actual inputs, observed outputs,
reproduction outcome, failures or timeouts, claims supported, unresolved
uncertainty, and regression candidate.

Keep bulk run output under ignored build or runtime storage. Commit only small,
diagnostic fixtures or replays selected through the active milestone workflow.

## Stop Conditions

- The requested mode's activation gate is not satisfied.
- Required revision, seed, initial state, commands, limits, or acceptance
  criteria cannot be recorded.
- The test would expose privileged world state to an ordinary player or agent.
- The evaluation surface changes during a fixed comparison or statistical run.
- Continuing would overwrite an existing golden result without an intentional
  behavior decision.
- The activity becomes unbounded or leaves the selected milestone slice.

## Validation

- The selected mode is implemented, not merely planned.
- Inputs and limits are sufficient to reproduce the activity where the mode
  supports reproduction.
- Result status follows the defined meanings.
- Exploratory reasoning is separate from authoritative evidence.
- Statistical conclusions use a predeclared cohort, metric, sample size, and
  tolerance.
- No playability, fidelity, replay, MCP, or remote-execution claim exceeds the
  exact mode that ran.

## Browser Human-Play Mode

- Activate human frontend play only when a browser bundle and start/error
  recovery path exist. Record browser/version, OS, GPU adapter/backend,
  viewport, DPR, build revision, seed, input ownership/focus, and audio state.
- Test WebGPU unavailable, device/GPU loss, resize/DPR changes, background-tab
  visibility, blocked or suspended audio, and page-scroll prevention as
  recovery cases. These presentation failures must not change turn, state, or
  replay output.
- Compare named scenes against approved legacy reference captures with stated
  tolerances and structured human review. A subjective impression without a
  capture manifest is `INCONCLUSIVE`.
- Keep DOM inventory actions and keyboard mappings as semantic-command
  assertions, and verify hidden actors/items never enter a player scene.
