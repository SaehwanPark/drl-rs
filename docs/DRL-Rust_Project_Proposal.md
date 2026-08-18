---
title: "DRL-Rust Project Proposal"
description: "Comprehensive project proposal for a modern Rust reimplementation of Doom the Roguelike (DRL), with macOS-native delivery, semantic fidelity to the legacy implementation, and first-class automated/AI playtesting."
project: "DRL-Rust"
repository: "drl-rust"
date: 2026-08-18
status: "Draft / Living Design Document"
---

# DRL-Rust Project Proposal

## 1. Executive Summary

**DRL-Rust** (`drl-rust`) is a ground-up Rust reimplementation of Doom the Roguelike (DRL), initially targeting a high-quality native macOS experience while keeping the core simulation portable and platform-independent.

The project is **not** intended to be a line-by-line Pascal port and is **not** constrained by backward compatibility with legacy saves, mods, binary formats, WAD internals, or execution traces. Instead, the existing Pascal and Lua implementation serves as the **canonical source of intended game rules, interactions, behavioral character, and design semantics**.

The central design principle is:

> **Preserve the game; rewrite the machinery.**

DRL-Rust should retain the identity and modeled behavior of DRL while redesigning its internals around modern Rust practices:

- algebraic data types (ADTs);
- strong type safety and domain-aware types;
- explicit state transitions;
- controlled mutation and functional-core principles;
- deterministic and reproducible simulation;
- clean architectural boundaries;
- testability as a first-class design requirement;
- modern native rendering and input on macOS;
- Lua as a pragmatic transitional content/scripting layer;
- MCP as a first-class interface for simulated players, AI-assisted playtesting, and integrated testing.

The result should be both a playable game and a substantially more inspectable, testable, maintainable, and extensible simulation platform than the legacy implementation.

---

## 2. Background and Motivation

The legacy DRL codebase contains a mature and feature-rich game developed over many years. Its implementation reflects the constraints and conventions of its original Pascal/Valkyrie/Lua architecture.

The existing codebase provides valuable canonical information about:

- combat rules;
- action economy;
- player progression;
- weapons and items;
- enemy behavior;
- damage and resistance systems;
- level generation;
- targeting;
- inventory and equipment;
- traits and challenges;
- special interactions;
- difficulty behavior;
- AI personality and tactical patterns;
- user-facing game feel.

At the same time, the implementation contains architectural characteristics that should not be reproduced mechanically:

- broad mutable global state;
- cross-layer coupling between game logic, rendering, audio, and scripting;
- large responsibility-heavy classes;
- runtime flags and byte/string-oriented domain representation;
- legacy serialization assumptions;
- platform-specific build and packaging logic;
- direct presentation side effects from domain logic;
- implementation details driven by Pascal inheritance and legacy engine abstractions.

Rust provides a natural opportunity to model the domain more explicitly and safely while retaining the game's intended behavior.

---

## 3. Project Vision

DRL-Rust should become:

1. **A faithful modern interpretation of DRL**
   - preserving the intended mechanics, interactions, probabilities, and behavioral character of the legacy game;

2. **A native-feeling macOS game**
   - with responsive input, appropriate application data locations, modern rendering, audio, packaging, signing, and distribution;

3. **A deterministic simulation**
   - where seeds and command streams can reproduce gameplay scenarios for debugging and testing;

4. **A strongly typed domain model**
   - where invalid states are difficult or impossible to represent;

5. **A testable game platform**
   - supporting unit tests, property tests, scenario tests, scripted bots, MCP-based agents, replay-based regression tests, and full frontend integration tests;

6. **A maintainable Rust codebase**
   - organized around domain concepts rather than translated Pascal class boundaries;

7. **A platform for future experimentation**
   - including AI-driven test players, automated balance studies, large-scale seeded simulation, and new developer tooling.

---

## 4. Core Project Doctrine

### 4.1 Canonical truth

The legacy Pascal and Lua code is the **canonical truth of design intention and modeled behavior**.

It is **not** the canonical truth of:

- call order;
- RNG draw order;
- object lifetime;
- mutation sequence;
- binary layout;
- exact pathfinding implementation;
- exact intermediate state;
- exact rendering frame sequence;
- turn-by-turn execution identity.

### 4.2 Semantic fidelity over operational fidelity

DRL-Rust should preserve:

