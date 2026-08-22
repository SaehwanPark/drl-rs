# Specification

The [roadmap](docs/DRL-Rust_Project_Roadmap.md) owns milestone order and
progress. This file expands exactly one active implementation slice.

## Past

- M0–M6 established the Rust workspace, deterministic headless simulation,
  FOV/fog, combat/items/levels, replay/scenario/bot infrastructure, and MCP.
- M0 steering was reconciled to browser-first delivery. ADRs 0007 and 0008
  record the accepted Rust/WASM/WebGPU and build-time-content decisions.
- M3 asset infrastructure imported the complete tracked legacy graphics atlas
  from Git revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`, with CC BY-SA
  attribution and SHA-256 checksums. Legacy audio, music, and fonts remain
  redistribution-gated; its controlled reference-capture gate is `NOT_RUN` on
  arm64 macOS and remains an M8 acceptance dependency.

## Delivered — M8 presentation and attestation slices

Status: The M7 browser slice passed functional acceptance locally and in
remote web CI. The delivered M8 pixel-grid, visibility-band, low-health tone,
and fair effect-span slices share pure presentation rules. The preceding M8
slices replaced placeholder atlas cells with measured 32-pixel slots, carried
registered source-layer sets through semantic descriptors, exposed normalized
UV geometry, built ordered layer draw plans with screen rectangles and UVs
from fair render scenes, attached imported texture-source paths and dimensions,
and carried fair visible/explored lighting into each layer draw entry. The
preceding bounded loader added a subpath-safe browser URL boundary and a WASM
image decode path that validates each imported layer's measured dimensions.
The GPU-upload follow-up builds a stable unique-source manifest and uploads
those decoded images into renderer-owned WebGPU texture/view resources. The
base/emissive follow-ups add nearest-filtered WGSL sampling, the verified
emissive lighting floor, the legacy alpha cutoff, a native shader contract, and
optional colorization-mask sampling with a neutral per-vertex tint boundary.
The preceding slices carry the legacy Green Armor and Phase Device colors into
fair item scene metadata and the existing vertex tint path, and the browser
shell now rebases its presentation clock on visibility changes. This slice
carries the pinned yellow `StairsDown` tile color through that same mask path.
Earlier M8 work carries the renderer-neutral `SpriteComposite::shadow` source
into an optional
outline-mask WebGPU binding, using the retained transparent fallback when an
atlas has no shadow source. The shader receives the resource but leaves output
unchanged. It also carries pinned two-frame/500 ms metadata for the current
player, actors, and Phase Device into renderer-neutral descriptors and grouped
draws. This slice adds a caller-supplied normalized-progress layer-plan helper
that selects those frame UVs deterministically while preserving the existing
frame-zero API; no wall-clock scheduling is introduced. It also exposes pure
elapsed-milliseconds selection with explicit loop/clamp policy over the pinned
metadata. Visible outline/glow equations, broader animation timing/content, and
capture-backed audiovisual equivalence remain open.
This slice also exposes elapsed-time layer-plan UV selection with explicit
loop/clamp policy while preserving the existing frame-zero and normalized-
progress APIs; browser scheduling remains outside the renderer boundary.
The WASM shell now exposes `WebGpuRenderer::render_at_elapsed` to forward that
selection into textured vertex generation without changing the frame-zero
entrypoint or owning a browser clock.
After boot, the WASM shell schedules caller-owned elapsed rendering from
`requestAnimationFrame` timestamps, skips frames while the document is hidden,
resets its presentation baseline on restart, and installs one idempotent
`visibilitychange` listener that rebases the clock even when hidden RAF
callbacks are throttled; simulation commands remain event-driven. The exact
legacy outline equation and broader tint sources remain capture/evidence
gated. The delivered capture-attestation slice added a read-only
`scripts/check-reference-capture.sh` preflight for the ignored legacy-capture
manifest, including `legacy_dirty_state` and clean-checkout gating, and the
proposal's evidence-classification vocabulary. That slice added
`rights_status`, validates comma-separated `sha256:<64-hex>` media hashes, and
requires `rights_status=cleared` plus valid hashes before a manifest can claim
`PASS`. Missing, dirty, non-observed, uncleared, or malformed manifests remain
`NOT_RUN`/`INCONCLUSIVE`; only a controlled Linux x86-64 capture from a clean
checkout can promote the fidelity gate.
The delivered low-health follow-up also exposes the pinned blood-pulse target as
`drl_render::low_health_pulse_target_alpha`: a pure function of fair player
health and caller-supplied elapsed milliseconds. It preserves the observed
`current < max / 3` threshold, five-radian-per-second sine term, positive
target guard, and `[0, 1]` bound without owning a clock or mutable smoothing
state. The active follow-up adds `drl_render::low_health_pulse_state_step`, a
caller-owned pure transition that preserves the legacy `aMSec / 500`
move-toward rule and independent pending-target decay without inventing
internal clamps. Low-life texture compositing, blur/LUT compositor execution,
and capture-backed audiovisual parity remain explicitly open.
The delivered post-process follow-up exposes the observed glow add, the
declared/effective blur weights, and channel-swizzled LUT-coordinate
normalization as pure renderer helpers. It
does not create blur framebuffers, sample a LUT, blend outline masks, or claim
capture-backed color parity.
The preceding follow-up exposed pure horizontal/vertical blur tap plans with
caller-supplied screen dimensions, normalized offsets, effective weights, and
the observed center-alpha source index, plus a five-sample RGB/center-alpha
reduction. It rejects zero dimensions and does not execute sampling or own a
render pass.
The preceding follow-up exposed `drl_render::post_process_pass_plan`, which
distinguishes direct scene drawing from captured-scene processing and preserves
the observed optional horizontal/vertical blur then composite order across
glow/LUT gates. It owns no framebuffer, texture, sampler, scheduling, or
capture-parity behavior.
The active follow-up now exposes `drl_render::explosion_mark_phase`, preserving
the pinned normalized-duration, three-bucket integer selector and its
post-duration second-phase fallback. Delay scheduling, lifecycle, palette
mapping, sprite rendering, and capture parity remain outside the helper.
It also exposes `drl_render::effect_segment_index_at_elapsed`, preserving the
signed quotient and sign correction used by cell/item animation draws while
rejecting zero durations and out-of-range results. Sprite, level, item, and
lifecycle ownership remain outside the helper.
The active follow-up also exposes
`drl_render::kill_animation_segment_index_at_elapsed`, preserving the
source's lead-delay branch, reverse-branch selection, integer segment quotient,
and terminal clamp with explicit invalid-input rejection. Actor, sprite-table,
light, and lifecycle ownership remain outside the helper.
It also exposes `drl_render::fx_animation_frame_index_at_elapsed`, preserving
the pinned FX quotient and terminal frame clamp while rejecting zero duration
or frame count. Sprite IDs, atlas columns, and effect lifecycle remain outside
the helper.
It also exposes `drl_render::move_animation_progress_at_elapsed`, preserving
the pinned normalized elapsed ratio and `[0, 1]` clamp with explicit
zero-duration rejection. Coordinate interpolation, lighting, entity state,
and lifecycle remain outside the helper.
It also exposes `drl_render::missile_step_index_at_elapsed`, preserving the
source's normalized step-delay derivation and elapsed quotient, including the
one-unit minimums for zero duration or path length. Path traversal, visibility,
particles, and lifecycle remain outside the helper.
It also exposes `drl_render::missile_ray_sample_distance_at_index`, preserving
the ray branch's strict pre-increment half-grid test, fixed 20-unit spacing,
and possible endpoint overshoot with checked arithmetic. Zero endpoint length
and unrepresentable intermediate/output values return `None`; endpoint metrics,
interpolation, path, visibility, particles, and rendering remain outside the
helper.
It also exposes `drl_render::screen_shake_fade_at_elapsed`, preserving the
screen-shake update's `1 - (elapsed / duration)^2` active envelope and its
zero-at-expiry behavior. Random frequencies, trigonometric offsets, strength,
direction, scheduling, sprite-map state, rendering, and lifecycle remain
outside the helper.
It also exposes `drl_render::particle_burst_origin_at_legacy_cell`, preserving
the pinned one-based `((cell - 1) * 32 + 16)` origin conversion and zero Z
coordinate with checked signed arithmetic. It does not convert current
zero-based positions or own direction, randomness, decals, particle-engine,
rendering, or lifecycle behavior.
The current M9 content slice adds `drl_protocol::MonsterKind::definition()` as
the Rust-owned typed table for the four implemented archetypes. Actor factories
and generated monster spawns consume this record, including knockback and death
drop metadata, while compatibility accessors and replay schemas remain stable.
The table preserves current DRL-Rust values and does not claim legacy Lua
numeric parity.
The current M9 follow-up routes all game death-drop construction through the
existing `Item::from_spawn_kind` factory, preserving item IDs, payloads,
positions, events, and replay behavior without adding item variants.
The current item-definition follow-up centralizes all nine existing
`ItemSpawnKind` families in an immutable Rust-owned table and routes the
convenience factories through it. Supplied ammunition counts, current item
properties, replay V1, and protocol schemas remain unchanged; legacy numeric
parity and broader item/level migration remain open.
The current generated-loot follow-up centralizes the procedural room-item
selection in an immutable six-entry table (`0..35` 9mm ammo with 20 rounds,
`35..55` small med-pack, `55..70` shells with 8 rounds, `70..85` shotgun,
`85..95` green armor, and `95..100` Phase Device). It preserves the single
roll, item-ID allocation, spawn ordering, and canonical item factory; it is
not a balance or legacy-parity claim.
The current generated-monster follow-up similarly centralizes the four existing
roll outcomes (`0..40` Former Human, `40..65` Imp, `65..85` Former Sergeant,
and `85..100` Demon). It preserves one roll per accepted candidate, spawn
metadata/order, and `MonsterKind`/replay/protocol boundaries; it does not claim
legacy random-placement parity or balance conclusions.
The current tile-definition follow-up adds one immutable protocol-owned table
for the five existing `TileKind` semantics and routes core `Tile` physical
flags through it. Map indexing, FOV, observations, replay V1, and protocol
schemas remain unchanged; broader level/asset migration and legacy parity stay
open.
The current level-content follow-up adds one immutable Rust-owned
`standard-procedural` definition for the existing core default (40x20, six
rooms, room sizes 4..=8, and two monster/item attempts per populated room).
Default construction and level descent read that record; caller-owned custom
configs and the MCP session's five-room policy remain explicit. Generation,
RNG consumption, replay, and MCP schemas remain unchanged; broader level
conversion and legacy parity stay open.
The current M10 follow-ups add a versioned fixed-M4 command-history snapshot
token with a bounded codec, transactional deterministic replay, a best-effort
WASM `localStorage` boundary, and a versioned same-origin service-worker cache
generated from the complete static bundle. Unsupported versions, malformed or
oversized tokens, unavailable storage, and commands rejected during restore do
not replace the active session. Replay V1, MCP schemas, and authoritative game
state remain unchanged; offline browser acceptance and cross-browser behavior
remain `NOT_RUN`, while broader PWA/cache policy remains M12 work.
Sampler edge/wrap addressing remains a backend and capture concern; the helper
does not select it.

## Delivered — M11 fixed-seed cohort and regression slices

Status: this bounded slice adds an explicit evaluation sample boundary around
the existing deterministic batch runner. `CohortConfig` records the starting
seed, episode count, and per-episode turn limit. `CohortReport` records the
caller-supplied policy identity, the exact sample definition, aggregate
`BatchSummary`, and per-seed `EpisodeRecord` replay evidence in cohort order.
Scenario and procedural cohort entrypoints reuse the existing batch behavior;
they do not alter simulation, replay, or MCP schemas.

The seed sequence is contiguous with wrapping `u64` arithmetic, so a cohort is
reproducible even when its range crosses `u64::MAX`. Repeating a cohort with
the same scenario/configuration/policy factory must produce an equal report.
This makes sample size, seed range, and timeout budget explicit before any
future balance or difficulty interpretation.

This follow-up adds `CohortTolerances` and a pure compatible-report
comparison. A comparison is available only when policy identity and the full
sample definition match; it reports absolute win-rate and average-turn deltas
and a boolean tolerance result. Non-finite or negative tolerances are rejected
without changing either report.

## Delivered — M11 cohort report integrity

Status: this delivered slice adds `CohortReport::validate`, a pure evidence gate
that requires the configured record count, contiguous wrapping seed sequence,
per-record replay seed identity, and exact `BatchSummary` recomputation to
agree. It reports the first integrity failure without rerunning episodes or
interpreting the resulting metrics.

## Delivered — M12 static release manifest and source-derived cache version

Status: this delivered slice extends `scripts/build-web.sh` with a deterministic
`dist/release-manifest.json`. The manifest records schema version, the source
Git revision (or an explicit `unknown` fallback), sorted non-generated bundle
artifacts with SHA-256 hashes, generated-file names, and the imported graphics
rights notice path (`assets/legacy/drl/graphics/LICENSE`).
`scripts/check-release-manifest.sh` verifies the schema,
path safety, hashes, rights entry, and service-worker precache coverage after a
bundle build.

The manifest is evidence for reproducible static packaging only. It is not a
signature, a cache invalidation policy, or proof of offline or cross-browser
acceptance.

The generated service-worker cache version derives from the manifest source
revision (`v1-` plus its first 12 characters), with
`v1-unknown` when the build revision cannot be resolved. The checked-in worker
template remains a marker-bearing source; only the generated bundle receives
the concrete version.

## Delivered — M12 browser support and error diagnostics

Status: this delivered slice adds a static, accessible diagnostics panel to the
browser shell. It reports missing WebGPU, startup/rendering failures, and
offline-cache registration warnings with actionable desktop-WebGPU guidance.
Diagnostics are local DOM text only: they do not inspect authoritative game
state, send telemetry, or broaden the WebGPU support claim.

## Present — M12 static-shell accessibility audit

Status: the active slice adds `scripts/check-browser-accessibility.sh`, a
deterministic static audit for the shipped HTML shell. It verifies document
language, landmark/live-region semantics, named controls and form association,
canvas keyboard focus, diagnostics semantics, inventory labeling, and a
summary for the support disclosure. It does not claim full WCAG conformance,
screen-reader acceptance, or dynamic keyboard-journey coverage.

### Observable behavior

- `BrowserSession` creates the fixed deterministic M4 arena and exposes only a
  `PlayerObservation` and `GameEvent` stream.
- Identical seed and semantic commands produce the same events, final state,
  and replay as direct `drl-core` execution.
- Keyboard movement supports arrows/WASD and numpad cardinal/diagonal keys;
  wait, pickup, reload, descend, ranged targeting, and DOM inventory actions
  map to ordinary `drl-protocol::Command` values.
- Rejected commands roll back the session and display an error without
  advancing simulation.
- `drl-render::RenderScene` consumes only the fair observation. It exposes map
  dimensions, visible/explored tiles, visible actors/items, target-ready
  positions, and HUD values; hidden world state is unavailable.
- `drl-core::batch::CohortConfig` produces a deterministic contiguous seed
  sequence, and the scenario/procedural cohort entrypoints preserve that
  definition and per-seed replay records in `CohortReport` without changing
  game execution.
- `CohortReport::compare_with` compares compatible reports using caller-owned
  non-negative finite win-rate and average-turn tolerances. It rejects policy
  or sample-definition mismatches and never mutates either report.
- `CohortReport::validate` rejects incomplete or tampered evidence when record
  count, wrapping seed order, replay seed identity, or aggregate summary no
  longer matches the cohort definition; it never reruns a simulation.
- `drl-audio` maps events to semantic cues. The WASM mixer uses generated tones,
  mute/volume settings, and user-gesture unlock; blocked audio never blocks
  gameplay.
- `drl-web::WebGpuRenderer` owns only a canvas surface. It handles DPR-aware
  resize, a deterministic scene-geometry pass, and recoverable surface
  errors; atlas layer descriptors are ready for the M8 texture compositor and
  presentation timing does not submit commands.
- The static HTTPS shell provides accessible start/status/HUD/log/inventory
  regions and prevents page scrolling only while the canvas owns focus.
- `scripts/check-browser-accessibility.sh` rejects missing names, labels,
  focusability, live-region attributes, or required shell landmarks before the
  web contract tests run.
- A release bundle contains a deterministic `release-manifest.json` with source
  revision, sorted artifact hashes, generated-file declarations, and graphics
  rights metadata; the release-manifest check rejects missing, unsafe, or
  mismatched entries and missing service-worker coverage.
- The generated service worker uses the manifest source revision prefix in its
  cache version, so two different known build revisions do not share the same
  cache name; the checked-in template remains unversioned marker input.
- Startup and presentation failures expose an accessible `#game-diagnostics`
  alert with a title, detail, and recovery guidance; it remains hidden after a
  successful start unless the offline-cache boundary reports a warning.
