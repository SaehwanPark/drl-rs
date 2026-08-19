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
- Milestone 4 established representative enemy archetypes (`FormerHuman`, `FormerSergeant`, `Imp`, `Demon`),
  tactical monster AI with ranged attacks and line-of-fire checks, monster death loot drops,
  target validation and auto-targeting, and special-use Phase Device teleportation.

## Present

### Milestone 4: Weapon Knockback & Spread Mechanics, Statistical Weapon Tests, and Milestone 4 Completion

Status: Active

This slice implements weapon kinetic knockback mechanics (specifically pump-action Shotgun and Former Sergeant
shotgun attacks pushing surviving targets away along the firing ray), bounds and obstacle collision checks for knockback,
statistical verification suites for stochastic weapon behaviors (accuracy scaling, distance penalties, uniform damage distributions),
and final exit criteria verification for Milestone 4.

Observable outcomes:

- `drl-protocol` defines knockback events and view representations:
  - `GameEvent::ActorKnockedBack { entity_id: EntityId, from: Position, to: Position }`;
  - `ItemView` includes `knockback: Option<u32>` indicating kinetic push power;
- `drl-core` implements weapon knockback properties and execution:
  - `WeaponProperties` contains `knockback: u32` (`Item::shotgun` has knockback 1, `Item::pistol` and `Item::combat_knife` have knockback 0);
  - `Actor` exposes `knockback(&self) -> u32` resolving equipped weapon properties or innate actor knockback (e.g. `Actor::former_sergeant`);
  - `Game::apply_knockback` computes normalized push direction vector from attacker to defender and relocates the defender
    along the vector if the destination tile is within map bounds, walkable, and unoccupied by any living actor;
  - If a destination tile is blocked by terrain, map boundary, or another actor, knockback safely halts with no clipping;
  - If the player character is knocked back, field of view (FOV) and fog-of-war exploration memory are updated immediately;
  - Lethal blows do not displace targets, dropping loot corpses at the exact point of fatality;
- `drl-core` provides statistical validation in `crates/drl-core/tests/stochastic_combat.rs`:
  - Statistical tests verify accuracy scaling and distance penalties over large sample distributions ($N \ge 1,000$);
  - Statistical tests verify uniform damage rolls and strict min/max bound enforcement for Pistol, Shotgun, Combat Knife, and monster attacks;
  - Integration tests verify Shotgun knockback displacement, obstacle blocking, monster blocking, and bit-exact replay determinism;
- `drl-app` displays knockback event telemetry during headless demo combat;
- All Milestone 4 roadmap items and exit criteria are satisfied;
- `sh scripts/check-repository.sh` runs all checks, formatting, clippy, and tests cleanly.

Verification:

- `sh scripts/check-repository.sh` succeeds locally;
- `cargo test --locked --workspace` passes all unit, integration, boundary, combat,
  visibility, inventory, generator, ai, targeting, special items, and stochastic combat tests;
- integration tests in `crates/drl-core/tests/stochastic_combat.rs` verify statistical hit distributions,
  damage roll bounds, knockback collision invariants, and multi-turn replay determinism;
- `cargo run` executes the headless demo demonstrating Shotgun knockback and bit-exact replay verification.

Out of scope:

- live Lua scripting integration (Milestone 3);
- MCP transport servers (Milestone 6);
- presentation/GUI rendering (Milestone 7) and audio (Milestone 8).

## Future

Proceed with Milestone 5 replay suite, scripted bots, and automated scenario frameworks.
