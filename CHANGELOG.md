# Changelog

All notable contributor- and user-visible changes to DRL-Rust will be
documented in this file.

## Unreleased

- Browser-first steering and playable-slice implementation: accepted ADRs
  0007/0008, reconciled proposal/roadmap/spec/architecture/README/contributor
  guidance, browser-aware agent harness rules, and dynamic repo-local skill
  validation.
- Added `drl-assets` with the complete tracked legacy graphics import,
  CC BY-SA attribution, pinned source revision, and SHA-256 manifest. Legacy
  audio/music/fonts remain redistribution-gated; controlled runtime captures
  are recorded as `NOT_RUN` on the arm64 macOS host, with a capture-to-M7/M8
  fidelity matrix.
- Added additive fair observation identifiers, pure `PresentationStep` and
  `RenderScene` builders, semantic audio cues, transactional `BrowserSession`,
  WASM/WebGPU/Web Audio bindings, accessible static browser shell, and web
  build/check/serve scripts. Native headless and MCP contracts remain intact.
- Verified the M7 functional gate with a Chrome 151 WebGPU smoke playthrough
  (Apple Metal-3, 1280x720, DPR 1; explicit gesture-gated audio state) and
  remote CI run `32538527707`; fixed startup and mute-control status races so
  the visible status reflects the actual or applied audio state, and serialized
  rapid audio-control events to prevent stale settings. Legacy reference-capture
  comparison remains open and explicitly `NOT_RUN`.
- Added the first bounded M8 presentation slice: pure `PixelViewport` layout
  math chooses centered integer square cells, and the WebGPU scene uses those
  rectangles for deterministic letterboxing. Focused render tests, local WASM
  compilation, native web contracts, asset checks, local browser smoke, and
  hosted run `32539486760` pass; capture-backed audiovisual parity remains
  `NOT_RUN`.
- Added the follow-up M8 lighting slice: pure `LightingBand`/`shade_color`
  rules derive full light versus fixed explored-tile fog from fair scene data,
  and WebGPU consumes the shared rule. Capture-backed lighting equivalence is
  still `NOT_RUN`.
- Moved the existing quarter-health WebGPU clear-color threshold into pure
  `drl-render::SceneTone`/`scene_clear_color` planning with focused tests; the
  browser renderer now consumes the shared tone rule.
- Added pure event-ordered `drl-render::EffectSpan` timing with fixed logical
  durations for presentation effects; frontend ticks cannot advance gameplay,
  and capture-backed animation timing remains `NOT_RUN`.
- Carried those ordered effect spans through successful browser
  `PresentationStep` results, so future frame mapping does not rebuild raw
  event semantics or cross the simulation boundary.
- Filtered browser effect spans against before/after visible actors while
  retaining direct player transitions, preventing hidden monster events from
  becoming presentation timing.
- Replaced placeholder atlas cells with measured 32-pixel legacy sprite slots
  for all current tile, actor, and item semantics. `drl-assets` now exposes
  imported PNG dimensions and pure rectangle bounds checks; texture compositing
  and capture-backed audiovisual parity remain open.
- Added deterministic registered source-layer metadata for every imported
  atlas and aligned semantic descriptors with their atlas-specific layer
  order. This is compositor input only; it does not claim blending or capture
  parity.
- Corrected the pinned legacy graphics revision everywhere to the exact
  40-character Git commit, keeping asset provenance and capture scripts
  reproducible.
- Added renderer-neutral normalized UV conversion for bounded sprite cells,
  with explicit top-left image origin and invalid-dimension rejection. Texture
  sampling remains a future compositor concern.
- Added `drl-render::layer_draw_plan`, a deterministic renderer-neutral plan of
  atlas layers, pixel destinations, and normalized UVs for fair scene sprites.
  It preserves explored tile memory and keeps texture upload/blending and
  capture-backed parity as future work.
- Added pure `AtlasTextureSource` bindings for every registered atlas layer;
  draw-plan entries now carry imported relative paths and measured dimensions
  for a future frontend upload boundary. No image loading or compositing is
  claimed.
- Carried the shared fair `LightingBand` into every `LayerDraw`, preserving the
  fixed explored-memory fog factor and full-light visible-sprite factor for a
  future compositor without exposing hidden state.
