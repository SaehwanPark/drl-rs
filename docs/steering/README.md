# Development Steering

Last reviewed: 2026-09-02
Repository baseline reviewed: `9df938f`
Latest pull request inspected: `#454`
Project version at review: `0.2.338`

## Purpose

`docs/steering/` contains the current steering layer for drl-rs: temporary
priority constraints, architecture decisions that need to guide near-term work,
and audit evidence that explains why those constraints exist.

These files are intentionally separate from the long-lived roadmap and ADR
history. They should make future development easier to steer without turning a
single audit into a permanent second roadmap.

## Authority and reading order

Use the repository documents in this order:

1. **Accepted ADRs and verified `ARCHITECTURE.md`** define enduring architecture
   and implemented invariants.
2. **`docs/DRL-RS_Project_Roadmap.md`** owns milestone scope, ordering, exit
   criteria, and progress tracking.
3. **`docs/steering/current-priorities.md`** constrains which candidate work
   should become active while the listed stop gates remain open.
4. **`SPEC.md`** expands exactly one active implementation slice into observable
   behavior, acceptance criteria, and exclusions.
5. **`CHANGELOG.md`** records delivered changes after evidence exists.
6. **Audit material in this directory** is evidence and rationale, not an
   implementation-status authority.

If two documents conflict, do not silently choose the more convenient one.
Resolve the conflict by updating the enduring document or re-scoping the active
slice before implementation proceeds.

## Current steering set

- [`current-priorities.md`](current-priorities.md) — near-term priority order,
  stop gates, slice-selection rules, and fidelity terminology.
- [`audit-2026-08-30-post-0.2.318.md`](audit-2026-08-30-post-0.2.318.md) —
  checkpointed progression audit that supersedes the 0.2.88 audit for current
  slice selection.
- [`decisions/atomic-command-transactions.md`](decisions/atomic-command-transactions.md)
  — rejected commands must be exact state identity; prefer validate/prepare/
  commit.
- [`decisions/replay-semantics-and-rng-stability.md`](decisions/replay-semantics-and-rng-stability.md)
  — unbiased bounded sampling, golden vectors, and separate replay wire versus
  gameplay semantics versioning.
- [`decisions/content-catalog-and-typed-behavior-model.md`](decisions/content-catalog-and-typed-behavior-model.md)
  — one authoritative compile-time catalog plus a bounded typed Rust behavior
  vocabulary instead of callback sprawl.
- [`decisions/required-review-and-branch-protection.md`](decisions/required-review-and-branch-protection.md)
  — attributable independent determinism-review receipts and required `main`
  branch settings for protected paths.
- [`audit-2026-08-23.md`](audit-2026-08-23.md) — audit findings that motivated
  the earlier 0.2.88 steering wave; retained as historical evidence.
- [`drop-in-manifest.md`](drop-in-manifest.md) — file-level map for the steering/harness overlay.

Legacy behavior evidence lives under [`../legacy-behavior/`](../legacy-behavior/);
the current Gate D stress-case notes are
[`medical-powerarmor.md`](../legacy-behavior/medical-powerarmor.md) and
[`subtle-knife.md`](../legacy-behavior/subtle-knife.md), plus
[`trigun.md`](../legacy-behavior/trigun.md). The delivered Anti-Freak Jackal
Fire-mitigation slice added
[`anti-freak-jackal-fire-profile.md`](../legacy-behavior/anti-freak-jackal-fire-profile.md);
the delivered Null Pointer Plasma-mitigation slice adds
[`null-pointer-plasma-resistance-profile.md`](../legacy-behavior/null-pointer-plasma-resistance-profile.md).
The delivered Rocket Launcher direct-Fire and ground-item slices are documented in
[`rocket-launcher-profile.md`](../legacy-behavior/rocket-launcher-profile.md).
The delivered Standard BFG 9000 direct-Plasma slice is documented in
[`bfg9000-explosion.md`](../legacy-behavior/bfg9000-explosion.md).


## Agent harness

The canonical repository-local agent skill tree remains `.agents/skills/`.
Steering-specific delivery gates live alongside the existing milestone-delivery
skill under `.agents/skills/drl-milestone-delivery/references/`, keeping one
discoverable skill tree and one repository-local workflow authority.

`docs/audit-feedback-20260823.md` is retained as a compatibility pointer to
`docs/steering/audit-2026-08-23.md`; it no longer owns independent steering
content.

## Promotion and retirement

Steering decisions are deliberately easier to add and retire than accepted
ADRs. When a steering decision has been implemented and its architecture is
stable:

- consolidate the durable part into the appropriate accepted ADR and
  `ARCHITECTURE.md`;
- remove temporary stop-gate language that no longer applies;
- keep the audit as historical evidence;
- update the roadmap and `SPEC.md` from verified implementation evidence.

Do not leave obsolete steering gates active after their acceptance criteria are
satisfied.
