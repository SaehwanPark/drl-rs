# Specification

Last reviewed: 2026-08-22
Current project version: `0.2.29`

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

## 2. Active Implementation Slice: M9 Representative Content-Evidence Coverage

### 2.1 Scope & Objective

Validate the existing pinned legacy-content converters with a reviewed
crosswalk. The gate checks source provenance, SHA-256 shape, deterministic
ordering/uniqueness, representative being/item/cell IDs, and complete coverage
of all 26 indexed special-level IDs. It also synchronizes those IDs with the
Rust descriptive catalog. It records evidence coverage only; it does not
import runtime Lua behavior or assert gameplay parity.

### 2.2 Predecessor Foundation (Delivered Slices)

1. **Pinned content converters (v0.2.17–v0.2.23)**:
   - Build-time converters already extract shallow scalar records for beings,
     item families, terrain cells, and special levels with provenance and
     explicit migration gaps.
2. **Typed Rust content boundary**:
   - Current Rust definitions and the descriptive special-level catalog remain
     runtime-owned; this slice adds only a validation gate over evidence.

### 2.3 Present Slice Acceptance Criteria

- [x] **Reviewed crosswalk**: `docs/content/evidence-coverage.json` pins the
  legacy revision, source paths, record counts, representative IDs, and the
  complete 26-level ID catalog.
- [x] **Provenance validation**: Every bundle source carries the pinned
  revision and a non-empty 64-character SHA-256 digest.
- [x] **Deterministic record validation**: Bundles are sorted and duplicate-free;
  source indices remain within the declared provenance list.
- [x] **Representative coverage**: Being, item-family, and terrain-cell bundles
  cover the reviewed current representatives; the level bundle matches all 26
  indexed special-level IDs.
- [x] **Rust catalog synchronization**: The reviewed level IDs match the
  `SPECIAL_LEVEL_DEFINITIONS` source IDs exactly.
- [x] **Fixture rejection coverage**: Validator tests reject duplicate IDs,
  unsorted records, missing representatives, and wrong revisions.
- [ ] **Full typed content migration and behavior validation**: Nested tables,
  callbacks, symbolic fields, assets, and gameplay semantics remain explicit
  migration gaps.

### 2.4 Pure Contract

- **Input**: Four provenance-bearing JSON evidence bundles and the reviewed
  crosswalk.
- **Output**: PASS only when pinned sources, deterministic records, reviewed
  representatives, and special-level coverage all match; otherwise a bounded
  diagnostic failure.
- **Ownership Boundary**:
  - `scripts/convert-legacy-content-bundle.py` and
    `scripts/convert-legacy-level-index.py` own extraction.
  - `scripts/check-content-evidence.py` owns crosswalk validation.
  - `crates/drl-core/src/special_level_definition.rs` owns the descriptive
    Rust ID catalog checked by the validator.
  - `docs/content/evidence-coverage.json` owns reviewed coverage scope.
  - No simulation, browser runtime, Lua interpreter, or inferred gameplay
    behavior changes.

---

## 3. Recent Delivered Slices

### M9 — Representative Content-Evidence Coverage (`VERSION` 0.2.29)

- [x] Added a reviewed crosswalk and validator for pinned being, item-family,
  terrain-cell, and special-level evidence bundles.
- [x] Added provenance, digest-shape, ordering, uniqueness, representative,
  and complete 26-level coverage checks plus fixture rejection cases.
- [x] Synchronized the reviewed 26-level ID list with the Rust descriptive
  `SPECIAL_LEVEL_DEFINITIONS` catalog and rejected fixture drift.
- [ ] Full typed migration, assets, callbacks, and gameplay parity remain open.

### M10 — Same-Release Offline-Cache Isolation (`VERSION` 0.2.27)

- [x] Service-worker reads now match only the current generated release cache;
  stale/unrelated namespaces cannot satisfy offline requests.
- [x] Node worker contracts cover current-cache hits and fail-closed stale
  isolation while preserving routing and activation behavior.
- [ ] Real browser offline install/control/reload acceptance remains open.

### M12 — Dynamic Interaction Accessibility Contract (`VERSION` 0.2.26)