- Browser support text identifies desktop Chromium WebGPU as the tested target;
  missing `navigator.gpu`, unavailable WebGPU initialization, and service-
  worker failures are presented locally without network telemetry.
- The browser shell applies mute status after the asynchronous audio-unlock
  retry, so the visible status cannot report a stale unlock result.
- Mute and volume controls serialize audio-unlock/settings operations, so rapid
  presentation events cannot apply a stale setting or advance simulation.
- `drl-render::PixelViewport` chooses the largest integer tile size that fits
  the physical canvas, centers the map, and exposes deterministic tile pixel
  rectangles for presentation backends.
- The WebGPU scene uses those rectangles, preserving square cells and
  letterboxing unused canvas space without changing simulation state.
- `drl-render::LightingBand` maps a visible tile to full light and an explored
  but currently hidden tile to a fixed fog factor; the shared shade function is
  deterministic and does not inspect hidden world state.
- WebGPU tile colors consume that shared lighting rule, so browser presentation
  does not maintain a second fog multiplier with different semantics.
- `drl-render::scene_clear_color` maps fair HUD health to the existing normal or
  low-health clear tone. It is deterministic, preserves the current quarter
  health threshold, and never advances simulation.
- `drl-render::low_health_pulse_target_alpha` maps optional fair player health
  and explicit elapsed milliseconds to the source-derived instantaneous blood
  pulse target. Healthy or unavailable health returns zero; low health follows
  the observed integer-divided one-third threshold and sine phase, with a
  bounded non-negative alpha. The helper owns no wall clock or smoothing state.
