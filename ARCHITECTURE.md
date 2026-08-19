# Architecture

Last Reviewed: 2026-08-18

Status: Verified

## Overview

DRL-Rust is currently a single Rust 2024 binary package. It is a repository
scaffold, not an implemented game architecture. The broader target design in
the [project proposal](docs/DRL-Rust_Project_Proposal.md) remains planned.

## Current Components

- `Cargo.toml` defines the `drl-rust` package without dependencies.
- `src/main.rs` is the only executable entry point and prints a placeholder
  message.
- `docs/DRL-Rust_Project_Roadmap.md` owns milestone planning and progress.
- `SPEC.md` expands the active roadmap slice.
- `AGENTS.md`, `docs/harness/drl-delivery/team-spec.md`, and repo-local skills
  define the development and test-play harness.
- `scripts/check-repository.sh` is the common local and CI verification entry
  point and includes deterministic harness-structure validation.

There is no library API, multi-crate workspace, game state, command protocol,
Lua runtime, MCP server, renderer, audio system, persistence layer, or gameplay
implementation yet.

## Current Flow

```text
Roadmap milestone
  -> active SPEC slice
  -> optional evidence specialists
  -> implementation and focused tests
  -> capability-gated test play and determinism review
  -> local and CI verification
  -> architecture, changelog, and roadmap reconciliation
```

The current executable flow is only:

```text
cargo run -> src/main.rs -> placeholder standard output
```

## Consequential Invariants

- The roadmap remains canonical for long-term scope and milestone status.
- Planned architecture must not be described as implemented.
- Future simulation code must remain independent from graphics, audio,
  operating-system, filesystem, and MCP concerns.
- Gameplay randomness must become explicit and reproducible.
- Human UI, bots, replay tools, and MCP should eventually use the same semantic
  command boundary.
- Legacy Pascal and Lua sources inform behavior, not Rust module structure or
  execution order.
- One milestone owner reconciles canonical documents; delegated workers are
  read-only by default and cannot convert exploratory findings directly into
  completion claims.
- Unsupported test-play capabilities are reported as `NOT_RUN`; missing or
  contradictory evidence remains `INCONCLUSIVE`.
- Repository-controlled text uses spaces with indentation and tab width 2.

## Planned Direction

The proposal describes a headless deterministic simulation core, shared
commands, observations and events, Lua-backed content, replay and test-agent
support, an MCP interface, and a native macOS presentation layer. Those
components are targets, not current dependencies or compatibility guarantees.

Crate boundaries and technology choices remain undecided until their roadmap
slices are specified and verified.
