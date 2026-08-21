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

## Present — M7 Browser-playable M4 slice

Status: Functional acceptance passed locally and in remote web CI; the
capture-backed fidelity comparison remains open.

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

### Public contracts

- Additive `PlayerObservation::{map_width, map_height, player_hp}`.
- Additive `ActorView::monster_kind`, `ItemArchetype`, and
  `ItemView::archetype`.
- `PresentationStep { before, command, events, after }`, `RenderScene`,
  `AudioCue`, `BrowserSession`, and WASM-only `WebGpuRenderer`.
- MCP JSON serialization and replay schemas remain unchanged. `drl-core` has
  no presentation, browser, audio, filesystem, network, or MCP dependency.

### Verification

Local checks:

```text
sh scripts/check-repository.sh              PASS (baseline plus new crates)
sh scripts/check-assets.sh                  PASS (32 PNGs, license, hashes)
cargo check --locked -p drl-web --target wasm32-unknown-unknown  PASS
scripts/check-web.sh                        PASS for native/WASM builds;
                                             browser runner NOT_RUN if Chrome absent
scripts/build-web.sh                         PASS (release bundle in ignored dist/)
Chrome 151 WebGPU smoke playthrough          PASS (Apple Metal-3, 1280x720, DPR 1;
                                             start with explicit gesture-gated
                                             audio state, move, mute, restart)
GitHub Actions run 32537824075               PASS (repository + Ubuntu WASM jobs)
```

The local and remote functional gates pass. The run records browser/version,
OS, adapter/backend, viewport, DPR, build revision, and audio unlock/mute
state, but the fidelity-matrix comparison remains `NOT_RUN` until a controlled
legacy capture is available.

### Explicit non-goals

- WebGL2 fallback, Firefox/Safari, mobile/touch, controllers, and native
  desktop packaging are post-1.0.
- No runtime Lua, new gameplay breadth, save persistence, accounts, backend
  service, or parallel JavaScript gameplay state.
- Legacy audio/music/fonts are not shipped until rights are documented.
- Full audiovisual equivalence is M8: capture-backed tolerances, visual
  regressions, cue timing, and structured human comparison.

## Next

After the controlled capture and reference-scene comparison pass, move this
slice to Past and activate the first bounded M8 audiovisual-parity slice. Do
not claim M8 fidelity from the current placeholder atlas rectangles or the
`NOT_RUN` legacy captures.
