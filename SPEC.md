# Specification

Last reviewed: 2026-08-22
Current project version: `0.2.17`

The [Roadmap](docs/DRL-Rust_Project_Roadmap.md) owns overall milestone scope,
ordering, and delivery tracking. This file expands **exactly one active
implementation slice** into observable behavior, pure contracts, acceptance
criteria, and verification boundaries.

---

## 1. Status Vocabulary

- `[x]` — **Delivered and Verified**: Supported by checked repository evidence,
  tests, or artifacts.
- `[ ]` — **Present or Future Work**: Open implementation task or acceptance
  gate.
- `NOT_RUN` — **Environment Unavailable**: Required prerequisites (e.g., Linux
  x86-64 binary or browser WebGPU) were not present; not an inferred pass.
- `INCONCLUSIVE` — **Unresolved Evidence**: Output exists but cannot
  definitively satisfy criteria without additional evidence.

---

## 2. Active Implementation Slice: M9 Legacy Content Evidence Converter

### 2.1 Scope & Objective

Provide a build-time, dependency-light extractor for the shallow declarative
fields in pinned legacy Lua content records. The converter emits a provenance-
bearing JSON table and records nested tables/functions as explicit migration
gaps. It is not a Lua interpreter and does not infer behavior.

### 2.2 Predecessor Foundation (Delivered Slices)

1. **Typed Rust content (v0.2.4–v0.2.13)**:
   - Current monster, item, tile, loot, and level definitions are owned by
     Rust and tested independently of legacy runtime code.
2. **Build-time boundary (ADR 0008)**:
   - Legacy files are pinned research inputs; the browser ships no Lua VM or
     legacy object model, and unknown behavior remains an explicit gap.

### 2.3 Present Slice Acceptance Criteria

- [x] **Pinned source**: Read a specified legacy Git revision (defaulting to the
  recorded DRL revision) and include path, revision, and SHA-256 provenance.
- [x] **Declarative extraction**: Extract scalar fields from shallow
  `register_being` and `register_item` records in deterministic ID order.
- [x] **Explicit gaps**: Preserve nested tables and function-valued fields as
  named migration gaps instead of dropping or guessing them.
- [x] **Fixture coverage**: Verify being/item records, scalar types,
  provenance, deterministic ordering, and gap retention without a Lua runtime.
- [ ] **Full content migration**: Expand typed coverage and validate behavior,
  assets, and fairness for all legacy content families.

### 2.4 Pure Contract

- **Input**: A pinned legacy Git revision and one Lua source path, or an explicit
  fixture input for conversion tests.
- **Output**: JSON with schema version, source provenance, sorted scalar fields,
  and explicit migration-gap entries.
- **Ownership Boundary**:
  - `scripts/convert-legacy-content.py` owns build-time extraction only.
  - `drl-core` remains the gameplay authority; conversion output is reviewed
    input, not a runtime dependency.
  - The browser bundle receives no Lua source, interpreter, or legacy object
    model.

---

## 3. Recent Delivered Slices

### M9 — Legacy Content Evidence Converter (`VERSION` 0.2.17)

- [x] Added pinned-source extraction for shallow being/item scalar records.
- [x] Added provenance fields and explicit nested/function migration gaps.
- [x] Added deterministic fixture coverage without a Lua runtime.
- [ ] Full content migration and behavior validation remain open.

### M11 — Cohort Study CLI (`VERSION` 0.2.16)

- [x] Added bounded deterministic `drl-rust cohort` reports for three bots.
- [x] Added stable metadata, outcome, and telemetry line fields with validation.
- [x] Added repeatability and invalid-option contract tests.
- [ ] Canonical difficulty targets and statistical interpretation remain open.

### M10 — Offline-Cache Readiness (`VERSION` 0.2.15)

- [x] Started service-worker registration independently of WebGPU startup.
- [x] Added explicit unavailable/installing/ready/failure diagnostics.
- [x] Added injected-capability tests and bootstrap ordering checks.
- [ ] Real browser offline installation and reload acceptance remain open.

### M12 — Detached Release Signing (`VERSION` 0.2.14)

- [x] Added optional detached manifest signing and public-key derivation.
- [x] Added fail-closed verification when signature artifacts are present.
- [x] Added ephemeral-key shell coverage for tamper rejection.
- [ ] Key custody, CI enforcement, and production trust-root governance remain
  open.

