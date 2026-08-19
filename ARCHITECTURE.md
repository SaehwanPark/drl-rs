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
  errors (`CommandError`), events (`GameEvent::AttackResolved`, `GameEvent::DamageApplied`, `GameEvent::ActorDied`,
  `GameEvent::ActorKnockedBack`, `GameEvent::PlayerTeleported`, `GameEvent::LevelTransitioned`, `GameEvent::ItemDropped`,
  `GameEvent::ItemPickedUp`, `GameEvent::WeaponReloaded`), observations (`Observation`, `TileView`, `ActorView`,
  `PlayerObservation`, `OmniscientObservation`), metrics (`RunOutcome`, `EpisodeMetrics`, `BatchSummary`),
  scenario fixtures (`ScenarioMap`, `ScenarioFixture`), and replay specifications (`ReplayLog`, `ReplayVersion`,
  `ReplayMetadata`, `PlayerSpawnConfig`, `MonsterSpawnSpec`, `ItemSpawnSpec`, `ItemSpawnKind`, `ReplayExecutionError`).
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
    ammo stacking, clip loading/consumption, Phase Device special-use teleportation, kinetic knockback power, and factory
    constructors for Pistol, Shotgun, Combat Knife, 9mm Ammo, Shells, MedPacks, Green Armor, and Phase Device);
  - `inventory`: bounded player backpack inventory (`Inventory`) with automatic ammo merge/stacking
    and equipped gear tracking (`Equipment` for weapon and armor slots);
  - `Actor`: combat stats, durability, speed, energy, inventory, equipment, dynamic weapon damage/accuracy,
    innate or weapon knockback, armor damage protection mitigation, living state, monster archetypes
    (`FormerHuman`, `FormerSergeant`, `Imp`, `Demon`), and death drop loot specifications;
  - `CombatResolver`: pure, deterministic combat calculation routines for melee and ranged attacks;
  - `Scheduler`: energy-based action scheduling algorithm executing actor turns by relative speeds;
  - `World`: physical level state, deterministic `BTreeMap` actor storage, ground items mapping
    (`ground_items: BTreeMap<ItemId, (Position, Item)>`), monster and item spawning,
    fog-of-war map exploration memory (`explored_tiles`), and perception filtering for player observations;
  - `Game`: turn progression kernel executing player commands (movement, bump-attacks, ranged attacks with
    clip ammo deduction, weapon reloading, kinetic knockback resolution with boundary/obstacle collision checks,
    item pickups/drops/equips/consumables, Phase Device teleportation, stairs descent and level transitions),
    monster AI responses, and deterministic event emissions;
  - `scenario`: declarative scenario fixtures (`Scenario`, `ScenarioRunner`) with ASCII grid parser (`Scenario::from_ascii`),
    arbitrary room layouts, monster and loot placements, custom player spawn configurations, and execution runners;
  - `agent`: automated bot policies (`AgentPolicy` trait) consuming strictly `PlayerObservation` without information leakage,
    featuring `RandomBot`, `GreedyCombatBot` (engaging enemies, reloading, healing, looting, stairs descent), and `ExplorerBot`;
  - `batch`: batch simulation runner (`BatchRunner`) executing large volumes of procedural and scenario episodes across diverse seeds
    and computing aggregate statistical metrics (`BatchSummary`);
  - `replay`: deterministic replay execution engine (`ReplayEngine`) with schema validation (`ReplayEngine::validate`),
    diagnostic execution (`run_with_diagnostics`), and bit-exact reproducibility verification across multi-level and scenario command streams.
- `crates/drl-app` is the executable runner (`drl-rust`) that runs headless simulation,
  tactical ranged monster combat, weapon knockback blasts, FOV visibility, item/equipment/reload mechanics,
  Phase Device teleportation, scenario fixture execution, automated agent play, batch simulation sweeps, and replay determinism verification.
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
- `crates/drl-core/tests/stochastic_combat.rs` verifies statistical accuracy scaling, uniform damage roll bounds,
  kinetic knockback obstacle resistance, and multi-turn replay determinism.
- `crates/drl-core/tests/targeting.rs` verifies targeting validation for Position/Entity/Direction targets,
  range bounds, LOS obstruction, and visible target queries.
- `crates/drl-core/tests/visibility.rs` verifies FOV shadowcasting, fog-of-war exploration
  memory persistence, player observation entity hiding, and line-of-fire obstacle blocking.
- `crates/drl-core/tests/scenarios.rs` verifies declarative scenario parsing, ASCII map layout loading,
  custom starting equipment, and scenario execution assertions.
- `crates/drl-core/tests/agents.rs` verifies `RandomBot`, `GreedyCombatBot`, and `ExplorerBot`
  operating solely on player observations and reproducing bit-exact replay streams.
- `crates/drl-core/tests/batch_simulation.rs` verifies `BatchRunner` multi-seed procedural sweeps,
  statistical metrics collection, and failure artifact reproducibility.
- `crates/drl-core/tests/replay_versioning.rs` verifies `ReplayVersion::V1` metadata headers,
  boundary validation, and rich turn/command diagnostic error reporting.

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
  -> SPEC slice
  -> drl-protocol schemas (commands, observations, events, metrics, fixtures, replays)
  -> drl-core deterministic simulation (map, FOV, AI, items, combat, scenarios, agents, batch)
  -> drl-core/tests verification suites (scenarios, agents, batch, replay versioning)
  -> drl-app headless demo & replay verification
```

## Milestone Boundaries

| Component | Responsibility in Milestone 5 |
| --- | --- |
| `drl-protocol` | Domain primitives, commands, observations, events, metrics, scenario fixtures, versioned replays |
| `drl-core` | Pure deterministic simulation, FOV, AI, items, scenarios, bot policies, batch runner, replay validation |
| `drl-app` | Headless execution, scenario bot demo, batch sweep metrics, replay determinism |
| `drl-script` | Placeholder workspace crate |
| `drl-mcp` | Placeholder workspace crate |
| `drl-render` | Placeholder workspace crate |
| `drl-audio` | Placeholder workspace crate |

## Next Architectural Invariants

- Keep `drl-core` pure Rust `std` with zero I/O, rendering, sound, or network dependencies.
- Ensure all agent policies consume exclusively `PlayerObservation` and submit `Command`s.
- Keep replays completely self-contained and reproducible without external asset dependencies.
- Expose all batch metrics and scenario fixtures through typed protocol schemas.
