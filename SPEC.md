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

## Present

### Milestone 1: Headless Simulation Core — Domain Types, Grid Map, Deterministic RNG, and Movement Commands

Status: Active

This slice establishes the fundamental headless simulation kernel in `drl-core` and
shared semantic protocols in `drl-protocol`.

Observable outcomes:

- `drl-protocol` defines domain types (`Position`, `Direction`, `Turn`, `EntityId`,
  `ItemId`, `LevelId`), commands (`Command::Move`, `Command::Wait`), typed errors
  (`CommandError`), events (`GameEvent`), observations (`Observation`, `TileView`,
  `ActorView`), and replay specifications (`ReplayLog`);
- `drl-core` implements deterministic seedable random number generation (`GameRng`)
  with no ambient or global RNG state, passing bit-exact reproducibility tests;
- `drl-core` implements 2D tile grid maps (`Map`, `Tile`) with bounds checking,
  walkability, blocking flags, and factory constructors (e.g., arena map);
- `drl-core` implements actor state and a minimal deterministic `World` with
  deterministic `BTreeMap` entity collections, actor spawning, and occupancy checking;
- `drl-core` implements a turn execution step (`Game::step`, `Game::execute_player_command`)
  validating movement legality against terrain and entity collisions, emitting ordered
  game events, and updating game state deterministically;
- `drl-core` implements replay playback (`ReplayEngine`) ensuring identical seeds and
  command sequences produce bit-for-bit identical state and event logs;
- `drl-app` provides an executable headless demonstration executing a deterministic
  movement scenario and printing structured observations and events;
- `sh scripts/check-repository.sh` runs formatting, clippy, harness, and all unit/integration
  tests across the workspace without warnings.

Verification:

- `sh scripts/check-repository.sh` succeeds locally;
- `cargo test --locked --workspace` passes all unit, integration, boundary, determinism,
  and replay tests;
- `cargo run` executes the headless simulation scenario demonstrating valid movement,
  collision rejection, wait turns, and determinism verification;
- tests verify that independent simulations with the same seed and commands yield
  identical world states.

Out of scope:

- combat, inventory, items, and AI behaviors (subsequent Milestone 1 slices);
- level generation algorithms (Milestone 2);
- live Lua scripting integration (Milestone 3);
- MCP transport servers (Milestone 6);
- presentation/GUI rendering and audio (Milestone 7 & 8).

## Future

Proceed with combat mechanics, damage calculations, item models, and basic monster
AI turns in subsequent Milestone 1 slices.