- `drl-render::low_health_pulse_state_step` applies the observed
  `elapsed_ms / 500` move-toward step to a caller-selected target and
  independently decays positive pending target state. It returns both
  caller-owned values without draw-time clamping.
- `drl-render::post_process_glow_color` and
  `drl-render::post_process_lut_coordinate` preserve the observed post-process
  RGB add, `xzy` channel order, scale, offset, and coordinate clamp from the
  pinned shader. They consume caller-supplied values only; blur generation,
  LUT sampling, outline blending, and capture parity remain outside the
  current boundary.
- `drl-render::post_process_blur_taps` emits five normalized horizontal or
  vertical taps for valid caller-supplied screen dimensions. It preserves the
  pinned `weights[abs(i)]` effective weights and center-alpha index while
  rejecting zero dimensions; it performs no texture sampling or scheduling.
- `drl-render::post_process_blur_rgba` applies those effective weights to five
  caller-supplied RGB samples without renormalization or clamping and copies
  only the center sample's alpha.
- `drl-render::post_process_pass_plan` returns the bounded logical stage order:
  direct scene when both gates are off, captured scene plus composite for
  LUT-only, and captured scene plus horizontal blur, vertical blur, and
  composite when glow is enabled. The returned feature flags do not validate
  resources or execute a backend pass.
