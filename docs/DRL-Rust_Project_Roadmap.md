---
title: "DRL-Rust Project Roadmap"
description: "Milestone-based execution roadmap and living progress checklist for the DRL-Rust reimplementation."
project: "DRL-Rust"
repository: "drl-rust"
date: 2026-08-18
status: "Living Roadmap"
---

# DRL-Rust Project Roadmap

## 1. Purpose

This document is the living execution tracker for **DRL-Rust** (`drl-rust`).

It should be revisited continuously and updated as work progresses.

This roadmap is the canonical project plan for milestone scope, ordering,
status, and exit criteria. Root `SPEC.md` may unpack one active milestone slice
into implementation-ready outcomes, verification, and non-goals, but it must
not replace or silently broaden this roadmap. `ARCHITECTURE.md` records verified
current structure and invariants; `CHANGELOG.md` records meaningful delivered
history.

Checkbox convention:

```text
[ ] Not started
[x] Complete
```

For partially complete work, keep the parent task unchecked and mark completed subtasks individually.

The roadmap is intentionally milestone-based rather than calendar-based. A milestone is complete when its **exit criteria** are satisfied, regardless of elapsed time.

---

## 2. Project Principles to Recheck at Every Milestone

- [ ] Preserve **legacy gameplay intention and modeled behavior**, not exact Pascal execution traces.
- [ ] Do not introduce backward compatibility requirements for legacy saves, mods, WAD formats, or RNG streams.
- [ ] Keep `drl-core` independent of graphics, audio, OS APIs, MCP, and filesystem concerns.
- [ ] Prefer typed domain models, ADTs, and explicit state transitions.
- [ ] Use functional-core / imperative-shell patterns where they improve clarity.
- [ ] Make gameplay randomness explicit and reproducible.
- [ ] Ensure human UI, bots, replay tools, and MCP ultimately operate through the same semantic command model.
- [ ] Keep Lua behind a narrow and intentional boundary.
- [ ] Avoid premature framework/ECS/trait abstraction.
- [ ] Add tests as behavioral rules become understood.
- [ ] Document deliberate deviations from legacy implementation.
- [ ] Keep the project runnable at every milestone.

---

# Milestone 0 — Repository Foundation and Rewrite Contract

## Goal

Establish the project structure, development standards, architectural doctrine, and legacy-behavior documentation process before substantial implementation.

## Deliverables

- Rust workspace
- baseline CI
- architecture documentation
- coding conventions
- behavioral-specification process
- initial ADR structure
- legal/provenance inventory

## Checklist

### Repository

- [x] Create GitHub repository `drl-rust`.
- [x] Add root `README.md`.
- [x] Add `LICENSE` and document licensing strategy.
- [ ] Add `CONTRIBUTING.md`.
- [ ] Add `CODE_OF_CONDUCT.md` if public contribution is expected.
- [x] Add `docs/` directory.
- [ ] Add `docs/adr/`.
- [ ] Add `docs/legacy-behavior/`.
- [ ] Add `tests/fixtures/`.
- [ ] Add `content/`.
- [x] Add `assets/`.

### Rust tooling

- [x] Initialize Cargo workspace.
- [x] Pin or document supported Rust toolchain/MSRV policy.
- [x] Configure `rustfmt`.
- [x] Configure `clippy`.
- [x] Configure `cargo test` in CI.
- [ ] Add dependency-audit/security tooling if appropriate.
- [ ] Decide dependency update policy.

### Initial crates

- [x] Create `drl-core`.
- [x] Create `drl-protocol`.
- [x] Create `drl-app`.
- [x] Create placeholder `drl-script`.
- [x] Create placeholder `drl-mcp`.
- [x] Create placeholder `drl-render`.
- [x] Create placeholder `drl-audio`.
- [x] Verify dependency direction prevents `drl-core` from depending on presentation/platform crates.

### Design doctrine

- [x] Add project principle: "Preserve the game; rewrite the machinery."
- [x] Document semantic fidelity vs operational fidelity.
- [x] Document explicit non-goals.
- [x] Document backward-compatibility opt-out.
- [x] Document Rust design priorities.
- [x] Document Clean Code principles adopted.
- [x] Document Clean Code/OO practices intentionally not adopted.
- [x] Document policy on globals/shared mutable state.
- [x] Document policy on deterministic randomness.
- [x] Document policy on side effects.

### Legacy archaeology

- [ ] Inventory major Pascal source units.
- [ ] Inventory major Lua source files.
- [ ] Map major gameplay domains to legacy files.
- [ ] Create initial `combat.md` behavioral-spec document.
- [ ] Create initial `movement.md`.
- [ ] Create initial `turn-economy.md`.
- [ ] Create initial `items.md`.
- [ ] Create initial `ai.md`.
- [ ] Create initial `generation.md`.
- [ ] Add a template for future behavior-spec notes.

