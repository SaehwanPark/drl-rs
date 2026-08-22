# DRL-Rust Project Roadmap

Last reviewed: 2026-08-22

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
  Linux x86-64; the read-only manifest preflight validates this boundary and
  records checkout dirty-state and evidence classification, requiring a clean
  checkout and directly observed evidence for promotion. The attestation gate
  also requires explicit rights and media-hash metadata; see
  `docs/reference-captures/manifest.md`.
- M7 browser session/WebGPU/WASM functional acceptance passed locally and in
  remote web-CI run `32538527707` for the merged M7 head; reference-scene
  comparison remains open because legacy captures are `NOT_RUN`.
- The first bounded M8 presentation slice adds deterministic pixel-grid
  letterboxing through `drl-render::PixelViewport`; hosted run `32539486760`
  passes the repository and WASM browser jobs. Full audiovisual parity is still
  planned and capture-gated.
- The delivered follow-up M8 slice centralizes visible-versus-explored lighting
  bands in `drl-render`; this remains a presentation rule, not a legacy parity
  claim.
- A further M8 slice centralizes the existing quarter-health scene tone in
  `drl-render::SceneTone`; capture-backed low-life LUT/glow parity remains open.
- The delivered M8 follow-up adds event-ordered `drl-render::EffectSpan` timing;
  capture-backed animation/effect durations remain open.
- The browser effect handoff now carries ordered spans in successful
  `PresentationStep` results with visibility filtering; visual frame mapping
  remains capture-gated.
- The delivered M8 atlas-descriptor slice replaces placeholder cells with
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
  optional colorization-mask sampling now carries evidence-backed Green Armor
  tint for visible ground items/player equipped armor, the byte-quantized
  Phase Device tint for visible ground items, and the pinned yellow StairsDown
  tile tint; other current archetypes remain neutral. Additional per-sprite
  tint sourcing and capture-backed parity remain open. The optional
  outline-mask source is now transported into the browser binding with a
  transparent fallback and composited behind the base sprite with a tested
  straight-alpha equation; exact legacy glow/outline parity remains
  capture-gated. Pure frontend effect progress
  and caller-supplied animation frame selection are now available; legacy
  player/actor/Phase Device frame metadata is now pinned at two frames and
  500 ms, and callers can select frame UVs from normalized progress or
  elapsed milliseconds with explicit loop/clamp policy; the WASM renderer now
  forwards that caller-owned selection through a bounded browser animation loop
  and rebases its presentation clock on visibility changes while broader
  content animation and capture parity remain open. A read-only capture
  manifest preflight now validates pinned metadata, rights, media hashes, and
  evidence status while preserving `NOT_RUN`.
- `SPEC.md` contains the single active M8 particle-decal-cell slice; it is
  not a second roadmap.
- The first bounded M11 cohort slice adds explicit fixed-seed sample
  definitions and reports around the existing headless batch runner. Reports
  retain policy identity, aggregate metrics, and per-seed replay evidence;
  the follow-up adds compatible-report win-rate and average-turn tolerance
  comparisons; the outcome-distribution follow-up keeps victory, death,
  turn-limit, stalled, and in-progress counts distinct with normalized rates;
  the compatible outcome-comparison follow-up reports absolute per-category
  rate deltas after integrity validation; the outcome-tolerance follow-up
  applies one finite non-negative bound to every category; balance conclusions,
  difficulty targets, and statistical interpretation remain open.
- Delivered M11 telemetry projections and compatible comparisons expose
  validated shot-accuracy, damage, kill, pickup, and item-use totals with
  caller-owned descriptive deltas and tolerances; they do not infer balance or
  statistical significance.
- Delivered M11 integrity validation checks cohort record count, wrapping seed
  order, replay seed identity, and aggregate-summary coherence before evidence
  is used for regression comparisons; it does not infer balance or significance.
- The first bounded M12 packaging slice now emits and verifies a static-bundle
  release manifest with project version, source revision, sorted artifact
  hashes, generated-file declarations, graphics rights metadata, and
  service-worker coverage. Signed releases and offline/cross-browser
  acceptance remain open.
- The M12 cache-version follow-up derives the generated service-worker cache
  name from the manifest project version and source-revision prefix; this is a
  deterministic invalidation policy, not a signed release or offline claim.
- The delivered M12 manifest-integrity follow-up emits a lowercase SHA-256 sidecar
  for the generated manifest, verifies exact bytes, and precaches the sidecar;
  this is packaging integrity evidence, not signing or offline acceptance.
- The delivered M12 service-worker follow-up runs dependency-free mocked lifecycle
  and fetch contracts for precache, stale-cache cleanup, navigation fallback,
  and same-origin GET gating; full browser-offline acceptance remains open.
