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

## Present

### Milestone 4: Procedural Level Generation, Stairs, Level Transitions, and Multi-Level Headless Mini-Run

Status: Active

This slice implements procedural level generation (connected rooms and corridors with reachability
invariants), exit stairs, player descent commands, seamless multi-level world transitions (carrying
over player stats, health, inventory, and equipped items), and a complete multi-level headless mini-run.

Observable outcomes:

- `drl-protocol` defines new semantic player commands and error conditions:
  - `Command::Descend` (descends stairs at current position to transition to the next level);
  - Typed error variant in `CommandError` (`NotOnStairs(Position)`);
- `drl-protocol` defines new semantic game events:
  - `GameEvent::LevelTransitioned { from_level: LevelId, to_level: LevelId }`;
- `drl-core` implements an isolated procedural map generator in `crates/drl-core/src/generator.rs`:
  - `LevelGenerator`: deterministic procedural generation of non-overlapping rectangular rooms connected
    by walkable L-shaped and straight corridors with surrounding perimeter walls;
  - Configurable room count, room dimension constraints, monster spawn density, and item spawn density;
  - Automatic placement of player spawn in the starting room and `Tile::StairsDown` in the exit room;
  - Formal reachability and connectivity verification (BFS flood-fill) ensuring all rooms and the exit
    stairs are reachable from the player spawn point;
  - Populates rooms with representative monsters (Former Humans, Imps) and floor loot (Ammo, MedPacks, Shotguns);
  - Deterministic generation: identical seed produces identical map layout, actor spawns, and floor items;
- `drl-core` implements level transition logic in `World` and `Game`:
  - `Command::Descend` verifies the player stands on `Tile::StairsDown`;
  - On valid descent, preserves player actor entity state (current/max HP, inventory backpack, equipped weapon
    with magazine clip state, equipped armor, and action energy) and transitions to `LevelId(current + 1)`;
  - Instantiates new `World` with generated map, resets fog-of-war exploration for the new floor, places player
    at new level spawn point, and populates floor monsters and loot;
  - Emits `GameEvent::LevelTransitioned`;
- `drl-core` supports multi-level deterministic replay execution:
  - `ReplayEngine` accurately records and replays multi-level command streams with bit-exact state reproduction;
- `drl-app` demonstrates a multi-level headless mini-run:
  - Level 1: exploration, engaging monsters, looting ammo/weapons, healing, reaching stairs, descending;
  - Level 2: continuation with preserved inventory/health, further combat and exploration;
  - Bit-exact replay verification across the full multi-level run;
- `sh scripts/check-repository.sh` runs all checks, formatting, clippy, and tests cleanly.

Verification:

- `sh scripts/check-repository.sh` succeeds locally;
- `cargo test --locked --workspace` passes all unit, integration, boundary, combat,
  visibility, inventory, generator, and replay determinism tests;
- unit tests in `crates/drl-core/src/generator.rs` verify deterministic generation, room count,
  and flood-fill reachability;
- integration tests in `crates/drl-core/tests/level_progression.rs` verify stairs descent validation,
  level transition events, player state persistence across level transitions, and multi-level replay determinism;
- `cargo run` executes the headless demo demonstrating procedural level exploration, combat, looting,
  stairs descent, and replay verification.

Out of scope:

- live Lua scripting integration (Milestone 3);
- MCP transport servers (Milestone 6);
- presentation/GUI rendering (Milestone 7) and audio (Milestone 8).

## Future

Proceed with Milestone 5 replay suite, scripted bots, and automated scenario frameworks.
