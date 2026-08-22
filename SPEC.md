# Specification

The [roadmap](docs/DRL-Rust_Project_Roadmap.md) owns milestone order and
progress. This file expands exactly one active implementation slice. It keeps
delivered evidence, the present slice, and future work separate so an open
item cannot be mistaken for an implemented capability.

## Status key

- `[x]` — delivered and verified by repository evidence.
- `[ ]` — present-slice work, future work, or an acceptance gate still open.
- `NOT_RUN` — required evidence was unavailable; it is not a pass.

## Past — delivered foundations

### M0–M6: deterministic game and agent foundations

- [x] Rust workspace, functional-core/imperative-shell boundaries, ADRs,
  contributor guidance, legacy behavior shells, and the repository harness.
- [x] Deterministic maps, explicit RNG, semantic commands, observations,
  movement, scheduling, combat, damage/death, armor, knockback, FOV/LOS,
  fog memory, inventory/equipment, consumables, levels, stairs, and transitions.
- [x] Versioned replay diagnostics, declarative ASCII scenarios, ordinary-
  observation bots, deterministic batch execution, and MCP JSON-RPC tools,
  resources, fairness boundaries, stdio transport, and virtual-agent tests.
- [x] MCP and replay schemas remain independent of presentation, audio,
  filesystem, network, and browser concerns.

### M3: assets and evidence

- [x] `drl-assets` imports the tracked graphics atlas from
  `17d9be1204751899b2d69d8d3a2dde247bd0cc5c` with CC BY-SA attribution,
  exact paths, dimensions, and SHA-256 manifests.
- [x] Legacy presentation evidence is catalogued in
  `docs/legacy-behavior/presentation.md`; audio, music, and fonts remain
  redistribution-gated.
- [ ] Controlled Linux x86-64 legacy captures, rights-cleared stills, and the
  capture-to-M7/M8 fidelity matrix are still open. The current arm64 macOS
  environment records this gate as `NOT_RUN`.

### M7: browser-playable M4 slice

- [x] `drl-web` provides a WASM/WebGPU shell with fair `PresentationStep` and
  `RenderScene` inputs, atlas descriptors, visibility/fog, actors/items,
  targeting/HUD, generated audio cues, gesture unlock, mute/volume, keyboard
  input, inventory, restart/game-over, and recoverable GPU/audio errors.
- [x] Host-agnostic web build/check/serve scripts and Ubuntu CI run pinned
  `wasm-pack 0.15.0` with headless Chrome.
- [x] Local and hosted functional checks include a Chrome 151 WebGPU smoke
  playthrough.
- [ ] Capture-backed reference-scene comparison remains open because legacy
  runtime captures are `NOT_RUN`.

## Past — delivered M8 presentation contracts

- [x] `PixelViewport` chooses centered integer square cells and deterministic
  letterboxing without changing simulation state.
- [x] `LightingBand`, scene tone, low-health pulse target/state transitions,
  and event-ordered `EffectSpan` timing are pure, fair-data contracts.
- [x] Atlas descriptors use measured 16-column/32-pixel slots. Layer metadata,
  normalized UVs, texture-source bindings, draw plans, grouped composites, and
  fair lighting are deterministic renderer inputs.
- [x] The browser uploads validated layer sources once, samples nearest-filtered
  base color, applies the emissive lighting floor and legacy `0.1` alpha cutoff,
  and carries evidence-backed Green Armor, Phase Device, and StairsDown tints.
- [x] Optional outline-mask sources have transparent fallbacks and a bounded
  straight-alpha compositing pass; exact legacy glow/outline parity remains
  capture-gated.
- [x] Player/actor/Phase Device two-frame/500 ms metadata, normalized-progress
  and elapsed frame selection, elapsed layer planning, WebGPU forwarding, and
  visibility-aware browser scheduling are implemented.
- [x] Pure post-process contracts cover glow/LUT coordinates, blur taps and
  reduction, pass order, explosion marks, effect segments, kill/FX/movement
  timing, missile steps/ray spacing, and screen-shake fading. They do not own
  GPU resources or claim capture parity.
- [x] Particle burst origin, direction, range sampling, decal cell mapping,
  decal pixel placement, caller-resolved in-bounds/non-liquid/non-blocking
  eligibility, and the caller-owned sprite insertion request are implemented.
- [x] Read-only capture-manifest preflight validates pinned revision/scenes,
  clean checkout, rights status, media hashes, and evidence classification;
  unavailable captures remain `NOT_RUN`/`INCONCLUSIVE`.

## Past — delivered M9–M12 slices

### M9 content tables

- [x] Rust-owned definitions cover current monster archetypes, death drops,
  item spawn families, procedural loot and monster rolls, tile semantics, and
  the standard procedural level policy.
