# Changelog

All notable contributor- and user-visible changes to DRL-Rust will be
documented in this file.

## Unreleased

### Added

- Procedural dungeon level generator (`generator`) in `drl-core` producing bounded
  maps with non-overlapping rectangular rooms connected by walkable L-shaped and
  straight corridors, border walls, and entry/exit placements.
- Invariant reachability and connectivity validation using Breadth-First Search (BFS)
  guaranteeing walkable paths from player spawn to down-stairs and between all rooms.
- Room-based entity and loot distribution spawning representative monsters (Former
  Humans, Imps) and floor items (9mm ammo, Shotgun shells, MedPacks, Shotguns, Armor).
- Exit stairs interaction and level transitions via `Command::Descend`, validating
  stairs presence with `CommandError::NotOnStairs` and transitioning the world to `LevelId(n + 1)`.
- Player state preservation across level transitions, carrying over player health,
  inventory backpack, equipped weapons/armor, clip ammunition, and energy into new levels.
- Replay recording and playback support for down-stairs positions (`ReplayLog.initial_stairs`)
  and multi-level command streams with bit-exact reproducibility.
- Semantic protocol event `GameEvent::LevelTransitioned { from_level, to_level }`
  and command `Command::Descend`.
- Comprehensive integration test suite in `crates/drl-core/tests/level_progression.rs`
  verifying procedural generator connectivity, stairs validation, player state retention,
  and multi-level replay determinism.
- Headless demo runner update in `drl-app` demonstrating combat, floor looting, stairs
  descent, level transition from Level 1 to Level 2, and replay determinism.

- Item domain model (`item`) in `drl-core` and `drl-protocol` with physical item
  properties, weapons, body armor, ammunition stacks, and consumables.
- Bounded player inventory (`Inventory`) with automatic ammunition stacking, stack
  draining, and capacity enforcement.
- Equipment system (`Equipment`) supporting dedicated weapon and armor slots,
  equipment swapping, and unequip validation.
- Weapon and ammunition mechanics with magazine clip tracking, ammo consumption
  on ranged attacks, clip exhaustion errors (`CommandError::NoAmmoInClip`), and
  reloading (`Command::Reload`) from reserve inventory ammo stacks.
- Representative weapons: Pistol (9mm caliber, 10-round clip), Shotgun (Shells,
  8-round clip), Combat Knife (melee).
- Representative items: 9mm Ammo, Shotgun Shells, Small MedPack (+10 HP), Large
  MedPack (+25 HP), Green Armor (+5 armor protection).
- Ground item tracking in `World` with deterministic `BTreeMap` storage, floor loot
  spawning, pickup (`Command::Pickup`), and dropping (`Command::Drop`).
- Perception filtering for ground items, exposing only floor items on explored
  fog-of-war tiles in `PlayerObservation.ground_items`.
- Armor damage protection mitigation reducing raw incoming damage in combat.
- Replay recording and playback support for initial item spawns (`ItemSpawnSpec`).
- Comprehensive integration test suite in `crates/drl-core/tests/inventory.rs`
  verifying pickups, drops, capacity limits, equip/unequip cycles, medpack use,
  weapon firing, and reload cycles.
- Headless demo runner update in `drl-app` demonstrating item pickups, weapon
  swapping, ranged combat, and healing with bit-exact replay determinism.

- Field of View (FOV) calculation and Line of Sight (LOS) ray tracing module
  (`fov`) in `drl-core` supporting deterministic perimeter raycasting, obstacle
  occlusion, and transparency checks.
- Fog-of-war map exploration memory in `World` tracking explored tiles and
  revealing previously seen terrain.
- Perception filtering in `PlayerObservation` strictly hiding unobserved entities
  and monsters behind obstacles or outside the active field of view.
- Line-of-fire obstacle checks for ranged attacks (`Command::AttackRanged`),
  rejecting blocked shots with `CommandError::LineOfSightBlocked`.
- Extended `TileView` in `drl-protocol` with `is_visible` flag distinguishing
  active FOV cells from remembered fog-of-war cells.