- The delivered M12 release-audit follow-up rejects malformed source revision
  identities before manifest/cache policy is accepted; signed history
  authenticity remains open.
- The delivered M12 checkout-identity follow-up requires a generated source
  revision to match Git `HEAD` (or `DRL_BUILD_REVISION`) when available, while
  preserving `unknown` for unverifiable source archives.
- The delivered M11 telemetry follow-up projects validated shot accuracy, damage,
  kill, pickup, and item-use totals/rates from fixed-seed reports without
  inferring balance or statistical significance.
- The active M11 comparison follow-up reports compatible telemetry deltas and
  applies separate caller-owned shot-accuracy and per-episode-average bounds;
  it remains descriptive evaluation evidence.
- The M12 diagnostics slice adds a local accessible browser-support and
  startup-error panel with recovery guidance; it sends no telemetry and does
  not extend the tested WebGPU browser claim.
- The delivered M12 accessibility slice statically audits shell landmarks,
  named controls, form labels, focus semantics, live regions, and support
  disclosure; dynamic WCAG and screen-reader acceptance remain open.
- `VERSION` is the canonical `x.y.z` project value. The delivery harness
  checks package and release-manifest projections and requires one valid
  component transition for code changes; document-only and setting-only
  changes do not bump it.

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

### M8 — Audiovisual parity (pixel-scale, outline-compositing, and particle slices delivered; parity planned)

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
cutoff, optional mask sampling with bounded Green Armor, Phase Device, and
StairsDown tint mappings, outline-mask GPU transport, pure caller-supplied
animation frame
selection, evidenced player/actor/Phase Device animation metadata, and
progress-selected frame UV plans plus elapsed-time loop/clamp selection,
caller-supplied elapsed-time layer-plan math, WebGPU forwarding, and bounded
browser scheduling, and the capture-manifest preflight with clean-checkout
provenance and evidence classification. The optional outline-mask pass now
resolves the shadow source behind the base sprite with straight-alpha weights
and applies the existing fragment cutoff to the combined alpha; opaque base
pixels are preserved and the exact legacy equation remains capture-gated.
The preceding follow-up also exposed a pure source-derived low-health pulse
target from fair HP and caller-supplied elapsed milliseconds. The current
follow-up adds caller-owned pure smoothing and pending-target decay using the
pinned `aMSec / 500` step; texture compositing and post-processing remain
backend/capture work.
The preceding bounded follow-up recorded the pinned declared/effective
post-process blur weights, glow add, and LUT-coordinate normalization as pure
renderer contracts without claiming an offscreen GPU pipeline or
capture-backed color parity.
The preceding follow-up added pure horizontal/vertical blur-tap plans with
caller-supplied screen dimensions and a five-sample RGB/center-alpha reduction.
The preceding follow-up added the pure direct-versus-captured post-process pass
order across glow/LUT gates. The preceding follow-up added the pure three-phase
explosion-mark selector with source duration normalization and fallback, plus
signed cell/item effect segment arithmetic with explicit invalid-input handling.
The current follow-up adds caller-owned kill-animation segment selection with
the pinned lead-delay, reverse-branch, quotient, and terminal-clamp rules;
the follow-up also adds caller-owned FX frame selection with the pinned
quotient and terminal clamp. Animation lifecycle, palette/sprite rendering,
backend sampling, and capture-backed parity remain open. The current follow-up
also adds caller-owned normalized movement progress with the pinned ratio and
clamp; coordinate interpolation, lighting, and entity lifecycle remain open.
The current follow-up also adds caller-owned missile step selection with the
pinned minimum-normalized delay and elapsed quotient; path traversal,
visibility, and particles remain open. The blur reduction preserves weighted
RGB and center-only alpha without renormalization or clamping.
The current follow-up also adds caller-owned missile ray-spacing selection with
the pinned strict pre-increment half-grid test, fixed 20-unit spacing, and
possible endpoint overshoot. Distance metrics, interpolation, path traversal,
visibility, particles, and rendering remain open.
The current follow-up also adds caller-owned screen-shake fade timing with the
pinned active quadratic envelope and zero-at-expiry guard. Random frequencies,
offsets, strength/direction scaling, scheduling, and rendering remain open.
The current follow-up also adds caller-owned particle-burst origin math for the
pinned one-based 32-pixel cell-center conversion. Direction, random burst
sampling, decals, particle-engine integration, and rendering remain open. The
delivered particle-direction follow-up adds caller-owned XY normalization,
zero-vector handling, and positive distance-scale arc-to-Z adjustment; random
sampling, decals, particle-engine integration, and rendering remain open.
The active range-sampling follow-up preserves the source's affine min/max
calculation for caller-owned unit samples, including reversed bounds without
hidden clamping; random generator ownership, decals, engine integration, and
rendering remain open.
The active decal follow-up preserves the source's rounded-position offset and
truncating 32-pixel mapping into one-based cells, while map bounds, liquid/block
eligibility, decal selection/storage, and rendering remain open.
The current M9 content slice adds a Rust-owned `MonsterKind::definition()` table
for the four implemented archetypes and routes actor factories/generated spawns
through it without changing current values or replay schemas. Legacy numeric
parity and broader Lua/content migration remain open.
The current follow-up also routes game death drops through the existing typed
`Item::from_spawn_kind` factory with all nine current spawn variants covered;
item balance, legacy parity, and broader content migration remain open.
The current item-definition follow-up centralizes those nine existing spawn
families in an immutable Rust-owned table while preserving supplied ammo
counts, current properties, and replay/protocol schemas. Broader item and
level migration, conversion tooling, and legacy parity remain open.
The current generated-loot follow-up centralizes the six existing procedural
room-item outcomes and their exact roll bounds/fixed ammo payloads while
preserving one-roll RNG consumption, item IDs, ordering, and factory behavior;
broader balance, conversion, and legacy parity remain open.
The current generated-monster follow-up centralizes the four existing
procedural monster outcomes and exact roll bounds while preserving one-roll
RNG consumption, spawn metadata/order, and shared monster definitions; broader
actor/level migration, balance, and legacy parity remain open.
The current tile-definition follow-up centralizes the five existing tile
semantics in an immutable protocol-owned table while preserving core map/FOV
behavior, observations, replay/protocol schemas, and separate core storage;
broader level/asset migration and legacy parity remain open.
The current level-content follow-up centralizes the existing core default and
level-descent policy in one immutable `standard-procedural` definition while
preserving custom generator configs, the MCP five-room policy, seeded output,
and replay/MCP boundaries; broader level conversion and legacy parity remain
open.
The current M10 follow-ups add a versioned fixed-M4 command-history snapshot,
bounded corruption/version handling, transactional replay restore, a
best-effort WASM localStorage boundary with save/load/clear controls, and a
versioned same-origin service-worker cache generated from the complete static
bundle. Offline browser acceptance and cross-browser behavior remain
`NOT_RUN`; signed release invalidation and broader PWA policy remain M12 work.
Remaining work is exact legacy outline/glow role parity, additional per-sprite
tint sourcing, broader content metadata;
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

