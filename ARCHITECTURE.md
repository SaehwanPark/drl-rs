# Architecture

Last reviewed: 2026-08-21

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
- `drl-assets`: platform-neutral atlas IDs, layer descriptors, rectangles,
  semantic tile/actor/item lookup, and legacy revision identity. It has no
  decoder or platform dependency and core does not depend on it.
- `drl-render`: deterministic `PresentationStep`, `RenderScene`, target
  candidates, bounded event-to-effect builders, observation-independent
  `PixelViewport`/`PixelRect` layout math, and visibility-derived
  `LightingBand`/`shade_color` rules. It consumes player observations/events
  only; GPU ownership is in the WASM shell.
- `drl-audio`: deterministic event-to-`AudioCue` mapping and a WASM Web Audio
  mixer with explicit user-gesture unlock, mute, and volume state.
- `drl-web`: `cdylib + rlib` browser session, Winit/DOM command mapping,
  static shell exports, WebGPU scene-geometry surface, DPR resize, deterministic
  square-cell pixel layout, and recoverable GPU/audio status. It never mirrors
  authoritative gameplay state into JavaScript.
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