- [x] Added item-qualified escaped inventory actions, one live status channel,
  canvas help association, focus-visible styling, and diagnostic focus parity.
- [x] Added static shell and native markup contract coverage.
- [ ] WCAG 2.1 AA, screen-reader, contrast, and broad browser acceptance remain
  open.

### M11 — Descriptive Cohort Depth Distribution (`VERSION` 0.2.25)

- [x] Added validated, sorted deepest-level buckets and sample rates to cohort
  reports and single-policy/matrix CLI output.
- [x] Added empty-cohort, invalid-evidence, deterministic-order, and stable
  formatting coverage.
- [ ] Canonical difficulty targets and externally approved progression metrics
  remain open.

### M10 — Replay-Compatible Save Migration (`VERSION` 0.2.24)

- [x] Added strict V2 command-count encoding and V1 decode compatibility.
- [x] Migrated successful V1 restores in place while preserving fail-closed
  quarantine and transactional replay.
- [ ] Real offline browser lifecycle acceptance remains open.

### M9 — Special-Level Identity Catalog (`VERSION` 0.2.23)

- [x] Added immutable Rust metadata for 26 active level IDs, names, and texts.
- [x] Preserved optional legacy depth values and missing scalar fields.
- [ ] Full typed special-level migration, assets, and behavior validation remain
  open.

### M9 — Special-Level Evidence Index (`VERSION` 0.2.22)

- [x] Added pinned index coverage across 24 level files and 26 active records.
- [x] Added long-bracket map-string handling and explicit dynamic gaps.

### M9 — Item-Family Evidence Bundle (`VERSION` 0.2.21)

- [x] Added multi-source base/expansion/user-item evidence bundling.
- [x] Added source provenance, source indices, sorted merge, duplicate rejection,
  and fixture/pinned coverage.
- [ ] Full typed item-family migration and behavior validation remain open.

### M11 — Cohort Policy Matrix (`VERSION` 0.2.20)

- [x] Added deterministic `--bot all` output for greedy, random, and explorer.
- [x] Preserved shared seed/episode/turn configuration and prefixed report
  fields.
- [x] Added native repeatability and repository contract coverage.
- [ ] Canonical difficulty targets and statistical interpretation remain open.

### M9 — Terrain-Cell Evidence Converter (`VERSION` 0.2.19)

- [x] Added pinned-source extraction for shallow `register_cell` scalar data.
- [x] Added deterministic byte/single-quoted string handling and explicit
  nested/function/symbolic migration gaps.
- [x] Added fixture repeatability and pinned `cells.lua` coverage.
- [ ] Full typed terrain migration, asset mapping, and parity validation remain
  open.

### M12 — Signed-Release CI Smoke (`VERSION` 0.2.18)

- [x] Added repository contract coverage for signing and mutation rejection.
- [x] Added hosted WASM CI coverage with a runner-local ephemeral RSA key.
- [x] Asserted signed artifacts are present and the private key stays outside
  `dist`.
- [ ] Production key custody, rotation, and trust-root governance remain open.

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

### Verified Baseline (`VERSION` 0.2.29)

872914c feat(content): synchronize special-level evidence (#120)
4263241 docs(spec): record merged evidence baseline
a3d8fd2 feat(content): add evidence coverage gate (#119)
0d919ec docs(spec): record merged cache isolation baseline
9f0d601 feat(web): isolate service-worker cache reads (#118)
afdc01f feat(web): harden dynamic accessibility semantics (#117)
6024a15 feat(eval): add cohort depth distribution (#116)
cae3b09 feat(web): migrate browser saves to v2 (#115)
d7e5602 feat(content): add special-level metadata catalog (#114)
1fd09d5 feat(content): add pinned special-level evidence index (#113)
d1a8903 feat(content): bundle legacy item family evidence (#112)
34a9578 feat(eval): add deterministic cohort policy matrix (#111)
8486e15 feat(content): add pinned terrain cell evidence conversion (#110)
2a5e149 ci(release): smoke-test ephemeral manifest signing (#109)
3cfe62e feat(content): add pinned legacy content evidence converter (#108)
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
