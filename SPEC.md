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

## Present

### Milestone 2: Action Economy, Actor Scheduling, and Minimal Combat (Melee & Ranged)

Status: Active

This slice implements DRL's action economy, energy scheduling, actor combat stats,
damage calculation, melee/ranged combat resolution, death handling, and scenario replay.

Observable outcomes:

- `drl-protocol` defines domain types for combat and action economy: `HitPoints` (current,
  max, damage/heal with clamping), `Speed` (relative percentage), `ActionCost` (time units,
  standard = 1000), `DamageAmount`, `DamageType` (Physical, Plasma, Acid, Fire),
  `DamageSource` (`Actor`, `Environment`), `DeathCause` (`MeleeAttack`, `RangedAttack`,
  `Environment`), and `AttackOutcome` (`Hit`, `Miss`, `Blocked`);
- `drl-protocol` defines combat commands: `Command::AttackMelee(Direction)`,
  `Command::AttackRanged(Position)`, and bump-to-attack semantics for `Command::Move`;
- `drl-protocol` defines combat events: `GameEvent::AttackResolved`, `GameEvent::DamageApplied`,
  `GameEvent::ActorDied`, and `GameEvent::TurnEnded`;
- `drl-protocol` extends `ActorView` to expose health status (`hp`), living state (`is_alive`),
  and speed for player and omniscient observations;
- `drl-core` implements an isolated, pure `combat` module for deterministic hit chance
  calculations and damage rolls, testable independently of `Game`;
- `drl-core` implements an energy-based action scheduler (`scheduler` module) executing actor
  turns according to `Speed` and `ActionCost`, breaking ties deterministically by `EntityId`;
- `drl-core` implements actor health tracking, damage application, death transitions, and
  dead actor occupancy cleanup (dead actors no longer block movement);
- `drl-core` implements monster AI turns (approach player or melee attack when adjacent)
  when scheduled between player actions;
- `drl-core` supports deterministic recording and playback of combat encounters via `ReplayEngine`;
- `drl-app` demonstrates a multi-turn combat encounter with ranged and melee attacks,
  verifying bit-for-bit replay determinism;
- `sh scripts/check-repository.sh` runs formatting, clippy, harness, and all unit/integration
  tests across the workspace without warnings.

Verification:

- `sh scripts/check-repository.sh` succeeds locally;
- `cargo test --locked --workspace` passes all unit, integration, boundary, combat,
  scheduling, and replay determinism tests;
- `cargo run` executes the headless combat scenario demonstrating ranged attacks, melee
  finishing blows, monster actions, death transitions, and determinism verification;
- tests verify that combat calculations and energy scheduling are deterministic and
  independent of external presentation concerns.

Out of scope:

- inventory management, equipment slots, and item pickups (Milestone 4);
- procedural level generation algorithms (Milestone 4);
- live Lua scripting integration (Milestone 3);
- MCP transport servers (Milestone 6);
- presentation/GUI rendering and audio (Milestone 7 & 8).

## Future

Proceed with Lua runtime boundary and transitional content loading in Milestone 3.
