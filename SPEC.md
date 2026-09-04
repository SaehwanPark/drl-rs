# Specification

Last reviewed: 2026-09-04
Current project version: `0.2.343`
Audited starting checkpoint: `main` at `5f747a0` (Linux and Fedora CI
coverage reconciliation)
Delivery checkpoint: **merged** in PR #460 as `ee38357`

The [Roadmap](docs/DRL-RS_Project_Roadmap.md) owns milestone scope, ordering,
and progress. [`docs/steering/current-priorities.md`](docs/steering/current-priorities.md)
constrains slice selection while its stop gates remain open. This file expands
**exactly one active implementation slice**. Delivered history belongs in the
roadmap, changelog, evidence notes, and Git rather than accumulating here.

## 1. Status vocabulary

- `[x]` — **Delivered and verified**: supported by checked implementation and
evidence.
- `[ ]` — **Open**: required by the active slice and not yet delivered.
- `NOT_RUN` — **Environment unavailable**: prerequisites were unavailable; no
pass or failure is inferred.
- `INCONCLUSIVE` — **Evidence unresolved**: available evidence cannot support
the claim.

## 2. Active implementation slice: native frontend boundary

Slice status: **delivered** on `main` merge `ee38357` (PR #460), based on
`main` at `5f747a0` (`0.2.342`); implementation candidate revision was `058011a`
(`0.2.343`).

### 2.1 Objective

Define and prove the first thin native frontend boundary described by
[`docs/steering/audit-2026-09-02.md`](docs/steering/audit-2026-09-02.md) §§4,
6–8: a native `drl-desktop` shell must submit existing semantic commands,
consume fair observations and `PresentationStep` values, render the shared
`RenderScene` geometry plan through `wgpu`, and keep window metrics outside
simulation. The slice is an architectural proof and not a desktop release.

### 2.2 Scope and ownership

- Add `crates/drl-desktop` as a workspace crate with a small library and
  executable. `drl-desktop` owns native `winit` window/event-loop and `wgpu`
  surface/device/pipeline lifecycle only.
- Add neutral `drl-render::SceneQuad`, `scene_quad_plan`, and
  `target_quad_plan` projections for the existing geometry fallback and target
  overlay. `drl-web` consumes those renderer-owned projections instead of
  retaining a second browser-only geometry policy.
- Add a native `DesktopSession` adapter around `drl-core::Scenario` and
  `Game::step`, plus a pure physical-key-to-`Command` mapping. It exposes only
  fair observations, `RenderScene`, and `PresentationStep` to the shell.
- Keep `drl-core` deterministic and platform-independent; keep `drl-protocol`
  semantic; keep `drl-render` free of GPU/window dependencies.
- Transition the code version exactly once from `0.2.342` to `0.2.343`.

### 2.3 Observable acceptance criteria

- [x] `drl-desktop` is a workspace member and `cargo check --locked
  -p drl-desktop` builds the native `winit`/`wgpu` boundary on the supported
  host; the same crate is included in the Fedora development-host check.
- [x] `DesktopSession` instantiates a caller-supplied `Scenario`, submits
  semantic commands, and produces `PlayerObservation`, `RenderScene`, and
  `PresentationStep` values without importing `drl-web` or exposing hidden
  `World` state to the window shell. Focused session tests and the redacted
  `Debug` contract cover this boundary.
- [x] Successful desktop commands use the same event-to-effect construction as
  the browser boundary; rejected commands retain the session's authoritative
  game state and record only a presentation-facing error. Focused desktop
  session tests pass.
- [x] `scene_quad_plan` and `target_quad_plan` preserve the existing geometry
  fallback draw order, colors, visibility projection, centered integer
  `PixelViewport`, and inset policy; browser fallback/target geometry and
  native geometry consume renderer-owned plans. Focused render tests pass.
- [x] Native resize uses the physical `winit::dpi::PhysicalSize` supplied by
  the window/surface abstraction, clamps zero dimensions safely, and does not
  multiply an already-physical size by a compositor scale factor. Viewport and
  renderer tests cover zero/letterbox inputs; fractional-scale behavior is
  handled at the window boundary and interactive compositor proof remains
  `NOT_RUN`.
- [x] The native renderer creates a `wgpu` surface and pipeline using Vulkan or
  Metal backends, renders the shared scene plan, and treats presentation failure
  as a shell error rather than a gameplay error. Native compilation and the
  display-independent `cargo run -p drl-desktop -- --validate` proof pass;
  asset upload, texture sampling, nearest-neighbor compositing, and interactive
  surface acceptance remain a later/native acceptance slice.
- [x] Existing browser, replay, RNG, MCP, and core contracts remain unchanged;
  no native gameplay branch, persistence, audio backend, launcher, menu,
  packaging, gamepad, accessibility, X11, or generic-Linux support is added.
- [x] Repository, web, version, Fedora host, and focused desktop/render checks
  pass locally. Hosted CI and interactive Fedora Wayland/Vulkan or macOS Metal
  window acceptance are recorded separately; unavailable capture surfaces
  remain `NOT_RUN`.
- [x] An attributable independent review of the final correction range
  `20a6bb6..058011a` returns `PASS`; the final review is merge-gating evidence.

### 2.4 Semantic and boundary impact

- **Command atomicity:** `DesktopSession` delegates acceptance/rejection to
  `Game::step`; no outer simulation clone or alternate transaction policy is
  introduced. Unit tests compare the private session game before and after a
  rejected command.
- **RNG/replay:** no command, replay wire, RNG sampling, generator, or gameplay
  semantics identity changes. The native adapter does not save or import replay
  files.
- **Content/catalog:** no content definitions or registration paths change;
  the demo is a `Scenario` fixture and the render plan uses existing stable
  scene descriptors.
- **Presentation:** `drl-render` owns pure scene geometry and `drl-desktop`
  owns physical surface metrics. Presentation timing and resize events cannot
  advance the simulation.
- **Rights/evidence:** no asset files are added and no legacy runtime,
  audiovisual parity, or GPU acceptance claim is inferred from compilation.

### 2.5 Non-goals

- No native persistence, audio, menus, launcher, configuration UI, installer,
  RPM/Flatpak, gamepad, desktop accessibility, generic Linux, X11-specific
  acceptance, or browser/Fedora compatibility claim.
- No `drl-gpu` extraction, no copy of `drl-web`, no migration of gameplay or
  presentation policy into a platform shell, and no change to public WASM
  exports or browser storage/replay semantics.
- No claim of controlled legacy runtime, audiovisual parity, or human gameplay
  acceptance unless the required environment and evidence are recorded.

### 2.6 Delivery evidence

Evidence is bound to final candidate revision `058011a` rather than inferred
from intent:

- `cargo test --locked -p drl-render -p drl-desktop -p drl-web`, workspace
  Clippy with `-D warnings`, and `cargo run --locked -p drl-desktop -- --validate`
  pass; the focused suites report 82 `drl-render`, 9 `drl-desktop`, and 100
  native `drl-web` tests passing.
- `sh scripts/check-repository.sh` passes, including the SPEC structural,
  version (`0.2.343`), harness, rights, replay/MCP, full workspace, and
  desktop tests. `sh scripts/check-web.sh` passes with 2 WASM browser tests;
  the only output warning is the available newer `wasm-pack` release.
- `DRL_VERSION_BASE=5f747a0 sh scripts/check-version.sh` passes. The local
  `sh scripts/check-fedora-dev.sh` passes after native `drl-desktop` compilation
  and deterministic core/protocol tests; the host capability probe reports
  `dri=device-present`, `vulkan=library-present`, and `WAYLAND_DISPLAY=wayland-0`
  but correctly records GPU/Wayland acceptance as `NOT_RUN`.
- A bounded native launch probe did not emit an immediate startup diagnostic
  before its timeout (`INCONCLUSIVE`); no interactive Fedora 43 GNOME/Mutter
  Wayland/Mesa/RADV Vulkan or macOS Metal acceptance is claimed, and those
  surfaces remain `NOT_RUN`.
- Final implementation diff `5f747a0..058011a` and independent review of
  correction range `20a6bb6..058011a` are recorded as `PASS`. Hosted CI run
  `33892975163` passed `Repository checks`, `Repository checks (Linux)`,
  `Fedora 43 development host`, and `WASM browser checks`. The hosted Review
  policy run `33892975327` failed closed because the sole-maintainer repository
  has no independent GitHub approver; PR #460 was merged with the documented
  administrator exception after the independent local review PASS.

## 3. Enduring invariants

The active slice must preserve:

1. no ambient state, platform APIs, filesystem, browser, or presentation policy
   in `drl-core`;
2. identical declared seed, commands, and semantics produce identical current
   simulation results;
3. incompatible histories fail explicitly before simulation;
4. rejected commands and rejected restores do not partially mutate authoritative
   simulation state;
5. renderers, browser code, MCP, and bots consume fair observations/events and
   do not inspect hidden core state;
6. presentation timing, resize, scale factor, surface loss, and storage side
   effects do not advance gameplay;
7. no runtime Lua or generic callback recreation;
8. current-Rust, cross-version, legacy, browser, audiovisual, and performance
   evidence remain separately labeled.