- what a mechanic means;
- what constraints it enforces;
- what outcomes it tends to produce;
- what probabilities matter;
- how systems interact;
- what tactical behaviors players experience.

It does not need to preserve the legacy execution path that produces those results.

For stochastic mechanics, the target is often **statistical or behavioral equivalence**, not identical random outcomes.

### 4.3 Rewrite principles

When porting a subsystem:

```text
Legacy Pascal/Lua implementation
            ↓
Infer intended domain behavior
            ↓
Write explicit behavioral specification
            ↓
Design idiomatic Rust representation
            ↓
Write tests for the specification
            ↓
Implement and validate
```

The goal is to avoid translating implementation accidents into the new architecture.

---

## 5. Scope

### 5.1 In scope

The project should ultimately include:

- core turn-based game simulation;
- player movement and actions;
- weapons, ammunition, firing, reloading, melee, and special attacks;
- damage, armor, resistances, knockback, explosions, and environmental hazards;
- monsters and AI behavior;
- item generation and item interactions;
- inventory and equipment;
- player progression, traits, perks, and class behavior;
- difficulty behavior;
- level generation and special levels;
- visibility and targeting;
- status effects and temporary state;
- game messages and event presentation;
- sound and music;
- graphical native frontend;
- settings and application data;
- save/load using a new versioned format;
- replay/command-log support;
- Lua scripting during early and intermediate phases;
- MCP interface for machine-driven play;
- deterministic simulation and testing infrastructure;
- macOS packaging and distribution;
- eventual portability to other desktop platforms where practical.

### 5.2 Explicit non-goals

Initial and medium-term development should **not** be constrained by:

- legacy save compatibility;
- legacy mod compatibility;
- old binary formats;
- old WAD encryption or exact packaging;
- old Valkyrie interfaces;
- legacy Steam Workshop support;
- exact legacy RNG sequences;
- exact replay compatibility;
- exact old graphical implementation;
- source-level Pascal class correspondence;
- one-to-one Lua API compatibility beyond what is useful for staged migration.

These exclusions are deliberate architecture simplifications, not omissions to be corrected later unless requirements change.

---

## 6. Fidelity Model

DRL-Rust should classify legacy behavior into three categories.

### 6.1 Preserve faithfully

These are part of the game identity:

- action costs and turn economy;
- combat semantics;
- damage types and resistances;
- weapon roles and firing behavior;
- reload semantics;
- enemy tactical identities;
- item effects;
- trait/perk effects;
- difficulty rules;
- progression;
- meaningful probabilities;
- important edge-case interactions;
- level-generation character;
- special-level behavior;
- challenge-mode behavior;
- information visibility and targeting rules.

### 6.2 Reinterpret carefully

These may be reimplemented using new algorithms while preserving observable character:

- pathfinding;
- procedural generation internals;
- monster tactical decision algorithms;
- animation scheduling;
- targeting implementation;
- resource loading;
- sound scheduling;
- UI navigation;
- exact timing of non-gameplay presentation events.

### 6.3 Discard as implementation artifacts

These should generally not survive unless independently justified:

- mutable globals;
- Pascal inheritance structure;
- raw stream serialization;
- byte-level command identifiers;
- stringly typed state;
- direct renderer/audio calls from combat logic;
- old build-system conditionals;
- WAD encryption mechanics;
- legacy application path assumptions;
- platform-specific launcher machinery.

---

## 7. Architectural Goals

The architecture should optimize for the following quality attributes.

### 7.1 Correctness

The domain model should encode important invariants directly in types and APIs.

### 7.2 Determinism

A simulation should be reproducible when given the same:

- game version;
- seed;
- initial configuration;
- content version;
- command stream.

### 7.3 Testability

Core mechanics should be testable without rendering, audio, filesystem access, or a native window.

### 7.4 Explicit dependencies

Subsystems should receive what they need rather than reading ambient global state.

### 7.5 Isolation of side effects

Filesystem, rendering, audio, clock, process, and operating-system behavior should remain at architectural boundaries.

### 7.6 Extensibility without framework overengineering

The codebase should be modular, but should not introduce abstractions merely to anticipate hypothetical future requirements.

### 7.7 Observability

The simulation should expose enough structured state and events to support:

- debugging;
- replays;
- MCP agents;
- automated test players;
- balance analysis;
- regression diagnosis.