- Added explicit renderer-neutral `LayerRole` metadata for the legacy shader's
  base-color, colorization-mask, outline-mask, and emissive-mask inputs;
  texture upload, blend equations, and capture parity remain future work.
- Added `drl-render::sprite_composite_plan`, grouping complete role sets into
  one deterministic compositor input per fair scene sprite while rejecting
  malformed groups; GPU sampling and blend equations remain future work.
- Added a subpath-safe `drl-web::browser_asset_url` and WASM
  `load_texture_source` preflight that decodes same-origin imported PNGs and
  validates their manifest dimensions before any future GPU upload.
- Added a deterministic 24-source manifest and renderer-owned WASM WebGPU
  texture/view cache. Each validated decoded PNG is uploaded once with the
  external-image copy API; shader sampling and role compositing remain open.
- Added the first nearest-filtered base-color WGSL pass using grouped fair
  sprite UVs, source-specific bind groups, alpha blending, and shared lighting;
  geometry remains the fallback and mask/outline/emissive compositing remains
  future work.
- Added the bounded emissive-role follow-up: each registered sprite pairs its
  base source with an optional emissive source, samples the emissive red channel
  as a lighting floor, and uses a transparent 1x1 fallback when absent. Mask,
  colorization, outline/glow, and capture-backed shader parity remain open.
- Added the verified legacy `0.1` fragment-alpha cutoff to the textured WGSL
  pass; transparent edge fragments are discarded before source-alpha blending,
  while the fair/emissive lighting floor remains unchanged.
- Aligned renderer-owned atlas and transparent role-fallback storage with the
  observed legacy `GL_RGBA8` contract by using linear normalized
  `Rgba8Unorm`; browser display color-space parity remains capture-gated.
- Added a native contract test over the shared textured WGSL source, guarding
  base/emissive sampling, fair-lighting `max`, alpha cutout, and output terms
  while the runtime shader remains WASM-only.
- Added the bounded colorization-mask role to the textured WGSL pass. Optional
  mask views use the retained transparent fallback, and the current fair scene
  path supplies a neutral zero tint until per-sprite tint provenance is
  implemented; outline/glow and capture-backed shader parity remain open.
- Added pure `active_effect_frames` progress mapping for fair effect spans;
  frontend ticks receive stable normalized progress without advancing gameplay
  or claiming legacy animation-frame parity.

- `CONTRIBUTING.md` added at the repository root, covering workspace crate
  map, prerequisites, code style (2-space indent, `rustfmt`, `clippy`), branch
  naming and commit conventions, pull request workflow, local check procedure
  (`sh scripts/check-repository.sh`), and architectural do-not-cross rules.
- `docs/adr/` directory created with six initial Architecture Decision Records:
  - `0001` — Project architecture principles (functional-core/imperative-shell,
    typed domain, ADTs, explicit state, no ambient state, clean boundaries,
    testability, no premature abstraction);
  - `0002` — No legacy backward compatibility (no saves, mods, WAD, or RNG
    stream compatibility with the Pascal implementation);
  - `0003` — Semantic command model (all clients submit `Command` through the
    same simulation API; no privileged mutation paths);
  - `0004` — Explicit deterministic RNG (`GameRng` wraps SplitMix64 +
    Xoshiro256++; no global or ambient RNG in `drl-core`);
  - `0005` — Lua transitional strategy (Lua behind a narrow typed boundary;
    Rust owns all simulation invariants; Lua errors are isolated);
  - `0006` — MCP semantic interface strategy (MCP as first-class agent/test
    interface via JSON-RPC 2.0 stdio; not a simulation bypass; player
    information boundaries enforced; replay determinism preserved).
- `docs/legacy-behavior/` directory created with four documents:
  - `_template.md` — reusable template distinguishing verified behaviors,
    inferred design intent, legacy implementation artifacts, deliberate
    DRL-Rust decisions, and open questions;
  - `movement.md` — movement semantics shell covering grid movement, bounds
    enforcement, occupancy, diagonal movement, level exit, and action cost;
  - `turn-economy.md` — action-cost semantics shell covering the energy-based
    scheduling model, actor speed, action cost uniformity, and dead actor
    handling;
  - `combat.md` — combat semantics shell covering hit resolution (accuracy
    roll, range penalty, LOS requirement), damage calculation (uniform roll,
    armor mitigation, HP clamping), death, knockback, and loot drops.
