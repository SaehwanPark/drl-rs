# Specification

Last reviewed: 2026-09-02
Current project version: `0.2.340`
Audited starting checkpoint: `main` at `2a089c6` (Laser Rifle direct-Plasma
delivery and canonical documentation reconciliation)
Delivery checkpoint: `main` merge commit `d855725` (PR #456, merged)

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

## 2. Active implementation slice: M8 — Modular browser shell

Slice status: **open** — the implementation and the local verification in §2.8
are delivered on branch `codex/drl-web-frontend-modularization` (PR #457, merge
pending); the hosted checks and the independent re-review in §2.8 are not closed.

### 2.1 Objective

Split the monolithic `crates/drl-web/src/lib.rs` browser module into focused
modules so each browser responsibility (assets, DOM markup, GPU contracts,
keyboard/input, session commands, persistence storage, renderer, scene
projection, winit input app, animation loop, and WASM exports) is owned by one
module, per audit 2026-09-02 §13 item 1.

This is a behavior-preserving modularization slice. It moves existing items,
keeps the WGSL shader sources byte-identical, and keeps every browser contract
string in the crate that now owns the behavior. It adds no native shell, no
`drl-desktop` crate, no Linux CI change, and no new public API.

### 2.2 Audited starting point

At audited starting revision `30ec8c6` (version `0.2.340`):

- `crates/drl-web/src/lib.rs` held 14,764 lines: the crate-root re-export
  surface, production session/DOM/GPU/asset helpers, the whole `wasm` shell
  (storage, textures, renderer, scene, winit app, DOM shell, animation loop,
  and `#[wasm_bindgen]` exports), 97 native boundary tests, and 2 WASM tests.
- `scripts/check-browser-diagnostics.sh` and `scripts/test-browser-controls.mjs`
  asserted browser contracts by grepping that single file, so any contract move
  silently weakened the boundary checks.
- Only `persistence.rs` and `texture.rs` were separate modules, and they reached
  shared helpers through crate-root re-exports.
- `cargo clippy -p drl-web --target wasm32-unknown-unknown --all-targets`
  emitted 7 warnings (too-many-arguments, collapsible `if`, type complexity).
  Re-running it at `30ec8c6` reproduces the same 7 warnings, so they pre-date
  this slice and the web gate does not treat them as errors.

### 2.3 Scope and ownership

- **Roadmap:** M8/M13 browser-shell structure; the first step of the audit's
  native-portability order, which requires refactoring `drl-web` rather than
  copying it.
- **Crate root:** `lib.rs` is now a 69-line module map plus the `pub(crate)`
  surface that the shell and boundary tests resolve by crate-root name.
- **Platform-independent browser helpers:** `animation`, `assets`, `dom`, `gpu`,
  `input`, `session` plus the existing `persistence` and `texture` modules.
- **Browser shell:** `wasm/mod.rs` keeps the module map, shared thread-local
  shell state, and re-exports; `wasm/storage`, `textures`, `renderer`, `scene`,
  `app`, `shell_dom`, `animation_loop`, and `exports` own one responsibility
  each.
- **Boundary tests:** `tests/mod.rs` holds shared helpers and imports; 11
  focused test modules and `wasm_tests.rs` own the cases.
- **Contract scripts:** both boundary scripts now grep the shell module set
  instead of one file.
- **Project version:** implementation advances `VERSION` from `0.2.340` to
  `0.2.341`.

### 2.4 Review and branch contract

- Every moved item keeps its original body; the only intended text changes are
  `pub(crate)` visibility, `use`/`mod` declarations, and `use super::*;` in
  submodules.
- The two WGSL shader constants are byte-identical to the pre-split strings
  (SHA-256 match over the 1,704- and 480-character shader texts), so pipeline
  behavior and the shader-retention contract tests are unchanged.
- All 100 native `drl-web` tests remain (97 relocated by name plus the 3
  existing `persistence` tests) and both `#[wasm_bindgen_test]` cases remain.
- No gameplay semantics change: gameplay semantics stay `142`, and the replay
  wire, RNG sampling, generator semantics, and ruleset identity are untouched.
- `crates/drl-web/**` stays a protected review path; this slice relies on the
  documented solo-maintainer Review-policy exception.

### 2.5 Acceptance criteria

- [x] `cargo check -p drl-web --all-targets` and
  `cargo clippy -p drl-web --all-targets -- -D warnings` pass with zero warnings.
- [x] `cargo check -p drl-web --target wasm32-unknown-unknown --all-targets`
  passes with zero warnings, and WASM-target clippy reports only the 7
  pre-existing warnings reproduced at `30ec8c6`.
- [x] The relocated boundary test set passes unchanged (100 native tests).
- [x] `scripts/check-browser-diagnostics.sh`, `scripts/test-browser-controls.sh`,
  `scripts/check-browser-accessibility.sh`, `scripts/check-service-worker.sh`,
  and `scripts/test-offline-cache.sh` pass against the new module layout.
- [x] `sh scripts/check-repository.sh` and `sh scripts/check-web.sh` pass on
  Fedora 43 x86-64.
- [x] `cargo fmt --all -- --check` and `sh scripts/check-version.sh` pass on the
  final commit.
- [ ] Hosted Repository and WASM checks pass on the final implementation commit;
  a green run on an earlier commit is not accepted (status recorded in §2.8).
- [ ] An attributable independent review returns `pass` on the final commit. The
  first read-only pass returned `fix` against `12d12a1`; the corrections are
  recorded in §2.8 and a focused re-review is required.

### 2.6 Non-goals

- No Linux CI or Fedora job change, no `drl-desktop` crate, no native desktop
  window, and no Fedora/Wayland/Vulkan acceptance claim.
- No change to rendering equations, persistence codec, save/quarantine policy,
  DOM markup, input mapping, animation cadence, or any `#[wasm_bindgen]` export
  signature.
- No cleanup of the 7 pre-existing WASM-target clippy warnings and no split of
  the large content-parity test modules beyond the current per-suite grouping.

### 2.7 Evidence boundary

This slice proves module-boundary equivalence for the browser shell on the
Fedora 43 x86-64 host: both target builds, the full native test set, the
browser contract scripts, and byte-identical shader sources. It does not prove
new browser runtime behavior beyond the existing headless Chrome WASM tests, no
new interactive Chromium acceptance record is claimed, controlled legacy
captures remain `NOT_RUN`, and nothing in this slice demonstrates native desktop
or Linux CI coverage.

### 2.8 Delivery evidence

Evidence is bound to a named revision; a later commit does not inherit an
earlier commit's checks.

- **Implementation:** `ccdee78` (modularization) then document and script
  corrections, on branch `codex/drl-web-frontend-modularization` against baseline
  `30ec8c6` (`0.2.340`). Merge into `main` is **pending** in PR #457.
- **Local verification, Fedora 43 x86-64 (GNOME/Wayland host):**
  `sh scripts/check-repository.sh` exits `0`; `sh scripts/check-web.sh` exits
  `0`, covering the service-worker, offline-cache, browser-control,
  support-classifier, diagnostics, and accessibility contracts, the 100-test
  native `drl-web` set, and the 2 WASM persistence tests in headless Chrome
  152.0.7977.75 with ChromeDriver 152.0.7977.75 (local wasm-pack 0.13.1; hosted
  CI pins 0.15.0). `cargo clippy -p drl-web --all-targets` is warning-free;
  `cargo clippy -p drl-web --target wasm32-unknown-unknown --all-targets` reports
  7 warnings, and the same command re-run at `30ec8c6` reproduces the identical 7,
  so none is introduced here. Mechanical fidelity (shader SHA-256 and lengths,
  `#[wasm_bindgen]` export signatures, item-name census, test-name diff,
  platform-import census) is in `/tmp/fidelity.md`.
- **Independent review, first pass:** read-only review of `30ec8c6..12d12a1` per
  `.agents/skills/drl-determinism-review/SKILL.md`, disposition **`fix`**. It
  confirmed behavior preservation at the inspected boundaries
  (`BrowserSession::submit` rollback contract, quarantine-before-remove storage
  order, export side-effect ordering, animation clock and `visibilitychange`
  rebasing, `escape_html` and markup helpers, module and `cfg` reachability,
  encapsulation census, gameplay semantics still `142`) and raised three
  findings: (1) this slice pre-claimed its own review verdict and hosted-check
  success against a stale implementation head and described PR #457 as shipped;
  (2) the aggregate diagnostics grep let the incompatible-save contract survive
  losing one intended owner; (3) `ARCHITECTURE.md` mis-described the remaining
  `texture.rs` platform binding as an animation callback. All three are
  corrected: this evidence ledger replaces the pre-authored verdict sentence, the
  boundary scripts now assert each contract string on its owning module (with the
  consumer asserted as the `== Some("Saved session incompatible")` comparison), and the architecture
  text names the texture-cache error type. Negative control for finding 2: with
  that literal removed from `wasm/shell_dom.rs`, both
  `scripts/check-browser-diagnostics.sh` and `scripts/test-browser-controls.mjs`
  exit non-zero; on the clean tree both pass.
- **Independent re-review:** pending on the final commit; this slice does not
  claim that verdict until it lands.
- **Hosted checks:** pending on the final commit. An earlier revision of this
  branch passed hosted Repository checks and WASM browser checks and failed the
  Review-policy check closed as the documented solo-maintainer
  `enforce_admins=false` exception; that run is not counted for acceptance here.
- **`NOT_RUN`:** controlled legacy reference captures and interactive
  Chromium/Wayland acceptance (no browser acceptance record is claimed beyond the
  headless WASM suite above).

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
6. presentation timing and storage side effects do not advance gameplay;
7. no runtime Lua or generic callback recreation;
8. current-Rust, cross-version, legacy, browser, audiovisual, and performance
   evidence remain separately labeled.
