# Steering Decision — Required Review and Branch Protection

**Status:** Active Gate D control-plane constraint; candidate for future ADR
consolidation

**Date:** 2026-08-31

---

## Context

The repository has a deterministic core and a review workflow that already
asks contributors for one approval. Before this decision, that convention was
not machine-checkable, and `main` had no branch protection. A replay-visible or
legacy-fidelity change could therefore merge without an attributable
independent determinism review or the ordinary repository/browser checks.

The repository currently has one maintainer, so a policy that enforces reviews
for administrators would prevent the maintainer from merging any change. The
exception must be explicit, narrow, and revisited before 1.0 or before a second
maintainer is added.

## Decision

### 1. Protected-path receipt

The protected paths are:

- `crates/drl-core/`
- `crates/drl-protocol/`
- `crates/drl-mcp/`
- `crates/drl-app/`
- `crates/drl-web/`
- `crates/drl-script/`
- `docs/legacy-behavior/`

The hosted `Review policy` check reads pull-request metadata with a read-only
token. For a pull request changing one of these paths, the latest submitted
review by each reviewer is considered. At least one reviewer other than the
author must have state `APPROVED` and a body containing the exact receipt:

```text
drl-determinism-review: PASS
```

A later `CHANGES_REQUESTED` or other review state from that reviewer removes
the receipt until the reviewer approves the current revision again. The check
does not judge review quality; the independent reviewer owns that judgment and
the linked review artifact.

### 2. Hosted execution boundary

`.github/workflows/review-policy.yml` runs from the pull-request base revision
on pull-request and review-state events. It checks out no pull-request code,
uses `contents: read` and `pull-requests: read`, and never writes repository or
branch settings. Missing metadata fails on hosted runs; local invocations may
report `NOT_RUN` when GitHub credentials are unavailable.

### 3. Main branch settings

`main` requires:

- at least one approving pull-request review;
- dismissal of stale approvals;
- strict required-status updates;
- `Repository checks`, `WASM browser checks`, and `Review policy` statuses.

The live settings intentionally use `enforce_admins: false` while there is one
maintainer. This is a temporary solo-maintainer exception, not a waiver for
external contributors. `scripts/check-branch-protection.sh` inspects the live
API response and accepts only this documented setting plus the required review
and status controls.

### 4. Deterministic fixtures

Both policy scripts accept explicit fixture environment variables, and their
fixture scripts run as part of the repository contract. Fixture execution is
read-only and does not call GitHub or mutate branch settings.

## Consequences

- Protected-path pull requests carry an attributable, searchable review
  receipt, and the required hosted status is branch-protection-compatible.
- Documentation-only and control-plane changes can still be checked without
  inventing a determinism review claim when they do not touch protected paths.
- The solo-maintainer exception is visible in code, documentation, and live
  setting evidence and must be removed or revisited before 1.0.
- The policy checks review metadata and settings only; they do not establish
  gameplay parity, replay compatibility, browser behavior, or review quality.