- Roadmap progress table updated: M0 status corrected to "Complete";
  M1, M2, M4, M5, M6 statuses updated to "Complete" with delivery summaries.
- M0 roadmap checklist items marked complete: `CONTRIBUTING.md`, `docs/adr/`,
  `docs/legacy-behavior/`, the three implemented behavior shells
  (`combat.md`, `movement.md`, `turn-economy.md`), behavior-spec template, and
  the harness/documentation checks. The earlier “six behavior areas” wording
  was corrected; three shells are present.
- `ARCHITECTURE.md` updated to document `docs/adr/` and `docs/legacy-behavior/`
  as recognized structural components.

- Full Model Context Protocol (MCP) server implementation (`crates/drl-mcp`) providing machine-operable semantic
  gameplay interfaces for AI-driven testing, playtesting, and evaluation.
- Zero-dependency JSON-RPC 2.0 communication engine in pure Rust `std` (`drl_mcp::json`, `drl_mcp::protocol`) supporting
  standard MCP protocol methods (`initialize`, `ping`, `tools/list`, `tools/call`, `resources/list`, `resources/read`).
- Semantic tool suite:
  - `game_start`: initialize seeded procedural dungeon sessions with configurable dimensions and turn limits;
  - `game_load_scenario`: parse and load declarative ASCII scenario layouts;
  - `game_get_observation`: retrieve fair player-visible world views (FOV tiles, visible actors, inventory, equipment);
  - `game_list_actions`: dynamically synthesize available legal actions (`Move`, `AttackRanged`, `Reload`, `Pickup`,
    `Use`, `Equip`, `Unequip`, `Drop`, `Wait`, `Descend`);
  - `game_step_action`: execute semantic actions directly through the simulation core;
  - `game_reset`: reset session back to starting configuration;
  - `game_get_metrics`: fetch real-time episode telemetry and terminal outcomes;
  - `game_save_replay`: export deterministic session replay logs;
  - `game_get_dev_state`: developer-only omniscient world state inspection gated by explicit `dev_mode` flag.
- Static and dynamic game resources (`drl://rules/game`, `drl://rules/actions`, `drl://session/metrics`, `drl://session/events`).
- Stdio transport runner (`McpServer::run_stdio`) and CLI integration in `drl-app` (`drl-rust --mcp` or `drl-rust mcp`).
- Comprehensive MCP integration test suites (`protocol_jsonrpc.rs`, `tools_gameplay.rs`, `security_and_fairness.rs`,
  `virtual_ai_player.rs`) verifying information boundaries, error handling, tool workflows, and bit-exact replay determinism.
- Completion of Milestone 6: MCP Game Interface deliverables and exit criteria.

- Versioned replay log schema (`ReplayVersion::V1`, `ReplayMetadata`) in `drl-protocol` and
  `drl-core` supporting engine version headers, custom player spawn configurations (`PlayerSpawnConfig`),
  and explicit tile override maps.
- Diagnostic replay error reporting with `ReplayExecutionError` capturing exact turn numbers,
  0-based command indices, failed commands, and underlying simulation error contexts.
- Replay validation engine (`ReplayEngine::validate`) ensuring all coordinates, bounds, entities,
  items, and stairs are physically consistent prior to execution.
- Declarative scenario fixture framework (`Scenario`, `ScenarioFixture`, `ScenarioMap`) in `drl-protocol`
  and `drl-core` supporting multi-room ASCII map parsing (`Scenario::from_ascii`), custom monster/item placements,
  starting equipment, stairs configurations, and fluent assertion runners (`ScenarioRunner`).
- Scripted test agent policies (`AgentPolicy` trait) consuming strictly `PlayerObservation` and emitting
  `Command`s without information leakage:
  - `RandomBot`: uniform random selection among legal walkable directions and interactions;
  - `GreedyCombatBot`: tactical survival bot prioritizing health restoration, weapon reloading,
    line-of-fire checks, ranged and melee engagements, item looting, and exit stairs descent;
  - `ExplorerBot`: goal-directed exploration bot navigating uncharted maze corridors and descending stairs.
