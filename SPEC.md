# Specification

## Document Contract

The [project roadmap](docs/DRL-Rust_Project_Roadmap.md) is the canonical plan
for milestone scope, order, status, and exit criteria. This file expands only
the active implementation slice into observable outcomes and verification. It
does not replace or duplicate the full roadmap.

## Past

- The repository, Rust 2024 binary scaffold, license, proposal, roadmap, and
  local legacy-asset research location were established before this
  specification workflow was adopted.
- The Milestone 0 documentation and harness foundation established durable
  agent guidance, team contracts, check scripts, and repository workflow.
- Milestone 0 multi-crate Cargo workspace and initial crates boundary scaffolding
  were established and validated with architectural boundary tests.
- Milestone 1 established the deterministic headless simulation kernel in `drl-core`
  and shared protocol contracts in `drl-protocol`, including 2D grid maps, seedable RNG,
  movement validation, and replay determinism.
- Milestone 2 established the action economy, energy scheduling, actor combat stats,
  pure combat calculations, melee/ranged resolution, and deterministic replay.

## Present

### Milestone 4: Field of View (FOV), Fog-of-War Map Memory, and Line-of-Fire Targeting

Status: Active

This slice implements DRL's field of view (FOV) calculation, line-of-sight (LOS) raycasting,
fog-of-war map exploration and memory, player observation filtering (preventing information
leaks for hidden entities), and line-of-fire validation for ranged attacks.

Observable outcomes:

- `drl-protocol` defines `CommandError::LineOfSightBlocked(Position)` when a ranged attack
  or targeting action cannot trace an unblocked line to the target cell;
- `drl-protocol` extends `TileView` with an `is_visible` flag distinguishing cells
  currently in the player's active field of view from cells remembered in fog of war;
- `drl-core` implements an isolated, pure `fov` module providing:
  - Bresenham-based discrete line-of-sight ray tracing (`has_line_of_sight`);
  - field of view calculation (`compute_fov`) for a configurable vision radius;
  - proper occlusion handling (opaque walls and closed doors block sight, while
    transparent floors, open doors, and stairs transmit sight);
  - perimeter illumination so walls facing the player are visible;
- `drl-core` implements fog-of-war map exploration tracking in `World`:
  - previously visited/seen tiles are remembered as explored;
  - unexplored tiles remain completely hidden from `PlayerObservation`;
- `drl-core` ensures `PlayerObservation` strictly filters entities:
  - `visible_actors` contains ONLY living actors that currently reside within the
    player's active field of view;
  - monsters behind walls or in unexplored/fog-of-war areas are never leaked to the player;
- `drl-core` enforces line-of-fire validation on `Command::AttackRanged`:
  - attacks targeting entities through walls or opaque obstacles are rejected with
    `CommandError::LineOfSightBlocked`;
- `drl-app` demonstrates FOV visibility in the headless scenario, reporting visible
  tile counts and confirming that hidden enemies become visible only upon entering LOS;
- `sh scripts/check-repository.sh` runs formatting, clippy, harness, and all unit/integration
  tests across the workspace without warnings.

Verification:

- `sh scripts/check-repository.sh` succeeds locally;
- `cargo test --locked --workspace` passes all unit, integration, boundary, combat,
  visibility, scheduling, and replay determinism tests;
- integration tests in `crates/drl-core/tests/visibility.rs` verify shadowcasting/LOS,
  actor filtering in observations, fog-of-war persistence, and line-of-fire blocking;
- `cargo run` executes the headless demo demonstrating FOV raycasting and line-of-fire checks.

Out of scope:

- inventory management, equipment slots, and item pickups;
- procedural level generation algorithms;
- live Lua scripting integration;
- MCP transport servers;
- presentation/GUI rendering and audio.

## Future

Proceed with inventory, equipment, weapons, and level flow in Milestone 4.
