# Specification

Last reviewed: 2026-09-07
Current project version: `0.2.353`

The roadmap owns milestone scope and ordering. This file expands exactly one
active implementation slice; delivered history belongs in the roadmap,
changelog, evidence notes, and Git.

## 1. Status vocabulary

- `[x]` — delivered and verified by checked implementation and evidence.
- `[ ]` — open and required by the active slice.
- `NOT_RUN` — prerequisites unavailable; no pass or failure is inferred.
- `INCONCLUSIVE` — available evidence cannot support the claim.

## 2. Active implementation slice: attributable review exception process (M0/M13)

Slice status: **delivered and verified locally**. This bounded control-plane
slice follows audit recommendation 1: make the temporary solo-maintainer review
exception explicit, attributable, and time-bounded before selecting another
protected gameplay slice. F1/F2/F3 and the M12 asset diagnostics slice are
delivered locally.

### 2.1 Objective

Document and enforce the information required when an eligible independent
reviewer is unavailable: reason, responsible maintainer, current head, expiry,
and follow-up owner. Keep the normal independent-review receipt mandatory for
protected paths; the exception is not a passing review and cannot silently
satisfy the hosted policy.

### 2.2 Scope and acceptance

- [x] Add an explicit exception section to the required-review decision with a
  fixed expiry of `2026-12-31`, a named accountable role, and required evidence.
- [x] Add PR-template fields for exception reason, current head, expiry, and
  reviewer-recruitment follow-up; preserve the exact independent receipt.
- [x] Link the process from the compact release-readiness summary and steering
  status, clearly distinguishing `PASS`, `INCONCLUSIVE`, and exception use.
- [x] Add fixture/document checks so the exception cannot be documented as an
  independent approval and the expiry remains visible.
- [x] Preserve code, gameplay, replay, RNG, MCP, and browser behavior; the
  document-check fixture is a code-path change and advances `VERSION` exactly
  once from `0.2.352` to `0.2.353`.

### 2.3 Non-goals

No GitHub branch-protection mutation, remote reviewer recruitment, gameplay or
content change, claim of hosted CI success, or conversion of an exception into
an independent determinism-review receipt is included.

### 2.4 Delivery evidence

- `sh scripts/test-review-policy.sh`: PASS; existing policy fixtures still
  distinguish independent receipts, stale reviews, author self-approval, and
  the new exception-document contract.
- `sh scripts/check-agent-harness.sh`: PASS, including version and policy
  fixtures.
- `DRL_VERSION_BASE=6f370b1 sh scripts/check-version.sh`: PASS for the exact
  `0.2.352` -> `0.2.353` code transition.
- `sh scripts/check-spec-structure.sh`, `cargo fmt --all -- --check`, and
  `git diff --check`: PASS.
- The decision, PR template, steering status, and release-readiness summary
  all identify the accountable role and `2026-12-31` expiry; the exception is
  explicitly `INCONCLUSIVE` and never a review receipt.
- Remote GitHub settings, eligible reviewer recruitment, and hosted CI remain
  `NOT_RUN`.

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
6. presentation timing, resize, scale factor, surface loss, and storage side
   effects do not advance gameplay;
7. no runtime Lua or generic callback recreation;
8. current-Rust, cross-version, legacy, browser, audiovisual, and performance
   evidence remain separately labeled.
