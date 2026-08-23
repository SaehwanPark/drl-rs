---
name: drl-milestone-delivery
description: Deliver one bounded DRL-Rust roadmap slice while keeping steering, specifications, architecture, history, implementation, and verification aligned.
---

# DRL Milestone Delivery

## When to Use

- Use this skill to plan, implement, resume, or review work from one roadmap
  milestone.
- Use it when legacy Pascal or Lua behavior must become an explicit Rust-facing
  behavioral contract.
- Do not use it to redesign the full roadmap, work across unrelated milestones,
  or copy the legacy architecture.

## Required Inputs

- `docs/DRL-Rust_Project_Roadmap.md`
- `docs/steering/README.md` and `docs/steering/current-priorities.md`
- `references/steering-gates.md`
- `docs/DRL-Rust_Project_Proposal.md`
- `SPEC.md`, `ARCHITECTURE.md`, and `CHANGELOG.md`
- the current implementation and tests
- relevant legacy sources when the selected item concerns game behavior

## Source-of-Truth Hierarchy

1. Accepted ADRs and verified `ARCHITECTURE.md` own enduring architecture and
   implemented invariants.
2. The roadmap owns milestone scope, ordering, exit criteria, and progress.
3. `docs/steering/current-priorities.md` constrains candidate-slice selection
   while its stop gates remain open.
4. `SPEC.md` expands exactly one active implementation slice into observable
   behavior, verification, and exclusions.
5. `CHANGELOG.md` records meaningful work only after delivery is supported by
   evidence.
6. The proposal supplies design direction; audit files supply evidence and
   rationale. Neither proves implementation.

If these sources conflict, stop and report the conflict instead of choosing one
silently. A steering decision that conflicts with an accepted ADR must be
resolved in the enduring document before implementation relies on it.

## Ownership and Team Use

The agent using this skill is the milestone owner. The milestone owner owns
scope, synthesis, implementation integration, final acceptance, and all writes
that reconcile `SPEC.md`, `ARCHITECTURE.md`, `CHANGELOG.md`, steering docs, or
the roadmap.

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

1. Read the steering index and `references/steering-gates.md` before selecting
   work. Identify which open gate the candidate slice closes or why it is a
   justified exemption.
2. Select one milestone and the smallest coherent checklist slice that can be
   implemented and verified without unrelated work.
3. Inspect current code, tests, documents, and relevant legacy evidence. Record
   uncertainty rather than inferring unsupported behavior.
4. Update `SPEC.md` before implementation:
   - keep the roadmap item identifiable;
   - state observable outcomes and verification;
   - state explicit non-goals;
   - keep exactly one active slice;
   - record command-atomicity, RNG/replay, content-catalog, behavior-model,
     protocol/domain, and rights impacts when applicable.
5. For legacy-facing behavior, distinguish observed behavior, inferred intent,
   implementation artifacts, ambiguity, and deliberate DRL-Rust decisions.
   Static-definition coverage is not behavior-complete by implication.
6. Implement the complete slice with focused tests. Keep randomness explicit,
   preserve headless execution, and prevent presentation/platform concerns from
   entering the simulation core.
7. For command changes, verify representative rejected paths satisfy
   `Err => before == after`, including RNG state.
8. For replay-visible changes, state whether gameplay/ruleset semantics are
   preserved or advanced. A stable wire schema alone does not prove
   cross-version compatibility.
9. For content work, prefer one authoritative compile-time registration path
   and typed behavior. Do not normalize broad scalar-only fan-out while the
   content/behavior steering gate remains open.
10. Run `sh scripts/check-repository.sh` plus slice-specific checks.
11. For consequential simulation or test-play changes, use
   `.agents/skills/drl-determinism-review/SKILL.md` as an independent,
   read-focused gate. Apply at most one focused fix pass before re-scoping.
12. Reconcile canonical documents only from final evidence. Run
   `scripts/check-version.sh`; code-path changes require one valid `x.y.z`
   transition, while documentation-only and setting-only changes do not bump
   `VERSION`.
13. Review the final diff for contradictions, accidental scope growth, stale
   steering gates, and claims that exceed evidence.

## Outputs

- an updated bounded slice in `SPEC.md`
- implementation and focused tests when the slice requires code
- targeted updates to `ARCHITECTURE.md`, `CHANGELOG.md`, roadmap, accepted ADRs,
  and steering docs only where the evidence requires them
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
and starting revision, with a local rerun suffix when needed. Each artifact must
name its predecessor artifact and revision. Start `00-scope.md` with
`predecessor: none` and the starting revision. Reject a mismatched run
identifier or broken revision lineage.

Every final handoff must identify the milestone and slice, run identifier,
owner and role, input and output revision or repository state, predecessor
artifact and revision, status, evidence inspected, files changed, checks and
exact outcomes, claims supported, unresolved uncertainty, skipped work, and
next owner. Use `PASS`, `FAIL`, `INCONCLUSIVE`, or `NOT_RUN` for execution
results.

## Stop Conditions

Stop and report rather than improvise when:

- the selected slice violates an active steering gate without a documented
  exemption tied to a named roadmap/release requirement;
- the selected slice requires a decision that changes another milestone;
- legacy evidence is unavailable or materially contradictory;
- implementation would violate a documented architecture invariant;
- verification cannot support a requested completion claim;
- concurrent changes overlap the same source-of-truth documents;
- a required delegated result failed or conflicts with another result and
  source evidence cannot resolve the conflict.

## Validation

- Exactly one implementation slice is active in `SPEC.md`.
- The slice identifies steering-gate impact or exemption.
- Every completed roadmap checkbox has repository or remote evidence.
- Current and planned architecture are clearly distinguished.
- Documentation, tests, and implementation describe the same behavior.
- Rejected-command atomicity and replay-semantic impact are checked where
  applicable.
- Delegation, when used, has one synthesis owner and no overlapping writes.
- Unsupported checks are `NOT_RUN`; missing evidence remains `INCONCLUSIVE`.
- `sh scripts/check-repository.sh` succeeds.

## Browser Slice Rules

- Treat browser/WASM presentation as an effect boundary: `drl-core` and
  `drl-protocol` remain free of GPU, Web Audio, DOM, filesystem, and wall-clock
  dependencies.
- Record WASM target, Rust/tool versions, browser version, GPU adapter/backend,
  viewport, DPR, and audio state.
- Require player-observation parity with direct headless execution.
- Asset imports require a source revision, dirty-state record, license/
  attribution, and checksum manifest. Unclear rights are
  `INCONCLUSIVE`/`NOT_RUN`, never silently bundled.
