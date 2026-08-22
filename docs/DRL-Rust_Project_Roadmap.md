# DRL-Rust Project Roadmap

Last reviewed: 2026-08-22

## Product direction

DRL-Rust is a browser-first Rust/WASM reimplementation of Doom the Roguelike.
The 1.0 target is desktop Chrome/Edge with WebGPU, a static HTTPS bundle, an
accessible DOM shell, and later offline/PWA productization. The deterministic
headless core, MCP interface, bots, and replay tools remain first-class.

The project preserves DRL semantics while replacing legacy implementation
machinery. Pascal/Lua is behavioral or content evidence; Lua is a build-time
conversion input and never a runtime WASM dependency. Imported assets require
attributable provenance and rights. Audiovisual equivalence requires approved
legacy captures, stated tolerances, automated regressions, and human review.

Native packaging, WebGL2 fallback, Firefox/Safari, mobile/touch, and controller
UX are post-1.0 portability work.

## Status key

- `[x]` — delivered and verified by the repository evidence named nearby.
- `[ ]` — planned, open, or not yet verified.
- `NOT_RUN` — the environment did not provide the required evidence; it is not
  a pass or an inferred failure.

## Current truth (`VERSION` 0.2.8)

- [x] M0–M2 and M4–M6 are delivered and covered by repository tests.
- [x] M0 macOS CI evidence is recorded for baseline commit `c07c44e` in
  [run 32408102745](https://github.com/SaehwanPark/drl-rust/actions/runs/32408102745).
- [x] M3 graphics import, attribution, SHA-256 checks, and manifest tooling are
  delivered.
- [ ] Controlled legacy runtime captures are still `NOT_RUN` on arm64 macOS:
  the available binary is Linux x86-64. Promotion still requires a clean
  checkout, directly observed evidence, explicit rights, and valid media
  hashes. See `docs/reference-captures/manifest.md`.
- [x] M7 browser/WebGPU/WASM functional acceptance passed locally and in remote
  web CI run `32538527707` for the merged M7 head.
- [ ] M7 reference-scene comparison remains open because legacy captures are
  `NOT_RUN`.
- [x] M8 has delivered pixel-grid layout, visibility lighting, scene tone,
  event-ordered effect spans, atlas/layer metadata, UV and draw planning,
  texture upload/base sampling, emissive and alpha-cutoff rules, evidence-
  backed tints, outline transport/compositing, elapsed animation selection,
  browser scheduling, and capture-manifest attestation helpers.
- [x] M8 particle contracts cover burst origin, direction, range sampling,
  decal cell mapping, pixel placement, and in-bounds/non-liquid/non-blocking
  eligibility.
- [ ] M8 exact legacy outline/glow parity, broader tint/content animation,
  particle insertion/storage/rendering, HUD typography, audio/music, and
  capture-backed audiovisual comparison remain open.
- [x] M11 fixed-seed cohorts, integrity validation, outcome distributions,
  compatible comparisons, tolerance gates, telemetry projections, and
  telemetry comparisons are delivered as descriptive evidence only.
- [ ] M11 balance studies, difficulty targets, statistical interpretation, and
  ordinary-player/developer observation separation remain open.
- [x] M12 release manifests, source-derived cache names, diagnostics,
  accessibility shell audit, manifest sidecar, worker lifecycle harness,
  source-identity syntax audit, and checkout binding are delivered.
- [ ] M12 signing, offline-after-first-load, dynamic accessibility, and
  untested-browser support remain open.
- [x] Version transitions are enforced by the agent harness: code changes
  require one valid `x.y.z` component increment; documentation-only and
  setting-only changes do not bump the version.

## Milestone checklist

### M0 — Truthful steering, documentation, and harness

- [x] Reconcile proposal, roadmap, SPEC, architecture, README, changelog, ADRs,
  contributor guidance, and repo-local agent skills to browser-first delivery.
- [x] Keep one active SPEC slice, serialize canonical writes, and gate browser
  test-play on browser/GPU/viewport/DPR/audio evidence.
- [x] Correct the earlier three behavior shells versus the false “six” claim.
- [x] Keep remote CI and rights/capture evidence incomplete until directly
  verified.

### M1 — Deterministic simulation kernel

- [x] Pure Rust maps, explicit RNG, semantic commands, observations, movement,
  and replay-safe state transitions.

### M2 — Turn economy and combat

- [x] Energy scheduling, melee/ranged combat, damage/death events, accuracy,
  armor, and knockback.

### M3 — Browser-compatible assets, provenance, and fidelity evidence

- [x] `drl-assets` owns atlas IDs, layers, rectangles, semantic mappings, and
  the complete tracked graphics import from the pinned Git revision.
- [x] CC BY-SA attribution, exact paths, SHA-256 checksums, and rights metadata
  are recorded; audio, music, and fonts remain redistribution-gated.
- [x] Legacy sprite, fog, particle, animation, HUD, audio-cue, and provenance
  evidence is catalogued in `docs/legacy-behavior/`.
- [ ] Capture lighting/fog, targeting, ranged combat, knockback/death, low
  health, inventory/HUD, and transitions in a rights-cleared Linux x86-64
  environment.
- [ ] Exit the capture gate only when every current M4 semantic maps to an atlas
  entry and capture checks are `PASS`; current execution is `NOT_RUN`.

### M4 — Perception and content foundations

- [x] FOV, LOS, fog memory, fair observations, inventory/equipment/consumables,
  procedural levels, stairs, and level transitions.

### M5 — Replays, scenarios, and test agents

- [x] Versioned replay diagnostics, declarative fixtures, ordinary-observation
  bots, deterministic batch execution, and replay-verified agent tests.

### M6 — MCP semantic interface

- [x] Zero-dependency JSON-RPC/MCP lifecycle, tools/resources, fairness
  boundaries, stdio runner, and virtual-agent determinism tests.

### M7 — Browser-playable M4 slice

- [x] `drl-web` WASM/WebGPU shell with pinned web dependencies and host-agnostic
  build/check/serve scripts.
- [x] Fair `PresentationStep`/`RenderScene`, atlas descriptors, geometry,
  visibility/fog, actors/items, targeting/HUD, generated audio cues,
  gesture-gated audio, accessibility shell, keyboard/numpad input, inventory,
  restart/game-over, and recoverable GPU/audio errors.
- [x] Functional checks pass locally and in remote Ubuntu web CI, including the
  Chrome 151 WebGPU smoke playthrough.
- [ ] Capture-backed reference-scene comparison and audiovisual fidelity gate.

### M8 — Audiovisual parity

#### Delivered

- [x] Pixel-scale square-cell viewport and deterministic letterboxing.
- [x] Visibility-derived lighting bands, quarter-health scene tone, pure
  low-health pulse target/state transitions, and event-ordered effect spans.
- [x] Measured 16-column/32-pixel atlas slots, ordered layer metadata,
  normalized UVs, texture-source bindings, layer draw plans, grouped sprite
  composites, and fair lighting propagation.
- [x] WebGPU texture upload/cache, nearest base sampling, emissive lighting
  floor, legacy `0.1` alpha cutoff, optional mask sampling, and evidence-backed
  Green Armor, Phase Device, and StairsDown tints.
- [x] Outline-mask transport and bounded straight-alpha compositing; exact
  legacy glow/outline equation remains capture-gated.
- [x] Pinned player/actor/Phase Device animation metadata, progress/elapsed
  frame selection, elapsed-time layer planning, WebGPU forwarding, and a
  visibility-aware browser animation loop.
- [x] Pure post-process glow/LUT coordinate, blur-tap/reduction, pass-order,
  explosion, effect-segment, kill, FX, movement, missile, and screen-shake
  contracts. These do not own GPU resources or claim capture parity.
- [x] Particle burst origin, direction, range, decal cell mapping, decal pixel
  placement, and caller-resolved decal eligibility contracts.
- [x] Read-only capture-manifest preflight with clean-checkout, rights, hash,
  and evidence classification gates; missing captures remain `NOT_RUN`.

#### Present slice (expanded in `SPEC.md`)

- [ ] Package the accepted placement plus caller-provided decal sprite into a
  renderer-neutral insertion request; selection, storage, and rendering remain
  outside this slice.

#### Open after the present slice

- [ ] Exact legacy outline/glow and lighting/LUT equivalence from approved
  captures.
- [ ] Broader tint sources, content animation/effect timing, HUD typography,
  particles, cleared replacement audio/music, and visual/audio regressions.
- [ ] Particle decal storage and renderer integration.

### M9 — Typed content migration and gameplay breadth

- [x] Rust-owned definitions for current monster archetypes, death drops, item
  spawn families, procedural loot, procedural monster rolls, tile semantics,
  and the standard procedural level policy.
- [ ] Broader actor/item/level migration and conversion tooling from legacy
  evidence; no runtime Lua in the browser.
- [ ] Every migrated content group passes fairness, replay, provenance, and
  asset-mapping gates.

### M10 — Browser persistence and PWA state

- [x] Versioned fixed-session snapshot codec, corruption/version handling,
  transactional replay restore, best-effort WASM localStorage controls, and a
  versioned same-origin service-worker cache.
- [ ] Offline-after-first-load behavior, broader migration/corruption policy,
  and replay-compatible save acceptance.
- [x] No accounts or gameplay backend are part of the scope.

### M11 — Balance and evaluation

- [x] Fixed-seed cohort definitions/reports with policy identity, sample bounds,
  turn budgets, aggregate metrics, and per-seed replay evidence.
- [x] Cohort integrity validation, compatible comparisons, outcome categories,
  outcome-rate tolerance, telemetry distributions, and telemetry comparisons.
- [ ] Balance/economy studies, difficulty targets, statistical interpretation,
  and ordinary-player/developer observation separation.

### M12 — Static web productization and release hardening

- [x] Reproducible release manifest with source revision, project version,
  artifact hashes, generated-file declarations, rights metadata, and worker
  coverage.
- [x] Project/source-derived cache naming, accessible diagnostics, static shell
  accessibility audit, manifest SHA-256 sidecar, mocked worker lifecycle/fetch
  checks, source-identity syntax audit, and checkout binding.
- [ ] Signed releases, offline acceptance, broader invalidation policy, dynamic
  accessibility, and untested-browser support.

### M13 — Browser-first 1.0 release

- [ ] Ship the accepted desktop Chromium WebGPU game with deterministic
  headless/MCP tooling, approved audiovisual matrix, PWA/offline behavior,
  release notes, rollback/version guidance, and a public rights inventory.

## Post-1.0 portability

- [ ] WebGL2 fallback, Firefox/Safari acceptance, mobile/touch and controller UX.
- [ ] Native desktop packaging, optional Linux/Windows frontends, and other
  distribution channels.

These targets must not reintroduce platform dependencies into `drl-core`.

## Delivery gates

- [ ] Every milestone uses the repo-local delivery harness and exactly one
  active SPEC slice.
- [ ] Run `sh scripts/check-repository.sh` after every slice.
- [ ] Asset slices also run `scripts/check-assets.sh`.
- [ ] Browser slices run the locked WASM check, `scripts/check-web.sh`, and the
  named remote job.
- [x] `PASS`, `FAIL`, `INCONCLUSIVE`, and `NOT_RUN` are explicit vocabulary.
- [x] Remote or reference-capture criteria are never marked complete from local
  inference.
