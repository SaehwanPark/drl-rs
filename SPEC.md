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
- Milestone 5 established versioned replays (`ReplayVersion::V1`), rich turn/command diagnostics
  (`ReplayExecutionError`), declarative scenario fixtures (`Scenario`, `ScenarioRunner`), automated bot
  policies (`AgentPolicy`, `RandomBot`, `GreedyCombatBot`, `ExplorerBot`), and headless batch simulation (`BatchRunner`).

## Present

### Milestone 6: Model Context Protocol (MCP) Game Interface, Semantic Tools, and Integration Suite

Status: Complete / Active Slice

This slice implements the complete Model Context Protocol (MCP) server, JSON-RPC 2.0 dispatching engine,
semantic session management, legal action synthesis, and integration test suites for Milestone 6.

Observable outcomes:

- `drl-mcp` implements a zero-external-dependency JSON and MCP JSON-RPC 2.0 communication engine:
  - `json` module providing pure-Rust recursive-descent parser and serializer (`JsonValue`, `JsonObject`, `JsonArray`);
  - `protocol` module defining JSON-RPC 2.0 envelopes (`JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError`),
    error codes (`PARSE_ERROR`, `INVALID_REQUEST`, `METHOD_NOT_FOUND`, `INVALID_PARAMS`, `SESSION_NOT_ACTIVE`, `PERMISSION_DENIED`),
    MCP version constants (`MCP_PROTOCOL_VERSION`, `DRL_MCP_VERSION`), and `ToolDefinition`/`ResourceDefinition` models;
  - `session` module (`McpSession`) managing procedural and scenario game simulation lifecycle, turn limits,
    cumulative `EpisodeMetrics`, `ReplayLog` recording, and fair `PlayerObservation` perception filtering;
  - Dynamic legal action synthesis (`compute_legal_actions`) generating available actions (`Move`, `AttackRanged`, `Reload`,
    `Pickup`, `Use`, `Equip`, `Unequip`, `Drop`, `Wait`, `Descend`) with structured tool parameter payloads;
  - `tools` module exposing complete game tools: `game_start`, `game_load_scenario`, `game_get_observation`,
    `game_list_actions`, `game_step_action`, `game_reset`, `game_get_metrics`, `game_save_replay`, and `game_get_dev_state`;
  - `resources` module exposing static and live resources: `drl://rules/game`, `drl://rules/actions`,
    `drl://session/metrics`, and `drl://session/events`;
  - `server` module (`McpServer`) implementing standard JSON-RPC dispatching and stdio transport loop (`run_stdio`);
- Security and fairness boundaries strictly enforced:
  - Default observation masks unseen entities and unrevealed tiles;
  - `game_get_dev_state` rejects omniscient world access unless explicit `dev_mode` is enabled;
  - No filesystem access or shell execution commands exposed;
- `drl-mcp` integration test suites in `crates/drl-mcp/tests/`:
  - `tests/protocol_jsonrpc.rs`: handshake, tool/resource listing, ping, and malformed request handling;
  - `tests/tools_gameplay.rs`: procedural and scenario game lifecycles, combat, reload, loot, and medpack usage;
  - `tests/security_and_fairness.rs`: dev mode access gating, visibility masking, and turn limit cutoffs;
  - `tests/virtual_ai_player.rs`: virtual AI agent playing a scenario run entirely over MCP JSON-RPC with replay verification;
- `drl-app` updates CLI entry point to support `--mcp` / `mcp` stdio server mode and MCP interactive demo;
- `sh scripts/check-repository.sh` runs all checks, formatting, clippy, and tests cleanly.

Verification:

- `sh scripts/check-repository.sh` succeeds locally;
- `cargo test --workspace` passes all unit, integration, protocol, and virtual AI player test suites;
- `cargo run` demonstrates MCP initialization, tool queries, and semantic actions;
- virtual AI player completes scenario execution over MCP JSON-RPC with bit-exact replay determinism.

Out of scope:

- live Lua scripting integration (Milestone 3);
- native macOS windowing and GPU rendering (Milestone 7);
- native audio engine (Milestone 8).

## Future

Proceed with Milestone 7 native macOS rendering and input interface.
