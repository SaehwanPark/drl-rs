# Architecture

Last Reviewed: 2026-08-19

Status: Verified

## Overview

DRL-Rust is organized as a multi-crate Cargo workspace. It provides the initial
modular crate boundaries for a headless simulation core, shared semantic
protocols, an executable application runner, and placeholder subsystems for
rendering, audio, scripting, and MCP. The gameplay simulation and presentation
implementations remain planned.

## Current Components

- Root `Cargo.toml` defines the Cargo workspace managing all crates under
  `crates/`.
- `crates/drl-core` is the deterministic headless simulation core library with
  no dependencies on rendering, audio, MCP, or operating-system APIs.
- `crates/drl-protocol` is the shared contract library for semantic commands,
  observations, and events.
- `crates/drl-app` is the executable runner (`drl-rust`) that coordinates
  crates and provides the default workspace entry point.
- `crates/drl-script`, `crates/drl-mcp`, `crates/drl-render`, and
  `crates/drl-audio` are placeholder workspace crates with bounded dependency
  declarations.
- `crates/drl-core/tests/boundaries.rs` enforces architectural dependency
  direction via automated tests.
- `docs/DRL-Rust_Project_Roadmap.md` owns milestone planning and progress.
- `SPEC.md` expands the active roadmap slice.
- `AGENTS.md`, `docs/harness/drl-delivery/team-spec.md`, and repo-local skills
  define the development and test-play harness.
- `scripts/check-repository.sh` is the common local and CI verification entry
  point and includes formatting, clippy, test, and harness-structure validation.

There is no active gameplay simulation, Lua runtime, live MCP server, GPU
renderer, audio backend, or persistence layer yet.

## Current Flow

```text
Roadmap milestone
  -> active SPEC slice
  -> optional evidence specialists
  -> implementation and focused tests
  -> capability-gated test play and determinism review
  -> local and CI verification
  -> architecture, changelog, and roadmap reconciliation
```

The current executable flow is:

```text
cargo run -> crates/drl-app/src/main.rs -> drl-core & drl-protocol -> scaffold status
```

## Consequential Invariants

- The roadmap remains canonical for long-term scope and milestone status.
- Planned architecture must not be described as implemented.
- `drl-core` and `drl-protocol` must remain independent of graphics, audio,
  operating-system, filesystem, and MCP concerns; automated tests enforce this.
- Gameplay randomness must become explicit and reproducible.
- Human UI, bots, replay tools, and MCP should eventually use the same semantic
  command boundary (`drl-protocol`).
- Legacy Pascal and Lua sources inform behavior, not Rust module structure or
  execution order.
- One milestone owner reconciles canonical documents; delegated workers are
  read-only by default and cannot convert exploratory findings directly into
  completion claims.
- Unsupported test-play capabilities are reported as `NOT_RUN`; missing or
  contradictory evidence remains `INCONCLUSIVE`.
- Repository-controlled text uses spaces with indentation and tab width 2.

## Planned Direction

The proposal describes a headless deterministic simulation core, shared
commands, observations and events, Lua-backed content, replay and test-agent
support, an MCP interface, and a native macOS presentation layer. Those
components are targets, not current dependencies or compatibility guarantees.