- [ ] Broader legacy content migration, conversion tooling, balance claims,
  and legacy numeric parity remain future work.

### M10 persistence boundary

- [x] Versioned fixed-session snapshots, bounded corruption/version handling,
  transactional deterministic restore, best-effort WASM `localStorage` controls,
  and a versioned same-origin service-worker cache.
- [ ] Offline-after-first-load acceptance and broader PWA migration policy.

### M11 evaluation evidence

- [x] Fixed-seed cohort definitions/reports retain policy identity, sample
  bounds, turn budgets, aggregate metrics, and per-seed replay evidence.
- [x] Integrity validation checks record count, wrapping seed order, replay seed
  identity, and aggregate-summary coherence.
- [x] Outcome distributions keep victory, death, turn-limit, stalled, and
  in-progress categories distinct; compatible comparisons and finite,
  non-negative tolerance gates are pure and descriptive.
- [x] Telemetry distributions and compatible comparisons cover shot accuracy,
  damage, kills, pickups, and item use with caller-owned tolerances.
- [ ] Balance, difficulty, statistical significance, and ordinary-player/
  developer observation separation are not claimed.

### M12 release hardening

- [x] Release manifests record project version, source revision, sorted artifact
  hashes, generated-file declarations, graphics rights metadata, and worker
  coverage.
- [x] Source-derived cache naming, accessible diagnostics, static shell
  accessibility checks, the manifest SHA-256 sidecar, mocked worker lifecycle/
  fetch checks, source-identity syntax checks, and checkout binding.
- [ ] Signed releases, offline acceptance, dynamic accessibility, broader
  invalidation policy, and untested-browser support.

## Delivered — M8 particle-decal insertion request

Status: version 0.2.9 packages the accepted placement and eligibility results
with the caller-provided decal sprite identifier. The request remains
renderer-neutral and introduces no map storage, particle-engine state, or
rendering side effects.

### Legacy evidence boundary

The legacy `DecalCallback` in `src/drlparticles.pas` performs these steps:

- [x] Rejects missing sprite-map/DRL/level context.
- [x] Maps the rounded world position to a one-based cell and offset pixel
  position.
- [x] Rejects out-of-bounds, liquid, or movement-blocking cells.
- [x] Preserves the accepted pixel position and caller-provided `aDecalSprite`
  as a pure insertion request; level decal storage remains outside this slice.

### Delivered pure contract

- Input: caller-rounded world position `[i32; 2]`, caller-resolved cell flags,
  and caller-provided decal sprite identifier `u32`.
- Output: `Some(ParticleDecalInsertion)` containing the existing
  `ParticleDecalPlacement` and the unchanged sprite identifier, or `None` when
  offset arithmetic overflows or eligibility is false.
- Ownership: the caller selects the sprite, owns decal storage and lifetime,
  and decides whether/how a renderer draws the request.

### Delivered acceptance criteria

- [x] Reuse the existing checked placement and eligibility helpers rather than
  repeating offset or flag logic.
- [x] Preserve the legacy cell, pixel, and sprite identifier exactly.
- [x] Reject every ineligible cell and every unrepresentable offset.
- [x] Add focused tests for accepted, liquid, blocked, out-of-bounds, combined-
  flag, and offset-overflow cases.
- [x] Keep the helper pure, deterministic, and independent of `drl-core`, map
  storage, RNG, particle-engine state, WebGPU resources, and browser timing.

## Present — M8 particle-decal storage boundary

Status: the next bounded slice will define how accepted insertion requests are
retained in deterministic order without coupling `drl-core` to renderer state.
It must preserve every request exactly and leave sprite selection, map ownership,
particle spawning, and GPU rendering to callers.

### Acceptance criteria

- [ ] Preserve insertion order and duplicate sprite/position requests.
- [ ] Keep storage deterministic, bounded by caller policy, and independent of
  map flags, RNG, particle-engine state, WebGPU resources, and browser timing.
- [ ] Reject or report capacity policy explicitly rather than silently dropping
  accepted requests.
- [ ] Add focused tests for empty, append, duplicate, and capacity-boundary
  behavior.

## Shared observable behavior

### Simulation and fairness

- [x] `BrowserSession` exposes only fair `PlayerObservation` and `GameEvent`
  streams; hidden simulation state is unavailable to renderers and agents.
- [x] Identical seed and semantic commands produce identical events, final
  state, and replay as direct `drl-core` execution.
- [x] Rejected commands roll back the session and do not advance simulation.
- [x] MCP JSON and replay schemas remain stable while presentation contracts
  remain additive and renderer-owned.

### Evaluation

