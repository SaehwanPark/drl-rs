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

## Present — M8 animation frame selector

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
This slice adds pure frame-index selection from caller-supplied normalized
effect progress. Asset-specific frame counts, legacy animation timing,
per-sprite tint sourcing, outline/glow equations, and capture-backed
audiovisual equivalence remain open.

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
- `drl-audio` maps events to semantic cues. The WASM mixer uses generated tones,
  mute/volume settings, and user-gesture unlock; blocked audio never blocks
  gameplay.
- `drl-web::WebGpuRenderer` owns only a canvas surface. It handles DPR-aware
  resize, a deterministic scene-geometry pass, and recoverable surface
  errors; atlas layer descriptors are ready for the M8 texture compositor and
  presentation timing does not submit commands.
- The static HTTPS shell provides accessible start/status/HUD/log/inventory
  regions and prevents page scrolling only while the canvas owns focus.
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
  audio, and animation work cannot submit a simulation command.
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
  current fair scene path supplies a neutral zero tint, and a transparent
  fallback is used when a mask source is unavailable.
- The WGSL source is shared with a native shader-contract test that checks the
  base/emissive/mask samples, fair-lighting `max`, neutral tint input, alpha
  cutout, and output path; native tests therefore guard the WASM-only runtime
  shader text.
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

### Public contracts

- Additive `PlayerObservation::{map_width, map_height, player_hp}`.
- Additive `ActorView::monster_kind`, `ItemArchetype`, and
  `ItemView::archetype`.
- `PresentationStep { before, command, events, effects, after }`, `RenderScene`,
  `AudioCue`, `BrowserSession`, and WASM-only `WebGpuRenderer`.
- MCP JSON serialization and replay schemas remain unchanged. `drl-core` has
  no presentation, browser, audio, filesystem, network, or MCP dependency.

### Verification

Local checks:

```text
sh scripts/check-repository.sh              PASS (baseline plus new crates)
sh scripts/check-assets.sh                  PASS (32 PNGs, license, hashes)
cargo check --locked -p drl-web --target wasm32-unknown-unknown  PASS
cargo test -p drl-render                      PASS (pixel-grid, lighting, tone, and timeline contracts)
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

The local and hosted functional gates pass for the preceding textured-pass
slices; this pure frame-selector slice must add its own hosted run to the
handoff artifact.

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
- Per-sprite tint sourcing, outline/glow, legacy animation timing/frame
  metadata, and capture-backed legacy shader equivalence remain future M8
  slices; this pass is renderer-neutral groundwork only.

## Next

The pixel-scale viewport, atlas metadata, UV geometry, draw-plan source
metadata, fair lighting factors, effect progress, layer input roles, grouped
sprite composites, validated browser source loading, renderer-owned GPU
texture upload, base-color sampling, the emissive lighting floor, the legacy
alpha cutoff, optional neutral-tint mask sampling, and caller-supplied frame
selection are covered by local checks and hosted WASM browser jobs. Continue
M8 with per-sprite tint sourcing, outline/glow, legacy animation timing,
or capture-backed measurement of lighting, effects, typography, and audio. Do
not claim audiovisual parity from renderer-neutral grouping or the `NOT_RUN`
legacy captures.
