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

## Present

### Milestone 0: Cargo Workspace and Initial Crates Boundary Scaffolding

Status: Active

This slice establishes the modular multi-crate Cargo workspace and validates
architectural dependency boundaries for DRL-Rust.

Observable outcomes:

- root `Cargo.toml` is configured as a multi-crate Cargo workspace managing all
  first-party crates in `crates/`;
- `crates/drl-core` is created as an independent, headless simulation library
  with no dependencies on rendering, audio, MCP, or operating-system concerns;
- `crates/drl-protocol` is created as the shared semantic contract library for
  commands, observations, and events;
- `crates/drl-app` is created as the application runner binary, configured as
  the default workspace runnable;
- placeholder crates `crates/drl-script`, `crates/drl-mcp`, `crates/drl-render`,
  and `crates/drl-audio` are defined within the workspace with explicit
  dependency directions;
- automated architectural tests verify that `drl-core` and `drl-protocol`
  remain free of disallowed presentation, audio, and MCP dependencies;
- `sh scripts/check-repository.sh` runs formatting, clippy, harness, and test
  checks across the workspace without warnings.

Verification:

- `sh scripts/check-repository.sh` succeeds locally;
- `cargo test --locked --workspace` passes all crate unit and architecture
  boundary tests;
- `cargo run` launches `drl-app` as the default workspace binary;
- architectural dependency invariants are verified by automated test.

Out of scope:

- implementing core gameplay simulation mechanics (deferred to Milestone 1);
- integrating external graphics or audio engine dependencies (deferred to
  Milestone 7 and Milestone 8);
- integrating a live Lua C/Rust runtime (deferred to Milestone 3);
- implementing the live MCP server transport (deferred to Milestone 6).

## Future

Proceed to Milestone 1 (Headless Simulation Kernel) starting with typed world
entities, grid positions, and deterministic turn/command structures after this
workspace foundation is verified.