### ADRs

- [ ] ADR: project architecture principles.
- [ ] ADR: no legacy backward compatibility.
- [ ] ADR: semantic command model.
- [ ] ADR: explicit deterministic RNG.
- [ ] ADR: Lua transitional strategy.
- [ ] ADR: MCP semantic interface strategy.

### Provenance

- [ ] Document legacy code license.
- [ ] Document art license.
- [ ] Inventory audio/music/font provenance.
- [ ] Document third-party asset redistribution questions.
- [ ] Decide what assets can safely enter the new repository.

## Exit Criteria

- [x] `cargo test --workspace` succeeds.
- [ ] CI runs on macOS.
- [x] Architectural boundaries are documented.
- [x] Rewrite fidelity doctrine is explicit.
- [ ] At least six major legacy behavior areas have an initial specification shell.
- [ ] No gameplay implementation has forced premature architectural compromise.

---

# Milestone 1 — Headless Simulation Kernel

## Goal

Build the smallest coherent game simulation capable of representing a map, player, turn state, commands, observations, and deterministic execution.

## Deliverables

- typed world model
- command model
- observation model
- deterministic RNG
- minimal headless turn loop
- replayable command sequence

## Checklist

### Core types

- [x] Define `EntityId`.
- [x] Define `ItemId`.
- [x] Define `LevelId`.
- [x] Define `Turn`.
- [x] Define `Position`.
- [x] Define map dimensions/types.
- [x] Define `Direction`.
- [x] Define `GameState`.
- [x] Define `Game`.
- [x] Define minimal `World`.
- [x] Define minimal `Actor`.

### Commands

- [x] Define `Command`.
- [x] Implement `Move`.
- [x] Implement `Wait`.
- [x] Define command validation.
- [x] Return typed invalid-command errors.
- [x] Separate command legality from input bindings.

### Observation

- [x] Define `Observation`.
- [x] Define player observation.
- [x] Define visible tile representation.
- [x] Define visible entity representation.
- [x] Implement player-visible observation generation.
- [x] Define explicit omniscient debug observation.

### Events

- [x] Define core `GameEvent`.
- [x] Emit movement events.
- [x] Emit turn/action events.
- [x] Add event ordering rules where observable ordering matters.
- [x] Ensure core events do not depend on renderer/audio types.

### RNG

- [x] Select deterministic RNG implementation.
- [x] Wrap it in domain-owned `GameRng`.
- [x] Prohibit ambient gameplay RNG.
- [x] Add seed initialization.
- [x] Add deterministic RNG unit tests.
- [x] Define deterministic iteration policy for simulation-relevant collections.

### Minimal map

- [x] Implement walkable/blocking cells.
- [x] Implement map bounds.
- [x] Implement occupancy.
- [x] Prevent invalid blocking overlap.
- [x] Implement basic movement legality.

### Headless executable

- [x] Add simple CLI/debug runner.
- [x] Start a fixed test map.
- [x] Accept a small sequence of commands.
- [x] Print structured observations/events.
- [x] Allow explicit seed input.

### Replay prototype

- [x] Define minimal replay structure.
- [x] Store seed.
- [x] Store initial setup.
- [x] Store command sequence.
- [x] Re-run replay deterministically.
- [x] Add replay regression test.

### Tests

- [x] Player cannot move outside map.
- [x] Player cannot move into blocked tile.
- [x] Player can wait.
- [x] Turn advances according to defined action semantics.
- [x] Same seed + same commands produce same state.
- [x] Player observation does not leak hidden world state.

## Exit Criteria

- [x] A headless game can start, move, wait, observe, and replay.
- [x] Simulation is deterministic.
- [x] `drl-core` has no renderer, audio, OS, Lua, or MCP dependency.
- [x] Command and observation models are stable enough to build the next vertical slice.

---

# Milestone 2 — Action Economy, Actors, and Minimal Combat

## Goal

Implement enough of DRL's actor/action model to support a representative combat encounter.

## Deliverables

- action costs
- actor scheduling
- HP/damage
- ranged or melee combat
- death
- combat events
- representative scenario tests

## Checklist

### Legacy specification

- [ ] Document legacy action-cost semantics.
- [ ] Document movement cost.
- [ ] Document attack cost.
- [ ] Document wait cost.
- [ ] Document speed interactions.
- [ ] Document basic hit/damage semantics.
- [ ] Identify behavior vs implementation artifacts.

### Domain types

