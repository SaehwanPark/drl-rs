# Specification

Last reviewed: 2026-08-22
Current project version: `0.2.11`

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

## 2. Active Implementation Slice: M8 Particle-Decal Renderer Integration

### 2.1 Scope & Objective

Connect retained particle decal requests from `ParticleDecalStore` into the
browser WebGPU rendering pipeline. The renderer consumes stored decal entries
in deterministic insertion order and maps caller-supplied decal sprite
identifiers to atlas draw passes without adding map ownership, particle
spawning, simulation commands, or browser timing to the store or simulation
core.

### 2.2 Predecessor Foundation (Delivered Slices)

1. **Decal Placement & Eligibility (v0.2.6–v0.2.8)**:
   - `particle_decal_cell_at_rounded_world`: Converts world coordinates
     `[i32; 2]` using legacy 16px offset and 32px cell division into 1-based
     cells.
   - `particle_decal_placement_at_rounded_world`: Retains both 1-based cell
     and derived pixel offset.
   - `particle_decal_cell_is_eligible`: Pure boolean check requiring cell to be
     in bounds, non-liquid, and non-blocking.
2. **Decal Insertion Request (v0.2.9)**:
   - `ParticleDecalInsertion`: Combines accepted `ParticleDecalPlacement` with
     caller-provided decal sprite ID (`u32`). Pure, renderer-neutral request.
3. **Decal Storage Boundary (v0.2.10)**:
   - `ParticleDecalStore::new(capacity)`: Initializes caller-owned storage with
     bounded capacity.
   - `try_insert`: Appends an insertion request preserving insertion order and
     duplicates; returns `CapacityExceeded` when full without mutating prior
     entries.
   - `entries()`: Exposes retained requests in deterministic order.

### 2.3 Present Slice Acceptance Criteria

- [x] **Deterministic Consumption**: Read requests from
  `ParticleDecalStore::entries()` in strict insertion order without altering
  duplicate entries; requests outside the visible viewport or without a
  caller-provided sprite descriptor are omitted without changing the store.
  Sprite IDs remain opaque caller-resolved handles (the legacy convention is
  a packed layer/cell handle), and stored pixel placement is retained for
  sub-cell draw geometry. The caller also supplies each decal's lighting band;
  the renderer does not infer hidden visibility.
- [x] **Renderer Decoupling**: Keep sprite texture lookup, layer resolution, and
  WebGPU resource binding strictly within `drl-render` and `drl-web`.
- [x] **Simulation Independence**: Ensure decal rendering does not access
  `World`, modify simulation state, spawn commands, or alter PRNG streams.
- [x] **Native Contract Tests**: Add focused unit tests verifying decal draw
  generation from store entries, insertion order, duplicate retention,
  viewport filtering, unknown-sprite omission, and store immutability.
- [x] **Browser Consumption**: Include the renderer-neutral decal plan in the
  existing textured WebGPU pass using the same source-specific bindings as
  scene sprites.
- [ ] **Capture Gate**: Keep full visual and display parity gated on approved
  reference captures (`NOT_RUN` on macOS arm64).

### 2.4 Pure Contract

- **Input**: `&ParticleDecalStore` entries, visible viewport bounds, and atlas
  sprite descriptors. The descriptor table resolves opaque caller-provided
  sprite IDs; the store does not infer an atlas or hard-code blood slots.
- **Output**: Render-ready decal draw plan or vertex buffer inputs, preserving
  request order and duplicates for accepted visible entries.
- **Ownership Boundary**:
  - `drl-render` owns draw planning and UV coordinate generation.
  - `drl-web` owns WebGPU pipeline bindings and texture sampling.
  - `drl-core` remains completely unaware of decal storage and rendering.

---

## 3. Recent Delivered Slices

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

### Verified Baseline (`VERSION` 0.2.11)

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