- `drl-render::explosion_mark_phase` maps caller-supplied elapsed/duration units
  through the source `(elapsed * 3) div max(duration, 1)` selector: buckets
  zero, one, and two select the first, second, and third phases, while later
  buckets use the observed second-phase fallback. It owns no delay queue,
  sprite, palette, or animation lifecycle.
- `drl-render::effect_segment_index_at_elapsed` maps caller-owned elapsed and
  duration units plus a signed target segment through the observed quotient and
  sign correction. It returns `None` for zero duration or an unrepresentable
  result and performs no sprite/level/item mutation.
- `drl-render::kill_animation_segment_index_at_elapsed` maps caller-owned
  elapsed time, stored total duration, segment count, lead delay, and reverse
  flag through the pinned kill-animation branch. It returns `None` for zero
  duration/count or an invalid forward lead delay and clamps to the terminal
  segment without consulting actor, sprite, light, or lifecycle state.
- `drl-render::fx_animation_frame_index_at_elapsed` maps caller-owned elapsed
  and duration units plus a frame count through the pinned FX integer quotient
  and terminal clamp. It returns `None` for zero duration/count and performs no
  sprite-ID, atlas-column, lifecycle, or backend work.
- `drl-render::move_animation_progress_at_elapsed` maps caller-owned elapsed
  and duration units through the pinned normalized movement ratio and clamp. It
  returns `None` for zero duration and performs no coordinate, light, entity,
  interpolation, lifecycle, or backend work.