- [x] Define `HitPoints`.
- [x] Define `DamageAmount`.
- [x] Define `DamageType`.
- [x] Define `ActionCost`.
- [x] Define `Speed`.
- [x] Define `AttackOutcome`.
- [x] Define `DeathCause`.
- [x] Define `DamageSource`.

### Turn/action system

- [x] Implement action scheduling.
- [x] Implement player action cost.
- [x] Implement monster action cost.
- [x] Validate deterministic actor ordering.
- [x] Test actor speed differences.
- [x] Test no actor receives invalid extra actions.

### Combat

- [x] Implement one melee attack.
- [x] Implement one ranged weapon.
- [x] Implement hit resolution.
- [x] Implement damage application.
- [x] Implement death.
- [ ] Implement basic knockback if required by chosen representative weapon.
- [x] Emit combat events.
- [x] Ensure combat functions do not perform presentation side effects.

### Scenario fixture

- [x] Create minimal player-vs-monster scenario.
- [x] Add deterministic command replay.
- [x] Add expected semantic outcomes.
- [x] Add statistical test if a hit probability is involved.

### Invariants

- [x] HP cannot violate defined bounds.
- [x] Dead actors cannot act.
- [x] Invalid targets are rejected.
- [x] Damage calculations are independently testable.
- [x] Actor occupancy remains valid after death/movement.

## Exit Criteria

- [x] A complete headless combat encounter is playable.
- [x] Core combat is expressed through domain types.
- [x] Combat calculations are testable independently of `Game`.
- [x] Action economy is consistent enough to expand to real DRL mechanics.

---

# Milestone 3 — Lua Runtime and Transitional Content Layer

## Goal

Integrate Lua as a constrained content/behavior layer without recreating legacy global coupling.

## Deliverables

- Lua runtime
- typed Rust/Lua boundary
- representative legacy content loaded
- Lua-driven behavior participating in simulation

## Checklist

### Runtime

- [ ] Select Lua integration crate/runtime.
- [ ] Enable compatibility with required legacy Lua semantics where useful.
- [ ] Add isolated Lua runtime crate/module.
- [ ] Implement controlled error propagation.
- [ ] Add Lua execution diagnostics.
- [ ] Add content reload strategy for development.

### Boundary design

- [ ] Define allowed Lua queries.
- [ ] Define allowed Lua commands/actions.
- [ ] Prohibit unrestricted mutable access to `Game`.
- [ ] Define typed conversion layer.
- [ ] Define stable entity references visible to Lua.
- [ ] Define lifetime/ownership rules for Lua references.
- [ ] Document API compatibility policy.

### Content migration

- [ ] Select a small representative legacy Lua subsystem.
- [ ] Port/load representative actor prototype.
- [ ] Port/load representative item.
- [ ] Port/load representative AI policy.
- [ ] Normalize Lua globals where needed.
- [ ] Remove assumptions tied to Pascal internals.

### Tests

- [ ] Lua can create/load representative content.
- [ ] Lua errors do not corrupt simulation state.
- [ ] Invalid Lua commands are rejected safely.
- [ ] Lua behavior is deterministic when supplied deterministic RNG.
- [ ] Lua cannot access hidden APIs outside its contract.

## Exit Criteria

- [ ] At least one real gameplay element is defined or controlled through Lua.
- [ ] Rust remains authoritative for world invariants.
- [ ] Lua does not receive uncontrolled mutable world access.
- [ ] The team is confident the boundary can scale to substantial existing content.

---

# Milestone 4 — Core DRL Gameplay Vertical Slice

## Goal

Create a small but recognizably DRL-like playable slice using representative weapons, enemies, items, visibility, and level flow.

## Deliverables

- visibility/FOV
- targeting
- inventory
- ammunition/reload
- several weapons
- several monsters
- item pickup/use
- one generated level
- basic level transition

## Checklist

### Visibility

- [ ] Specify legacy visibility behavior.
- [x] Implement field of view.
- [x] Implement fog-of-war/memory policy.
- [x] Ensure observations expose only legal information.
- [x] Add visibility property/scenario tests.

### Targeting

- [ ] Specify legacy targeting mechanics.
- [x] Define `Target`.
- [x] Implement target legality.
- [x] Implement line-of-fire checks.
- [x] Implement target selection metadata.
- [x] Test blocked shots.
- [x] Test out-of-range/invalid targets.

### Inventory and equipment

- [ ] Specify legacy capacity/equipment rules.
- [x] Define inventory model.
- [x] Define equipment slots.
- [x] Implement pickup.
- [x] Implement drop.
- [x] Implement equip/unequip.
- [x] Implement use.
- [x] Add capacity/invariant tests.

