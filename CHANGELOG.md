# Changelog

All notable contributor- and user-visible changes to DRL-Rust will be
documented in this file.

## Unreleased

### Added

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