---

## 8. High-Level Architecture

A preferred conceptual architecture is:

```text
                    ┌───────────────────────┐
                    │      macOS UI         │
                    │ input/render/audio    │
                    └──────────┬────────────┘
                               │
                           Command
                               │
                               v
┌───────────────┐       ┌───────────────┐       ┌────────────────┐
│ MCP / Agents  │──────>│   drl-core    │<─────>│  drl-script    │
└───────────────┘       │  simulation   │       │ Lua/content API│
                        └──────┬────────┘       └────────────────┘
                               │
                        Events/Observation
                               │
             ┌─────────────────┼─────────────────┐
             │                 │                 │
             v                 v                 v
        Presentation       Test harness      Replay tooling
```

The **simulation core** is the semantic center of the project.

Human input, MCP input, scripted test players, and replay systems should all interact with the game through the same command-oriented API.

---

## 9. Proposed Cargo Workspace

A likely workspace organization:

```text
drl-rust/
├── Cargo.toml
├── crates/
│   ├── drl-core/
│   │   ├── world/
│   │   ├── actor/
│   │   ├── item/
│   │   ├── combat/
│   │   ├── actions/
│   │   ├── progression/
│   │   ├── visibility/
│   │   ├── pathfinding/
│   │   ├── generation/
│   │   ├── rng/
│   │   └── simulation/
│   │
│   ├── drl-protocol/
│   │   ├── command/
│   │   ├── observation/
│   │   ├── event/
│   │   ├── replay/
│   │   └── versioning/
│   │
│   ├── drl-script/
│   │   ├── runtime/
│   │   ├── api/
│   │   ├── compatibility/
│   │   └── content/
│   │
│   ├── drl-assets/
│   │
│   ├── drl-render/
│   │
│   ├── drl-audio/
│   │
│   ├── drl-mcp/
│   │
│   ├── drl-tools/
│   │
│   └── drl-app/
│
├── content/
│   ├── core/
│   └── drl/
│
├── tests/
│   ├── scenarios/
│   ├── properties/
│   ├── replays/
│   └── fixtures/
│
└── docs/
```

This is a target shape rather than an immutable commitment. Crates should be split when there is a meaningful dependency boundary, not merely for organizational aesthetics.

---

## 10. Core Domain Model

### 10.1 Stable identities

Avoid raw integers throughout the domain.

```rust
pub struct EntityId(u32);
pub struct ItemId(u32);
pub struct LevelId(u16);
pub struct Turn(u64);
```

### 10.2 Domain-specific values

Examples:

```rust
pub struct HitPoints(u16);
pub struct ArmorValue(u16);
pub struct DamageAmount(u16);
pub struct ActionCost(u32);
pub struct Accuracy(i16);
pub struct Speed(u16);
```

These types should enforce invariants where appropriate.

### 10.3 Algebraic data types

Use enums to represent meaningful alternatives:

```rust
pub enum DamageType {
  Bullet,
  Melee,
  Fire,
  Plasma,
  Acid,
  Explosive,
  Other,
}

pub enum EquipmentSlot {
  Weapon,
  Armor,
  Boots,
}

pub enum Target {
  Entity(EntityId),
  Position(Position),
}
```

The specific cases should be derived from actual DRL behavior rather than invented prematurely.

### 10.4 Invalid states should be difficult to represent

Prefer:

```rust
enum GameState {
  MainMenu(MainMenuState),
  CharacterCreation(CharacterCreationState),
  Playing(PlayingState),
  LevelTransition(LevelTransitionState),
  GameOver(GameOverState),
}
```

over:

```text
state flag
+ optional player
+ optional level
+ optional transition
+ optional menu
```

---

## 11. Command-Oriented Simulation

The game should expose player intent as structured commands.

Example:

```rust
pub enum Command {
  Move(Direction),
  Wait,
  Fire {
    weapon: WeaponSlot,
    target: Target,
    mode: FireMode,
  },
  Reload {
    weapon: WeaponSlot,
    mode: ReloadMode,
  },
  Pickup {
    item: ItemId,
  },
  Use {
    item: ItemId,
    target: Option<Target>,
  },
  Equip {
    item: ItemId,
    slot: EquipmentSlot,
  },
}
```

A central simulation API might resemble:

```rust
impl Game {
  pub fn apply(&mut self, command: Command) -> ActionResult;
  pub fn observe(&self, observer: Observer) -> Observation;
}
```

Commands should be semantic. Frontends translate physical inputs into commands.

---

## 12. Events and Effects

Simulation logic should avoid direct presentation side effects.

Instead of a combat function:

- playing a sound;
- writing a message;
- triggering camera shake;
- calling a renderer;
- starting an animation;

it should return structured results.

For example:

```rust
pub enum GameEvent {
  EntityMoved { entity: EntityId, from: Position, to: Position },
  DamageApplied { source: DamageSource, target: EntityId, amount: DamageAmount },
  EntityDied { entity: EntityId, cause: DeathCause },
  ItemPickedUp { entity: EntityId, item: ItemId },
  SoundRequested(SoundCue),
  Message(GameMessage),
}
```

The presentation layer interprets presentation-oriented events.

This preserves a clear separation:

```text
Input
  ↓
Simulation
  ↓
Events / Effects
  ↓
Presentation
```

---

## 13. Functional-Core / Imperative-Shell Strategy

DRL-Rust should use functional programming ideas where they improve clarity and correctness.

Good candidates for pure or nearly pure functions include:

- hit resolution;
- damage calculation;
- armor/resistance calculation;
- knockback calculation;
- action-cost calculation;
- trait modification;
- item-stat derivation;
- probability decisions when supplied an explicit random roll;
- visibility calculations;
- targeting legality;
- AI utility scoring.

Example:

```rust
fn resolve_attack(
  attacker: &Combatant,
  defender: &Combatant,
  weapon: &Weapon,
  roll: AttackRoll,
) -> AttackOutcome
```

The outer simulation remains intentionally stateful:

```rust
fn apply_command(game: &mut Game, command: Command)
```

The goal is **controlled mutation with pure inner calculations**, not total immutability.

---

## 14. Randomness and Determinism

Randomness must be an explicit simulation dependency from the beginning.

Avoid untracked global or thread-local randomness inside gameplay logic.

Conceptually:

```rust
pub struct GameRng {
  // deterministic internal state
}
```

All gameplay randomness should flow through this abstraction.

Important goals:

- deterministic seeded runs;
- reproducible bugs;
- replayable command streams;
- test injection of fixed or scripted random outcomes;
- statistical validation of stochastic mechanics;
- large-scale automated balance testing.

Exact legacy RNG sequences are not a requirement.

---

## 15. Observation Model

The game should not expose the entire internal world to frontends or AI agents.

Define an explicit observation model representing information available to a player.

```rust
pub struct Observation {
  pub turn: Turn,
  pub player: PlayerObservation,
  pub visible_tiles: Vec<ObservedTile>,
  pub visible_entities: Vec<ObservedEntity>,
  pub visible_items: Vec<ObservedItem>,
  pub recent_messages: Vec<GameMessage>,
  pub available_actions: Vec<ActionDescriptor>,
}
```

This provides:

- anti-cheating boundaries for MCP agents;
- a stable interface for UI;
- explicit fog-of-war semantics;
- testable information visibility;
- cleaner replay and debugging tools.

Special observers may be allowed:

```rust
pub enum Observer {
  Player(PlayerId),
  OmniscientDebug,
  Replay,
}
```

---

## 16. Entity and World Representation

The project should not automatically adopt a full ECS merely because the implementation is in Rust.

DRL has relatively modest turn-based entity counts and strong domain-specific behavior. A simpler design may be preferable:

```text
World owns entities
  ↓
stable typed IDs
  ↓
systems operate over explicit domain state
```

Potential internal storage options:

- generational arena;
- slot map;
- index map with versioned handles;
- custom stable-identifier store.

The selected representation should prioritize:

- stable references;
- safety against stale IDs;
- easy serialization;
- deterministic iteration where required;
- straightforward debugging.

The architecture should be validated against actual gameplay needs before introducing ECS complexity.

---

## 17. Lua Strategy

### 17.1 Purpose

Lua should be treated as a pragmatic migration and content-definition layer.

It allows the project to preserve substantial existing behavioral knowledge while the Rust engine is rebuilt.

Likely early Lua responsibilities:

- entity prototypes;
- item definitions;
- traits/perks;
- AI policies;
- level definitions;
- procedural-generation rules;
- challenges;
- special-level logic;
- other content-heavy configuration.