- `drl-render::missile_step_index_at_elapsed` maps caller-owned elapsed,
  duration, and path-length units through the pinned `max`-normalized step delay
  and integer quotient. It returns `None` for a quotient outside the Rust step
  range and performs no path traversal, visibility, particle, lifecycle, or
  backend work.
- `drl-render::missile_ray_sample_distance_at_index` maps a caller-owned
  endpoint length, grid size, and sample ordinal through the pinned strict
  pre-increment ray schedule. It returns `None` for an ineligible sample or
  checked-arithmetic failure and preserves the source's possible overshoot
  without selecting a distance metric, interpolation coordinate, visibility,
  particle, lifecycle, or backend behavior.
- `drl-render::screen_shake_fade_at_elapsed` maps caller-owned elapsed and
  duration units to the source-derived quadratic fade, returning zero for zero
  or expired duration. It owns no random stream, offset, direction, strength,
  scheduling, sprite-map, rendering, lifecycle, or backend state.
- `drl-render::particle_burst_origin_at_legacy_cell` maps explicit one-based
  legacy cell coordinates to a centered integer pixel origin, returning `None`
  for signed overflow. It performs no current-position conversion, direction
  normalization, random sampling, decal selection, particle spawn, rendering,
  lifecycle, or backend work.
- `MonsterKind::definition()` returns the current Rust-owned name, HP, speed,
  combat, knockback, and death-drop record for each implemented archetype.
  Actor construction and generated spawns consume the same record; no Lua
  runtime, legacy balance migration, or replay-schema change is introduced.
