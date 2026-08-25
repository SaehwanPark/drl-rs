# Vertical Subtle Knife Encounter — Verification

## Focused checks

- `cargo test -p drl-core --test scenarios subtle_knife_vertical_scenario_preserves_visibility_and_replay --quiet` — PASS (1/1).
- `cargo test -p drl-web subtle_knife_browser_boundary_matches_direct_core_presentation --quiet` — PASS (1/1; command replay events/state also compared).
- `cargo test -p drl-core --test scenarios --quiet` — PASS (6/6).
- `cargo test -p drl-web --quiet` — PASS (29/29).

## Repository checks

- `cargo test --workspace --quiet` — PASS (all workspace suites).
- `sh scripts/check-repository.sh` — PASS (repository, content, MCP, and
  compile/test contracts).
- `sh scripts/check-version.sh` — PASS (`0.2.144`).
- `git diff --check` — PASS.

## Boundary status

The Rust scenario/replay/core/browser-boundary evidence is verified. Controlled
legacy runtime, browser capture, audio, WebGPU, audiovisual, armor/resistance,
and broad monster/AI parity are `NOT_RUN`, not inferred passes.
