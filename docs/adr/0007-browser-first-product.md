# ADR 0007: Browser-first product and presentation boundary

- Status: Accepted
- Date: 2026-08-21

## Context

The original plan prioritized a native macOS executable, but the product now
needs a broadly reachable game with the original DRL presentation richness.
The simulation and protocol are already platform-neutral; rendering/audio are
placeholders and there is no reason to make operating-system packaging a
prerequisite for the first playable slice.

## Decision

DRL-Rust 1.0 is a browser-first Rust/WASM product. `drl-core` remains a pure,
deterministic simulation and `drl-protocol` remains the only semantic boundary.
`drl-render` builds deterministic scenes and the first browser backend uses
`wgpu` WebGPU on a canvas with nearest-neighbor pixel scaling. Semantic DOM
regions own start/error/help/HUD/inventory accessibility. `drl-audio` maps
events to semantic cues and unlocks Web Audio from a user gesture. The product
is distributed as a static HTTPS bundle; PWA/offline productization is a
later milestone. Desktop Chromium (Chrome/Edge) is the initial acceptance
matrix. WebGL2 fallback, Firefox/Safari, touch/controller input, and native
desktop packaging are post-1.0.

## Consequences

- Presentation failures, animation, tab visibility, resize/DPR, and audio
  policy can never advance simulation.
- Browser acceptance requires a reproducible seed/command parity check and
  browser/GPU/viewport/DPR/audio metadata.
- The renderer must consume player observations only; hidden world state stays
  in the core.
- Native headless `drl-app` and MCP stdio remain supported for tooling and
  deterministic regression tests.