### M11 — Cohort Telemetry Integrity (`VERSION` 0.2.13)

- [x] Added typed telemetry-invariant diagnostics for shot counts, level
  identity, and configured turn budgets.
- [x] Enforced the invariants before outcome or telemetry projection without
  mutating reports or accessing player observations.
- [x] Added focused integration coverage for each rejected metric shape.
- [ ] Automated large-scale balance and canonical difficulty studies remain
  open.

### M10 — Browser-Save Corruption Recovery (`VERSION` 0.2.12)

- [x] Quarantined malformed, unsupported, oversized, and replay-invalid save
  values in a bounded browser-owned diagnostic slot.
- [x] Cleared rejected active values when storage permits and kept boot/load
  playable when storage access or cleanup fails.
- [x] Preserved transactional restore and tested active-session immutability.
- [ ] Explicit replay-compatible migration for recognized older formats.

### M8 — Particle-Decal Renderer Integration (`VERSION` 0.2.11)

- [x] Resolved opaque caller-provided sprite handles through presentation-only
  descriptor tables without guessing a legacy blood atlas or slot.
- [x] Preserved stored-pixel sub-cell placement and inserted decals between
  terrain and ordinary objects in renderer-neutral plans.
- [x] Added browser WebGPU entry points and native ordering/immutability tests;
  capture-backed visual parity remains `NOT_RUN`.

### M8 — Particle-Decal Storage Boundary (`VERSION` 0.2.10)

- [x] Implemented `ParticleDecalStore` with caller-configured maximum capacity.
- [x] Preserved insertion order and duplicate requests deterministically.
- [x] Explicit `CapacityExceeded` error reporting without dropping prior
  entries.
- [x] Comprehensive test coverage for empty, append, duplicate, and capacity
  limits.

### M8 — Particle-Decal Insertion Request (`VERSION` 0.2.9)

- [x] Packaged accepted placement coordinates with caller-provided sprite ID.
- [x] Reused checked placement and eligibility math without duplicate logic.
- [x] Preserved legacy cell, pixel offset, and sprite ID representations.

### M12 — Release Hardening & Integrity (`VERSION` 0.2.1)

- [x] Git checkout-identity binding in release manifest generator.
- [x] Release manifest SHA-256 sidecar (`release-manifest.sha256`).
- [x] Versioned service-worker cache invalidation based on version and commit.
- [x] Mocked service worker lifecycle and fetch contract test harness.
- [x] Static HTML shell accessibility audit (landmarks, focus, live regions).

### M11 — Cohort Telemetry & Outcome Projections (`VERSION` 0.1.1–0.2.3)

- [x] Fixed-seed cohort reports (`CohortConfig`, `CohortReport`) with integrity
  validation.
- [x] Cohort outcome distributions (victory, death, turn limit, stalled, in
  progress).
- [x] Pure outcome-rate and telemetry tolerance gates for regression detection.
- [x] Descriptive telemetry metrics: shot accuracy, damage dealt/taken, kills,
  pickups, and item usage.

---

## 4. Shared Observable Behavior & Invariants

### 4.1 Simulation & Fairness

- **Information Decoupling**: `BrowserSession` and MCP tools expose only fair
  `PlayerObservation` and `GameEvent` streams. Hidden world state is never
  accessible.
- **Replay Determinism**: Identical seed and command sequences produce bit-exact
  identical events and end states across headless, MCP, and browser runners.
- **Transactional Rollback**: Rejected or illegal commands roll back session
  state without advancing the simulation turn or consuming energy.

### 4.2 Browser & WebGPU Presentation

- **Input Translation**: Browser shell maps keyboard/numpad, wait, pickup,
  reload, descend, ranged targeting, and DOM inventory clicks directly to
  protocol `Command`s.
- **No Simulation Side Effects**: Canvas resize, DPR change, animation tick,
  audio playback, tab visibility change, or WebGPU device loss never advance
  simulation time.
- **Square-Cell Viewport**: Integer scaling and centering ensure square game
  tiles without non-uniform axis stretching.
- **Visibility-Derived Lighting**: Full light for tiles in active FOV; fixed fog
  factor for remembered explored tiles. Presentation never queries hidden
  tiles.