- Headless batch simulation runner (`BatchRunner`) executing high-throughput procedural and scenario sweeps
  across arbitrary seeds with configurable episode limits, recording `EpisodeRecord` artifacts, and aggregating
  statistical summaries (`BatchSummary`: win rates, average turns, total kills, damage dealt/taken).
- Runtime metrics accumulation (`RunOutcome`, `EpisodeMetrics`) tracking completion status, damage telemetry,
  kill distributions, item pickups, ammo expenditure, and level progression.
- Integration test suites in `crates/drl-core/tests/`:
  - `scenarios.rs`: ASCII grid parsing, custom hero loadouts, and multi-step scenario metrics;
  - `agents.rs`: headless policy execution, combat room clearing, maze navigation, and bit-exact replay determinism;
  - `batch_simulation.rs`: multi-seed procedural batches, statistical validation, and sweep determinism;
  - `replay_versioning.rs`: metadata header validation, boundary rejection, and diagnostic error locations.
- Headless application runner update in `drl-app` executing declarative scenario fixtures, automated bot play,
  batch sweeps, and replay determinism verification.
- Completion of Milestone 5: Replay, Scenario, and Test-Agent Infrastructure deliverables and exit criteria.

- Weapon kinetic knockback mechanics in `drl-core` (`apply_knockback`) and `drl-protocol`
  (`GameEvent::ActorKnockedBack { entity_id, from, to }`), enabling pump-action Shotgun
  and Former Sergeant shotgun attacks to push surviving targets 1 tile backwards along the firing vector.
- Map boundary, terrain obstacle, and actor collision checks for knockback resolution, ensuring
  actors never clip into walls, out-of-bounds cells, or occupied tiles.
- Weapon property `knockback: u32` in `WeaponProperties` and `ItemView`, configured with 1 for
  `Item::shotgun` and `Actor::former_sergeant`.
- Immediate FOV and fog-of-war exploration updates when the player character is knocked back by enemies.
- Comprehensive statistical test suite in `crates/drl-core/tests/stochastic_combat.rs` validating
  empirical accuracy scaling across distances, 3-sigma confidence intervals, uniform damage distributions,
  and bit-exact multi-turn knockback replay determinism.
- Completion of Milestone 4: Core DRL Gameplay Vertical Slice roadmap deliverables and exit criteria.
- Headless demo runner update in `drl-app` displaying real-time kinetic knockback event telemetry.

- Enemy archetypes domain and factory constructors in `drl-protocol` and `drl-core`

  (`FormerHuman`, `FormerSergeant`, `Imp`, `Demon`) with distinct health, speed,
  melee, ranged attack ranges, accuracies, and death loot drop tables.
- Tactical Monster AI decision module (`ai`) in `drl-core` (`MonsterAi::decide_action`)
  supporting adjacent melee attacks, ranged projectile/fireball attacks with line-of-sight checks,
  and pathfinding pursuit towards the player.
- Targeting system module (`targeting`) in `drl-core` (`TargetingSystem`) providing
  pure validation for `Target::Position`, `Target::Entity`, and `Target::Direction`
  with out-of-bounds, range limit, and line-of-sight obstruction checks, as well as visible
  target listing and nearest enemy auto-targeting.
- Special-use consumable item `Phase Device` in `drl-protocol` and `drl-core` allowing
  emergency spatial relocation to random walkable unoccupied cells, updating FOV and fog of war.
- Monster death loot drop mechanics spawning floor items upon lethal combat resolution
  at the monster's exact position and emitting `GameEvent::ItemDropped`.
- Semantic protocol event `GameEvent::PlayerTeleported { from, to }`, `Target` enum,
  `MonsterKind` enum, and `ItemCategory::PhaseDevice`.
- Replay logging support for ranged monster specs with builder `with_ranged_combat`
  and `ItemSpawnKind::PhaseDevice`.
- Integration test suites in `crates/drl-core/tests/monsters_ai.rs`, `crates/drl-core/tests/special_items.rs`,
  and `crates/drl-core/tests/targeting.rs` verifying tactical AI behaviors, Phase Device safety,
  target validation, and bit-exact replay determinism.
- Headless demo runner update in `drl-app` demonstrating tactical ranged monster combat,
  loot drops, Phase Device emergency teleportation, and replay determinism.


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