- WebGPU uses that shared clear-color rule; health-tone presentation remains an
  effect and cannot reveal hidden world state.
- `drl-render::effect_timeline` preserves event order and assigns each bounded
  `PresentationEffect` a fixed logical duration. Spans are sequential and
  deterministic for identical event lists.
- Successful `BrowserSession::submit` results carry the corresponding ordered
  `EffectSpan` list in `PresentationStep::effects`; ordinary actor effects
  require endpoint visibility, while terminal hit/death targets use pre-step
  visibility so visible outcomes survive removal. Direct player transitions
  stay observable. Rejected commands carry no presentation step and do not
  mutate the session.
- Presentation ticks are frontend timing units only; tab visibility, resize,
  audio, and animation work cannot submit a simulation command. Visibility
  transitions reset the presentation baseline without changing simulation or
  effect state.
- `drl-assets::AtlasId::dimensions` records the imported PNG dimensions needed
  to validate sprite rectangles without loading image data in Rust.
- Current tile, actor, and item descriptors use the legacy 16-column, 32-pixel
  sprite-sheet slots for their corresponding semantic entries; the fallback
  entry remains a bounded FX cell.
- Descriptor bounds tests reject any current rectangle that exceeds its
  imported atlas dimensions.
- `drl-assets::AtlasId::layers` exposes the registered source-layer order for
  each imported atlas; semantic descriptors reference the matching static set.
- Layer metadata is descriptive input for a future compositor. It does not
  load, blend, or sample image data.
- The asset manifest, import/check scripts, provenance notes, and
  `LEGACY_REVISION` all use the exact 40-character pinned Git commit.
- `SpriteRect::uv_rect` returns normalized atlas coordinates for valid
  rectangles and returns `None` for zero-sized atlases or out-of-bounds cells.
- Every current semantic descriptor has in-range UV coordinates under its
  imported atlas dimensions.
- `drl-assets::AtlasId::texture_source` resolves each registered atlas layer
  to its imported relative path and measured pixel dimensions.
- `drl-assets::SpriteLayer::role` maps each registered source to the explicit
  legacy shader input role: base color, colorization mask, outline mask, or
  emissive mask.
- `drl-render::layer_draw_plan` emits ordered atlas/layer entries with a
  `PixelRect` destination, normalized UVs, and the resolved texture source for
  each visible scene sprite, including its layer role.
- `drl-render::sprite_composite_plan` groups contiguous, complete layer sets by
  stable sprite index, preserving scene order, UVs, destinations, fair
  lighting, and optional role-specific sources. Incomplete or reordered sets
  are omitted defensively.
- The browser asset URL helper accepts only relative imported PNG basenames and
  keeps deployments under a subpath; the WASM loader decodes a same-origin
  image and rejects dimensions that differ from the pinned manifest.
- `drl-web::texture_source_manifest` emits the 24 unique registered layer
  sources in stable atlas order, and the WASM renderer uploads each validated
  source once into a persistent linear `Rgba8Unorm` texture/view cache using
  the WebGPU external-image copy API, matching the observed legacy `GL_RGBA8`
  storage contract. The cache is renderer-owned and does not advance gameplay;
  display color-space parity remains capture-gated.
- The WASM renderer samples each composite's base source with a nearest-filtered
  bind group, top-left-origin UVs, alpha blending, and the fair visible or
  explored lighting factor. The paired emissive source samples its red channel
  and raises, but never lowers, that fair lighting floor. A missing emissive
  source binds a transparent 1x1 fallback; missing base cache entries skip only
  the textured pass, and the deterministic geometry fallback still presents the
  scene.
- The textured fragment shader discards sampled base alpha below the verified
  legacy `0.1` cutoff before source-alpha blending; exact-threshold fragments
  survive and all other lighting behavior is unchanged.
- The textured fragment shader samples an optional colorization-mask source and
  adds its RGB contribution multiplied by the supplied per-vertex tint. The
  fair scene path supplies the evidence-backed Green Armor tint for the
  applicable visible ground-item/player draws, the byte-quantized Phase
  Device tint for visible ground items, the pinned yellow StairsDown tile tint,
  and a neutral zero tint for other current roles; a transparent fallback is
  used when a mask source is unavailable.
