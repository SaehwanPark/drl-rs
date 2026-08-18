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

## Workflow

1. Select one milestone and the smallest coherent checklist slice that can be
   implemented and verified without unrelated work.
2. Inspect the current code, tests, documents, and relevant legacy evidence.
   Record uncertainty rather than inferring unsupported behavior.
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
7. Reconcile documentation from evidence:
   - update architecture only for changed, verified structure or invariants;
   - move completed specification outcomes out of active state only after
     verification;
   - add meaningful changelog entries;
   - mark roadmap tasks complete only when their stated result exists;
   - leave remote-CI criteria incomplete until the remote run passes.
8. Review the final diff for contradictions, accidental scope growth, and
   claims that exceed test or inspection evidence.

## Outputs

- an updated bounded slice in `SPEC.md`
- implementation and focused tests when the slice requires code
- targeted updates to `ARCHITECTURE.md`, `CHANGELOG.md`, and the roadmap
- a handoff reporting files changed, checks run, deviations, and unresolved
  risks

Write canonical outputs directly to the repository. Do not create `_workspace/`
handoffs unless later coordination requirements explicitly justify them.

## Stop Conditions

Stop and report rather than improvise when:

- the selected slice requires a decision that changes another milestone;
- legacy evidence is unavailable or materially contradictory;
- implementation would violate a documented architecture invariant;
- verification cannot support a requested completion claim;
- concurrent changes overlap the same source-of-truth documents.

## Validation

- Exactly one implementation slice is active in `SPEC.md`.
- Every completed roadmap checkbox has repository or remote evidence.
- Current and planned architecture are clearly distinguished.
- Documentation, tests, and implementation describe the same behavior.
- `sh scripts/check-repository.sh` succeeds.