### 4.3 Evaluation & Telemetry

- **Contiguous Seeds**: Cohort reports record contiguous seed ranges and
  preserve per-seed replay logs.
- **Descriptive Metrics**: Projections report descriptive counts and rates; they
  do not assert balance conclusions or statistical significance.

### 4.4 Release Packaging

- **Manifest Validation**: Release scripts verify manifest schema, sorted
  SHA-256 hashes, license metadata, worker coverage, sidecar digests, and source
  commit identity.

---

## 5. Public Contracts & Boundaries

### 5.1 Protocol & Simulation (`drl-protocol`, `drl-core`)

- **Domain Types**: `Position`, `Direction`, `Turn`, `EntityId`, `ItemId`,
  `LevelId`, `HitPoints`, `Speed`, `ActionCost`.
- **Commands**: `Command::Move`, `Command::Wait`, `Command::Pickup`,
  `Command::Drop`, `Command::Equip`, `Command::Unequip`, `Command::Reload`,
  `Command::Use`, `Command::AttackMelee`, `Command::AttackRanged`,
  `Command::Descend`.
- **Observations**: `PlayerObservation` (FOV tiles, visible actors, ground
  items, inventory, equipment, player HP, map dimensions).
- **Events**: `GameEvent::AttackResolved`, `GameEvent::DamageApplied`,
  `GameEvent::ActorDied`, `GameEvent::ActorKnockedBack`,
  `GameEvent::PlayerTeleported`, `GameEvent::LevelTransitioned`, etc.

### 5.2 Presentation & Assets (`drl-render`, `drl-assets`, `drl-web`)

- **Layout**: `PixelViewport`, `PixelRect`, integer cell scaling.
- **Shading & Tone**: `LightingBand`, `shade_color`, `SceneTone`,
  `low_health_pulse_target_alpha`, `LowHealthPulseState`.
- **Draw Plans**: `layer_draw_plan`, `sprite_composite_plan`, `LayerRole`,
  `AtlasTextureSource`.
- **Decals & Particles**: `ParticleDecalPlacement`, `ParticleDecalInsertion`,
  `ParticleDecalStore`, burst origins/directions/ranges.

### 5.3 Decoupling Invariants

- `drl-core` and `drl-protocol` have **zero** dependencies on WebGPU, Web
  Audio, DOM, filesystem, network, or MCP crates.

---

## 6. Verification Gates

### Verified Baseline (`VERSION` 0.2.17)

f305503 feat(eval): add deterministic cohort study CLI (#107)
22e0e8b feat(web): start offline cache registration at bootstrap (#106)
af0fcf8 feat(release): add optional manifest signing (#105)
- [x] `sh scripts/check-repository.sh` — Full repository test suite, formatting,
  clippy, and harness checks.
- [x] `sh scripts/check-assets.sh` — 32 imported legacy PNGs, licensing, and
  SHA-256 checksums.
- [x] `scripts/test-reference-capture.sh` — Preflight fixture test suite for
  capture manifests.
- [x] `sh scripts/check-web.sh` — WASM target compilation and native/WASM web
  contract tests.
- [x] `sh scripts/build-web.sh && sh scripts/check-release-manifest.sh` — Static
  bundle build and 44-artifact manifest validation.
- [x] `scripts/check-version.sh` — Project version consistency and transition
  rules.

### Environment-Gated Verification

- [ ] Controlled Linux x86-64 reference captures (`NOT_RUN` on macOS arm64).
- [ ] Direct visual/audio diff against legacy reference captures (`NOT_RUN`).

---

## 7. Explicit Non-Goals

- **Pre-1.0 Multiplatform**: No WebGL2 fallback, mobile/touch UI, gamepad
  navigation, or native desktop packaging before the 1.0 desktop Chromium
  release.
- **No Runtime Scripting**: No Lua VM or JavaScript gameplay logic. All
  simulation is pure compiled Rust.
- **No Online Services**: No player accounts, multiplayer networking, or
  remote backend servers.
- **No Unattributed Assets**: No bundling of legacy audio, music, or fonts
  without documented redistribution rights.
- **No False Parity Claims**: Pure mathematical contracts and generated audio
  tones are not claimed as full legacy audiovisual equivalence until validated
  against reference captures.