- `drl-render::item_colorization_tint` maps the currently implemented
  `GreenArmor` item to `[0, 255, 0, 255]` and the Phase Device ground item to
  the evidence-backed, byte-quantized `[0, 0, 179, 255]` tint; every other
  archetype remains neutral. `drl-render::tile_colorization_tint` maps only
  `StairsDown` to `[255, 255, 0, 255]`; other current tile kinds remain
  neutral. Draw plans and grouped composites preserve these fair mappings.
- `TextureBinding` and `TextureBatch` carry the optional `SpriteLayer::Shadow`
  source as an outline-mask binding. Atlases without a registered shadow layer
  bind the transparent 1x1 fallback; the WGSL contract declares and samples
  the resource but does not blend it into the output in this transport-only
  slice.
- `SpriteDescriptor` carries pinned two-frame/500 ms metadata for the player,
  current actors, and Phase Device. Layer draws and grouped composites preserve
  that metadata; `frame_rect` advances one atlas row with bounds checks, while
  presentation still chooses progress and timing.
- `drl-render::layer_draw_plan_at_progress` accepts finite normalized progress,
  selects frame-specific UVs for evidenced animated descriptors, and keeps
  static descriptors on frame zero. Invalid progress returns `None`; the
  existing `layer_draw_plan` remains unchanged.
- `drl-render::animation_frame_index_at_elapsed` converts caller-supplied
  elapsed milliseconds through explicit `Loop` or `Clamp` policy. It rejects
  zero metadata and owns no wall clock or effect/sprite scheduling.
- `drl-render::layer_draw_plan_at_elapsed` applies that caller-supplied elapsed
  time to evidenced animated descriptors, keeps static descriptors on frame
  zero, and preserves one selected UV across each sprite composite group.
- `WebGpuRenderer::render_at_elapsed` forwards caller-supplied elapsed time and
  playback policy to the textured pass; malformed elapsed plans take the
  existing geometry fallback, while `render` remains frame zero.
- The browser shell's bounded `requestAnimationFrame` loop converts finite,
  monotonic callback timestamps to elapsed milliseconds, skips hidden-document
  frames, and never submits simulation commands. A scheduling failure leaves
  gameplay available.
- The WGSL source is shared with a native shader-contract test that checks the
  base/emissive/mask samples, fair-lighting `max`, tint forwarding, neutral
  fallback input, alpha cutout, and output path; native tests therefore guard
  the WASM-only runtime shader text.
- Every layer draw carries its fair `LightingBand`: explored tile memory uses
  the fixed explored factor, while visible tiles/items/actors use full light.
- `drl-render::active_effect_frames` returns normalized `[0, 1)` progress for
  active effect spans at a presentation tick, preserving input order and
  omitting zero-duration or overflowed spans.
- `drl-render::animation_frame_index` maps finite normalized progress and a
  caller-supplied nonzero frame count to a deterministic zero-based frame
  index; invalid progress or frame counts return `None`.
- Layer draw planning consumes only `RenderScene`; it cannot inspect hidden
  simulation state or advance gameplay.
- `scripts/check-reference-capture.sh` validates required capture metadata,
  pinned revision/scenes, executable/hash consistency, status vocabulary,
  placeholder policy, recorded legacy dirty state, evidence classification,
  rights status, and media-hash syntax without executing the legacy binary or
  changing `NOT_RUN`.

### Public contracts

- Additive `PlayerObservation::{map_width, map_height, player_hp}`.
- Additive `ActorView::monster_kind`, `ItemArchetype`, and
  `ItemView::archetype`.
- `PresentationStep { before, command, events, effects, after }`, `RenderScene`,
  `AudioCue`, `BrowserSession`, `tile_colorization_tint`, and WASM-only
  `WebGpuRenderer`.
- MCP JSON serialization and replay schemas remain unchanged. `drl-core` has
  no presentation, browser, audio, filesystem, network, or MCP dependency.
- `drl_core::{CohortConfig, CohortReport}` and the two `BatchRunner` cohort
  entrypoints are headless evaluation helpers; they do not expose hidden state
  to agents or frontends.
- `CohortTolerances` and `CohortComparison` are regression-gate math only;
  they do not claim statistical significance, balance correctness, or a
  difficulty target.
- The release manifest is unsigned metadata; signing, cache invalidation,
  offline acceptance, and cross-browser claims remain later M12/M13 work; the
  source-derived cache version is only deterministic naming, not a release
  invalidation policy. The diagnostics panel is a local recovery surface, not
  a browser compatibility guarantee or telemetry channel. Cohort report
  integrity validation is an evidence-consistency gate, not a balance,
  difficulty, or statistical-significance claim. The static accessibility
  audit is a shell contract, not a full WCAG or screen-reader acceptance
  result.

