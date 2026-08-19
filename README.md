# DRL-Rust

![DRL-Rust concept banner](assets/drl-rust-banner.png)

The banner is concept art, not a gameplay capture.

**DRL-Rust** (`drl-rust`) is an independent, ground-up Rust
reimplementation of [Doom the Roguelike (DRL)](https://drl.chaosforge.org/).
The initial product target is a high-quality native macOS experience, while
the planned simulation core remains portable and platform-independent.

## Project Status

DRL-Rust is in its repository-foundation stage and is **not playable**. The
current implementation is a multi-crate Rust 2024 workspace managing `drl-core`,
`drl-protocol`, `drl-app`, and subsystem placeholders; running it prints a
scaffold status message. Gameplay mechanics, live Lua integration, MCP server
transport, GPU rendering, and audio playback have not been implemented.

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

These are design goals, not claims about the current scaffold.

## Project Documents

- [Roadmap](docs/DRL-Rust_Project_Roadmap.md): canonical milestone plan,
  progress tracker, and exit criteria.
- [Specification](SPEC.md): implementation-ready expansion of the one active
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

The current output is only the scaffold's placeholder message.

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
