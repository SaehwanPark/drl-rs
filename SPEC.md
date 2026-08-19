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
- Milestone 4 established Field of View (FOV) calculation, line-of-sight raycasting,
  fog-of-war exploration memory, entity observation filtering, and line-of-fire obstacle blocking.
- Milestone 4 established item domain models, player inventory capacity, equipment slots (weapon
  and armor), ground item pickup/drop, weapon reload mechanics, ammo tracking, and consumable medpacks.
- Milestone 4 established procedural dungeon level generation (`generator`), non-overlapping
  rooms, BFS reachability validation, exit stairs (`Tile::StairsDown`), level transitions
  (`Command::Descend`), player state persistence across level boundaries, and multi-level replay determinism.
- Milestone 4 established weapon kinetic knockback mechanics, bounds collision safety,
  stochastic combat statistical validation suites, and completed Milestone 4 exit criteria.

## Present

### Milestone 5: Versioned Replays, Scenario Fixture Framework, Scripted Agent Policies, and Episode Metrics

Status: Active

This slice implements the automated testing, scenario fixture, replay diagnostics, scripted bot policy,
and batch simulation infrastructure for Milestone 5.

Observable outcomes:

- `drl-protocol` defines versioned replay schema, scenario fixtures, and simulation metrics:
  - `ReplayVersion` (`V1`) enum and `ReplayMetadata` with engine versioning;
  - `PlayerSpawnConfig` recording custom starting HP, speed, inventory, and equipment;
  - `ReplayLog` updated with versioning, metadata, and optional `PlayerSpawnConfig`;
  - `ReplayExecutionError` capturing failed turn number, command index, offending command, and `CommandError`;
  - `ScenarioMap` and `ScenarioFixture` representing explicit scenario layouts, spawns, and configurations;
  - `RunOutcome`, `EpisodeMetrics`, and `BatchSummary` capturing runtime telemetry (damage dealt/taken, kills, turns survived, win rate);
- `drl-core` implements scenario parsing, execution, replay validation, and automated bot agents:
  - `scenario` module providing ASCII grid parsing (`Scenario::from_ascii`), scenario instantiation, execution, and fluent assertion helpers (`ScenarioRunner`);
  - `replay` module with `ReplayEngine::validate` checking schema consistency and `run_with_diagnostics` returning `EpisodeMetrics` and `ReplayExecutionError`;
  - `agent` module defining the `AgentPolicy` trait operating strictly on `PlayerObservation` and emitting `Command`s;
  - Built-in agent policies: `RandomBot`, `GreedyCombatBot` (engaging enemies, reloading, healing, looting), and `ExplorerBot` (dungeon exploration and stairs descent);
  - `batch` module (`BatchRunner`) executing large automated episode runs across arbitrary seeds and aggregating statistical `BatchSummary`;
- `drl-core` provides comprehensive test suites in `crates/drl-core/tests/`:
  - `tests/scenarios.rs`: scenario fixture parsing, custom monster/item setups, and fluent scenario assertions;
  - `tests/agents.rs`: automated bot policies operating headlessly through observations without state leakage;
  - `tests/batch_simulation.rs`: multi-seed batch runs, metrics collection, and failure artifact reproducibility;
  - `tests/replay_versioning.rs`: versioned replay validation and error context reporting;
- `drl-app` updates its CLI demonstration to include scenario execution, automated agent play, and batch metrics summary;
- `sh scripts/check-repository.sh` runs all checks, formatting, clippy, and tests cleanly.

Verification:

- `sh scripts/check-repository.sh` succeeds locally;
- `cargo test --locked --workspace` passes all unit, integration, scenario, agent, batch, and replay tests;
- automated agent policies successfully complete deterministic scenario fixtures and multi-level procedural runs;
- `cargo run` executes scenario simulations and batch summaries headlessly.

Out of scope:

- live Lua scripting integration (Milestone 3);
- MCP JSON-RPC wire servers (Milestone 6);
- presentation/GUI rendering (Milestone 7) and audio (Milestone 8).

## Future

Proceed with Milestone 6 MCP game interface and Milestone 7 native macOS rendering.