### 17.2 Boundary design

Lua should interact through a narrow, explicit API rather than receiving arbitrary mutable access to internal Rust structures.

Preferred direction:

```text
Lua policy/content
      ↓
typed compatibility API
      ↓
Rust domain commands / queries
```

Avoid:

```text
Lua
 ↓
unrestricted mutable Game internals
```

### 17.3 Migration path

A possible evolution:

```text
Phase 1: Rust core + substantial legacy Lua
Phase 2: Rust core + cleaned/normalized Lua
Phase 3: Rust core + selected Rust implementations of mature subsystems
Phase 4: Lua retained primarily as a content DSL, if still useful
```

Full Lua elimination is not a required project objective.

---

## 18. MCP as a First-Class Interface

MCP should be designed as a semantic game interface, not GUI automation.

### 18.1 Core principle

AI test players should operate through the same conceptual command model as human players.

Prefer:

```text
observe_game
move
wait
fire
reload
pickup
use
equip
choose_trait
```

over:

```text
press_key
move_mouse
read_pixels
```

GUI-level automation can exist separately for end-to-end testing.

### 18.2 MCP capabilities

Potential MCP tools/resources:

- start a seeded game;
- inspect player-visible state;
- list legal/available actions;
- submit a command;
- inspect recent events;
- save a replay artifact;
- reset a scenario;
- load a test fixture;
- run a bounded episode;
- request omniscient debug state in explicit developer mode.

### 18.3 Fairness boundary

Ordinary test agents should only receive `Observation`, not internal `World` state.

This allows meaningful AI-driven playtesting without accidental omniscience.

---

## 19. Testing Strategy

Testing is a first-class architecture concern.

### 19.1 Unit tests

For pure domain calculations:

- damage;
- action cost;
- resistance;
- inventory constraints;
- trait modifiers;
- targeting;
- visibility;
- progression.

### 19.2 Property-based tests

Examples:

- damage never becomes negative;
- resistance does not increase raw damage unless explicitly designed to;
- inventory capacity constraints are preserved;
- dead actors cannot perform ordinary actions;
- two blocking living entities do not occupy the same tile;
- ammunition never becomes negative;
- legal movement does not leave map bounds.

### 19.3 Scenario tests

Represent small game situations as fixtures:

```text
initial world
+ seed
+ command sequence
→ expected semantic outcome
```

### 19.4 Statistical tests

Appropriate for stochastic systems:

- hit probabilities;
- proc chances;
- item generation;
- level generation distributions;
- AI action preferences;
- damage distributions.

The test should validate intended behavior rather than identical legacy random sequences.

### 19.5 Scripted bot tests

Simple policies can exercise many turns cheaply:

```text
if adjacent enemy:
  melee
else if visible enemy:
  fire
else if item underfoot:
  pickup
else:
  explore
```

### 19.6 MCP/LLM playtests

Reasoning-capable agents can test:

- discoverability;
- tactical coherence;
- unusual state transitions;
- long-run gameplay;
- edge cases that deterministic scripts may not explore.

### 19.7 Frontend integration tests

Separate tests should validate:

```text
physical input
→ command mapping
→ simulation
→ event stream
→ rendered/presented response
```

---

## 20. Replay and Reproducibility

DRL-Rust should support semantic replay artifacts.

A replay may contain:

- build/version identifier;
- content version;
- RNG seed;
- initial character configuration;
- ordered command stream;
- optional checkpoints;
- outcome metadata.

Conceptually:

```text
seed
+ initial state
+ command 1
+ command 2
+ ...
→ reproducible run
```

This can support:

- bug reports;
- CI regression artifacts;
- MCP playtest analysis;
- balance studies;
- failure reproduction.

Full event sourcing is not required.

---

## 21. Native macOS Frontend

The initial product target is a high-quality macOS-native desktop application.

### 21.1 Proposed technical direction

A lean Rust-native stack is preferred over adopting a full game engine prematurely.

Potential components:

- `winit` for native window/event-loop integration;
- `wgpu` for portable GPU rendering with Metal on macOS;
- a focused audio crate/backend selected after prototype validation;
- platform-appropriate filesystem/path handling;
- standard macOS app bundle packaging.

### 21.2 Native application behavior

