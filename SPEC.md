# Specification

Last reviewed: 2026-09-04
Current project version: `0.2.343`
Audited starting checkpoint: `main` at `5f747a0` (Linux and Fedora CI
coverage reconciliation)
Delivery checkpoint: **open** on branch `codex/native-frontend-boundary`

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

Slice status: **in progress** on branch `codex/native-frontend-boundary`,
based on `main` at `5f747a0` (`0.2.342`); candidate code version is `0.2.343`.

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
- Add a neutral `drl-render::SceneQuad`/`scene_quad_plan` projection for the
  existing geometry fallback. `drl-web` consumes that same projection instead
  of retaining a second browser-only geometry policy.
- Add a native `DesktopSession` adapter around `drl-core::Scenario` and
  `Game::step`, plus a pure physical-key-to-`Command` mapping. It exposes only
  fair observations, `RenderScene`, and `PresentationStep` to the shell.
- Keep `drl-core` deterministic and platform-independent; keep `drl-protocol`
  semantic; keep `drl-render` free of GPU/window dependencies.
- Transition the code version exactly once from `0.2.342` to `0.2.343`.

### 2.3 Observable acceptance criteria

- [ ] `drl-desktop` is a workspace member and `cargo check -p drl-desktop`
  builds the native `winit`/`wgpu` boundary on the supported host.
- [ ] `DesktopSession` instantiates a caller-supplied `Scenario`, submits
  semantic commands, and produces `PlayerObservation`, `RenderScene`, and
  `PresentationStep` values without importing `drl-web` or exposing hidden
  `World` state to the window shell.
- [ ] Successful desktop commands use the same event-to-effect construction as
  the browser boundary; rejected commands retain the session's authoritative
  game state and record only a presentation-facing error.
- [ ] `scene_quad_plan` preserves the existing geometry fallback draw order,
  colors, visibility projection, and centered integer `PixelViewport`; browser
  geometry and native geometry consume this one plan.
- [ ] Native resize uses the physical `winit::dpi::PhysicalSize` supplied by
  the window/surface abstraction, clamps zero dimensions safely, and does not
  multiply an already-physical size by a compositor scale factor. Viewport
  tests cover fractional-scale/letterbox inputs.
- [ ] The native renderer creates a `wgpu` surface and pipeline using Vulkan or
  Metal backends, renders the shared scene plan, and treats presentation failure
  as a shell error rather than a gameplay error. Asset upload, texture sampling,
  and the nearest-neighbor sprite compositor remain a later native slice.
- [ ] Existing browser, replay, RNG, MCP, and core contracts remain unchanged;
  no native gameplay branch, persistence, audio backend, launcher, menu,
  packaging, gamepad, accessibility, X11, or generic-Linux support is added.
- [ ] Repository, web, version, and focused desktop/render checks pass. Hosted
  CI and interactive Fedora Wayland/Vulkan or macOS Metal window acceptance are
  recorded separately; unavailable capture surfaces remain `NOT_RUN`.
- [ ] An attributable independent review of the final correction range returns
  `pass` before the slice is accepted.

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

Evidence will be bound to the final named revision rather than inferred from
intent. The final record must include:

- implementation and focused test commands with outputs;
- `sh scripts/check-repository.sh`, `scripts/check-web.sh`, and
  `scripts/check-version.sh` results where the host can run them;
- the native launch/capability result, with Fedora 43 GNOME/Mutter Wayland
  Mesa/RADV Vulkan and macOS Metal separately labeled `PASS`, `FAIL`,
  `INCONCLUSIVE`, or `NOT_RUN`;
- final diff and independent-review disposition;
- any hosted workflow result, without treating hosted checks as a substitute
  for the independent review.

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
