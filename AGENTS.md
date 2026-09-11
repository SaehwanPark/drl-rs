# Repository Agents Guide

## What

- drl-rs is a ground-up Rust reimplementation of Doom the Roguelike.
- The project is a Rust 2024 workspace with a deterministic headless kernel
  and a bounded browser/WASM playable slice; full audiovisual parity remains
  staged.
- `docs/DRL-RS_Project_Roadmap.md` is the canonical project plan and progress
  tracker. `docs/steering/current-priorities.md` constrains near-term slice
  selection while its stop gates remain open. `SPEC.md` expands only the active
  milestone slice.
- `ARCHITECTURE.md` records verified current structure and invariants.
  `CHANGELOG.md` records delivered contributor- or user-visible changes.
- The legacy Pascal and Lua implementation is a behavioral reference, not an
  architecture to translate mechanically.

## Why

- Preserve game semantics while replacing legacy implementation machinery.
- Keep the future simulation deterministic, headless, and independent from
  graphics, audio, operating-system, filesystem, and MCP concerns.
- Distinguish implemented facts from the target design in the project proposal.
- Keep correctness and canonical behavior ahead of scalar content breadth when
  the current steering gates identify unresolved foundation work.

## How

- Before changing a milestone item, read the roadmap, `docs/steering/README.md`,
  the active `SPEC.md` slice, applicable architecture constraints, and relevant
  implementation or legacy evidence.
- Use `.agents/skills/drl-milestone-delivery/SKILL.md` for milestone work.
- For coordinated work, follow `docs/harness/drl-delivery/team-spec.md`; keep
  one milestone owner and serialize canonical-document writes.
- Use `.agents/skills/drl-test-play/SKILL.md` for test play. Run only modes
  enabled by implemented repository capabilities.
- Use spaces with an indentation and tab width of 2. Run:

  ```sh
  sh scripts/check-repository.sh
  ```

- Update the roadmap only from verified evidence. Keep incomplete work active
  and do not claim remote CI success until the remote check has passed.
- Apply the stop gates in `docs/steering/current-priorities.md`: rejected
  commands must be atomic, replay/RNG semantics explicit, content registration
  single-sourced, and callback-heavy behavior typed before broad scalar-only
  content migration resumes.
- Browser acceptance records browser/version, OS, GPU backend, viewport, DPR,
  build revision, and audio state. Unsupported WebGPU/audio or unavailable
  Linux legacy captures are `NOT_RUN`, not inferred passes.
- Project versioning is canonical in `VERSION` and follows
  [docs/VERSIONING.md](docs/VERSIONING.md): code changes require one valid
  `x.y.z` transition, while documentation-only and setting-only changes do
  not bump the version. Run `scripts/check-version.sh`; CI supplies
  `DRL_VERSION_BASE` so the agent harness can enforce the transition.
- Be aware the usage limits of AI subscripts you use and when the limits are reset. You may check with `codexbar` to know the used percentages and/or when the limits are reset. See `codexbar.md` for details and default policy.

## Subagents

Use subagents proactively to reduce main-context growth.

* Delegate bounded, self-contained investigation or implementation tasks when the parent mainly needs the result, not the working process.
* Prefer subagents for work that requires reading many files, logs, tests, documentation, or other large intermediate context.
* Give subagents only the context and scope needed for their task; avoid copying the full parent conversation unless necessary.
* Ask subagents to return concise findings, evidence/references, risks, and recommended actions rather than raw working context.
* Keep architectural decisions, cross-component integration, and final verification with the parent agent.
* Avoid redundant subagents inspecting the same scope unless independent review is intentional.
* If a subagent's scope expands substantially, it should escalate back to the parent rather than absorbing unrelated work.
* Use the main context for decisions; use subagent contexts for discovery.

See `docs/subagents_policy.md` for detailed delegation patterns and guidance.

## Asynchronous GitHub Communication

Use GitHub proactively as the durable communication channel when human collaborators are unavailable or work may continue across sessions.

* Prefer remote branches, commits, PRs, and GitHub discussions/comments over keeping important state only in local context.
* Push meaningful work to a remote branch regularly when it is safe and useful to preserve progress.
* Open a draft PR early for non-trivial work when it provides a useful place for status, design notes, review, and human steering.
* Keep PR descriptions and comments updated with current status, key decisions, unresolved questions, risks, and next steps.
* Use commits and PRs to leave a durable trail that another human or agent can resume without reconstructing the full conversation.
* When blocked on a human decision, record the question and relevant context in the PR or issue rather than leaving it only in transient agent context.
* Prefer small, reviewable commits and branches with clear scope.
* Do not merge, close, force-push shared work, or perform other irreversible repository actions unless explicitly authorized or clearly permitted by project policy.
* Never commit secrets, credentials, private data, or machine-specific sensitive artifacts.

Use local context for active reasoning; use GitHub for durable project state and asynchronous human communication.

## Agentic Loop

Use agentic loops for long-running tasks or when pursuing goals.

One loop is defined by

1. Select target slice (what to implement/examine/do)
2. Design a plan
3. Execute the plan
4. Test and verify
5. Update documents if necessary
6. PR handoff and merge autonomously
7. Move on to the next task or slice
