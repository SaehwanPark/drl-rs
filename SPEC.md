# Specification

Last reviewed: 2026-09-07
Current project version: `0.2.351`

The roadmap owns milestone scope and ordering. This file expands exactly one
active implementation slice; delivered history belongs in the roadmap,
changelog, evidence notes, and Git.

## 1. Status vocabulary

- `[x]` — delivered and verified by checked implementation and evidence.
- `[ ]` — open and required by the active slice.
- `NOT_RUN` — prerequisites unavailable; no pass or failure is inferred.
- `INCONCLUSIVE` — available evidence cannot support the claim.

## 2. Active implementation slice: public release-readiness entry point (audit feedback)

Slice status: **delivered and verified locally**. This bounded documentation
slice follows the audit's recommendation to make M12/M13 release gaps visible
at the product entry point. The audit remediation findings F1/F2/F3 are
delivered in commits `0af1bed`, `ae8a451`, and `578702e`.

### 2.1 Objective

Publish one compact, linked release-readiness summary that states what can be
run today, which public-release gates remain open or `NOT_RUN`, and which checks
support each claim. Correct the README workspace count from nine to ten crates
without duplicating a second roadmap or acceptance checklist.

### 2.2 Scope and acceptance

- [x] Add `docs/release-readiness.md` with current version, supported entry
  points, verified local evidence, open M12/M13 blockers, and explicit
  `NOT_RUN`/`INCONCLUSIVE` labels.
- [x] Link the summary from the README and documentation portal.
- [x] Correct the README architecture statement to ten workspace crates.
- [x] Keep the summary compact and linked to the canonical roadmap, audit,
  release-rights policy, and browser acceptance records rather than copying
  their full checklists.
- [x] Preserve code, gameplay, replay, RNG, MCP, browser behavior, and version;
  documentation-only changes did not bump `VERSION`.

### 2.3 Non-goals

No deployment, signing-key custody, WCAG or screen-reader certification,
audiovisual/reference capture, external MCP compatibility, native interactive
acceptance, or gameplay/content implementation is claimed or delivered here.
Those remain named release gates in the roadmap.

### 2.4 Delivery evidence

- `DRL_VERSION_BASE=578702e sh scripts/check-version.sh`: PASS with no version
  transition for the documentation-only diff.
- `sh scripts/check-spec-structure.sh` and `git diff --check`: PASS.
- README, portal, roadmap, audit, release-rights, and browser-acceptance links
  were checked against the repository paths; the summary labels the full-suite
  and hosted/runtime gaps `INCONCLUSIVE` or `NOT_RUN` rather than claiming
  release readiness.
- No code, gameplay, replay, RNG, MCP, browser behavior, or release metadata
  changed.

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
