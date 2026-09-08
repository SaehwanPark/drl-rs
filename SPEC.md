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

## 2. Active implementation slice: executable-module version classification (audit F3)

Slice status: **delivered and verified locally**. This is the third
remediation slice recommended by `docs/project-audit-2026-09-07.md` and is
bounded to version-policy classification and its fixture coverage. F1 and F2
are delivered in commits `0af1bed` and `ae8a451`.

### 2.1 Objective

Ensure shipped executable JavaScript modules with the `.mjs` suffix are treated
as code by `scripts/check-version.sh`. A runtime `.mjs` change must require
exactly one valid version transition, while documentation-only and
settings-only changes remain valid without a bump.

### 2.2 Scope and acceptance

- [x] Classify `.mjs` as executable code in the version checker.
- [x] Add repeatable temporary-Git fixtures proving an `.mjs` behavior change
  fails without a bump and passes with exactly one allowed transition.
- [x] Cover ordinary Rust and shell code as code, and documentation/settings
  changes as non-code; reject an unnecessary bump for non-code changes.
- [x] Run the fixture suite through the repository verification path without
  changing gameplay, replay, RNG, MCP, browser behavior, or release metadata.

### 2.3 Non-goals

No version-policy redesign, semantic-version carry behavior change, package
release, browser productization, or gameplay/content work is part of this slice.
The release-readiness gaps and independent review operating model remain open
outside F3.

### 2.4 Delivery evidence

- `sh scripts/test-version.sh`: PASS for `.mjs`, Rust, shell,
  documentation, settings, no-bump, exact-bump, and over-bump fixtures.
- `DRL_VERSION_BASE=ae8a451 sh scripts/check-version.sh`: PASS for the exact
  `0.2.350` -> `0.2.351` code transition.
- `sh scripts/check-agent-harness.sh`: PASS; the fixture suite is now part of
  the harness verification path.
- `sh scripts/check-spec-structure.sh`, `cargo fmt --all -- --check`, and
  `git diff --check`: PASS.
- No gameplay, replay, RNG, MCP, browser behavior, or release metadata changed
  beyond the required canonical version projection.
- Unavailable native, browser-capture, controlled-legacy, audiovisual, and
  human acceptance remain `NOT_RUN`.

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
