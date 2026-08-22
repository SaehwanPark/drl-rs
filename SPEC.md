# Specification

The [roadmap](docs/DRL-Rust_Project_Roadmap.md) owns milestone order and
progress. This file expands exactly one active implementation slice.

## Past

- M0–M6 established the Rust workspace, deterministic headless simulation,
  FOV/fog, combat/items/levels, replay/scenario/bot infrastructure, and MCP.
- M0 steering was reconciled to browser-first delivery. ADRs 0007 and 0008
  record the accepted Rust/WASM/WebGPU and build-time-content decisions.
- M3 asset infrastructure imported the complete tracked legacy graphics atlas
  from Git revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c5`, with CC BY-SA
  attribution and SHA-256 checksums. Legacy audio, music, and fonts remain
  redistribution-gated; its controlled reference-capture gate is `NOT_RUN` on
  arm64 macOS and remains an M8 acceptance dependency.

## Present — M8 measured atlas descriptors

Status: The M7 browser slice passed functional acceptance locally and in
remote web CI. The delivered M8 pixel-grid, visibility-band, low-health tone,
and fair effect-span slices share pure presentation rules. This bounded
follow-up replaces placeholder atlas cells with measured 32-pixel slots from
the pinned legacy sprite-sheet grid and exposes atlas dimensions for bounds
validation. It remains platform-neutral and is not a capture-backed parity
claim.

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
cargo test -p drl-assets                      PASS (measured slot mappings and atlas bounds)
cargo test -p drl-web                         PASS (effect handoff preserves event/timeline parity)
scripts/check-web.sh                        PASS for native/WASM builds;
                                             browser runner NOT_RUN if Chrome absent
scripts/build-web.sh                         PASS (release bundle in ignored dist/)
Chrome 151 WebGPU smoke playthrough          PASS (Apple Metal-3, 1280x720, DPR 1;
                                             start with explicit gesture-gated
                                             audio state, move, mute, restart;
                                             pixel-grid scene visible after move)
GitHub Actions run 32545325186               PASS (repository + Ubuntu WASM
                                             jobs for the measured atlas
                                             descriptor slice)
```

The local and remote functional gates pass for this slice. The remote run
records repository and Ubuntu WASM checks; the local browser runner is
`NOT_RUN`. The existing Chrome run records browser/version, OS,
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
- Texture upload, shader sampling, animation frame selection, and layer
  compositing remain future M8 slices.

## Next

The pixel-scale viewport is covered by local in-app Chrome smoke and the
hosted WASM browser job. Continue M8 only with capture-backed measurement of
atlas rectangles, lighting, effects, typography, and audio. Do not claim
audiovisual parity from the current placeholder atlas rectangles or the
`NOT_RUN` legacy captures.