- [x] Cohort reports use contiguous wrapping seeds and preserve per-seed replay
  evidence; invalid or incompatible reports cannot produce comparisons.
- [x] Outcome and telemetry projections report descriptive counts, totals, and
  normalized rates only after integrity checks.
- [x] Caller-owned finite, non-negative tolerances report deltas within bounds
  without implying balance, difficulty, or statistical significance.

### Browser and presentation

- [x] The browser maps keyboard/numpad, wait, pickup, reload, descend, ranged
  targeting, and DOM inventory actions to ordinary protocol commands.
- [x] Presentation consumes only fair scenes, preserves visibility/fog rules,
  and never advances gameplay from resize, animation, audio, or tab visibility.
- [x] WebGPU texture/shader contracts are guarded by native tests; malformed
  texture plans fall back to deterministic geometry where supported.
- [x] Audio unlock, mute, volume, diagnostics, and accessibility are local
  recovery/presentation surfaces and do not send telemetry.

### Release evidence

- [x] Release checks validate manifest schema, path safety, sorted hashes,
  generated-file declarations, rights metadata, worker coverage, sidecar bytes,
  source identity, and checkout identity.
- [ ] Offline behavior, signed release authenticity, dynamic WCAG/screen-reader
  acceptance, and untested-browser support remain open.

## Public contracts

- [x] Additive fair data: `PlayerObservation::{map_width, map_height,
  player_hp}`, `ActorView::monster_kind`, `ItemArchetype`, and
  `ItemView::archetype`.
- [x] Presentation APIs: `PresentationStep`, `RenderScene`, `AudioCue`,
  `BrowserSession`, `PixelViewport`, layer/composite plans, texture-source
  manifests, and WASM-only `WebGpuRenderer`.
- [x] Pure evaluation APIs: `CohortConfig`, `CohortReport`, integrity checks,
  outcome/telemetry projections, compatible comparisons, and tolerance gates.
- [x] Pure M8 particle APIs: burst origin/direction/range, decal cell mapping,
  `ParticleDecalPlacement`, `particle_decal_cell_is_eligible`,
  `ParticleDecalInsertion`, and its constructor.
- [x] `drl-core` and `drl-protocol` have no presentation, browser, audio,
  filesystem, network, or MCP dependency.

## Verification

### Baseline already verified on 0.2.9

- [x] `sh scripts/check-repository.sh` — repository tests and harness checks.
- [x] `sh scripts/check-assets.sh` — 32 imported PNGs, license, and hashes.
- [x] `scripts/test-reference-capture.sh` — fixture coverage.
- [x] `sh scripts/check-web.sh` — native/WASM contracts; browser runner is
  `NOT_RUN` when Chrome is unavailable.
- [x] `sh scripts/build-web.sh && sh scripts/check-release-manifest.sh` —
  release bundle and 44-artifact manifest.
- [x] `scripts/check-version.sh main` — version projections and transition.
- [x] `cargo fmt --all -- --check` and `git diff --check`.
- [x] Hosted PR checks for the insertion slice, including repository run
  `32600541249` and WASM browser job `97098028542`.
- [ ] Reference-capture execution and audiovisual comparison remain `NOT_RUN`.

### Present storage-slice verification

- [ ] Focused `drl-render` tests cover all storage acceptance criteria.
- [ ] Full repository, asset, web, build, manifest, formatting, and version
  checks pass after implementation.
- [ ] Hosted repository and WASM jobs pass for the storage PR.

## Future

- [ ] M8 exact legacy outline/glow and lighting/LUT equivalence from approved
  captures.
- [ ] M8 broader tint sources, content animation/effect timing, HUD typography,
  particle decal rendering, cleared replacement audio/music, and
  automated visual/audio regression.
- [ ] M9 broader typed content migration with fairness, replay, provenance, and
  asset-mapping gates.
- [ ] M10 offline-after-first-load and replay-compatible save acceptance.
- [ ] M11 balance/economy studies with declared samples and human review.
- [ ] M12 signing, dynamic accessibility, offline hardening, and untested
  browser support.
- [ ] M13 accepted desktop Chromium WebGPU 1.0 release and public rights
  inventory.

## Explicit non-goals

- WebGL2 fallback, Firefox/Safari, mobile/touch, controllers, and native desktop
  packaging before M13.
- Runtime Lua, parallel JavaScript gameplay state, accounts, or a gameplay
  backend service.
- Shipping legacy audio/music/fonts before rights are documented.
- Claiming full audiovisual equivalence from renderer-neutral contracts,
  generated tones, or `NOT_RUN` captures.
- Treating static accessibility as full WCAG or screen-reader acceptance.
- Treating release manifests, cache names, sidecars, or mocked worker contracts
  as signed release authenticity or browser-offline proof.