### Weapons and ammunition

- [x] Define weapon domain types.
- [x] Define ammo types.
- [x] Implement ammunition consumption.
- [x] Implement reload.
- [x] Implement representative pistol.
- [x] Implement representative shotgun.
- [x] Implement representative melee weapon.
- [x] Implement weapon-specific action costs.
- [x] Implement representative spread/knockback behavior.
- [x] Add statistical tests for stochastic weapon behavior.

### Monsters

- [x] Implement several representative enemy archetypes.
- [x] Implement basic melee AI.
- [x] Implement basic ranged AI.
- [ ] Integrate Lua AI where appropriate.
- [x] Preserve behavioral character rather than exact old state-machine transitions.

### Items

- [x] Implement health item.
- [x] Implement armor.
- [x] Implement ammunition pickup.
- [x] Implement one special-use item.
- [x] Add item interaction tests.


### Level

- [x] Implement stairs/exit.
- [x] Implement level transition.
- [x] Create one simple procedural generator.
- [x] Add seed-based generation tests.
- [x] Validate connectivity/reachability invariants.

### Headless gameplay

- [x] Start a complete mini-run.
- [x] Fight enemies.
- [x] Pick up equipment.
- [x] Reload/fire.
- [x] Reach exit.
- [x] Transition to next level.

## Exit Criteria

- [x] A recognizably DRL-like headless vertical slice exists.
- [x] A test player can complete the slice without renderer support.
- [x] Core observations are sufficiently rich for automated agents.
- [x] Major gameplay domains have stable architectural homes.

---

# Milestone 5 — Replay, Scenario, and Test-Agent Infrastructure


## Goal

Turn the core simulation into a serious automated testing environment.

## Deliverables

- versioned replay format
- scenario fixtures
- scripted agents
- batch-run tooling
- metrics collection

## Checklist

### Replay format

- [x] Version replay schema.
- [x] Record build/content version.
- [x] Record seed.
- [x] Record initial character/configuration.
- [x] Record command stream.
- [x] Add replay validation.
- [x] Add replay error reporting with turn/command context.
- [x] Store selected failing replays as regression tests.

### Scenario framework

- [x] Define fixture format.
- [x] Allow explicit maps.
- [x] Allow actor placement.
- [x] Allow inventory configuration.
- [x] Allow RNG seed.
- [x] Allow scripted random outcomes for focused tests if needed.
- [x] Add reusable assertion helpers.

### Scripted bots

- [x] Define bot/agent trait or policy interface only if concretely useful.
- [x] Implement random legal-action bot.
- [x] Implement simple combat bot.
- [x] Implement simple exploration bot.
- [x] Implement survival-oriented bot.
- [x] Ensure bots consume ordinary `Observation`.
- [x] Ensure bots submit ordinary `Command`.

### Batch simulation

- [x] Run many seeds headlessly.
- [x] Add configurable episode limits.
- [x] Collect outcome metrics.
- [x] Collect failure/crash artifacts.
- [x] Record pathological seeds.
- [x] Add machine-readable summary output.

### Metrics

- [x] Run completion status.
- [x] Turns survived.
- [x] Damage dealt/taken.
- [x] Death cause.
- [x] Weapon usage.
- [x] Ammo consumption.
- [x] Item usage.
- [x] Level reached.
- [x] Enemy kill distribution.

### CI

- [x] Add fast smoke-agent run.
- [x] Add replay regressions.
- [x] Separate long simulation suite from normal PR checks.
- [x] Upload failing replay artifacts.

## Exit Criteria

- [x] Hundreds or thousands of headless episodes can run without GUI.
- [x] Failures are reproducible from artifacts.
- [x] Agent code does not bypass ordinary player information boundaries.
- [x] The project can detect behavioral regressions before native UI work dominates development.

---

# Milestone 6 — MCP Game Interface

## Goal

Expose DRL-Rust as a machine-operable semantic environment for AI-driven testing and integrated test play.

## Deliverables

- MCP server
- observation tools/resources
- semantic action tools
- scenario lifecycle tools
- developer-mode controls
- integration tests

## Checklist

### MCP protocol design

- [x] Define MCP capabilities document.
- [x] Version MCP-facing schema.
- [x] Define game/session lifecycle.
- [x] Define player-visible observation representation.
- [x] Define legal-action representation.
- [x] Define semantic action submission.
- [x] Define error format.

### Core tools

- [x] Start new seeded game.
- [x] Get current observation.
- [x] List available/legal actions.
- [x] Submit move.
- [x] Submit wait.
- [x] Submit fire.
- [x] Submit reload.
- [x] Submit pickup.
- [x] Submit use.
- [x] Submit equip.
- [x] Advance/query game only through ordinary simulation.

