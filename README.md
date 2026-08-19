# DRL-Rust

![DRL-Rust concept banner](assets/drl-rust-banner.png)

The banner is concept art, not a gameplay capture.

**DRL-Rust** (`drl-rust`) is an independent, ground-up Rust
reimplementation of [Doom the Roguelike (DRL)](https://drl.chaosforge.org/).
The initial product target is a high-quality native macOS experience, while
the planned simulation core remains portable and platform-independent.

## Project Status

DRL-Rust has established its **Milestone 4 Procedural Level Generation, Combat, Inventory, Equipment, and Level Transitions**. The
current implementation is a multi-crate Rust 2024 workspace featuring:

- pure, deterministic headless simulation core (`drl-core`) with 2D tile maps,
  grid coordinates, seedable PRNG, and turn-based movement, combat, inventory, and weapon mechanics;
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
  Small/Large MedPacks, and Green Armor;
- line-of-fire validation for ranged weapon attacks rejecting obstructed shots;
- action economy and energy-based actor scheduling (`Scheduler`) supporting relative
  speeds and deterministic turn progression;
- combat resolution engine (`CombatResolver`) supporting melee bump-attacks, direct
  melee strikes, and ranged weapon fire with damage clamping and death handling;
- shared semantic protocol schemas (`drl-protocol`) for commands, combat/item domain models,
  observations, events, and replays;
- an executable application runner (`drl-app` / `drl-rust`) that runs a multi-level headless
  combat, inventory, reload, and stairs descent simulation demonstration and verifies replay reproducibility;
- automated architectural boundary tests, pure combat unit tests, FOV integration tests,
  inventory integration tests, level progression tests, and deterministic scenario replay verification.

Live Lua scripting integration (Milestone 3), MCP server transport (Milestone 6),
GPU rendering (Milestone 7), and audio playback (Milestone 8) are scheduled in subsequent roadmap milestones.

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

The project is early and the roadmap still contains foundational decisions.
Before changing code or documentation:

- choose a bounded active roadmap slice;
- preserve the roadmap/SDD source-of-truth hierarchy;
- distinguish verified current behavior from planned design;
- keep unrelated milestone work out of the change;
- run `sh scripts/check-repository.sh`;
- report checks, deviations, and unresolved provenance or design questions.

Dedicated contribution policy and conduct documents remain Milestone 0 work.

## Licensing and Provenance

Project-authored code and documentation in this repository are provided under
the [MIT License](LICENSE).

Doom the Roguelike, its legacy source, names, art, audio, fonts, and other
third-party materials may have different owners and license terms. The root MIT
license must not be treated as permission to redistribute imported legacy
assets. Keep research assets untracked until their provenance and redistribution
rights are documented.
