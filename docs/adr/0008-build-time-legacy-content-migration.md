# ADR 0008: Build-time migration of legacy content

- Status: Accepted
- Date: 2026-08-21
- Supersedes: [ADR 0005](0005-lua-transitional-strategy.md)

## Context

Runtime Lua was previously considered as a transitional compatibility layer.
Shipping a Lua VM in WASM would expand the security, determinism, bundle, and
licensing surface while preserving legacy implementation machinery the project
is explicitly replacing.

## Decision

Lua and legacy data remain research and conversion inputs only. Conversion
tools may read a pinned legacy revision and emit typed Rust/content tables, but
the browser bundle ships no Lua VM, Lua scripts, or runtime legacy object
model. `drl-core` owns gameplay authority and deterministic RNG. Imported
assets require per-group provenance, license/attribution, and checksums. The
graphics directory is eligible under its recorded CC BY-SA 4.0 terms; audio,
music, and fonts remain excluded until separately cleared. Unknown content is
represented as an explicit migration gap rather than silently inferred.

## Consequences

- `drl-script` is a future conversion/content boundary, not a runtime Lua
  dependency.
- M3 records evidence and stable semantic asset identifiers before M7 wires
  presentation.
- Lua behavior gaps are tracked as evidence/roadmap work and tested through
  Rust commands and observations.
