# drl-rs

[![CI](https://github.com/SaehwanPark/drl-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/SaehwanPark/drl-rs/actions/workflows/ci.yml)
[![Pages](https://github.com/SaehwanPark/drl-rs/actions/workflows/pages.yml/badge.svg)](https://saehwanpark.github.io/drl-rs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 2024](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)

**drl-rs** is a ground-up, deterministic Rust reimplementation of [Doom the Roguelike (DRL)](https://drl.chaosforge.org/), originally created by Kornel Kisielewicz and [ChaosForge](https://chaosforge.org/).

The project delivers a pure, deterministic headless simulation core, an interactive browser-playable slice rendered with WebGPU, and a native Model Context Protocol (MCP) server for automated AI agent playtesting.

📖 **[Explore the Full Documentation Portal](https://saehwanpark.github.io/drl-rs/)**

---

## ✨ Key Features

- **Strict Simulation Determinism**: Pure PRNG rejection sampling (`GameRng`), explicit command-driven turn execution, zero ambient state, and bit-exact replay reproducibility.
- **Transactional Command Safety**: All state-mutating commands are protected by atomic transaction guards. Rejected commands (e.g. empty clips, blocked moves, out-of-range targets) are guaranteed no-ops (`before == after`).
- **WebGPU Browser Edition**: High-performance pixel-art graphics rendered via native WebGPU shaders in desktop Chromium browsers, with an accessible HTML shell and offline PWA service worker caching.
- **Model Context Protocol (MCP) Interface**: Full stdio JSON-RPC 2.0 tool suite (`step`, `observe`, `list_actions`, `verify_replay`) allowing AI assistants (Claude, Antigravity, custom agents) to play and evaluate scenarios.
- **Rich Tactical Arsenal**: Modeled weapon behaviors including first-, second-, third-, fourth-, fifth-, sixth-, seventh-, eighth-, ninth-, and tenth-level BFG 10K chainfire, first-, second-, and third-level Chaingun chainfire, first-, second-, and third-level Minigun chainfire, first- and second-level Plasma Rifle chainfire, first-, second-, third-, fourth-, fifth-, and sixth-level Laser Rifle chainfire, first-, second-, third-, fourth-, fifth-, and sixth-level Nuclear Plasma chainfire, deterministic BFG 10K radius-2 actor fanout with thresholded loose-ammo destruction, Standard and Nuclear BFG 9000 radius-8 actor fanout with source self-safety and thresholded ordinary ground-item destruction, single-shell shotgun reloads, kinetic pellet knockback, plasma energy volleys, and exotic unique artifacts (Trigun, Subtle Knife, Grammaton).
- **Procedural Dungeon Generation**: Deterministic level layouts with rooms, corridors, stairwells, fluid hazard terrains (acid, lava, mud), and dynamic monster spawning.

---

## ⚡ Quickstart

### 1. 🖥️ WebGPU Browser Edition
Run the local web server and open the browser playable slice:
```bash
sh scripts/serve-web.sh
```
Open [http://127.0.0.1:8000](http://127.0.0.1:8000) in Google Chrome or Microsoft Edge with WebGPU enabled.

### 2. ⌨️ Terminal & Headless CLI
Run the standalone executable for interactive demos and batch procedural cohort studies:
```bash
# Run headless demo suite
cargo run -p drl-app --bin drl-rs

# Run deterministic procedural cohort study
cargo run -p drl-app --bin drl-rs -- cohort --seed 42 --episodes 100 --bot greedy
```

### 3. 🤖 AI Agent via Model Context Protocol (MCP)
Start the stdio JSON-RPC MCP server:
```bash
cargo run -p drl-app --bin drl-rs -- --mcp
```

---

## 🏛️ Workspace Architecture

drl-rs follows a strict **Functional Core, Imperative Shell** design pattern across 9 focused crates:

```text
crates/
  ├── drl-protocol   # Shared semantic domain types, commands, events, observations, replays
  ├── drl-core       # Pure deterministic headless simulation kernel (zero I/O, zero graphics)
  ├── drl-mcp        # Model Context Protocol (MCP) JSON-RPC 2.0 server and tool suite
  ├── drl-app        # Standalone executable CLI, demo runners, and batch cohort runner
  ├── drl-script     # Build-time content and legacy converter boundary (no runtime Lua)
  ├── drl-assets     # Platform-neutral sprite atlas identifiers, UVs, and provenance
  ├── drl-render     # Pure presentation layer and GPU composition planning
  ├── drl-audio      # Semantic sound cue mappings and Web Audio procedural synthesizer
  └── drl-web        # Browser WASM entry point, WebGPU pipeline, DOM shell, and offline PWA
```

---

## 📚 Documentation Directory

Detailed guides and specifications are hosted on the **[DRL-rs Documentation Portal](https://saehwanpark.github.io/drl-rs/)**:

### Player Manuals & Tactical Guides
- [**Installation & Launch Guide**](https://saehwanpark.github.io/drl-rs/guides/installation.html): Prerequisites, compilation, and launch commands.
- [**How to Play & Controls**](https://saehwanpark.github.io/drl-rs/guides/how-to-play.html): Movement, targeting, ranged combat, reload mechanics, and inventory.
- [**WebGPU Browser Edition**](https://saehwanpark.github.io/drl-rs/guides/browser-slice.html): WebGPU features, keyboard/touch controls, and offline PWA caching.
- [**Weapons & Items Catalog**](https://saehwanpark.github.io/drl-rs/guides/weapons-and-items.html): Complete firearm catalog, energy weapons, armors, and exotic artifacts.
- [**Monsters & Combat Tactics**](https://saehwanpark.github.io/drl-rs/guides/monsters-and-tactics.html): Enemy speed ratios, AI fallback rules, and survival tactics.

### AI & Developer References
- [**AI & MCP Playtesting Guide**](https://saehwanpark.github.io/drl-rs/guides/mcp-playtesting-guide.html): Connecting AI agents, tool schemas, and prompt recipes.
- [**Determinism & Architecture**](https://saehwanpark.github.io/drl-rs/guides/architecture-and-determinism.html): ECS state models, transaction guards, and presentation decoupling.
- [**Asset Licensing Policy**](https://saehwanpark.github.io/drl-rs/reference/asset-licensing.html): Legacy asset provenance and copyright boundaries.
- [**Versioning Policy**](https://saehwanpark.github.io/drl-rs/reference/versioning.html): Three-component version transitions and CI validation.

### Repository Specifications
- `SPEC.md`: Active implementation milestone slice.
- `ARCHITECTURE.md`: Verified current architecture invariants.
- `CHANGELOG.md`: Contributor and user-visible change history.
- `docs/DRL-RS_Project_Roadmap.md`: Canonical project roadmap and milestone gates.
- `docs/steering/README.md`: Active steering decisions and stop gates.

---

## 🛠️ Contributing

Contributions are welcome! Follow these steps to build and verify your changes:

1. **Clone the repository**:
   ```bash
   git clone https://github.com/SaehwanPark/drl-rs.git
   cd drl-rs
   ```
2. **Run full verification suite**:
   ```bash
   sh scripts/check-repository.sh
   sh scripts/check-web.sh
   ```
3. **Coding Standards**:
   - Rust 2024 edition, 2-space indentation, no literal tabs, no trailing whitespace.
   - Code changes require a single `x.y.z` version bump in `VERSION` and `Cargo.toml`.
   - See [CONTRIBUTING.md](CONTRIBUTING.md) for full pull request and review guidelines.

---

## 📜 License

- **Source Code**: Licensed under the [MIT License](LICENSE).
- **Legacy Graphics**: Original 2D graphics imported from ChaosForge under attribution in `assets/legacy/drl/graphics/LICENSE`.