### Test utilities

- [x] Reset game.
- [x] Load approved scenario fixture.
- [x] Save replay.
- [x] Return run summary.
- [x] Set bounded episode/turn limit.
- [x] Expose recent semantic events.

### Security boundaries

- [x] Ordinary MCP agent cannot read omniscient world state.
- [x] Developer-only omniscient mode is clearly separated.
- [x] MCP does not expose arbitrary filesystem access.
- [x] MCP does not expose arbitrary shell execution.
- [x] Validate all tool arguments.
- [x] Bound resource usage for long episodes.

### Integration tests

- [x] MCP can complete the Milestone 4 vertical slice.
- [x] MCP action results match direct simulation calls.
- [x] MCP observations match standard player observations.
- [x] Invalid actions fail cleanly.
- [x] Replay from an MCP-driven run reproduces the episode.

### Practical agent tests

- [x] Run a rule-based agent through MCP.
- [x] Run an LLM-driven exploratory test session.
- [x] Capture examples of useful agent-found issues.
- [x] Document intended role of MCP in CI vs exploratory testing.

## Exit Criteria

- [x] A remote/model-driven agent can play a complete headless episode semantically.
- [x] MCP uses the same core command model as all other clients.
- [x] Replays from MCP sessions are reproducible.
- [x] Player-information boundaries are preserved.

---

# Milestone 7 — Native macOS Rendering and Input

## Goal

Create the first real native macOS graphical frontend while preserving the headless core.

## Deliverables

- app window
- GPU renderer
- keyboard input
- sprite/tile rendering
- HUD
- targeting UI
- menus
- high-DPI support

## Checklist

### Platform foundation

- [ ] Select window/event-loop stack.
- [ ] Select GPU rendering stack.
- [ ] Create macOS app target.
- [ ] Verify Apple Silicon build.
- [ ] Verify high-DPI/Retina behavior.
- [ ] Define window/fullscreen settings.
- [ ] Use platform-appropriate app-data directories.

### Renderer

- [ ] Render tile map.
- [ ] Render player.
- [ ] Render monsters.
- [ ] Render items.
- [ ] Render fog/visibility.
- [ ] Render targeting cursor.
- [ ] Render effects/events.
- [ ] Implement pixel-art scaling policy.
- [ ] Handle window resize.

### Input

- [ ] Map keyboard events to semantic commands.
- [ ] Implement movement bindings.
- [ ] Implement fire/targeting bindings.
- [ ] Implement reload.
- [ ] Implement pickup/use.
- [ ] Implement inventory navigation.
- [ ] Keep physical key codes out of `drl-core`.

### UI

- [ ] HUD.
- [ ] Message log.
- [ ] Inventory screen.
- [ ] Character/trait screen.
- [ ] Main menu.
- [ ] Pause/options.
- [ ] Targeting interface.
- [ ] Death/game-over screen.

### Integration

- [ ] Presentation consumes core events.
- [ ] Renderer does not mutate gameplay state directly.
- [ ] UI actions become `Command`s.
- [ ] Headless tests continue to pass unchanged.

## Exit Criteria

- [ ] The Milestone 4 gameplay slice is playable in a native macOS window.
- [ ] The same run remains playable headlessly.
- [ ] No core architecture was weakened to accommodate rendering.

---

# Milestone 8 — Audio, Animation, and Game Feel

## Goal

Move from functional frontend to a convincing DRL-like player experience.

## Deliverables

- sound
- music
- animation/effect system
- feedback tuning
- UI responsiveness

## Checklist

### Audio

- [ ] Select audio backend.
- [ ] Implement semantic sound cue mapping.
- [ ] Weapon sounds.
- [ ] Monster sounds.
- [ ] Item sounds.
- [ ] UI sounds.
- [ ] Music playback.
- [ ] Music transitions.
- [ ] Volume settings.
- [ ] Mute controls.

### Animation

- [ ] Define presentation-only animation model.
- [ ] Movement animation.
- [ ] Projectile/shot effects.
- [ ] Damage feedback.
- [ ] Death effects.
- [ ] Explosion effects.
- [ ] Knockback visualization.
- [ ] Screen/camera effects where appropriate.
- [ ] Level transitions.

### Game feel

- [ ] Tune animation duration.
- [ ] Tune input responsiveness.
- [ ] Tune message timing.
- [ ] Tune targeting UX.
- [ ] Tune HUD readability.
- [ ] Validate pixel-art scaling.
- [ ] Conduct focused human playtests.

### Architectural validation

