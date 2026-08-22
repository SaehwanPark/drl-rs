# Architecture

Last reviewed: 2026-08-22

Status: Verified for the current headless and browser-slice implementation;
full audiovisual parity remains planned.

## Boundaries

```text
HTML/DOM input and accessibility shell
  -> drl-web::BrowserSession / semantic Command
  -> drl-core::Game (deterministic simulation)
  -> PlayerObservation + GameEvent
  -> drl-render::RenderScene -> WASM WebGPU canvas
  -> drl-audio::AudioCue -> user-unlocked Web Audio
```

`drl-core` is the authority for world state, legality, action costs, seeded
randomness, replay, and events. It does not depend on rendering, audio,
browser APIs, filesystem, network, or MCP. `drl-protocol` contains semantic
commands, observations, events, item/actor identifiers, scenarios, and replay
types shared by core, MCP, bots, and frontends.

## Workspace crates

- `drl-core`: pure deterministic maps, FOV/fog, combat, AI, items, levels,
  scenarios, bots, batches, and replay execution.
- `drl-protocol`: stable semantic boundary. Player observations now include
  map dimensions and player HP; actor/item views include stable presentation
  identifiers. MCP wire serialization and replay schema remain compatible.
- `drl-assets`: platform-neutral atlas IDs, imported PNG dimensions, measured
  32-pixel rectangles, deterministic registered layer sets and shader input
  roles, semantic
  tile/actor/item lookup, normalized UV geometry, deterministic texture-source
  bindings, and legacy revision identity.
  It has no decoder or platform dependency and core does not depend on it.
- `drl-render`: deterministic `PresentationStep`, `RenderScene`, target
  candidates, bounded event-to-effect builders, observation-independent
  `PixelViewport`/`PixelRect` layout math, visibility-derived
  `LightingBand`/`shade_color` rules, health-derived `SceneTone`/clear color,
  event-ordered `EffectSpan` timing, and renderer-neutral `LayerDraw` plans
  carrying atlas layers, imported source metadata, explicit layer roles, fair
  lighting, evidence-backed Green Armor/Phase Device colorization tint,
  optional outline-mask transport, evidenced sprite animation metadata and
  caller-supplied progress-selected UVs, pixel destinations, and normalized
  UVs;
  `sprite_composite_plan`
  groups complete role sets for a future backend. It consumes player
  observations/events
  only; `active_effect_frames` maps those fair spans to frontend progress;
  GPU ownership is in the WASM shell.
  `drl-web` carries fair, visibility-filtered spans through each successful
  `PresentationStep`.
- `drl-audio`: deterministic event-to-`AudioCue` mapping and a WASM Web Audio
  mixer with explicit user-gesture unlock, mute, and volume state.
- `drl-web`: `cdylib + rlib` browser session, Winit/DOM command mapping,
  static shell exports, WebGPU scene-geometry surface, DPR resize, deterministic
  square-cell pixel layout, validated same-origin texture-source decode, a
  renderer-owned WebGPU texture/view cache, and recoverable GPU/audio status. It
  never mirrors authoritative gameplay state into JavaScript.
- `drl-mcp`: zero-dependency JSON-RPC/MCP semantic server and fairness boundary.
- `drl-app`: native headless demo and MCP stdio runner, retained for tooling.
- `drl-script`: conversion/content boundary placeholder; no runtime Lua.

## Data and effect rules

- Browser flow is `Command -> Game -> before observation/events/after
  observation -> scene/cues -> effects`. A failed command restores the browser
  session checkpoint and reports an error.
- Rendering, animation, audio, tab visibility, resize/DPR, and GPU loss never
  advance the simulation.
- `PixelViewport` chooses an integer square cell size from physical canvas
  dimensions and centers the map; presentation backends may letterbox but may
  not stretch logical cells independently by axis.
- `LightingBand` derives only from the fair tile visibility bit: visible tiles
  use full light and explored memory uses the fixed fog factor. It cannot
  reveal or consult hidden simulation state.
- `SceneTone` derives only from fair player HP and preserves the existing
  quarter-health threshold; its clear color is a presentation effect, never a
  simulation or hidden-state channel.
- `EffectSpan` uses fixed logical durations and event order only. Frontends may
  map ticks to frames, but presentation timing cannot advance the simulation.
- `active_effect_frames` reports normalized progress only for the supplied
  spans at the supplied frontend tick. It omits zero-duration/overflowed spans
  and cannot create new events or inspect simulation state.
- `animation_frame_index` maps that normalized progress to a caller-supplied
  nonzero frame count. It is pure frontend math and does not infer asset
  metadata or legacy timing.