- End-to-end integration test suite in `crates/drl-core/tests/visibility.rs`
  verifying shadowcasting, fog-of-war exploration persistence, entity filtering,
  and line-of-fire validation.
- Headless demo update in `drl-app` displaying active FOV and explored fog-of-war
  tile metrics per turn.
- Action economy and energy-based actor scheduling system (`Scheduler`) in `drl-core`
  supporting relative actor speeds and deterministic turn ordering.
- Pure, deterministic combat calculation module (`CombatResolver`) in `drl-core`
  resolving melee and ranged attacks with explicit seedable RNG.
- Melee bump-attacks, direct melee attacks (`Command::AttackMelee`), and targeted
  ranged attacks (`Command::AttackRanged`) with range and obstacle validation.
- Domain models in `drl-protocol` for combat stats (`HitPoints`, `Speed`, `ActionCost`,
  `DamageAmount`, `DamageType`, `DamageSource`, `DeathCause`, `AttackOutcome`).
- Combat and scheduling events (`GameEvent::AttackResolved`, `GameEvent::DamageApplied`,
  `GameEvent::ActorDied`, `GameEvent::ActionCostPaid`).
- Autonomous monster AI turn execution during scheduled energy intervals, reacting
  to player positions and executing attacks.
- Actor health tracking, damage deduction with clamping, death state transitions,
  and dead actor occupancy unblocking.
- Replay support for monster spawns and combat command streams via `MonsterSpawnSpec`.
- Headless combat demonstration in `drl-app` running a multi-turn tactical scenario
  and verifying bit-for-bit replay determinism.
- Comprehensive unit and end-to-end integration test suites in `crates/drl-core/tests/combat.rs`.
- Headless simulation kernel (`drl-core`) with deterministic seedable `GameRng`
  (SplitMix64 + Xoshiro256++), 2D bounded tile maps (`Map`, `Tile`), and physical
  world state (`World`) with deterministic entity storage.
- Shared semantic protocol contracts (`drl-protocol`) including domain types
  (`Position`, `Direction`, `Turn`, `EntityId`, `ItemId`, `LevelId`), commands
  (`Command::Move`, `Command::Wait`), typed errors (`CommandError`), events
  (`GameEvent`), observations (`Observation`, `TileView`, `ActorView`), and replay
  logs (`ReplayLog`).
- Deterministic turn loop execution kernel (`Game::step`) with movement validation,
  collision detection against terrain and entities, and ordered event emission.
- Deterministic replay execution engine (`ReplayEngine`) and validation tests
  verifying bit-for-bit identical state reproduction across independent runs.
- Executable headless simulation demonstration in `drl-app` running a multi-step
  scenario and verifying replay determinism.
- Comprehensive unit and end-to-end integration tests for movement, terrain bounds,
  occupancy collisions, PRNG reproducibility, and observation snapshots.
- Multi-crate Cargo workspace managing `drl-core`, `drl-protocol`, `drl-app`,
  `drl-script`, `drl-mcp`, `drl-render`, and `drl-audio`.
- Deterministic headless simulation core library (`drl-core`) and shared
  protocol contract library (`drl-protocol`).
- Default workspace application executable (`drl-app` / `drl-rust`).
- Automated architectural boundary tests ensuring `drl-core` and `drl-protocol`
  remain free of presentation, audio, and MCP dependencies.
- A repo-local milestone-delivery harness with durable repository guidance.
- A staged development and test-play team contract with explicit ownership,
  deterministic handoffs, and bounded delegation.
- Reusable legacy-archaeology, capability-gated test-play, and independent
  determinism-review skills.
- Repository checks for skill structure, required harness paths, and handoff
  and result-status vocabulary.
- Lightweight specification, architecture, and changelog documents governed by
  the canonical project roadmap.
- Dependency-light two-space formatting checks shared by local development and
  macOS CI.
- Contributor-facing README guidance for the current scaffold, project
  direction, legacy research setup, and licensing boundaries.
