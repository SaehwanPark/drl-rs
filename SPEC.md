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
- Milestone 2 established action economy, energy-based actor scheduling, melee and ranged
  combat resolution, HP/damage domain types, death transitions, and combat event emission.
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
- Milestone 6 established the complete Model Context Protocol (MCP) server (`drl-mcp`): zero-dependency
  JSON-RPC 2.0 engine, semantic tool suite (`game_start`, `game_get_observation`, `game_list_actions`,
  `game_step_action`, `game_reset`, `game_get_metrics`, `game_save_replay`, `game_get_dev_state`),
  static/live resources, security and fairness boundaries, stdio transport runner, and comprehensive
  integration test suites including a virtual AI player test with bit-exact replay verification.

## Present

### Milestone 0: Repository Foundation — Documentation Completion Slice

Status: Active

This slice closes the remaining open M0 checklist items that are directly
deliverable now that the domain is understood through M6.

Observable outcomes:

- `CONTRIBUTING.md` at the repository root documents the developer onboarding
  path: workspace crate map, code style (2-space indent, `rustfmt`, `clippy`),
  branch naming convention, pull request workflow, local check procedure
  (`sh scripts/check-repository.sh`), and architectural do-not-cross rules
  (no ambient RNG, no `drl-core` presentation dependencies);
- `docs/adr/` directory created with six initial Architecture Decision Records:
  - `0001` — Project architecture principles (functional-core/imperative-shell,
    typed domain, ADTs, explicit state, no ambient state);
  - `0002` — No legacy backward compatibility (no saves, mods, WAD, or RNG
    stream compatibility with the legacy Pascal implementation);
  - `0003` — Semantic command model (all clients use the same `Command` type);
  - `0004` — Explicit deterministic RNG (`GameRng` wraps SplitMix64 +
    Xoshiro256++; no global or ambient RNG);
  - `0005` — Lua transitional strategy (Lua behind a narrow typed boundary;
    Rust owns all simulation invariants);
  - `0006` — MCP semantic interface strategy (MCP as first-class agent and test
    interface via JSON-RPC 2.0, not a simulation bypass);
- `docs/legacy-behavior/` directory created with four documents:
  - `_template.md` — reusable template for future behavior-spec notes;
  - `movement.md` — movement semantics shell: verified behaviors, open
    questions, and explicit uncertainty;
  - `turn-economy.md` — action-cost semantics shell: energy model, speed
    interactions, and scheduling contract;
  - `combat.md` — combat semantics shell: hit resolution, damage calculation,
    death, and known unknowns;
- The roadmap progress table updated to reflect actual M0–M6 delivery status.

Verification:

- `sh scripts/check-repository.sh` succeeds with no new failures;
- All new files pass tab and trailing-whitespace checks;
- `cargo fmt`, `cargo clippy`, and `cargo test` are unaffected (no Rust code changed).

Out of scope for this slice:

- `CODE_OF_CONDUCT.md` (requires public-contribution policy decision);
- `tests/fixtures/` and `content/` directories (belong with gameplay milestones);
- Dependency-audit tooling and dependency update policy;
- Remote CI macOS run (requires CI infrastructure action);
- Provenance and asset-licensing inventory;
- Milestone 3 Lua runtime implementation;
- Milestone 7 native macOS rendering.

## Future

Proceed with Milestone 7: native macOS rendering and input interface.
