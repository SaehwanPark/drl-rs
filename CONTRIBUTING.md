# Contributing to DRL-Rust

Thank you for your interest in DRL-Rust. This document covers everything you
need to start contributing: workspace layout, code style, branch and PR
conventions, and the architectural rules that keep the project healthy.

---

## Table of Contents

- [Workspace layout](#workspace-layout)
- [Prerequisites](#prerequisites)
- [Running checks](#running-checks)
- [Code style](#code-style)
- [Branch and commit conventions](#branch-and-commit-conventions)
- [Pull request workflow](#pull-request-workflow)
- [Architectural rules](#architectural-rules)
- [Documentation and specification](#documentation-and-specification)
- [Reporting issues](#reporting-issues)

---

## Workspace layout

DRL-Rust is a Cargo workspace. All crates live under `crates/`.

| Crate | Role |
|---|---|
| `drl-protocol` | Shared semantic domain types, commands, events, observations, fixtures, replays |
| `drl-core` | Deterministic headless simulation kernel (no I/O, no rendering, no audio) |
| `drl-mcp` | MCP JSON-RPC 2.0 server and semantic tool suite |
| `drl-app` | Executable entry point: headless demo, bot play, batch sweeps, MCP stdio runner |
| `drl-script` | Build-time content/Lua conversion boundary; no runtime Lua |
| `drl-assets` | Platform-neutral atlas identifiers, provenance, and mappings |
| `drl-render` | Pure scene/presentation builders consumed by browser renderers |
| `drl-audio` | Semantic cues and WASM Web Audio mixer |
| `drl-web` | Browser session, WASM exports, wgpu surface, DOM/keyboard shell |

Key documents:

| File | Purpose |
|---|---|
| `docs/DRL-Rust_Project_Roadmap.md` | Canonical milestone plan and progress tracker |
| `SPEC.md` | Active implementation slice: outcomes, verification, and non-goals |
| `ARCHITECTURE.md` | Verified current structure and architectural invariants |
| `CHANGELOG.md` | Contributor- and user-visible history |
| `docs/adr/` | Architecture Decision Records |
| `docs/legacy-behavior/` | Behavioral specification notes from legacy research |

---

## Prerequisites

- **Rust** — stable toolchain, Rust 2024 edition. See `rust-toolchain.toml` or
  `Cargo.toml` for the current MSRV policy.
- **Git** — standard version control.
- Browser work additionally needs the `wasm32-unknown-unknown` target and
  pinned `wasm-pack 0.15.0`.

No additional tools are required. The project uses only `std` and a small set
of declared workspace dependencies.

---

## Running checks

Always run the repository check script before pushing:

```sh
sh scripts/check-repository.sh
```

This script runs in order:

1. Rejects literal tab characters in tracked files.
2. Rejects trailing whitespace.
3. Runs `git diff HEAD --check`.
4. Validates the agent harness structure (`scripts/check-agent-harness.sh`).
5. Runs `cargo fmt --all -- --check`.
6. Runs `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`.
7. Runs `cargo test --locked --workspace`.

All seven steps must pass before a PR is opened. Browser changes also run:

```sh
sh scripts/check-assets.sh
scripts/check-web.sh
```

The web script reports a missing local Chrome runner as `NOT_RUN`; remote web
CI and manual Chrome/Edge acceptance remain separate gates.

---

## Code style

### Formatting

- **Indentation**: 2 spaces. No tabs anywhere.
- **Trailing whitespace**: none.
- **Line endings**: LF (Unix).
- `rustfmt` is authoritative for Rust source. Run `cargo fmt --all` to format.
- `rustfmt.toml` at the repository root configures any non-default options.

### Rust conventions

- Prefer explicit types and domain-aware newtypes over primitive obsession.
- Prefer `match` over long `if let` chains where exhaustiveness aids clarity.
- Prefer `Result` and typed error variants over panics in simulation code.
- Keep functions small and single-purpose.
- Avoid deeply nested blocks; extract helpers instead.
- No `unwrap()` or `expect()` in simulation paths unless the invariant is
  explicitly documented and enforced by construction.
- No `println!` or `eprintln!` in `drl-core` or `drl-protocol`.
- Randomness must flow through `GameRng`; see the RNG ADR.

### Clippy

All Clippy warnings are treated as errors (`-D warnings`). Address lint
suggestions before opening a PR. If suppressing a lint, add a comment
explaining why.

### Comments and documentation

- Prefer self-documenting names over comments that restate the code.
- Use `///` doc-comments on public types and functions.
- Use `//` inline comments only for non-obvious decisions or invariants.
- Do not leave commented-out code.

---

## Branch and commit conventions

### Branch names

```
feature/<short-description>
fix/<short-description>
docs/<short-description>
refactor/<short-description>
test/<short-description>
```

Use lowercase with hyphens (the repository default branch prefix is `codex/`
for agent-created branches). Keep descriptions concise.

### Commit messages

- Write in the imperative mood: "Add FOV raycasting", not "Added FOV".
- Keep the subject line under 72 characters.
- Reference the milestone or roadmap item in the body when relevant.
- One logical change per commit. Avoid mixing refactors with feature additions.

---

## Pull request workflow

1. **Branch** from `main`:
   ```sh
   git checkout main && git pull && git checkout -b feature/my-change
   ```

2. **Implement** your change with tests where applicable.

3. **Check locally**:
   ```sh
   sh scripts/check-repository.sh
   ```

4. **Push** your branch and **open a PR** against `main` on GitHub.

5. **PR description** should include:
   - Which roadmap item or SPEC.md slice this addresses.
   - What observable outcomes the change delivers.
   - How it was verified (test commands, manual steps).
   - Any known limitations or follow-up items.

6. **Review** — at least one review approval is required before merging.

7. **Merge** using squash-merge to keep `main` history linear.

8. **Delete** the remote and local branch after merge.

---

## Architectural rules

These constraints are load-bearing. Violating them can corrupt the simulation's
determinism guarantees or couple presentation concerns into the simulation core.

### drl-core must stay pure

`drl-core` must have zero dependencies on rendering, audio, OS APIs, Lua,
MCP, network, or filesystem I/O. It depends only on `drl-protocol` and the
Rust standard library. This is enforced by automated boundary tests in
`crates/drl-core/tests/boundaries.rs`.

### drl-protocol must be dependency-free

`drl-protocol` contains only data types and must not depend on `drl-core`,
`drl-mcp`, or any presentation crate.

### No ambient or global RNG

All randomness in the simulation must pass through the explicit `GameRng`
parameter. Never use `rand::thread_rng()` or any other ambient source inside
`drl-core`. See `docs/adr/0004-explicit-deterministic-rng.md`.

### All clients use the same Command model

Human input, scripted bots, and MCP agents must all submit `drl-protocol::Command`
values through the standard simulation API. No client may modify world state
directly or bypass command validation. See `docs/adr/0003-semantic-command-model.md`.

### Deterministic collection ordering

Any collection iterated during simulation must use deterministic ordering.
Use `BTreeMap`/`BTreeSet` instead of `HashMap`/`HashSet` where order affects
outputs. Document exceptions explicitly.

### Presentation side-effects belong outside drl-core

Game events (`GameEvent`) are the only output mechanism from simulation steps.
Rendering, audio, and UI reactions must be driven by consuming those events
outside the core.

### Browser boundary

Browser input must map to ordinary `drl-protocol::Command` values. `drl-web`
may consume only `PlayerObservation` and `GameEvent`; it must not read
`World`, expose hidden actors/items, or let animation, audio policy,
resize/DPR, tab visibility, or GPU loss advance simulation. Start/error/help,
HUD, and inventory controls remain semantic DOM regions even when the world is
drawn on a pixel-scaled WebGPU canvas.

### Asset provenance

Import only from the pinned legacy Git revision recorded in the asset manifest,
never from a dirty checkout. Keep source path, attribution, license, and
checksum records with every asset group. The graphics atlas is CC BY-SA 4.0;
legacy code, audio/music, and fonts require separate rights decisions.

---

## Documentation and specification

- Before implementing a milestone item, read `SPEC.md` (active slice) and the
  relevant section of the roadmap.
- Update `SPEC.md` to reflect the active slice before writing code.
- Record verified architectural changes in `ARCHITECTURE.md`.
- Add changelog entries in `CHANGELOG.md` after delivery.
- Mark roadmap tasks complete only when the stated result exists and is verified.
- When documenting legacy behavior, distinguish observed fact from inferred
  design intent. See `docs/legacy-behavior/_template.md`.

---

## Reporting issues

Open a GitHub issue describing:

- Steps to reproduce (minimal scenario or command sequence).
- Expected behavior (referencing legacy DRL if appropriate).
- Actual behavior.
- Relevant game seed and replay log if available.

If you have a reproducible replay, attach it. The replay log format makes
defects much easier to isolate and verify.
