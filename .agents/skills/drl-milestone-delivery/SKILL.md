---
name: drl-milestone-delivery
description: Deliver one bounded DRL-Rust roadmap slice while keeping specifications, architecture, history, implementation, and verification aligned.
---

# DRL Milestone Delivery

## When to Use

- Use this skill to plan, implement, resume, or review work from one roadmap
  milestone.
- Use it when legacy Pascal or Lua behavior must be converted into an explicit
  Rust-facing behavioral contract.
- Do not use it to redesign the full roadmap, work across unrelated milestones,
  or copy the legacy architecture.

## Required Inputs

- `docs/DRL-Rust_Project_Roadmap.md`
- `docs/DRL-Rust_Project_Proposal.md`
- `SPEC.md`, `ARCHITECTURE.md`, and `CHANGELOG.md`
- the current implementation and tests
- relevant legacy sources when the selected item concerns game behavior

## Source-of-Truth Hierarchy

1. The roadmap owns milestone scope, ordering, exit criteria, and progress.
2. `SPEC.md` expands the one active implementation slice into observable
   behavior, verification, and exclusions.
3. `ARCHITECTURE.md` describes verified current structure and consequential
   invariants. Planned structure must remain visibly labeled as planned.
4. `CHANGELOG.md` records meaningful work only after delivery is supported by
   evidence.
5. The proposal supplies design direction but does not prove implementation.

If these sources conflict, stop and report the conflict instead of choosing one
silently.

## Ownership and Team Use

The agent using this skill is the milestone owner. The milestone owner owns
scope, synthesis, implementation integration, final acceptance, and all writes
that reconcile `SPEC.md`, `ARCHITECTURE.md`, `CHANGELOG.md`, or the roadmap.

Read `docs/harness/drl-delivery/team-spec.md` before delegating. Keep work
direct unless legacy research, independent review, isolated tests, or precisely
disjoint implementation work provides concrete value. Delegation depth is one;
specialists do not create subordinate teams.

Delegated workers are read-only by default. Serialize canonical-document
writes. Prefer isolated checkouts for parallel code writes. Shared-checkout
writes require a disjoint path manifest and must not run repository-wide
formatters, dependency or lockfile operations, cross-cutting generators, or
git state-changing commands. The milestone owner remains the synthesis owner.

## Workflow

1. Select one milestone and the smallest coherent checklist slice that can be
   implemented and verified without unrelated work.
2. Inspect the current code, tests, documents, and relevant legacy evidence.
   Record uncertainty rather than inferring unsupported behavior. Decide
   whether the slice should remain direct or use one or more specialists from
   the team specification.
3. Update `SPEC.md` before implementation:
   - keep the roadmap item identifiable;
   - state observable outcomes and verification;
   - state explicit non-goals;
   - keep only one active slice;
   - link to the roadmap instead of reproducing its complete checklist.
4. For legacy-facing behavior, distinguish:
   - observed behavior;
   - inferred design intent;
   - legacy implementation accidents;
   - deliberate DRL-Rust decisions.
5. Implement the complete slice with focused tests. Keep randomness explicit,
   preserve headless execution, and prevent presentation or platform concerns
   from entering the simulation core.
6. Run `sh scripts/check-repository.sh` plus any slice-specific checks.
7. For consequential simulation or test-play changes, use
   `.agents/skills/drl-determinism-review/SKILL.md` as an independent,
   read-focused review gate. Apply at most one focused fix pass before
   re-scoping or reporting a block.
8. After any focused fix pass, rerun affected checks. Reconcile documentation
   from the final evidence:
   - update architecture only for changed, verified structure or invariants;
   - move completed specification outcomes out of active state only after
     verification;
   - add meaningful changelog entries;
   - mark roadmap tasks complete only when their stated result exists;
   - leave remote-CI criteria incomplete until the remote run passes.
9. Review the final diff for contradictions, accidental scope growth, and
   claims that exceed test or inspection evidence.

## Outputs

- an updated bounded slice in `SPEC.md`
- implementation and focused tests when the slice requires code
- targeted updates to `ARCHITECTURE.md`, `CHANGELOG.md`, and the roadmap
- a handoff reporting files changed, checks run, deviations, and unresolved
  risks

Write canonical outputs directly to the repository. Do not create `_workspace/`
handoffs for direct work. When delegation, interruption, or auditability
justifies a durable handoff, use:

```text
_workspace/drl/{milestone}-{slice}/
  00-scope.md
  01-evidence.md
  02-test-plan.md
  03-review.md
  04-verification.md
  final-handoff.md
```

Omit inapplicable intermediate files. Assign one run identifier from the slice
and starting revision, with a local rerun suffix when needed. Artifacts may
reference successive revisions as work progresses, but each must name its
predecessor artifact and revision. Start `00-scope.md` with
`predecessor: none` and the starting revision. Reject a mismatched run
identifier or broken revision lineage; replace or locally archive old run files
before continuing.

Every final handoff must identify the milestone and slice, run identifier,
owner and role, input and output revision or repository state, predecessor
artifact and revision, status, evidence inspected, files changed, checks and
exact outcomes, claims supported, unresolved uncertainty, skipped work, and
next owner. Use `PASS`, `FAIL`, `INCONCLUSIVE`, or `NOT_RUN` for execution
results.

## Stop Conditions

Stop and report rather than improvise when:

- the selected slice requires a decision that changes another milestone;
- legacy evidence is unavailable or materially contradictory;
- implementation would violate a documented architecture invariant;
- verification cannot support a requested completion claim;
- concurrent changes overlap the same source-of-truth documents;
- a required delegated result failed or conflicts with another result and
  source evidence cannot resolve the conflict.

## Validation

- Exactly one implementation slice is active in `SPEC.md`.
- Every completed roadmap checkbox has repository or remote evidence.
- Current and planned architecture are clearly distinguished.
- Documentation, tests, and implementation describe the same behavior.
- Delegation, when used, has one synthesis owner and no overlapping writes.
- Unsupported checks are reported as `NOT_RUN`; missing evidence remains
  `INCONCLUSIVE`.
- `sh scripts/check-repository.sh` succeeds.

## Browser Slice Rules

- Treat browser/WASM presentation as an effect boundary: `drl-core` and
  `drl-protocol` remain free of GPU, Web Audio, DOM, filesystem, and wall-clock
  dependencies.
- For a browser slice, record the WASM target, Rust/tool versions, browser
  version, GPU adapter/backend, viewport, device-pixel ratio, and audio unlock
  state in the handoff.
- Require player-observation parity with direct headless execution. A scene or
  cue builder may consume observations and semantic events only; animation,
  audio failure, resize, tab visibility, and GPU loss must not submit a
  command or advance simulation.
- Asset imports require a source revision, dirty-state record, license/
  attribution, and checksum manifest. Unclear audio, music, or font rights are
  `INCONCLUSIVE`/`NOT_RUN`, never silently bundled.