The application should:

- use appropriate macOS writable data/configuration directories;
- keep bundled resources read-only;
- support Retina/high-DPI scaling;
- handle fullscreen/windowed modes cleanly;
- support keyboard-first gameplay;
- add gamepad support if justified;
- eventually support signing and notarization.

### 21.3 Platform isolation

macOS-specific behavior belongs in platform/application crates, not `drl-core`.

---

## 22. Rendering Strategy

The renderer should consume game state and presentation events without owning simulation logic.

Likely concerns:

- tile/sprite rendering;
- camera;
- animation;
- particle/effect presentation;
- UI layers;
- HUD;
- menus;
- targeting overlays;
- minimap;
- transitions;
- text rendering;
- scaling and pixel-art handling.

The game should remain playable in headless form without the renderer.

A minimal text/debug frontend may be valuable during core development.

---

## 23. Audio Strategy

Audio should be event-driven.

The simulation requests semantic cues:

```text
weapon fired
monster died
door opened
level changed
pickup obtained
```

The audio layer maps those to:

- samples;
- volume;
- stereo/spatial behavior;
- music transitions.

The core should not depend on a particular audio library.

---

## 24. Asset and Content Pipeline

Backward compatibility with the legacy WAD format is not required.

During development, prefer transparent loose assets:

```text
content/
assets/
config/
```

This improves:

- iteration speed;
- debuggability;
- version control clarity;
- testability.

A new packaged asset format may be introduced later if deployment benefits justify it.

If useful, a one-time conversion tool can ingest legacy assets/content into the new project structure.

---

## 25. Save System

New saves should use a versioned, structured format.

Desirable characteristics:

- explicit schema;
- version field;
- migration support for DRL-Rust versions;
- no dependence on Rust memory layout;
- no dependence on serialized pointer/ID internals;
- human-inspectable format where practical, or a documented binary schema if performance warrants it.

Legacy Pascal save compatibility is explicitly out of scope.

---

## 26. Error Handling

Expected recoverable failures should use typed errors.

Examples:

- invalid command;
- illegal target;
- unavailable item;
- malformed content;
- missing asset;
- save-version mismatch.

Panics should indicate violated programmer assumptions or truly unrecoverable states, not ordinary gameplay conditions.

Error messages should preserve enough domain context to diagnose failures.

---

## 27. Logging and Diagnostics

Structured diagnostics should support:

- simulation state transitions;
- command processing;
- Lua errors;
- content loading;
- asset failures;
- renderer initialization;
- replay execution;
- MCP sessions;
- assertion/invariant failures.

Developer/debug builds should make it easy to attach:

- seed;
- turn;
- player state;
- current level;
- command;
- relevant entity IDs;

to error reports.

---

## 28. Clean Code Principles Adapted for Rust

The project should use useful principles from *Clean Code* without mechanically copying Java/OO idioms.

### 28.1 Adopt

- meaningful names;
- high cohesion;
- small conceptual units;
- single responsibility at module/function level;
- explicit dependencies;
- separation of policy from mechanism;
- clear abstraction levels;
- automated tests;
- local reasoning;
- removal of duplication where it represents duplicated knowledge.

### 28.2 Do not apply mechanically

Avoid unnecessary:

- tiny traits;
- interface-per-type structures;
- getter/setter layers;
- factory hierarchies;
- wrapper abstractions;
- object-oriented inheritance translations.

Rust-native tools should be preferred:

- enums;
- pattern matching;
- composition;
- ownership;
- newtypes;
- modules;
- free functions;
- iterators;
- traits where actual polymorphism is needed.

---

## 29. Rust Coding Principles

### 29.1 Favor domain language

Use names from game semantics rather than infrastructure whenever possible.

### 29.2 Make illegal states unrepresentable

Use constructors, newtypes, enums, and restricted field visibility.

### 29.3 Keep mutation localized

Avoid shared mutable state and uncontrolled aliasing.

### 29.4 Avoid primitive obsession

Do not pass generic integers or strings when a domain type exists.

### 29.5 Avoid premature abstraction

Duplication can be temporarily preferable to an incorrect shared abstraction.

### 29.6 Prefer deterministic iteration

Where simulation outcomes depend on iteration order, use containers/orderings that make this explicit.

### 29.7 Treat `unsafe` as exceptional

