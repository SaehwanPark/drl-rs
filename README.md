# DRL-Rust

![DRL-Rust concept banner](assets/drl-rust-banner.png)

The banner is concept art, not a gameplay capture.

**DRL-Rust** (`drl-rust`) is an independent, ground-up Rust
reimplementation of [Doom the Roguelike (DRL)](https://drl.chaosforge.org/).
The initial product target is a high-quality native macOS experience, while
the planned simulation core remains portable and platform-independent.

## Project Status

DRL-Rust has delivered its **Milestone 6: MCP Game Interface**. The
current implementation is a multi-crate Rust 2024 workspace featuring:

- pure, deterministic headless simulation core (`drl-core`) with 2D tile maps,
  grid coordinates, seedable PRNG, and turn-based movement, combat, inventory, and weapon mechanics;
- Model Context Protocol (MCP) server engine (`drl-mcp`) exposing standard JSON-RPC 2.0
  game session tools, resources, and semantic actions for AI test agents and playtesting;
- comprehensive MCP tool suite: `game_start`, `game_load_scenario`, `game_get_observation`,
  `game_list_actions`, `game_step_action`, `game_reset`, `game_get_metrics`, `game_save_replay`, and `game_get_dev_state`;
- static and dynamic MCP game resources (`drl://rules/game`, `drl://rules/actions`, `drl://session/metrics`, `drl://session/events`);
- zero-external-dependency JSON parser, serializer, and JSON-RPC 2.0 dispatcher implemented in pure standard Rust;
- strict fairness and security boundaries ensuring AI agents receive only standard `PlayerObservation` unless operating in guarded developer mode;
- stdio transport runner for MCP clients and AI agent integration via `cargo run -p drl-app -- --mcp`;
- versioned replay log schema (`ReplayVersion::V1`, `ReplayMetadata`) in `drl-protocol` and
  `drl-core` with engine versioning, custom hero spawn configurations, explicit tile maps, and schema validation (`ReplayEngine::validate`);
- diagnostic replay error reporting with `ReplayExecutionError` capturing exact turn numbers,
  0-based command indices, failed commands, and underlying simulation error contexts;
- declarative scenario fixture framework (`Scenario`, `ScenarioFixture`, `ScenarioMap`) with ASCII map
  parsing (`Scenario::from_ascii`), custom monster/item placements, hero equipment configurations, and fluent assertion runners (`ScenarioRunner`);
- scripted test agent policies (`AgentPolicy` trait) consuming strictly `PlayerObservation` without state leakage:
  `RandomBot`, `GreedyCombatBot` (engaging enemies, reloading, healing, looting, and stairs descent), and `ExplorerBot`;
- headless batch simulation runner (`BatchRunner`) executing large procedural and scenario sweeps across arbitrary seeds,
  recording `EpisodeRecord` artifacts, and calculating statistical summaries (`BatchSummary`: win rates, average turns, damage, kills);
- runtime metrics accumulation (`RunOutcome`, `EpisodeMetrics`) tracking completion status, damage telemetry,
  kill distributions, item pickups, ammo expenditure, and level progression;
- weapon kinetic knockback mechanics (`apply_knockback`) and `GameEvent::ActorKnockedBack`, pushing surviving targets
  away along the shot vector with obstacle, wall, and occupant collision safety;
- tactical monster AI decision engine (`ai`) with line-of-sight checks, ranged projectile/fireball attacks,
  adjacent melee strikes, and pathfinding pursuit;
- representative enemy archetypes (`FormerHuman`, `FormerSergeant`, `Imp`, `Demon`) with distinct health,
  speed, melee, ranged capabilities, innate/weapon knockback, and death loot drop tables;
- targeting validation system (`targeting`) verifying `Target::Position`, `Target::Entity`, and `Target::Direction`
  against bounds, range limits, and line-of-sight obstruction, with visible enemy listing and nearest auto-selection;
- special-use consumable item `Phase Device` enabling emergency spatial teleportation to safe walkable tiles;
- monster death loot drop mechanics spawning ground items upon lethal combat defeat;
- procedural dungeon level generation (`generator`) with non-overlapping room carving,
  walkable corridor linking, down-stairs exit placement, BFS reachability validation, and monster/loot spawning;
- down-stairs interaction and seamless level transitions (`Command::Descend`) carrying over player
  health, inventory backpack, equipped weapon/armor, clip ammunition, and action energy;
- field of view (FOV) calculation and line-of-sight (LOS) raycasting (`fov`) with
  deterministic perimeter raycasting, obstacle occlusion, and perimeter illumination;
- fog-of-war map exploration memory in `World` and strict perception filtering in
  `PlayerObservation` preventing information leaks for hidden enemies and floor items;
- item domain models (`item`) and bounded backpack inventory (`inventory`) supporting
  ammunition stacking, floor pickups (`Command::Pickup`), and drops (`Command::Drop`);
- equipment slots (`Equipment`) for weapons and body armor with damage protection mitigation;
- weapon and ammunition mechanics with magazine clip tracking, ammo consumption
  on ranged fire, clip exhaustion rejection, and weapon reloading (`Command::Reload`);
- representative equipment and items: Pistol, Shotgun, Combat Knife, 9mm Ammo, Shells,
  Small/Large MedPacks, Green Armor, and Phase Device;
- line-of-fire validation for ranged weapon attacks rejecting obstructed shots;
- action economy and energy-based actor scheduling (`Scheduler`) supporting relative
  speeds and deterministic turn progression;
- combat resolution engine (`CombatResolver`) supporting melee bump-attacks, direct
  melee strikes, and ranged weapon fire with damage clamping and death handling;
- shared semantic protocol schemas (`drl-protocol`) for commands, combat/item domain models,
  observations, events, targets, metrics, scenario fixtures, and replays;
- an executable application runner (`drl-app` / `drl-rust`) that runs headless simulation,
  tactical ranged monster combat, weapon knockback blasts, scenario fixtures, automated bot play, batch sweeps, stdio MCP server, and replay determinism verification;
- automated architectural boundary tests, pure combat unit tests, FOV integration tests,
  inventory integration tests, AI & archetype tests, targeting tests, special item tests, level progression tests,
  stochastic combat statistical validation suites, declarative scenario tests, agent policy tests, batch simulation tests,
  replay versioning tests, and MCP JSON-RPC protocol/virtual AI player tests.

Live Lua scripting integration (Milestone 3), GPU rendering (Milestone 7), and audio playback (Milestone 8)
are scheduled in subsequent roadmap milestones.

The project intends to preserve DRL's modeled behavior and tactical character
without translating the legacy Pascal architecture or execution traces
line-by-line:

> **Preserve the game; rewrite the machinery.**

Planned design priorities include:

- a deterministic, headless simulation core;
- explicit commands, observations, events, and random state;
- typed domain models and controlled side effects;
- one semantic command boundary for human input, replays, bots, and MCP;
- a narrow, intentional Lua boundary;
- native macOS presentation outside the portable simulation.

## Project Documents

- [Roadmap](docs/DRL-Rust_Project_Roadmap.md): canonical milestone plan,
  progress tracker, and exit criteria.
- [Specification](SPEC.md): implementation-ready expansion of the active
  roadmap slice.
- [Architecture](ARCHITECTURE.md): verified current structure, invariants, and
  clearly labeled planned direction.
- [Proposal](docs/DRL-Rust_Project_Proposal.md): broader living design
  rationale.
- [Changelog](CHANGELOG.md): meaningful delivered changes.
- [Agent guide](AGENTS.md): durable repository workflow for contributors and
  coding agents.

The roadmap remains authoritative. SDD documents unpack and record milestone
work; they do not create a competing plan.

## Prerequisites

Current development requires:

- Git;
- the current stable Rust toolchain with Rust 2024 edition support;
- the `rustfmt` and `clippy` Rust components;
- a POSIX shell and standard command-line tools available on macOS or Linux.

With [rustup](https://rustup.rs/):

```sh
rustup toolchain install stable --profile minimal \
  --component rustfmt \
  --component clippy
rustup default stable
```

The project does not declare a minimum supported Rust version yet. Stable Rust
is the documented development and CI policy until the roadmap records a
different decision.

Lua, the legacy checkout, and legacy assets are not required to build the
current scaffold. They become optional research inputs for relevant roadmap
work.

## Quick Start

```sh
git clone https://github.com/SaehwanPark/drl-rust.git
cd drl-rust
cargo run
```

This runs the headless simulation demo, executes a deterministic command sequence,
and verifies replay reproducibility.

Run the same repository checks used by CI:

```sh
sh scripts/check-repository.sh
```

This checks the repo-wide spaces-only policy with indentation and tab width 2,
the agent-harness structure, Rust formatting, Clippy warnings, and tests. CI
runs the same command on macOS.

## Repository Layout

```text
.
├── .agents/                         Repo-local delivery specialist skills
├── .github/workflows/               macOS CI
├── assets/                          Tracked project assets
├── crates/                          Multi-crate workspace members
│   ├── drl-app/                     Application runner executable
│   ├── drl-audio/                   Audio layer placeholder
│   ├── drl-core/                    Deterministic simulation core
│   ├── drl-mcp/                     Model Context Protocol placeholder
│   ├── drl-protocol/                Shared command/observation/event schema
│   ├── drl-render/                  Presentation layer placeholder
│   └── drl-script/                  Scripting/content integration placeholder
├── docs/                            Project plans and harness team contract
├── scripts/                         Shared repository checks
├── AGENTS.md                        Durable agent and contributor guidance
├── ARCHITECTURE.md                  Verified architecture state
├── CHANGELOG.md                     Delivered project history
└── SPEC.md                          Active milestone slice
```

## Milestone Development Workflow

1. Select the smallest coherent item from one roadmap milestone.
2. Expand only that item in the `Present` section of `SPEC.md`, including
   observable outcomes, verification, and non-goals.
3. Inspect current code and relevant legacy evidence before implementation.
4. Implement and test the bounded slice.
5. Reconcile the specification, architecture, changelog, and roadmap from
   verification evidence.

The repo-local
[`drl-milestone-delivery`](.agents/skills/drl-milestone-delivery/SKILL.md)
skill contains the complete workflow and stop conditions.

## Optional Agent Team Workflow

Keep small, tightly coupled changes with one milestone owner. When legacy
research, stage-aware test play, or an independent determinism review provides
clear value, use the
[DRL delivery team specification](docs/harness/drl-delivery/team-spec.md).
It defines role selection, serialized ownership of canonical documents,
deterministic handoffs, and partial-failure behavior.

The repository currently has no playable simulation. Test play at this stage
means scaffold smoke checks or structured legacy-behavior probes, not DRL-Rust
gameplay. The
[`drl-test-play`](.agents/skills/drl-test-play/SKILL.md) skill activates seeded
headless scenarios, replays, bots, MCP sessions, statistical studies, and
human play only after the corresponding capabilities are implemented.

Runtime handoffs are optional and ignored under `_workspace/`. Direct work
should continue to report a bounded handoff without creating coordination
files.

## Optional Legacy Research Setup

Behavioral archaeology uses the
[legacy codebase fork](https://github.com/SaehwanPark/doom-the-roughlike-original)
as a reference. The expected local layout is:

```text
../doom-the-roughlike-original/
├── drlhq/          Locally extracted sound and music research assets
└── fpcvalkyrie/    Legacy Valkyrie dependency checkout
```

- Follow the original project's
  [asset extraction instructions](https://github.com/chaosforgeorg/drl/blob/master/README.md)
  rather than committing extracted files.
- Clone [fpcvalkyrie](https://github.com/chaosforgeorg/fpcvalkyrie) into the
  location shown above when legacy builds require it.
- Local research assets may also be placed in `assets/legacy-drlhq/`; that
  directory is intentionally ignored by Git.
- Install Lua, for example with `brew install lua`, only when a selected
  milestone actually needs legacy Lua inspection or a future runtime.

Legacy implementation details are evidence to interpret. They do not dictate
new Rust ownership, module, mutation, or random-number architecture.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full developer guide covering
workspace layout, prerequisites, code style, branch and PR conventions, and
the architectural rules that must not be broken.

Quick reference:

- choose a bounded active roadmap slice;
- read `SPEC.md` before writing code;
- preserve the roadmap/SPEC source-of-truth hierarchy;
- distinguish verified current behavior from planned design;
- keep unrelated milestone work out of the change;
- run `sh scripts/check-repository.sh`;
- report checks, deviations, and unresolved design questions.

Architecture Decision Records in [`docs/adr/`](docs/adr/) document the
key accepted decisions for architecture principles, backward compatibility
policy, command model, RNG strategy, Lua integration, and MCP interface design.

## Licensing and Provenance

Project-authored code and documentation in this repository are provided under
the [MIT License](LICENSE).

Doom the Roguelike, its legacy source, names, art, audio, fonts, and other
third-party materials may have different owners and license terms. The root MIT
license must not be treated as permission to redistribute imported legacy
assets. Keep research assets untracked until their provenance and redistribution
rights are documented.
