# DRL-Rust

DRL-Rust is a ground-up Rust reimplementation of Doom the Roguelike. The
product direction is browser-first: a deterministic Rust/WASM game rendered
with WebGPU in desktop Chrome/Edge, with an accessible HTML shell and
gesture-unlocked Web Audio. Headless Rust and MCP remain supported for agents,
replays, and regression testing.

## Current capabilities

- Deterministic simulation:
  - Complete M4 headless game loop with combat, FOV/fog, AI, levels, replay,
    scenarios, bots, batches, inventory, and MCP tooling.
  - Stable tile, item, monster, and standard-level definitions, canonical item
    factories, and table-driven generated item/monster selection with preserved
    RNG boundaries.
  - Fixed-seed cohort reports preserve sample definitions, policy identity,
    aggregate metrics, and per-seed replay evidence for evaluation.
  - Cohort regression math applies explicit win-rate and average-turn
    tolerances without mutating simulation or claiming balance parity.
  - Cohort report validation rejects inconsistent sample/evidence metadata
    before a regression comparison is used.
  - Cohort outcome distributions preserve distinct terminal counts and
    sample-normalized rates without interpreting balance or significance.
  - Compatible cohort comparisons report absolute per-outcome rate deltas
    after integrity validation without adding tolerance or significance claims.
  - Outcome comparisons accept one finite, non-negative per-category rate
    tolerance and expose a deterministic pass/fail gate.
  - Cohort telemetry projections and compatible comparisons expose validated
    shot accuracy, damage, kill, pickup, and item-use totals/rates without
    inferring balance conclusions.
- Versioned delivery:
  - `VERSION` is the canonical `x.y.z` project value (currently `0.2.4`),
    projected into Cargo, MCP, and release manifests; the agent harness rejects
    invalid code-change transitions and ignores document/setting-only diffs.
- Browser and presentation slice:
  - M7 functional checks pass locally and in remote web CI.
  - M8 provides square pixel-grid letterboxing, shared lighting bands, measured
    atlas slots, normalized UVs, renderer-neutral layer/composite plans, fair
    observation-derived presentation, and validated texture-source loading.
  - Native-tested base/mask/emissive rendering includes evidence-backed Green
    Armor, Phase Device, and StairsDown tint boundaries, an emissive lighting
    floor, optional outline-mask compositing, and animation frame
    metadata/selection.
  - Pure contracts cover effect timing, low-health tone/pulse, explosion marks,
    movement and missile progress, screen-shake fade, particle origins and
    burst directions, and post-process glow/LUT math without claiming full
    backend fidelity.
- Staged work:
  - Full audiovisual equivalence, broader content migration, PWA persistence,
    and support for other browsers remain roadmap work.
  - Release builds emit a hashed static-bundle manifest with graphics rights
    metadata; signing and offline/cross-browser acceptance remain open.
  - Placeholder M7 atlas rectangles are not a fidelity claim.

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
scripts/check-version.sh
scripts/check-web.sh
scripts/check-browser-diagnostics.sh # also run by check-web.sh
scripts/check-browser-accessibility.sh # also run by check-web.sh
scripts/check-reference-capture.sh
scripts/check-release-manifest.sh  # after scripts/build-web.sh
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
- `crates/drl-assets`:
  - Atlas IDs/dimensions, measured rectangles, registered layers and shader
    roles, normalized UVs, texture-source bindings, and semantic asset mapping.
  - Pinned legacy revision identity and licensing metadata.
- `crates/drl-render`:
  - Pure scene construction, pixel viewport layout, layer/composite plans,
    lighting, animation selection, and observed tint mappings.
  - Source-derived contracts for health tone/pulse, effect and missile timing,
    screen shake, particle origins, and post-process glow/LUT math.
  - Renderer/backend and full audiovisual equivalence remain staged work.
- `crates/drl-audio`: semantic cues and WASM Web Audio mixer.
- `crates/drl-web`:
  - Browser session, fair observation boundary, validated texture loading,
    renderer-owned WebGPU uploads, and the partial textured pass.
  - Animation playback, bounded scheduling, fixed-session snapshots,
    best-effort localStorage, generated-bundle service-worker cache, and the
    project-version/source-revision cache policy and manifest digest sidecar
    recorded by release manifests, with a mocked service-worker lifecycle
    contract and source-identity audit.
  - Local accessible browser-support/startup diagnostics with recovery guidance;
    no telemetry or untested-browser support claim.
  - Static shell accessibility audit for names, labels, focus, and live regions;
    dynamic WCAG/screen-reader acceptance remains open.
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
`docs/reference-captures/manifest.md`, which records checkout dirty-state and
evidence classification, rights, and media hashes while keeping capture
promotion gated on a clean controlled checkout with directly observed evidence.

## Contributing

Read `AGENTS.md`, `CONTRIBUTING.md`, the active `SPEC.md`, and the roadmap
before changing a milestone. Preserve deterministic headless behavior and run
`sh scripts/check-repository.sh`. Browser changes also need WASM/build
evidence, browser metadata, and an explicit statement of any unavailable
WebGPU/audio/reference-capture checks.
