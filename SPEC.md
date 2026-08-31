# Specification

Last reviewed: 2026-08-31
Current project version: `0.2.324`
Audited starting checkpoint: `main` at
`22795a70b13b360bb0d94b28e1b591bc30543fd6` (merged PR #435; M0 guard reconciled)
Delivery checkpoint: `main` at
`49add3aecf7886dea40590497132fabe4b56f06b` (merged PR #436; policy delivered)

The [Roadmap](docs/DRL-RS_Project_Roadmap.md) owns milestone scope, ordering,
and progress. [`docs/steering/current-priorities.md`](docs/steering/current-priorities.md)
constrains slice selection while its stop gates remain open. This file expands
**exactly one active implementation slice**. Delivered history belongs in the
roadmap, changelog, evidence notes, and Git rather than accumulating here.

## 1. Status vocabulary

- `[x]` — **Delivered and verified**: supported by checked implementation and
  evidence.
- `[ ]` — **Open**: required by the active slice and not yet delivered.
- `NOT_RUN` — **Environment unavailable**: prerequisites were unavailable; no
  pass or failure is inferred.
- `INCONCLUSIVE` — **Evidence unresolved**: available evidence cannot support
  the claim.

## 2. Active implementation slice: M0 — required review and branch policy

Slice status: **delivered and verified** at the delivery checkpoint above;
live `main` settings were inspected after merge.

### 2.1 Objective

Record and enforce an attributable independent determinism-review receipt for
replay-visible or legacy-fidelity pull requests, then make the `main` branch
require that policy alongside the repository and browser checks. The policy
must remain read-only with respect to pull-request code and must make the
single-maintainer exception explicit while the repository has no second
collaborator.

This is a Gate D control-plane slice. It changes no game behavior, replay
identity, RNG sampling, content catalog, protocol schema, or presentation
boundary.

### 2.2 Audited starting point

At audited starting revision `22795a7` (version `0.2.323`):

- `SPEC.md` has an executable structural guard and a deterministic fixture
  contract; delivered slice history remains outside the active specification.
- Contributors are asked for one approval, but no automated check verifies an
  independent determinism-review receipt on protected changes.
- The GitHub `main` branch is not protected, so required checks and review
  policy are advisory rather than enforced.

### 2.3 Scope and ownership

- **Steering gate:** Gate D — canonical scope and review must remain auditable.
- **Primary owner:** `scripts/check-review-policy.sh` owns pull-request receipt
  validation; `scripts/check-branch-protection.sh` owns the inspectable GitHub
  settings contract; fixture scripts own their bounded positive and negative
  cases; the workflow owns the hosted status check.
- **Project version:** implementation advances `VERSION` from `0.2.323` to
  `0.2.324`.
- **Gameplay/replay semantics:** no gameplay, replay, RNG-sampling, generator,
  ruleset, snapshot, protocol, or content identity changes.

### 2.4 Review and branch contract

- A protected path is any change under `crates/drl-core/`,
  `crates/drl-protocol/`, `crates/drl-mcp/`, `crates/drl-app/`,
  `crates/drl-web/`, `crates/drl-script/`, or `docs/legacy-behavior/`.
- A pull request that changes a protected path passes only when a reviewer
  other than the pull-request author has a current-head `APPROVED` review whose
  body contains the exact receipt `drl-determinism-review: PASS`. Reviews for
  an older head are not current evidence.
- The `Review policy` workflow runs from the base revision with read-only
  permissions on pull-request open, synchronize, reopen, ready-for-review,
  and review-state changes. It reports `NOT_RUN` only when no pull request or
  no local GitHub credentials are available; hosted pull requests fail closed
  if their metadata cannot be inspected.
- The branch checker requires one approving review, stale-review dismissal,
  strict required-status updates, and the `Repository checks`, `WASM browser
  checks`, and `Review policy` contexts on `main`.
- The repository currently has one maintainer. Branch protection therefore
  records `enforce_admins: false` as an explicit temporary exception; external
  contributors still face the required review and status checks, and the
  exception must be revisited before a second maintainer is added or 1.0 is
  declared.
- Local fixture inputs make the policy deterministic without mutating GitHub
  settings or requiring network access.

### 2.5 Acceptance criteria

- [x] `scripts/check-review-policy.sh` identifies protected paths and rejects
  missing, self-authored, stale, or receipt-free review evidence while
  accepting one current independent receipt.
- [x] `scripts/test-review-policy.sh` covers no-protected-change, rejection,
  independent-approval, and latest-review-state fixtures.
- [x] `scripts/check-branch-protection.sh` validates the required review and
  status settings, reports an unprotected branch as a failure, and supports a
  deterministic fixture input.
- [x] `scripts/test-branch-protection.sh` covers the passing settings and each
  required-setting failure.
- [x] `.github/workflows/review-policy.yml`, `.github/pull_request_template.md`,
  and the steering decision document make the receipt and setting contract
  discoverable without executing pull-request code.
- [x] `main` has the required settings applied and the branch checker passes
  against the live GitHub API; the documented solo-maintainer exception is
  visible in the setting evidence.
- [x] The policy has no production-crate dependency and changes no gameplay,
  replay, RNG, protocol, content, or browser behavior.
- [x] Local shell, repository, version, and documentation checks pass; hosted
  `Repository checks`, `WASM browser checks`, and `Review policy` checks pass
  for the reviewed policy workflow revision.

### 2.6 Non-goals

- No attempt to judge the quality of a review beyond the attributable exact
  receipt; the independent reviewer remains responsible for the review record.
- No write access from the hosted workflow, no automatic reviewer assignment,
  and no branch-settings mutation from repository checks.
- No parser for Markdown prose, checkbox counts, or historical roadmap entries.
- No changes to Rust crates, gameplay semantics, replay formats, or runtime
  behavior.

### 2.7 Evidence boundary

The policy scripts prove only the declared receipt and branch-setting shape
against the supplied pull-request or API metadata. They do not prove review
quality, semantic correctness of the reviewed change, or completion of any
roadmap item beyond the checked control-plane contract.

## 3. Enduring invariants

The active slice must preserve:

1. no ambient state, platform APIs, filesystem, browser, or presentation policy
   in `drl-core`;
2. identical declared seed, commands, and semantics produce identical current
   simulation results;
3. incompatible histories fail explicitly before simulation;
4. rejected commands and rejected restores do not partially mutate authoritative
   simulation state;
5. renderers, browser code, MCP, and bots consume fair observations/events and
   do not inspect hidden core state;
6. presentation timing and storage side effects do not advance gameplay;
7. no runtime Lua or generic callback recreation;
8. current-Rust, cross-version, legacy, browser, audiovisual, and performance
   evidence remain separately labeled.
