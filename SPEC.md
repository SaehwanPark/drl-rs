# Specification

Last reviewed: 2026-08-31
Current project version: `0.2.323`
Audited starting checkpoint: `main` at
`cdb3660035576c23b88e0b8fa7473781d3161467` (merged PR #433; Gate C records reconciled)
Delivery checkpoint: `main` at
`e598ae2365a6c610f8a181d74e6f773f30c9d2f4` (merged PR #434; SPEC guard delivered)

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

## 2. Active implementation slice: M0 — SPEC structural guard

Slice status: **delivered and verified** at the delivery checkpoint above.
The next M0 policy slice will be selected after this reconciliation; this
section remains the bounded specification for the just-delivered structural
guard.

### 2.1 Objective

Add a repository-level structural check that keeps `SPEC.md` as one bounded
active implementation slice instead of allowing delivered slices to accumulate
as a historical ledger. The check must validate the canonical top-level shape,
reject duplicate or extra slice sections, and run as part of the repository
contract before implementation work is accepted.

This is a Gate D control-plane slice. It changes no game behavior, replay
identity, RNG sampling, content catalog, protocol schema, or presentation
boundary.

### 2.2 Audited starting point

At audited starting revision `cdb3660` (version `0.2.322`):

- `SPEC.md` documents one active implementation slice and keeps delivered
  history in the roadmap, changelog, evidence, and Git.
- `scripts/check-repository.sh` runs the existing harness and contract checks,
  but no checker enforces the `SPEC.md` top-level shape.
- The roadmap keeps the remaining M0 item open: add a structural repository
  check that prevents a historical multi-slice SPEC ledger.

### 2.3 Scope and ownership

- **Steering gate:** Gate D — canonical scope and review must remain auditable.
- **Primary owner:** `scripts/check-spec-structure.sh` owns the structural
  contract; `scripts/test-spec-structure.sh` owns its positive and negative
  fixture cases; `scripts/check-repository.sh` invokes both.
- **Project version:** implementation advances `VERSION` from `0.2.322` to
  `0.2.323`.
- **Gameplay/replay semantics:** no gameplay, replay, RNG-sampling, generator,
  ruleset, snapshot, protocol, or content identity changes.

### 2.4 Structural contract

- The canonical `SPEC.md` has exactly these level-two headings, in order:
  `1. Status vocabulary`, `2. Active implementation slice: ...`, and
  `3. Enduring invariants`.
- CommonMark's up-to-three-leading-space ATX heading form is recognized, so
  indentation cannot bypass the guard.
- Exactly one level-two heading begins with `## 2. Active implementation
  slice:`; any second active-slice heading or any extra level-two section is
  rejected as a possible historical ledger.
- Subsections under the active slice remain allowed, but a second active-slice
  marker at any heading depth is rejected.
- The checker accepts a caller-provided path for isolated fixture tests and
  reports failures without mutating the repository.

### 2.5 Acceptance criteria

- [x] `scripts/check-spec-structure.sh SPEC.md` accepts the canonical shape and
  rejects duplicate active slices, extra top-level history sections, and nested
  active-slice markers.
- [x] `scripts/test-spec-structure.sh` exercises the accepted fixture and each
  rejection case with deterministic, bounded temporary files.
- [x] `scripts/check-repository.sh` invokes the structural checker and its
  fixture contract before the broader repository checks.
- [x] The check is shell/POSIX-only, has no production-crate dependency, and
  does not alter gameplay, replay, RNG, protocol, content, or browser behavior.
- [x] M0 roadmap/steering records identify the delivered guard while the
  independent-review/branch-protection policy item remains explicitly open.
- [x] Local format, check, test, clippy, repository, and version checks pass;
  hosted checks pass for the reviewed merge revision.

### 2.6 Non-goals

- No enforcement of GitHub branch protection or review permissions in this
  slice; that is the separate remaining M0 checklist item.
- No parser for Markdown prose, checkbox counts, or historical roadmap entries.
- No changes to Rust crates, gameplay semantics, replay formats, or runtime
  behavior.

### 2.7 Evidence boundary

The checker proves only the declared structural shape of the selected
`SPEC.md` path and its fixture cases. It does not prove review enforcement,
branch protection, semantic correctness of slice content, or completion of any
roadmap item beyond the checked structural contract.

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
