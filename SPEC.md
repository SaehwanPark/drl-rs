# Specification

Last reviewed: 2026-08-31
Current project version: `0.2.326`
Audited starting checkpoint: `main` at
`7735d47` (merged PR #440; M13 JSON compatibility reconciled)
Delivery checkpoint: pending final implementation/review merge

The [Roadmap](docs/DRL-RS_Project_Roadmap.md) owns milestone scope, ordering,
and progress. [`docs/steering/current-priorities.md`](docs/steering/current-priorities.md)
constrains slice selection while its stop gates remain open. This file expands
**exactly one active implementation slice**. Delivered history belongs in the
roadmap, changelog, evidence notes, and Git rather than accumulating here.

## 1. Status vocabulary

- `[x]` — **Delivered and verified**: supported by checked implementation and
  evidence.
- `[ ]` — **Open**: required by the active slice and not yet delivered.
- `NOT_RUN` — **Environment unavailable**: prerequisites were unavailable; no
  pass or failure is inferred.
- `INCONCLUSIVE` — **Evidence unresolved**: available evidence cannot support
  the claim.

## 2. Active implementation slice: M13/M6 — replay-file verification CLI

Slice status: **implemented and locally verified**; final review and merge are
pending.

### 2.1 Objective

Add a native `drl-rs replay verify [path|-]` command that reads the canonical
`drl-rs-replay-v2` JSON envelope from a file or standard input, decodes it
through the public MCP replay decoder, and verifies the replay twice with
`ReplayEngine::verify_determinism`.

This is a bounded replay-file IO slice. It changes no replay schema, game
rules, RNG sampling, content catalog, MCP transport, browser behavior, or
presentation boundary.

### 2.2 Audited starting point

At audited starting revision `7735d47` (version `0.2.325`):

- `drl-mcp::replay_json::from_json_value` already decodes and safety-checks the
  exact V2 envelope, while `ReplayEngine::verify_determinism` already performs
  two independent current-engine executions.
- `drl-app` supported the demo, cohort, and MCP dispatch paths but had no
  filesystem or stdin replay verification command.
- Replay-file migration, cross-version interchange, and transport reconnect
  remained explicitly open.

### 2.3 Scope and ownership

- **Roadmap:** M13 tooling and M6 replay interface completion.
- **Primary owner:** `crates/drl-app/src/replay_cli.rs` owns argument parsing,
  file/stdin reads, diagnostics, and process-facing errors.
- **Decoder/execution:** `drl_mcp::replay_json::from_json_value` and
  `drl_core::ReplayEngine::verify_determinism` remain the sole semantic owners.
- **Project version:** implementation advances `VERSION` from `0.2.325` to
  `0.2.326`.
- **Gameplay/replay semantics:** no schema, command, RNG, generator, ruleset,
  or content identity changes.

### 2.4 Review and branch contract

- The only accepted input format is the canonical V2 JSON envelope; unknown
  JSON properties retain the decoder's existing tolerance.
- A path names a UTF-8 file; `-` reads all UTF-8 input from stdin. Missing,
  unreadable, malformed, unsafe, incompatible, or execution-invalid input
  fails closed with a deterministic diagnostic and non-zero status.
- Successful verification emits byte-identical output across repeated runs and
  across file/stdin sources.
- No filesystem, process, or stream concerns enter `drl-core`.

### 2.5 Acceptance criteria

- [x] `drl-rs replay verify [path|-]` accepts a valid canonical V2 replay from
  both a file and stdin.
- [x] Malformed JSON, unsafe dimensions/containers, and incompatible metadata
  fail before replay execution with stable diagnostics.
- [x] Repeated verification and file/stdin verification produce identical
  success output; failures return a non-zero process status.
- [x] Focused CLI tests, formatting, clippy, repository gate, web contracts,
  and version transition pass on the final revision.
- [ ] An attributable independent determinism-review receipt covers the exact
  final implementation commit; hosted Repository/WASM checks pass. Any
  sole-maintainer Review policy failure is recorded truthfully with the live
  documented exception.

### 2.6 Non-goals

- No replay migration, legacy V1/V2 translation, network replay IO, or broad
  external-client interchange claim.
- No changes to gameplay semantics, replay metadata identities, JSON schema,
  MCP lifecycle, browser persistence, or presentation.
- No claim of browser, audiovisual, human, legacy-runtime, or cross-version
  replay acceptance.

### 2.7 Evidence boundary

The CLI proves current-Rust decoding and deterministic verification for a
caller-supplied canonical V2 replay. It does not prove migration,
cross-version compatibility, arbitrary external replay interchange, or
browser, human, audiovisual, and legacy-runtime behavior; those surfaces
remain open or `NOT_RUN` in the roadmap.

## 3. Enduring invariants

The active slice must preserve:

1. no ambient state, platform APIs, filesystem, browser, or presentation policy
   in `drl-core`;
2. identical declared seed, commands, and semantics produce identical current
   simulation results;
3. incompatible histories fail explicitly before simulation;
4. rejected commands and rejected restores do not partially mutate authoritative
   simulation state;
5. renderers, browser code, MCP, and bots consume fair observations/events and
   do not inspect hidden core state;
6. presentation timing and storage side effects do not advance gameplay;
7. no runtime Lua or generic callback recreation;
8. current-Rust, cross-version, legacy, browser, audiovisual, and performance
   evidence remain separately labeled.
