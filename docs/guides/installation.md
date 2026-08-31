---
title: "Installation & Launch"
description: "How to install prerequisites, build drl-rs from source, and run CLI, MCP, and Web targets."
---

# Installation & Launch Guide

**drl-rs** is written in pure Rust (2024 edition) and designed to build and run reliably across macOS, Linux, and Windows.

---

## Prerequisites

1. **Rust Toolchain**: Install stable Rust (version 1.85+ with Rust 2024 edition support) via [rustup.rs](https://rustup.rs):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **WASM Target** (for building the browser edition):
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
3. **wasm-pack** (for browser bundling):
   ```bash
   cargo install wasm-pack --version 0.15.0 --locked
   ```
4. **Node.js** (for running browser/service worker regression tests):
   Node.js v18 or later.

---

## Building & Running the CLI

### Running the Headless Application
To launch the default demo application which validates headless simulation, bot runs, batch execution, and MCP interfaces:
```bash
cargo run -p drl-app --bin drl-rs
```

### Running Procedural Cohort Studies
The `cohort` subcommand runs batch procedural simulations with deterministic evaluation bots:
```bash
# Run 100 episodes with the Greedy combat bot
cargo run -p drl-app --bin drl-rs -- cohort --seed 42 --episodes 100 --bot greedy

# Run with all bots (greedy, random, explorer)
cargo run -p drl-app --bin drl-rs -- cohort --seed 42 --episodes 50 --bot all
```

---

## Running the MCP Server for AI Agents

To launch the Model Context Protocol (MCP) server over stdio for automated agent integration:
```bash
cargo run -p drl-app --bin drl-rs -- --mcp
```
Configure your MCP host (such as Claude Desktop or Antigravity) with:
```json
{
  "mcpServers": {
    "drl-rs": {
      "command": "cargo",
      "args": ["run", "-q", "-p", "drl-app", "--bin", "drl-rs", "--", "--mcp"]
    }
  }
}
```

---

## Building & Serving the Web Edition

### Building the Web Bundle
To build the WebAssembly bundle and package release assets:
```bash
sh scripts/build-web.sh
```

### Serving Locally
To start a lightweight local HTTP server and play in your browser:
```bash
sh scripts/serve-web.sh
```
Open [http://127.0.0.1:8000](http://127.0.0.1:8000) in desktop Google Chrome, Microsoft Edge, or another Chromium browser with WebGPU enabled.

---

## Preparing Sound, Music & Font Assets (Optional)

The repository includes license-cleared 2D pixel-art graphics. Sound effects, music tracks, and bitmap fonts are approved for in-game use but are excluded from repository tracking and release binaries.

### For Players & External Contributors
Download official DRL game binaries directly from [https://drl.chaosforge.org/](https://drl.chaosforge.org/):
1. Download an official game release package (e.g. `doomrl-win-0997.zip` or `doomrl-linux-x64-0.10.0.tar.gz`).
2. Extract the archive locally.
3. Import the assets using the preparation script:
   ```bash
   sh scripts/prepare-legacy-assets.sh /path/to/extracted-doomrl-folder
   ```

### For Internal & Local Development
If you have the pre-downloaded legacy repository at `../doom-the-roughlike-original`:
```bash
sh scripts/prepare-legacy-assets.sh
```

---

## Running Quality Checks

Ensure the repository passes all formatting, linter, determinism, and integration contracts:
```bash
sh scripts/check-repository.sh
sh scripts/check-assets.sh
sh scripts/check-web.sh
```