Any unsafe code should be isolated, documented, justified, and tested.

---

## 30. Legacy-Code Archaeology Process

For each subsystem, maintain a small behavioral note containing:

1. relevant Pascal modules;
2. relevant Lua modules;
3. observed rules;
4. inferred intent;
5. ambiguous cases;
6. decisions made in DRL-Rust;
7. associated tests;
8. known deliberate differences.

This becomes the bridge between the legacy implementation and the new executable specification.

Suggested location:

```text
docs/legacy-behavior/
  combat.md
  movement.md
  ai.md
  items.md
  progression.md
  generation.md
  ...
```

---

## 31. Decision Records

Use lightweight architecture decision records (ADRs) for choices with lasting consequences.

Examples:

- entity storage strategy;
- RNG implementation;
- save format;
- rendering stack;
- Lua runtime;
- content schema;
- replay versioning;
- MCP protocol design;
- deterministic iteration policy.

Suggested location:

```text
docs/adr/
```

---

## 32. Continuous Integration

CI should eventually include:

- formatting;
- linting;
- unit tests;
- property tests;
- scenario tests;
- headless simulation tests;
- replay tests;
- Lua/content validation;
- MCP protocol tests;
- macOS build;
- release bundle build;
- dependency/security audit where appropriate.

Long-running simulation suites can be separated from fast pull-request checks.

---

## 33. Automated Playtesting and Analysis

One major architectural advantage of DRL-Rust is the ability to simulate large numbers of runs cheaply.

Potential batch experiments:

```text
10,000 seeds
× multiple difficulties
× multiple bot policies
× multiple builds
```

Metrics can include:

- completion/win rate;
- death location;
- survival curves;
- weapon utilization;
- ammunition pressure;
- incoming damage sources;
- level-specific failure rates;
- item pickup/use frequency;
- trait selection;
- enemy kill distribution;
- exploration efficiency;
- pathological generation seeds.

These analyses should support development and balancing, not replace human playtesting.

---

## 34. Human and AI Playtesting Roles

Different test agents answer different questions.

### Scripted bots

Best for:

- deterministic regression;
- stress;
- performance;
- broad state-space traversal;
- repeated balancing studies.

### MCP/LLM agents

Best for:

- reasoning-driven behavior;
- interface comprehensibility;
- unusual strategy exploration;
- semantic integration testing;
- practical long-running sessions.

### Human players

Essential for:

- game feel;
- aesthetic judgment;
- difficulty perception;
- ergonomics;
- fun;
- pacing;
- frustration;
- audiovisual quality.

DRL-Rust should support all three rather than treating one as a substitute for the others.

---

## 35. Security and MCP Boundaries

MCP should not become an unrestricted process-control surface.

The server should expose narrowly scoped capabilities.

Default agent access should be limited to:

- player-visible observations;
- legal player actions;
- bounded scenario control.

Developer-only capabilities such as omniscient state or fixture loading should be clearly separated.

Filesystem or shell access should not be exposed through the game MCP surface unless independently justified.

---

## 36. Performance Expectations

DRL is turn-based and is not expected to be simulation-bound under ordinary play.

Optimization priorities should therefore be:

1. correctness;
2. clarity;
3. deterministic behavior;
4. responsiveness;
5. profiling-driven optimization.

Potential performance-sensitive areas:

- visibility/FOV;
- pathfinding;
- procedural generation;
- rendering batches;
- Lua boundary overhead during large simulation batches;
- automated high-throughput test runs.

Avoid speculative micro-optimization.

---

## 37. Compatibility Philosophy

### Legacy compatibility

Not required.

### Internal DRL-Rust compatibility

Once DRL-Rust produces public releases, compatibility policies should be deliberate.

Potential guarantees:

- save migrations between selected DRL-Rust versions;
- replay-version detection;
- stable content schema within a major release;
- versioned MCP API.

These guarantees should be added only after the corresponding formats stabilize.

---

## 38. Licensing and Asset Considerations

The legacy repository identifies its code and art under different licenses. DRL-Rust should maintain explicit provenance for:

- source code derived from or informed by legacy code;
- art;
- music;
- sound effects;
- fonts;
- third-party libraries.

Before public distribution, the project should confirm that its licensing, attribution, and asset distribution model are consistent with all inherited and third-party obligations.

