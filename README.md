# DRL-Rust

DRL-Rust is a ground-up Rust reimplementation of Doom the Roguelike. The
product direction is browser-first: a deterministic Rust/WASM game rendered
with WebGPU in desktop Chrome/Edge, with an accessible HTML shell and
gesture-unlocked Web Audio. Headless Rust and MCP remain supported for agents,
replays, and regression testing.

The current repository contains a playable browser-slice implementation and a
complete deterministic M4 simulation. M7 functional checks pass locally and in
remote web CI; M8 presentation work now keeps browser map cells square with
deterministic pixel-grid letterboxing, shared visible/explored lighting bands,
measured legacy atlas slots, registered layer metadata, normalized sprite UVs,
renderer-neutral layer draw plans with imported source metadata, fair lighting
factors, explicit legacy shader input roles, grouped sprite composite plans,
and validated browser texture-source loading, renderer-owned GPU texture
uploads using linear normalized atlas storage, a native-tested partial
nearest-filtered base/mask/emissive textured pass with evidence-backed Green
Armor and Phase Device ground colorization tint boundaries, the verified
emissive lighting floor, optional outline-mask GPU transport, and evidenced
player/actor/Phase Device animation metadata,
plus pure effect progress, caller-supplied animation frame selection, a
low-health scene tone, and event-ordered timing. Full
audiovisual equivalence, broader content, PWA persistence, and other browsers
remain staged roadmap work. Do not read the placeholder M7 atlas rectangles as
a fidelity claim.

## Quick start

### Headless and MCP

```sh
cargo run -p drl-app
cargo run -p drl-app -- --mcp
```

### Browser slice

Prerequisites: Rust, the `wasm32-unknown-unknown` target, and `wasm-pack
0.15.0`.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0 --locked
scripts/build-web.sh
scripts/serve-web.sh
```

Open `http://localhost:8080` over the local static server, press Start to
unlock audio/WebGPU, and focus the canvas. Arrows/WASD and numpad move;
Space/`.` waits; G picks up; R reloads; F selects the nearest visible enemy,
Enter fires, and Escape cancels; `>` descends. Numpad 7/9/1/3 are the
documented diagonal bindings. Inventory controls are exposed through the
semantic DOM region.

If WebGPU is unavailable, the page shows an explicit unsupported-device
status. Audio may remain suspended until a trusted user gesture; that state is
recoverable and never advances the game.

## Verification

```sh
sh scripts/check-repository.sh
sh scripts/check-assets.sh
scripts/check-web.sh
```

`check-web.sh` compiles the WASM target and runs native contract tests. It runs
headless Chrome WASM tests when Chrome is installed; otherwise it reports the
browser run as `NOT_RUN`. Remote Ubuntu CI owns the required web-CI evidence.

## Architecture

```text
DOM / keyboard -> drl-protocol::Command -> drl-core::Game
               -> PlayerObservation + GameEvent
               -> drl-render / drl-audio -> WebGPU canvas / Web Audio
```

The core has no rendering, audio, browser, filesystem, network, or MCP
dependency. Frontends consume fair player observations, never `World`.
`drl-assets` contains platform-neutral semantic atlas descriptors and licensed
legacy graphics metadata; it is not a dependency of the core.

## Workspace layout

- `crates/drl-core`: deterministic simulation, combat, FOV, AI, levels, items,
  scenarios, bots, batches, and replays.
- `crates/drl-protocol`: commands, observations, events, identifiers, and
  compatibility-sensitive MCP/replay contracts.
- `crates/drl-assets`: atlas IDs, dimensions, measured rectangles, registered
  layers and shader input roles, normalized UVs, texture-source bindings,
  semantic asset mapping, and
  legacy revision identity.
- `crates/drl-render`: pure scene construction, deterministic pixel viewport
  layout, atlas/layer draw-plan geometry with source metadata, roles, grouped
  composites, lighting, normalized effect progress, and caller-supplied
  animation frame selection and the observed Green Armor/Phase Device tint
  mappings, outline-mask transport, and renderer-neutral animation metadata.
- `crates/drl-audio`: semantic cues and WASM Web Audio mixer.
- `crates/drl-web`: browser session, validated same-origin texture-source
  loading, renderer-owned WebGPU texture uploads, the partial base/mask/
  emissive textured pass with Green Armor/Phase Device colorization tint,
  optional outline-mask transport, renderer-neutral animation metadata, and
  legacy alpha cutoff, Winit/WebGPU scene
  surface, DOM shell, and WASM exports.
- `crates/drl-mcp`: JSON-RPC/MCP server and fairness boundary.
- `crates/drl-app`: native headless demo and MCP stdio runner.
- `docs/DRL-Rust_Project_Roadmap.md`: canonical milestones and gates.
- `SPEC.md`: the one active implementation slice.
- `ARCHITECTURE.md`, `CHANGELOG.md`, `docs/adr/`, `docs/legacy-behavior/`, and
  `docs/reference-captures/`, `docs/harness/`: verified structure, history,
  decisions, evidence, and agent workflow.

## Legacy assets and licensing

The imported graphics under `assets/legacy/drl/graphics/` come from the pinned
legacy Git revision recorded in `MANIFEST.txt` and `SHA256SUMS`, with the
upstream CC BY-SA 4.0 license and attribution. The repository's MIT license
does not relicense them. Legacy code is GPL; audio, music, and fonts are not
bundled until their separate redistribution rights are recorded. See
`docs/legacy-behavior/asset-provenance.md` and
`docs/reference-captures/manifest.md`.

## Contributing

Read `AGENTS.md`, `CONTRIBUTING.md`, the active `SPEC.md`, and the roadmap
before changing a milestone. Preserve deterministic headless behavior and run
`sh scripts/check-repository.sh`. Browser changes also need WASM/build
evidence, browser metadata, and an explicit statement of any unavailable
WebGPU/audio/reference-capture checks.
