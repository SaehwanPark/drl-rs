# DRL-Rust Project Roadmap

Last reviewed: 2026-08-21

## Product direction

DRL-Rust is a browser-first Rust/WASM reimplementation of Doom the
Roguelike. Version 1.0 targets desktop Chrome and Edge with WebGPU, a static
HTTPS bundle, an accessible DOM shell, and offline/PWA productization later in
the roadmap. The deterministic headless core, MCP interface, bots, and replay
tools remain first-class. Native desktop packaging, WebGL2 fallback,
Firefox/Safari, mobile/touch, and controller UX are post-1.0 portability work.

The project preserves DRL semantics while replacing legacy implementation
machinery. Legacy Pascal/Lua is behavioral/content evidence; Lua is a
build-time conversion input, never a runtime WASM dependency. Every imported
asset has attributable provenance and rights. Full audiovisual equivalence is
measured with approved legacy reference captures, stated tolerances,
automated regressions, and human review.

## Current truth

- M0–M2 and M4–M6 are delivered and covered by repository tests.
- M0 macOS CI evidence is complete for the baseline commit
  `c07c44e` ([run](https://github.com/SaehwanPark/drl-rust/actions/runs/32408102745)).
- M3 graphics import/checks are delivered. Controlled legacy runtime captures
  are `NOT_RUN` on the current arm64 macOS host because the available binary is
  Linux x86-64; see `docs/reference-captures/manifest.md`.
- M7 browser session/WebGPU/WASM functional acceptance passed locally and in
  remote web-CI run `32538527707` for the merged M7 head; reference-scene
  comparison remains open because legacy captures are `NOT_RUN`.
- The first bounded M8 presentation slice adds deterministic pixel-grid
  letterboxing through `drl-render::PixelViewport`; hosted run `32539486760`
  passes the repository and WASM browser jobs. Full audiovisual parity is still
  planned and capture-gated.
- The active follow-up M8 slice centralizes visible-versus-explored lighting
  bands in `drl-render`; this remains a presentation rule, not a legacy parity
  claim.
- A further M8 slice centralizes the existing quarter-health scene tone in
  `drl-render::SceneTone`; capture-backed low-life LUT/glow parity remains open.
- The active M8 follow-up adds event-ordered `drl-render::EffectSpan` timing;
  capture-backed animation/effect durations remain open.
- The browser effect handoff now carries ordered spans in successful
  `PresentationStep` results with visibility filtering; visual frame mapping
  remains capture-gated.
- The current M8 atlas-descriptor slice replaces placeholder cells with
  pinned-legacy 32-pixel sprite slots and verified imported-sheet dimensions;
  the follow-up layer metadata now preserves registered source-layer order;
  normalized UV geometry, a renderer-neutral layer draw plan, and deterministic
  imported texture-source bindings are now available; fair lighting factors
  explicit legacy shader input roles are carried into draw entries, and
  complete role sets are grouped into deterministic compositor inputs, while
  a subpath-safe WASM source loader now validates decoded image dimensions;
  renderer-owned GPU texture upload and a partial base-color textured pass are
  now available; the emissive red-channel lighting floor is also wired through
  the browser pass, plus the legacy textured-fragment alpha cutoff, while
  optional colorization-mask sampling with a neutral tint boundary is now
  wired through the browser pass; per-sprite tint sourcing, outline/glow role
  compositing and capture-backed parity remain open. Pure
  frontend effect progress is now available; legacy animation timing remains
  capture-gated.
- `SPEC.md` contains the single active M8 slice; it is not a second roadmap.

## Milestones

### M0 — Truthful steering, documentation, and harness (complete)

Reconcile proposal, roadmap, spec, architecture, README, contributor guide,
changelog, ADRs, and agent skills to browser-first delivery. Keep one active
spec slice, serialize canonical writes, discover every repo-local skill, and
gate browser test-play on recorded browser/GPU/viewport/DPR/audio evidence.
Correct the earlier three behavior shells versus the false “six” claim. Keep
remote web CI and rights/capture evidence incomplete until directly verified.

### M1 — Deterministic simulation kernel (complete)

Pure Rust maps, explicit RNG, semantic commands, observations, movement, and
replay-safe state transitions.

### M2 — Turn economy and combat (complete)

Energy scheduling, melee/ranged combat, damage/death events, accuracy, armor,
and knockback.

### M3 — Browser-compatible assets, provenance, and fidelity evidence (implementation delivered; capture gate open)

- `drl-assets` owns platform-neutral atlas IDs, layers, rectangles, and
  semantic mappings for all implemented tiles, actors, and items.
- Import every tracked file in the legacy graphics directory from the pinned
  Git revision, with CC BY-SA attribution, exact paths, and SHA-256 checksums.
- Keep audio, music, and fonts out until rights are documented separately.
- Catalog layered sprites, LUTs, fog, particles, animation, HUD, audio cues,
  and unresolved provenance in `docs/legacy-behavior/`.
- Capture lighting/fog, targeting, ranged combat, knockback/death, low health,
  inventory/HUD, and transitions in a controlled Linux x86-64 environment;
  commit only rights-cleared stills and manifests, keeping unclear media in
  ignored `_workspace/`.
- Exit only when every current M4 semantic maps to an atlas entry and the
  provenance/capture checks are `PASS`; the capture-to-M7/M8 mapping lives in
  `docs/reference-captures/fidelity-matrix.md`, and current capture execution
  is `NOT_RUN`.

### M4 — Perception and content foundations (complete)

FOV, LOS, fog memory, fair player observations, inventory/equipment/consumables,
procedural levels, stairs, and level transitions.

### M5 — Replays, scenarios, and test agents (complete)

Versioned replay diagnostics, declarative fixtures, ordinary-observation bots,
and deterministic batch execution.

### M6 — MCP semantic interface (complete)

Zero-dependency JSON-RPC/MCP lifecycle, tools/resources, fairness boundaries,
stdio runner, and replay-verified virtual-agent tests.

### M7 — Browser-playable M4 slice (functional acceptance passed; fidelity gate open)

- Add `drl-web` as `cdylib + rlib` with pinned `wgpu 30.0.0`, `winit 0.30.12`,
  `wasm-bindgen 0.2.127`, matching web bindings, and `wasm-bindgen-test`.
- Use WebGPU first on desktop Chrome/Edge. Keep shader/presentation choices
  within a WebGL2-compatible subset without claiming fallback support.
- Add additive map dimensions/player HP/actor kind/item archetype protocol
  fields while preserving MCP JSON and replay schemas.
- Implement deterministic `PresentationStep`/`RenderScene`, atlas-layer
  descriptors and geometry presentation, visibility/fog, actors/items,
  targeting/HUD, generated Web Audio cues,
  gesture unlock, mute/volume, accessible DOM state, keyboard/numpad input,
  inventory actions, restart/game-over, and recoverable GPU/audio errors.
- Add host-agnostic web build/check/serve scripts, ignored `dist/`, and an
  Ubuntu web-CI job with the pinned `wasm-pack 0.15.0` and headless Chrome.
- Functional exit checks pass locally and in remote web CI, including a current
  Chrome 151 WebGPU smoke playthrough. Final exit still requires the
  capture-backed reference-scene comparison; legacy capture execution remains
  `NOT_RUN` on this host.

### M8 — Audiovisual parity (pixel-scale slice delivered; parity planned)

The first bounded pixel-scale layout slice is delivered: measure-free,
observation-independent viewport math chooses centered integer square cells for
the WebGPU surface. The baseline visible-versus-explored fog band is also
centralized in `drl-render`; capture-backed lighting/LUT parity remains open.
Measured atlas rectangles for all currently implemented semantics now come from
the pinned 16-column legacy grid, with pure bounds checks. Registered layer
metadata now carries exact atlas source sets in deterministic order, pure UV
geometry, texture-source metadata, fair lighting factors, normalized effect
progress, explicit base/mask/outline/emissive input roles, grouped sprite
composite inputs, and validated browser source loading are available, and
`drl-render::layer_draw_plan` converts fair scenes into ordered pixel/UV
operations; the renderer now retains a 24-source GPU texture cache and samples
base-color sprites with linear normalized atlas storage, a native-tested WGSL
contract, fair lighting, the emissive red-channel floor, the legacy alpha
cutoff, and optional mask sampling with a neutral tint boundary.
Remaining work is per-sprite tint sourcing and outline/glow role-specific
shader sampling/layer compositing; implement
capture-backed lighting/LUT parity,
particles, animation/effect timing, HUD typography, cleared legacy or
replacement audio, music transitions, and automated visual/audio regressions
against the M3 capture matrix. Use human review with tolerances;
generated M7 tones are not an equivalence claim.

### M9 — Typed content migration and gameplay breadth (planned)

Extend actors/items/levels from evidence using typed Rust/content tables and
conversion tooling. Migrate Lua behavior at build time only; no Lua runtime in
the browser. Every content group passes fairness, replay, provenance, and
asset mapping gates.

### M10 — Browser persistence and PWA state (planned)

Add versioned save/restart policy, browser storage boundaries, migration and
corruption handling, service-worker offline-after-first-load behavior, and
replay-compatible save tests. No accounts or gameplay backend.

### M11 — Balance and evaluation (planned)

Run fixed-seed bot cohorts, metrics, difficulty/economy studies, and
regressions with declared samples/tolerances. Keep ordinary-player and
developer observations separate.

### M12 — Static web productization and release hardening (planned)

Produce reproducible static HTTPS/PWA bundles, asset license notices,
cache/version policy, accessibility audit, browser support/error screens,
telemetry-free diagnostics, and signed release manifests. WebGPU remains the
1.0 baseline; do not imply support for untested browsers.

### M13 — Browser-first 1.0 release (planned)

Ship the accepted desktop Chromium WebGPU game with deterministic headless and
MCP tooling, approved audiovisual matrix, PWA/offline behavior, release
notes, rollback/version guidance, and a public rights inventory.

## Post-1.0 portability

Implement WebGL2 fallback, Firefox/Safari acceptance, mobile/touch and
controller UX, native desktop packaging, optional Linux/Windows frontends,
and additional distribution channels only after M13. These are portability
targets, not reasons to reintroduce platform dependencies into `drl-core`.

## Delivery gates

Every milestone uses the repo-local delivery harness and one active `SPEC.md`
slice. Run `sh scripts/check-repository.sh` after each slice. Asset work also
runs `scripts/check-assets.sh`; browser work runs
`cargo check --locked -p drl-web --target wasm32-unknown-unknown`,
`scripts/check-web.sh`, and the named remote job. `PASS`, `FAIL`,
`INCONCLUSIVE`, and `NOT_RUN` remain explicit. A remote or reference-capture
criterion is never marked complete from local inference.