- Atlas descriptors convert the pinned legacy one-based, sixteen-column
  sprite-sheet slots to bounded 32-pixel cells. Dimensions are metadata from
  the imported PNGs; no image decoding or texture upload occurs in this crate.
- Each descriptor carries the exact available source-layer set for its atlas in
  registration order. The list is metadata only; no blending or sampling is
  performed at this boundary.
- `SpriteLayer::role` names the independent legacy shader input represented by
  each source: base color, colorization mask, outline mask, or emissive mask.
  It does not prescribe backend blend equations.
- `SpriteRect::uv_rect` converts bounded image-space cells to normalized
  top-left-origin UVs. A backend owns any texture-origin inversion.
- `layer_draw_plan` emits atlas layers in scene order (tiles, items, actors),
  retaining explored tile memory for fog presentation while omitting unknown
  tiles and invalid/off-viewport geometry. It is metadata and geometry only:
  no decoder, texture upload, blending, or sampling occurs here.
- `sprite_composite_plan` groups one complete registered layer set per stable
  sprite index. It rejects incomplete, reordered, duplicate, or mismatched
  groups and still performs no image or GPU work.
- `AtlasTextureSource` records the relative imported path and measured
  dimensions for a registered atlas layer. Frontends own file loading and
  image/GPU resource lifetime.
- `drl-web::browser_asset_url` rejects absolute, traversal, query, fragment,
  and unsupported-character paths before constructing a subpath-safe graphics
  URL. The WASM loader decodes the image and validates its natural dimensions;
  no GPU object is created by this preflight boundary.
- `drl-web::texture_source_manifest` keeps the 24 unique registered layer
  sources in deterministic order. WASM boot uploads each validated source once
  with `Queue::copy_external_image_to_texture` into linear `Rgba8Unorm` storage
  and retains its texture/view;
  the base-color pass samples those views with nearest filtering, the paired
  emissive view raises the fair lighting floor from its red channel, and the
  optional colorization-mask view is sampled with a neutral zero per-vertex
  tint except for the evidence-backed Green Armor value on visible ground
  items/player equipped armor and the byte-quantized Phase Device value on
  visible ground items. Missing optional roles use a retained transparent 1x1
  fallback; outline-mask sources are now transported and bound but not blended;
  additional per-sprite tint sources and visible outline/glow compositing remain
  later boundaries. Player/current actor/Phase Device descriptor metadata is
  carried through layer plans and grouped composites; callers may provide
  normalized progress for pure UV selection, while timing remains a frontend
  responsibility. The WGSL pass
  discards base fragments below the verified `0.1`
  alpha cutoff before source-alpha blending. The WGSL source is defined at the
  crate boundary so a native contract test can guard its binding and
  compositing terms.
- `LayerDraw::lighting` carries the fair visibility band for the source sprite;
  explored tile memory is shaded by the shared fog factor and visible scene
  sprites use full light. A compositor must not derive this from hidden state.
- `PresentationStep::effects` is computed at the command boundary from the
  returned event list and visible actor sets. Ordinary effects require both
  endpoints; terminal hit/death effects use pre-step visibility. Rejected
  commands produce no step and no effects.
- Player scenes use only visible/explored observations. Omniscient views remain
  debug-only and are not available to ordinary browser input.
- Imported legacy graphics are tracked under `assets/legacy/drl/graphics/`
  with license, source revision, and checksums. Audio/music/fonts require
  separate rights records.
- WebGPU is the initial desktop Chromium backend. The shader/presentation
  design stays within a WebGL2-compatible subset, but the fallback and other
  browsers are post-1.0.

## Verification

- `crates/drl-core/tests/boundaries.rs` guards dependency direction and
  determinism.
- `scripts/check-assets.sh` verifies the imported atlas manifest.
- `crates/drl-web` native tests verify command mapping, transactional errors,
  and observation-scene construction; the WASM target check compiles WebGPU,
  winit, Web Audio, and bindings.
- `scripts/check-web.sh` and the Ubuntu remote job own WASM/browser checks.
- Reference-capture metadata lives in
  `docs/reference-captures/manifest.md`; current legacy runtime capture is
  `NOT_RUN` because the available binary is Linux x86-64 and the local host is
  arm64 macOS.

## Invariants

- No ambient RNG, wall-clock gameplay, platform I/O, or unstable iteration in
  `drl-core`.
- All clients (headless, MCP, browser, future native) submit ordinary
  `drl-protocol::Command` values.
- Any future presentation backend must consume semantic observations/events,
  not `World`, and must preserve bit-exact replay behavior.