This is a project-governance requirement and should be tracked separately from technical compatibility.

---

## 39. Key Risks and Mitigations

### Risk: semantic drift from DRL

**Mitigation**

- behavioral specification documents;
- legacy archaeology;
- focused comparison sessions;
- experienced human playtesting;
- tests around core mechanics.

### Risk: overengineering the rewrite

**Mitigation**

- prioritize vertical slices;
- resist premature ECS/framework adoption;
- avoid trait proliferation;
- implement abstractions only after concrete use cases emerge.

### Risk: Lua boundary recreates legacy coupling

**Mitigation**

- narrow query/command API;
- no unrestricted mutable world exposure;
- type-checked conversions;
- explicit ownership of scripting responsibilities.

### Risk: simulation and presentation become coupled again

**Mitigation**

- `drl-core` must compile and test headlessly;
- presentation consumes observations/events;
- no renderer/audio dependencies in core.

### Risk: MCP becomes a parallel game API

**Mitigation**

- MCP translates to the same `Command` model used by humans;
- observation model shared with frontend semantics;
- no MCP-only gameplay rules.

### Risk: excessive focus on exact legacy parity

**Mitigation**

- semantic fidelity doctrine;
- no exact RNG/output requirement;
- document intentional algorithmic differences.

### Risk: too much scope before first playable build

**Mitigation**

- milestone-based vertical development;
- maintain a continuously runnable headless game;
- prioritize minimal complete loops before breadth.

---

## 40. Definition of Architectural Success

The architecture can be considered successful when:

- the core game runs without a GUI;
- the same simulation accepts commands from human UI, scripted bots, replays, and MCP;
- core mechanics do not call rendering/audio/platform APIs;
- gameplay randomness is controlled and reproducible;
- domain invariants are represented by types and tests;
- legacy behavioral intent can be documented without preserving legacy structure;
- Lua content can execute through a constrained Rust boundary;
- a seeded failing run can be reproduced from a compact artifact;
- new systems can be added without broad changes to unrelated layers.

---

## 41. Definition of Product Success

A successful first major release should:

- feel recognizably and convincingly like DRL;
- preserve the game's major mechanics and tactical identity;
- run as a polished native macOS application;
- remain stable during long play sessions;
- support versioned saves;
- support deterministic replays;
- expose a usable MCP testing interface;
- include a meaningful automated test suite;
- provide maintainable documentation for future development;
- establish a codebase clearly superior to the legacy architecture in safety, modularity, and testability.

---

## 42. Recommended Initial Development Strategy

The project should begin with **behavioral modeling and a headless vertical slice**, not graphics.

Recommended order:

```text
1. Establish repository/workspace/tooling
2. Define core protocol: Command, Observation, Event
3. Implement deterministic RNG
4. Implement minimal map/player/turn loop
5. Add representative combat
6. Integrate Lua for selected legacy content
7. Build scenario/replay testing
8. Add scripted test player
9. Add MCP interface
10. Add native graphical frontend
11. Expand toward gameplay completeness
12. Package, polish, and release
```

This approach validates the most consequential architecture before investing heavily in presentation.

---

## 43. Guiding Questions for Ongoing Review

When implementing any feature, ask:

1. What is the intended DRL behavior?
2. Is this behavior canonical, or merely a legacy implementation detail?
3. Can the rule be expressed more directly in the type system?
4. Can invalid states be ruled out structurally?
5. Can the calculation be pure or isolated from side effects?
6. Is randomness explicit?
7. Can this be tested headlessly?
8. Can a human UI and MCP agent use the same semantic command?
9. Does Lua need this capability, or are we exposing too much?
10. Are we creating an abstraction because it is needed now, or because it sounds clean?
11. Could this design recreate one of the legacy coupling problems?
12. How would a future developer understand the intended rule without reading Pascal?

---

## 44. Final Project Statement

DRL-Rust is not a preservation effort for the legacy implementation.

It is a preservation and modernization effort for the **game design encoded by that implementation**.

The legacy code should tell us what the game means. Rust should let us express that meaning more explicitly, safely, and testably.

The core architectural target is therefore:

> **A deterministic, domain-oriented Rust simulation with explicit commands, observations, events, typed invariants, controlled side effects, pragmatic Lua integration, and first-class machine-driven testing—presented to players through a polished native macOS frontend.**