- [ ] No animation timing changes simulation outcomes.
- [ ] Audio failures do not affect gameplay.
- [ ] Presentation can be disabled for fast headless simulation.
- [ ] Semantic events remain stable enough for alternative frontends.

## Exit Criteria

- [ ] Native gameplay feels responsive and coherent.
- [ ] Presentation is recognizably aligned with DRL's identity.
- [ ] Simulation remains fully decoupled from presentation timing.

---

# Milestone 9 — Gameplay Breadth and Legacy Semantic Coverage

## Goal

Expand from the vertical slice to broad DRL gameplay coverage.

## Deliverables

- weapon families
- enemy roster
- items
- traits
- classes
- difficulty
- generated levels
- special levels
- challenges
- progression
- end-game flow

## Checklist

### Tracking framework

- [ ] Create master legacy-feature matrix.
- [ ] For every feature, track:
  - [ ] legacy source identified;
  - [ ] intended behavior documented;
  - [ ] Rust design decided;
  - [ ] implementation complete;
  - [ ] tests complete;
  - [ ] human verification complete.

### Combat breadth

- [ ] Complete damage types.
- [ ] Complete armor/resistance semantics.
- [ ] Complete major weapon categories.
- [ ] Complete reload variants.
- [ ] Complete explosions.
- [ ] Complete knockback.
- [ ] Complete environmental damage.
- [ ] Complete special attack modes.

### Monsters

- [ ] Implement major monster roster.
- [ ] Implement major AI archetypes.
- [ ] Implement special abilities.
- [ ] Implement boss behavior.
- [ ] Validate tactical identity through scenario/agent tests.

### Items and equipment

- [ ] Implement major consumables.
- [ ] Implement armor families.
- [ ] Implement boots/equipment.
- [ ] Implement special/unique items.
- [ ] Implement item generation rules.
- [ ] Validate drop/use distributions statistically.

### Player progression

- [ ] Classes.
- [ ] Experience.
- [ ] Level-up.
- [ ] Traits/perks.
- [ ] Trait prerequisites.
- [ ] Character statistics.
- [ ] Score/progression semantics.

### Difficulty

- [ ] Document difficulty parameters.
- [ ] Implement difficulty modifiers.
- [ ] Add batch-agent comparisons by difficulty.
- [ ] Verify expected ordering of challenge.

### Level generation

- [ ] Implement core generators.
- [ ] Validate connectivity.
- [ ] Validate item/enemy placement.
- [ ] Validate structural distributions.
- [ ] Add pathological-seed detection.
- [ ] Port/replace selected Lua generation logic.

### Special levels/challenges

- [ ] Inventory special levels.
- [ ] Document intended semantics.
- [ ] Implement prioritized special levels.
- [ ] Implement challenge modes.
- [ ] Add representative scenario tests.

### End-to-end progression

- [ ] Start new game.
- [ ] Play through multi-level run.
- [ ] Progress character.
- [ ] Reach late game.
- [ ] Implement victory/ending.
- [ ] Implement death/mortem summary.

## Exit Criteria

- [ ] The majority of canonical DRL gameplay systems are represented.
- [ ] Remaining gaps are explicitly listed.
- [ ] Automated agents can play long runs.
- [ ] Human testers recognize the game as behaviorally faithful.

---

# Milestone 10 — New Save System and Stable Internal Formats

## Goal

Introduce durable DRL-Rust persistence after major domain structures have stabilized.

## Deliverables

- versioned save format
- save/load UX
- migration policy
- stable replay schema
- content-version checks

## Checklist

### Save schema

- [ ] Choose serialization format.
- [ ] Add explicit save version.
- [ ] Add game/content version.
- [ ] Define serialization DTOs separate from internal representation where appropriate.
- [ ] Avoid serializing raw memory layout.
- [ ] Avoid leaking unstable internal IDs where possible.

### Save/load

- [ ] Manual save.
- [ ] Automatic save policy.
- [ ] Load.
- [ ] Corrupt-save error handling.
- [ ] Version mismatch handling.
- [ ] Atomic write strategy.
- [ ] Backup/recovery behavior.

### Migration

- [ ] Define compatibility policy across DRL-Rust releases.
- [ ] Implement at least one test migration fixture.
- [ ] Document when save migrations may be dropped.

### Replay stability

- [ ] Version replay schema independently if appropriate.
- [ ] Validate old replay rejection/migration behavior.
- [ ] Preserve useful regression replays.

## Exit Criteria

- [ ] Saves survive ordinary application restarts and upgrades within the defined policy.
- [ ] Persistence format is decoupled from Rust memory layout.
- [ ] Legacy Pascal saves remain intentionally unsupported.

---

# Milestone 11 — Balance, Regression, and Large-Scale Automated Playtesting

