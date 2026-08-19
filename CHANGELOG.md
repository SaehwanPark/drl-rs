# Changelog

All notable contributor- and user-visible changes to DRL-Rust will be
documented in this file.

## Unreleased

### Added

- Multi-crate Cargo workspace managing `drl-core`, `drl-protocol`, `drl-app`,
  `drl-script`, `drl-mcp`, `drl-render`, and `drl-audio`.
- Deterministic headless simulation core library (`drl-core`) and shared
  protocol contract library (`drl-protocol`).
- Default workspace application executable (`drl-app` / `drl-rust`).
- Automated architectural boundary tests ensuring `drl-core` and `drl-protocol`
  remain free of presentation, audio, and MCP dependencies.
- A repo-local milestone-delivery harness with durable repository guidance.
- A staged development and test-play team contract with explicit ownership,
  deterministic handoffs, and bounded delegation.
- Reusable legacy-archaeology, capability-gated test-play, and independent
  determinism-review skills.
- Repository checks for skill structure, required harness paths, and handoff
  and result-status vocabulary.
- Lightweight specification, architecture, and changelog documents governed by
  the canonical project roadmap.
- Dependency-light two-space formatting checks shared by local development and
  macOS CI.
- Contributor-facing README guidance for the current scaffold, project
  direction, legacy research setup, and licensing boundaries.
