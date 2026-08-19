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

## Present

### Milestone 4: Enemy Archetypes, Tactical Monster AI, Target Legality & Selection, and Special-Use Phase Device

Status: Active

This slice implements representative enemy archetypes (Former Human, Former Sergeant, Imp, Demon/Pinky),
tactical monster AI with ranged attack capabilities and line-of-fire evaluation, monster death loot drops,
target validation and metadata querying, and special-use consumable items (Phase Device teleportation).

Observable outcomes:

- `drl-protocol` defines target domain models, enemy classifications, new events, and error variants:
  - `Target` enum (`Target::Entity(EntityId)`, `Target::Position(Position)`, `Target::Direction(Direction)`);
  - `MonsterKind` enum (`FormerHuman`, `FormerSergeant`, `Imp`, `Demon`) for typed spawns and replays;
  - `GameEvent::PlayerTeleported { from: Position, to: Position }`;
  - `GameEvent::ItemDropped { item_id: ItemId, position: Position, item_name: String }`;
  - `CommandError::InvalidTarget(String)`, `CommandError::NoTargetInRange`;
- `drl-core` implements enemy archetype models and death drop configurations:
  - `Actor::former_human`: armed with pistol, ranged attacks, drops 9mm ammo on death;
  - `Actor::former_sergeant`: armed with shotgun, high damage close/mid-range attacks, drops shells or shotgun;
  - `Actor::imp`: hurling fireballs at range with LOS, slashing in melee, drops medpack;
  - `Actor::demon`: fast melee charger (Speed 130), high HP, biting melee strikes;
- `drl-core` implements tactical AI decision engine in isolated module `crates/drl-core/src/ai.rs`:
  - `MonsterAi::decide_action`: pure evaluation determining whether a monster executes a melee strike,
    ranged attack with line-of-fire validation, moves closer to close distance, or waits;
  - Monsters with ranged capabilities fire upon the player when within maximum range and line-of-fire is clear;
  - Monsters unable to fire or with pure melee attacks navigate towards the player's position;
  - On monster death, configured loot drops are automatically placed on the ground at the death location;
- `drl-core` implements target legality checking and target query helpers in `crates/drl-core/src/targeting.rs`:
  - `TargetingSystem::validate_target`: verifies bounds, range, living status, and line of fire;
  - `TargetingSystem::find_visible_targets`: queries and sorts hostile actors in the player's current field of view;
  - `TargetingSystem::find_nearest_target`: selects closest visible hostile target for auto-targeting;
- `drl-core` implements special-use consumable item mechanics:
  - `Item::phase_device`: special consumable that teleports user to a safe, random walkable floor tile;
  - `Game::execute_use_item` handles phase device usage, relocates player, updates FOV/fog-of-war exploration,
    and emits `GameEvent::PlayerTeleported`;
- `drl-core` integrates archetypes and special items into procedural generation:
  - `LevelGenerator` populates diverse monster archetypes and phase devices across dungeon rooms;
- `drl-app` demonstrates tactical monster engagements with ranged Former Sergeants, fast charging Demons,
  and emergency Phase Device teleportation, verified by bit-exact replay determinism;
- `sh scripts/check-repository.sh` runs all checks, formatting, clippy, and tests cleanly.

Verification:

- `sh scripts/check-repository.sh` succeeds locally;
- `cargo test --locked --workspace` passes all unit, integration, boundary, combat,
  visibility, inventory, generator, ai, targeting, and replay determinism tests;
- unit tests in `crates/drl-core/src/ai.rs` and `crates/drl-core/src/targeting.rs` verify AI decision trees,
  line-of-fire range checks, target querying, and sorting;
- integration tests in `crates/drl-core/tests/monsters_ai.rs` verify monster ranged attacks, monster movement,
  death loot drops, demon speed advantages, and replay determinism;
- integration tests in `crates/drl-core/tests/special_items.rs` verify Phase Device consumption, player relocation,
  bounds/walkability safety, and replay determinism;
- `cargo run` executes the headless demo demonstrating multi-archetype combat, phase device escape, and replay verification.

Out of scope:

- live Lua scripting integration (Milestone 3);
- MCP transport servers (Milestone 6);
- presentation/GUI rendering (Milestone 7) and audio (Milestone 8).

## Future

Proceed with Milestone 5 replay suite, scripted bots, and automated scenario frameworks.