## Goal

Use the architecture's testing advantages to characterize gameplay and identify regressions systematically.

## Deliverables

- batch simulation suite
- balance dashboards/reports
- regression thresholds
- MCP exploratory workflows
- curated pathological-seed corpus

## Checklist

### Batch experiment framework

- [ ] Run thousands of seeds per build.
- [ ] Parameterize difficulty.
- [ ] Parameterize player archetype.
- [ ] Parameterize bot policy.
- [ ] Persist aggregate metrics.
- [ ] Persist anomalous replays.

### Metrics

- [ ] Win rate.
- [ ] Survival curve.
- [ ] Level reached.
- [ ] Death cause.
- [ ] Weapon utilization.
- [ ] Ammo scarcity.
- [ ] Damage source composition.
- [ ] Item consumption.
- [ ] Trait choice.
- [ ] Boss success rates.
- [ ] Generation failures.

### Regression policy

- [ ] Define metrics that should remain approximately stable.
- [ ] Define acceptable stochastic tolerance.
- [ ] Avoid treating every distribution change as a bug.
- [ ] Require investigation for unexplained large shifts.
- [ ] Link significant shifts to code/behavior changes.

### MCP workflows

- [ ] Create standard exploratory-test prompts/tasks.
- [ ] Test new features with MCP agents before merge where useful.
- [ ] Capture agent reasoning separately from authoritative test results.
- [ ] Convert discovered bugs into deterministic scenarios/replays.

### Human validation

- [ ] Run structured human playtests.
- [ ] Compare human feedback with automated metrics.
- [ ] Identify mechanics where automated agents are poor proxies.
- [ ] Tune based on game feel as well as statistics.

## Exit Criteria

- [ ] Automated testing catches meaningful behavioral regressions.
- [ ] MCP testing has demonstrated practical value beyond scripted bots.
- [ ] Balance changes can be evaluated with reproducible evidence.

---

# Milestone 12 — macOS Productization

## Goal

Turn the developer build into a polished distributable macOS application.

## Deliverables

- app bundle
- icon/metadata
- signing
- notarization
- settings
- crash diagnostics
- release build pipeline

## Checklist

### Application bundle

- [ ] Bundle resources correctly.
- [ ] Set application identifier.
- [ ] Add app icon.
- [ ] Add version metadata.
- [ ] Validate read-only bundled resource assumptions.
- [ ] Validate writable app-data paths.

### Configuration

- [ ] Video settings.
- [ ] Audio settings.
- [ ] Key bindings.
- [ ] Accessibility-related options where practical.
- [ ] Reset settings.
- [ ] Safe defaults.

### macOS behavior

- [ ] Window restoration policy.
- [ ] Fullscreen behavior.
- [ ] Retina rendering.
- [ ] Keyboard focus handling.
- [ ] App quit handling.
- [ ] Crash/recovery behavior.

### Distribution

- [ ] Release profile optimization.
- [ ] Code signing.
- [ ] Hardened runtime if required.
- [ ] Notarization.
- [ ] Stapling.
- [ ] Test clean-machine installation.
- [ ] Test Gatekeeper behavior.
- [ ] Generate distributable artifact.

### CI/release automation

- [ ] Build signed release in controlled CI/release workflow.
- [ ] Generate checksums.
- [ ] Generate release notes.
- [ ] Archive symbols/debug artifacts as appropriate.

## Exit Criteria

- [ ] A non-developer macOS user can install and run DRL-Rust normally.
- [ ] Application data is stored in appropriate macOS locations.
- [ ] Release artifacts are reproducible and documented.

---

# Milestone 13 — Release Candidate and 1.0 Readiness

## Goal

Freeze major architecture, close critical gameplay gaps, validate stability, and prepare the first major public release.

## Deliverables

- feature-complete candidate
- stability validation
- documentation
- known-issues list
- 1.0 release

## Checklist

### Gameplay completeness

- [ ] Review legacy-feature matrix.
- [ ] Classify every remaining gap:
  - [ ] must fix before 1.0;
  - [ ] intentional difference;
  - [ ] post-1.0;
  - [ ] obsolete legacy artifact.
- [ ] Resolve all must-fix gameplay gaps.

### Stability

- [ ] Long human sessions without crashes.
- [ ] Long scripted-bot sessions without invariant failures.
- [ ] Large MCP sessions without protocol corruption.
- [ ] Save/load stress testing.
- [ ] Replay stress testing.
- [ ] Lua error-path testing.
- [ ] Renderer/device-loss handling as applicable.

### Performance