### M11 — Balance and evaluation (cohort, integrity, outcome, telemetry, and comparison slices delivered; evaluation open)

Run fixed-seed bot cohorts, metrics, difficulty/economy studies, and
regressions with declared samples/tolerances. The delivered cohort slices now
make seed ranges, sample size, policy identity, turn budgets, replay evidence,
compatible-report tolerance deltas, and evidence-integrity validation
explicit; balance studies, statistical interpretation, and ordinary-player/
developer observation separation remain open. The delivered outcome
distribution keeps terminal categories distinct and exposes normalized rates
without claiming a balance result. The delivered compatible comparison reports
absolute per-category rate deltas without adding a tolerance or significance
claim. The delivered tolerance gate applies a caller-owned finite non-negative
bound without adding statistical interpretation. The delivered telemetry
distribution adds validated combat/economy totals and descriptive rates for
shot accuracy, damage, kills, pickups, and item use without a balance claim.
The delivered telemetry comparison reports compatible absolute deltas and
applies separate caller-owned accuracy/average bounds without statistical
interpretation.

### M12 — Static web productization and release hardening (manifest, cache, diagnostics, accessibility, sidecar, worker-contract, source-audit, and checkout-binding slices delivered; hardening open)

Produce reproducible static HTTPS/PWA bundles, asset license notices,
cache/version policy, accessibility audit, browser support/error screens,
telemetry-free diagnostics, and signed release manifests. The first manifest
slice records source revision, artifact hashes, rights metadata, and
service-worker coverage; its cache-version follow-up derives the generated
worker name from the project version and source-revision prefix; the diagnostics
slice adds a local accessible browser-support/startup-diagnostics panel with recovery
 guidance and no telemetry; the delivered follow-up statically audits shell
 landmarks and control semantics. The delivered release-hardening follow-up adds a
 deterministic SHA-256 sidecar for the manifest and precaches it. The delivered
 follow-up runs dependency-free mocked service-worker lifecycle/fetch checks;
 full browser-offline acceptance remains open. The delivered release-audit
 follow-up validates source revision identity syntax, and the delivered
 checkout-binding follow-up requires that identity to match the built checkout
 when available. Signing, release audits, broader
invalidation policy, dynamic accessibility acceptance, and untested-browser
support remain open. WebGPU remains the 1.0 baseline; do not imply support for
untested browsers.

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
