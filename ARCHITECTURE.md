# Architecture

Last Reviewed: 2026-08-19

Status: Verified

## Overview

DRL-Rust is organized as a multi-crate Cargo workspace. It provides
modular crate boundaries for a headless deterministic simulation core (`drl-core`),
shared semantic protocol contracts (`drl-protocol`), an executable application runner
(`drl-app`), and placeholder subsystems for rendering, audio, scripting, and MCP.

## Current Components

- Root `Cargo.toml` defines the Cargo workspace managing all crates under
  `crates/`.
- `crates/drl-protocol` is the shared contract library for semantic domain types
  (`Position`, `Direction`, `Turn`, `EntityId`, `ItemId`, `LevelId`), commands
  (`Command::Move`, `Command::Wait`), errors (`CommandError`), events (`GameEvent`),
  observations (`Observation`, `TileView`, `ActorView`), and replay specifications
  (`ReplayLog`).
- `crates/drl-core` is the deterministic headless simulation core library containing:
  - `GameRng`: deterministic seedable PRNG (SplitMix64 + Xoshiro256++) with no ambient
    or global state;
  - `Map` & `Tile`: 2D bounded grid representation with walkability and transparency;
  - `World`: physical level state, deterministic `BTreeMap` actor storage, and collision checks;
  - `Game`: turn progression kernel executing player commands, emitting ordered events,
    and managing game state;
  - `ReplayEngine`: deterministic replay execution and bit-exact state verification.
- `crates/drl-app` is the executable runner (`drl-rust`) that runs headless simulation
  demonstrations and verifies replay reproducibility.
- `crates/drl-script`, `crates/drl-mcp`, `crates/drl-render`, and
  `crates/drl-audio` are placeholder workspace crates with bounded dependency
  declarations.
- `crates/drl-core/tests/boundaries.rs` enforces architectural dependency
  direction via automated tests.
- `crates/drl-core/tests/simulation.rs` verifies end-to-end multi-step movement, collision,
  observation, and replay determinism.
- `docs/DRL-Rust_Project_Roadmap.md` owns milestone planning and progress.
- `SPEC.md` expands the active roadmap slice.
- `AGENTS.md`, `docs/harness/drl-delivery/team-spec.md`, and repo-local skills
  define the development and test-play harness.
- `scripts/check-repository.sh` is the common local and CI verification entry
  point and includes formatting, clippy, test, and harness-structure validation.

There is no live Lua runtime, live MCP server, GPU renderer, audio backend,
or persistence layer yet.

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
cargo run -> crates/drl-app/src/main.rs -> drl-core (Game::new, Game::step) & drl-protocol -> headless simulation & replay verification
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