- [ ] Profile startup.
- [ ] Profile rendering.
- [ ] Profile FOV/pathfinding.
- [ ] Profile Lua-heavy scenarios.
- [ ] Profile batch simulation.
- [ ] Fix only evidence-based bottlenecks.

### Documentation

- [ ] User README.
- [ ] Build instructions.
- [ ] Developer architecture overview.
- [ ] MCP usage documentation.
- [ ] Replay documentation.
- [ ] Save compatibility policy.
- [ ] Asset/license attribution.
- [ ] Contribution workflow.
- [ ] Known intentional differences from legacy DRL.

### Release validation

- [ ] Fresh macOS installation.
- [ ] New game.
- [ ] Multi-level run.
- [ ] Save/load.
- [ ] Death.
- [ ] Victory.
- [ ] Audio.
- [ ] Fullscreen/windowed.
- [ ] Settings persistence.
- [ ] MCP smoke test.
- [ ] Replay smoke test.
- [ ] Release artifact notarization validation.

## Exit Criteria

- [ ] No known critical crashes.
- [ ] No known major invariant violations.
- [ ] Core DRL identity is preserved.
- [ ] Native macOS experience is polished.
- [ ] Automated and MCP testing are functioning.
- [ ] Documentation is sufficient for users and contributors.
- [ ] DRL-Rust 1.0 is released.

---

# Post-1.0 Candidate Directions

These should **not** influence the initial architecture unless concrete requirements emerge.

- [ ] Linux frontend/distribution.
- [ ] Windows frontend/distribution.
- [ ] New mod/content API designed specifically for DRL-Rust.
- [ ] Stable public Lua content SDK.
- [ ] New non-Lua content format.
- [ ] Headless tournament/agent environment.
- [ ] Rich telemetry and comparative balance tooling.
- [ ] Spectator/replay viewer.
- [ ] Web-based replay analysis.
- [ ] Accessibility expansion.
- [ ] Controller-focused UI.
- [ ] Alternative renderer/frontends.
- [ ] Formal benchmark suite for AI agents.
- [ ] New gameplay/content beyond canonical DRL.

---

# Cross-Milestone Definition of Done

A feature should not be considered complete merely because it works in one playthrough.

For gameplay features, the preferred completion checklist is:

- [ ] Intended legacy behavior identified.
- [ ] Implementation artifact vs canonical behavior distinguished.
- [ ] Rust domain model designed.
- [ ] Invariants documented.
- [ ] Unit/scenario/property tests added as appropriate.
- [ ] Randomness is explicit.
- [ ] Headless execution works.
- [ ] Observation representation is correct.
- [ ] Replay behavior is reproducible.
- [ ] MCP access works if the feature is player-operable.
- [ ] Native presentation works if applicable.
- [ ] Human playtest performed when game feel matters.
- [ ] Documentation updated.
- [ ] Deliberate differences recorded.

---

# Suggested Progress Summary

Update this table as milestones advance.

| Milestone | Status | Notes |
|---|---|---|
| M0 — Repository Foundation | In progress | Staged delivery/test-play harness, SDD, formatting, and CI foundations added; remote CI and remaining M0 deliverables are pending. |
| M1 — Headless Simulation Kernel | Not started | |
| M2 — Action Economy + Combat | Not started | |
| M3 — Lua Runtime | Not started | |
| M4 — Core Gameplay Vertical Slice | Not started | |
| M5 — Replay + Test Agents | Not started | |
| M6 — MCP Interface | Not started | |
| M7 — Native macOS Frontend | Not started | |
| M8 — Audio + Game Feel | Not started | |
| M9 — Gameplay Breadth | Not started | |
| M10 — Save System | Not started | |
| M11 — Automated Playtesting | Not started | |
| M12 — macOS Productization | Not started | |
| M13 — Release / 1.0 | Not started | |

---

# Immediate Starting Checklist

If beginning implementation now, the first concrete tasks are:

- [x] Create `drl-rust`.
- [x] Add this proposal and roadmap to `docs/`.
- [x] Initialize Cargo workspace.
- [x] Create `drl-core`, `drl-protocol`, and `drl-app`.
- [x] Add macOS CI.
- [ ] Write first ADRs.
- [ ] Create the legacy behavior-spec template.
- [ ] Document movement semantics.
- [ ] Document turn/action-cost semantics.
- [ ] Document minimal combat semantics.
- [ ] Define `EntityId`, `Position`, `Turn`, and `GameState`.
- [ ] Define initial `Command`.
- [ ] Define initial `Observation`.
- [ ] Define initial `GameEvent`.
- [ ] Select and wrap deterministic RNG.
- [ ] Build the first headless fixed-map movement test.

At that point, DRL-Rust will have crossed from proposal into an executable architecture.