### Verification

Local checks:

```text
sh scripts/check-repository.sh              PASS (baseline plus new crates)
sh scripts/check-assets.sh                  PASS (32 PNGs, license, hashes)
scripts/check-reference-capture.sh          PASS (`NOT_RUN` on arm64 macOS)
scripts/test-reference-capture.sh           PASS (fixture coverage)
scripts/check-release-manifest.sh           PASS (after `scripts/build-web.sh`)
scripts/check-browser-diagnostics.sh        PASS (via `scripts/check-web.sh`)
scripts/check-browser-accessibility.sh      PASS (via `scripts/check-web.sh`)
cargo check --locked -p drl-web --target wasm32-unknown-unknown  PASS
cargo test -p drl-render                      PASS (pixel-grid, lighting, tone, and timeline contracts)
cargo test -p drl-core --test batch_simulation PASS (fixed-seed cohort sample,
                                             identity, wrapping, replay
                                             determinism, tolerance-gate, and
                                             integrity contracts)
cargo test -p drl-assets                      PASS (slot mappings, atlas bounds,
                                             layer sets, roles, descriptor order,
                                             UVs)
cargo test -p drl-render                      PASS (layer draw ordering,
                                             screen/UV geometry, grouped
                                             sources, lighting factors, and
                                             effect progress)
cargo test -p drl-web                         PASS (asset URL/path, dimension,
                                             and effect-handoff contracts)
scripts/check-web.sh                        PASS for native/WASM builds;
                                             browser runner NOT_RUN if Chrome absent
scripts/build-web.sh                         PASS (release bundle in ignored dist/)
Chrome 151 WebGPU smoke playthrough          PASS (Apple Metal-3, 1280x720, DPR 1;
                                             start with explicit gesture-gated
                                             audio state, move, mute, restart;
                                             pixel-grid scene visible after move)
GitHub Actions run 32548718408               PASS (repository + Ubuntu WASM
                                             browser jobs for the preceding
                                             effect-progress slice)
```

The local and hosted functional gates pass for the preceding presentation
slices; this preflight slice must add its own hosted run to the handoff
artifact. Legacy runtime and audiovisual comparison remain `NOT_RUN`.

local browser execution remains `NOT_RUN` when the runner is unavailable. The
existing Chrome run records browser/version, OS,
adapter/backend, viewport, DPR, build revision, and audio unlock/mute state,
but the fidelity-matrix comparison remains `NOT_RUN` until a controlled legacy
capture is available.

### Explicit non-goals

- WebGL2 fallback, Firefox/Safari, mobile/touch, controllers, and native
  desktop packaging are post-1.0.
- No runtime Lua, new gameplay breadth, save persistence, accounts, backend
  service, or parallel JavaScript gameplay state.
- Legacy audio/music/fonts are not shipped until rights are documented.
- Full audiovisual equivalence is M8: capture-backed tolerances, visual
  regressions, cue timing, and structured human comparison.
- Visible outline/glow equations, effect ownership, broader content-specific
  animation timing, additional per-sprite tint sources, and capture-backed
  legacy shader equivalence remain future M8 slices. The M11 cohort and
  tolerance/integrity boundaries intentionally record and validate evidence
  and declared deltas without claiming a balance result, difficulty target, or
  statistical significance.

## Next

The fixed-seed cohort report and integrity gate are covered by focused
headless tests; the next M11 slice is balance/evaluation work. The pixel-scale
viewport,
atlas metadata, UV geometry, draw-plan source
metadata, fair lighting factors, effect progress, layer input roles, grouped
sprite composites, validated browser source loading, renderer-owned GPU
texture upload, base-color sampling, the emissive lighting floor, the legacy
alpha cutoff, optional mask sampling, the Green Armor, Phase Device, and
StairsDown tint boundaries, outline-mask GPU transport, caller-supplied frame
selection, and
the evidenced player/actor/Phase Device animation metadata, progress-driven
frame plans, elapsed-time layer plans, caller-driven elapsed WebGPU forwarding,
and bounded browser scheduling with visibility-lifecycle rebasing are covered
by local checks and hosted WASM browser jobs. Continue M8 with visible
outline/glow compositing, broader content, additional tint sources, or
capture-backed measurement of
lighting, effects, typography, and audio. Do not claim audiovisual parity from
renderer-neutral grouping or the `NOT_RUN` legacy captures.
