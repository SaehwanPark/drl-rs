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
  (`Position`, `Direction`, `Turn`, `EntityId`, `ItemId`, `LevelId`, `HitPoints`,
  `Speed`, `ActionCost`, `DamageType`, `DamageSource`, `DeathCause`, `AttackOutcome`, `Target`, `MonsterKind`),
  item types (`AmmoType`, `EquipmentSlot`, `ItemCategory`, `ItemView`, `GroundItemView`),
  commands (`Command::Move`, `Command::AttackMelee`, `Command::AttackRanged`, `Command::Wait`,
  `Command::Pickup`, `Command::Drop`, `Command::Equip`, `Command::Unequip`, `Command::Use`, `Command::Reload`, `Command::Descend`),
  errors (`CommandError`), events (`GameEvent`), observations (`Observation`, `TileView`, `ActorView`, `PlayerObservation`, `OmniscientObservation`),
  and replay specifications (`ReplayLog`, `MonsterSpawnSpec`, `ItemSpawnSpec`, `ItemSpawnKind`).
- `crates/drl-core` is the deterministic headless simulation core library containing:
  - `GameRng`: deterministic seedable PRNG (SplitMix64 + Xoshiro256++) with no ambient
    or global state;
  - `Map` & `Tile`: 2D bounded grid representation with walkability, transparency, and exit stairs (`Tile::StairsDown`);
  - `fov`: pure, deterministic field-of-view (`compute_fov`), line-of-sight raycasting (`has_line_of_sight`),
    and discrete ray tracing (`line_points`);
  - `targeting`: pure targeting validation (`TargetingSystem::validate_target`), visible targets query (`find_visible_targets`),
    and nearest enemy auto-selection (`find_nearest_target`);
  - `ai`: deterministic monster tactical AI decision kernel (`MonsterAi::decide_action`, `MonsterAction`)
    resolving melee attacks, ranged attacks with line-of-sight checks, and pathfinding pursuit;
  - `generator`: procedural dungeon level generator (`LevelGenerator`, `LevelGeneratorConfig`, `GeneratedLevel`, `Room`, `MonsterSpawn`)
    with non-overlapping room carving, L-shaped/straight corridor connections, down-stairs placement, BFS reachability validation,
    and deterministic monster/floor loot distribution;
  - `item`: domain item models (`Item`, `WeaponProperties`, `ArmorProperties`, `ConsumableProperties`,
    ammo stacking, clip loading/consumption, Phase Device special-use teleportation, and factory constructors for Pistol,
    Shotgun, Combat Knife, 9mm Ammo, Shells, MedPacks, Green Armor, and Phase Device);
  - `inventory`: bounded player backpack inventory (`Inventory`) with automatic ammo merge/stacking
    and equipped gear tracking (`Equipment` for weapon and armor slots);
  - `Actor`: combat stats, durability, speed, energy, inventory, equipment, dynamic weapon damage/accuracy,
    armor damage protection mitigation, living state, monster archetypes (`FormerHuman`, `FormerSergeant`, `Imp`, `Demon`),
    and death drop loot specifications;
  - `CombatResolver`: pure, deterministic combat calculation routines for melee and ranged attacks;
  - `Scheduler`: energy-based action scheduling algorithm executing actor turns by relative speeds;
  - `World`: physical level state, deterministic `BTreeMap` actor storage, ground items mapping
    (`ground_items: BTreeMap<ItemId, (Position, Item)>`), monster and item spawning,
    fog-of-war map exploration memory (`explored_tiles`), and perception filtering for player observations;
  - `Game`: turn progression kernel executing player commands (movement, bump-attacks, ranged attacks with
    clip ammo deduction, weapon reloading, item pickups/drops/equips/consumables, Phase Device teleportation,
    stairs descent and level transitions), monster AI responses, and deterministic event emissions;
  - `ReplayEngine`: deterministic replay execution and bit-exact state verification across multi-level command streams.
- `crates/drl-app` is the executable runner (`drl-rust`) that runs headless simulation,
  tactical ranged monster combat, FOV visibility, item/equipment/reload mechanics, Phase Device teleportation,
  and multi-level stairs descent demonstrations and verifies replay reproducibility.
- `crates/drl-script`, `crates/drl-mcp`, `crates/drl-render`, and
  `crates/drl-audio` are placeholder workspace crates with bounded dependency
  declarations.
- `crates/drl-core/tests/boundaries.rs` enforces architectural dependency
  direction via automated tests.
- `crates/drl-core/tests/combat.rs` verifies end-to-end combat encounters, ranged attacks,
  monster response, death transitions, and replay determinism.
- `crates/drl-core/tests/inventory.rs` verifies ground item pickups, drops, inventory capacity limits,
  weapon swapping, armor damage mitigation, ammo consumption, reloading, and medpack healing.
- `crates/drl-core/tests/level_progression.rs` verifies procedural generation connectivity, stairs descent
  validation, player state persistence across level boundaries, and multi-level replay determinism.
- `crates/drl-core/tests/monsters_ai.rs` verifies monster archetypes, tactical ranged AI with LOS checks,
  speed turn frequencies, monster death loot drops, and combat replay determinism.
- `crates/drl-core/tests/simulation.rs` verifies multi-step movement, collision,
  observation, and replay determinism.
- `crates/drl-core/tests/special_items.rs` verifies Phase Device teleportation, destination safety invariants,
  FOV exploration updates, and replay determinism.
- `crates/drl-core/tests/targeting.rs` verifies targeting validation for Position/Entity/Direction targets,
  range bounds, LOS obstruction, and visible target queries.
- `crates/drl-core/tests/visibility.rs` verifies FOV shadowcasting, fog-of-war exploration
  memory persistence, player observation entity hiding, and line-of-fire obstacle blocking.

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
