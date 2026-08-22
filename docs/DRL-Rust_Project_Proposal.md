# DRL-Rust Project Proposal

Version: browser-first steering revision, 2026-08-21

## Executive summary

DRL-Rust is a ground-up Rust reimplementation of Doom the Roguelike that
preserves the original's turn-based rules while making the first product
reachable in a modern desktop browser. Rust/WASM owns the deterministic game
and presentation contracts; WebGPU/wgpu renders the pixel-rich world to a
canvas; semantic DOM regions provide start, error, help, HUD, and inventory
accessibility; Web Audio supplies event-driven cues after a user gesture. The
bundle is static-hostable and becomes offline-capable through later PWA work.

Native headless execution and MCP remain supported for agents, regression
tests, and tooling. Native desktop packaging is optional post-1.0 portability,
not the product gate.

## Vision and success criteria

The product should feel like DRL: deterministic turn semantics, readable fog
and targeting, layered sprites, clear combat feedback, responsive animation,
and meaningful sound. Success is measured in layers:

1. Core parity: identical seed/command streams produce identical simulation
   events, final state, and replay in headless, MCP, and browser sessions.
2. Fairness: a browser renderer consumes only player observations and semantic
   events; hidden actors/items never enter ordinary frontend state.
3. Playability: desktop Chrome/Edge WebGPU users can start a fixed M4 run,
   move, wait, loot, equip/use/drop, reload, target, fight, descend, die, and
   restart with keyboard and accessible DOM controls.
4. Fidelity: M8 compares named scenes and effect sequences with approved
   legacy captures, stated tolerances, automated regressions, and structured
   human review.
5. Distribution: M12/M13 publish a reproducible static HTTPS/PWA bundle with
   license notices and explicit browser support/error behavior.

The full graphics atlas may be reused under its recorded CC BY-SA 4.0 terms.
Legacy code (GPL), audio/music binaries, and fonts are not implicitly covered
by the repository MIT license and stay out of the bundle until separately
cleared.

## Product boundary

```text
DOM start/input/accessibility
  -> winit/browser key binding
  -> drl-protocol::Command
  -> drl-web::BrowserSession
  -> drl-core::Game
  -> PlayerObservation + GameEvent
  -> drl-render::RenderScene / drl-audio::AudioCue
  -> WebGPU canvas / Web Audio
```

`drl-core` has no GPU, DOM, Web Audio, filesystem, network, MCP, or wall-clock
dependency. Presentation timing, resize/DPR, page visibility, GPU loss, and
audio policy cannot submit commands or advance turns. JavaScript exposes only
WASM boot and semantic input; authoritative gameplay state remains in Rust.

## Technical approach

### Simulation and protocol

Keep the existing `drl-core` deterministic kernel, `drl-protocol` commands,
events, observations, scenarios, and replay schema. Add only additive fair
frontend fields: map width/height, player HP, actor `MonsterKind`, and stable
`ItemArchetype`. MCP JSON remains the existing wire shape and replay V1 stays
bit-compatible.

### Assets and content

`drl-assets` is a platform-neutral semantic atlas descriptor crate. The full
tracked legacy graphics directory is imported from the pinned Git revision
with source paths, attribution, license, and checksums. `drl-script` is a
build-time conversion/content boundary; Lua is research input, not a WASM
runtime. Each unresolved content or rights question remains explicit.

### Browser presentation

`drl-render` builds `PresentationStep` and `RenderScene` from observations.
`drl-web` is `cdylib + rlib` and uses pinned `wgpu 30.0.0`, `winit 0.30.12`,
`wasm-bindgen 0.2.127`, web bindings, and `wasm-bindgen-test`. WebGPU is the
initial desktop Chromium backend; shaders remain in a future WebGL2-compatible
subset. M7 presents deterministic scene geometry with fog/explored tinting,
actors/items, targeting, HUD, pixel scaling, and bounded effects; the
descriptor-driven base/emissive/shadow texture compositor is an M8 task.

`drl-audio` maps events to semantic `AudioCue` values and generates placeholder
tones in M7. A start-button gesture unlocks Web Audio; mute, volume, blocked
audio, and suspension are recoverable presentation states. Cleared legacy or
replacement cues arrive with M8 evidence.

### Distribution and operations

Build with `wasm-pack 0.15.0` into ignored `dist/`; serve from any static HTTPS
host. The M10 bundle now has a manually versioned same-origin service-worker
cache for offline-after-first-load, while M12 adds broader PWA cache/version
policy and release hardening. An Ubuntu remote job installs the WASM target and
pinned wasm-pack, builds the release bundle, and runs headless Chrome tests; no
gameplay backend is required.

## Evidence and fidelity

Legacy source evidence is classified as observed, inferred intent,
implementation artifact, ambiguous, or DRL-Rust decision. Reference captures
record legacy revision, dirty state, executable hash, frontend/configuration,
viewport, DPR, ordered actions, capture tools, media hashes, and rights. On the
current arm64 macOS host the available legacy binary is Linux x86-64, so runtime
capture is `NOT_RUN` and remains a controlled Linux x86-64 task. The
scene-to-milestone acceptance map is maintained in
`docs/reference-captures/fidelity-matrix.md`; no comparison or golden
expectation is promoted from an unrecorded or unlicensed capture.

## Delivery and quality

The roadmap is canonical and `SPEC.md` contains one active slice. The
milestone owner serializes writes to canonical documents; the repo-local
harness discovers and validates every skill. Every slice runs
`sh scripts/check-repository.sh`; M3 runs asset checks; M7 runs WASM/build and
browser checks; M8 adds capture-backed visual/audio regressions. Local, remote,
playability, replay, MCP, and fidelity statuses remain independently reported
as `PASS`, `FAIL`, `INCONCLUSIVE`, or `NOT_RUN`.

## Scope exclusions

No runtime Lua, legacy architecture translation, new gameplay breadth beyond
the bounded M4 slice, save/account/backend work before its milestones, mobile
or touch UX, WebGL/Firefox/Safari acceptance, controller support, or native
desktop packaging is included in the first browser playable slice.

## Decision record

See [ADR 0007](adr/0007-browser-first-product.md) for the product/backend
decision and [ADR 0008](adr/0008-build-time-legacy-content-migration.md) for
the build-time Lua/content migration decision. Earlier ADRs remain valid where
they do not conflict; ADR 0005 is superseded.
