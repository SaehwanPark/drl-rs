# Specification

Last reviewed: 2026-09-07
Current project version: `0.2.352`

The roadmap owns milestone scope and ordering. This file expands exactly one
active implementation slice; delivered history belongs in the roadmap,
changelog, evidence notes, and Git.

## 1. Status vocabulary

- `[x]` — delivered and verified by checked implementation and evidence.
- `[ ]` — open and required by the active slice.
- `NOT_RUN` — prerequisites unavailable; no pass or failure is inferred.
- `INCONCLUSIVE` — available evidence cannot support the claim.

## 2. Active implementation slice: browser asset-pack diagnostics (M12)

Slice status: **delivered and verified locally**. This is a bounded named M12
release-hardening slice selected after the audit remediation and
release-readiness summary work. It improves the public browser entry point
without changing simulation semantics.

### 2.1 Objective

Report the presence of required graphics and optional HQ/LQ audio and bitmap
font packs during browser startup. Missing optional packs must state the
procedural-audio and DOM/browser-text fallbacks; missing required graphics must
remain visible as a startup problem rather than an implied release pass.

### 2.2 Scope and acceptance

- [x] Add a small browser-only asset-pack probe with explicit pack paths,
  required/optional classification, safe fetch failure handling, and stable
  user-facing summary text.
- [x] Show the summary in an accessible startup status region without stealing
  focus or blocking gameplay; preserve existing graphics/audio diagnostics.
- [x] Add deterministic mocked-fetch fixtures for all-present, optional-missing,
  required-missing, and fetch-failure states.
- [x] Run the fixture through `sh scripts/check-web.sh` and keep release-rights
  policy unchanged: optional legacy audio/fonts remain external/unbundled.
- [x] Preserve core, replay, RNG, MCP, save, service-worker, and gameplay
  behavior; advance the code version exactly once from `0.2.351` to `0.2.352`.

### 2.3 Non-goals

No audio decoding, font rendering, asset bundling, production HTTPS/PWA
installation, WCAG certification, legacy audiovisual parity, or native frontend
change is included. A probe result is a diagnostic, not proof that an optional
pack is rights-cleared or that a browser environment is supported.

### 2.4 Delivery evidence

- `node scripts/test-asset-status.mjs`: PASS for all-present,
  optional-missing, required-missing, network failure, and unavailable-fetch
  states.
- `sh scripts/check-web.sh`: PASS, including static service-worker/manifest,
  browser diagnostics, assets/render/audio/web tests, and headless Chrome.
- `sh scripts/check-service-worker.sh`, `sh scripts/check-browser-diagnostics.sh`,
  `node scripts/test-browser-controls.mjs`, `cargo fmt --all -- --check`,
  `sh scripts/check-spec-structure.sh`, and `git diff --check`: PASS.
- `DRL_VERSION_BASE=10ebd81 sh scripts/check-version.sh`: PASS for the exact
  `0.2.351` -> `0.2.352` code transition.
- Release-rights policy remains unchanged: optional legacy audio and fonts are
  detected but not copied into the distributable bundle.
- Unavailable runtime, reference-capture, audiovisual, native, and human
  acceptance remain `NOT_RUN`.

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
