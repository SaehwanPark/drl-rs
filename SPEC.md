# Specification

Last reviewed: 2026-09-07
Current project version: `0.2.350`

The roadmap owns milestone scope and ordering. This file expands exactly one
active implementation slice; delivered history belongs in the roadmap,
changelog, evidence notes, and Git.

## 1. Status vocabulary

- `[x]` — delivered and verified by checked implementation and evidence.
- `[ ]` — open and required by the active slice.
- `NOT_RUN` — prerequisites unavailable; no pass or failure is inferred.
- `INCONCLUSIVE` — available evidence cannot support the claim.

## 2. Active implementation slice: bounded MCP framing (audit F2)

Slice status: **delivered and verified locally**. This is the second
remediation slice recommended by `docs/project-audit-2026-09-07.md` and is
bounded to external MCP transport and JSON parsing. F1 fair ground-item
observations is delivered in `0af1bed`.

### 2.1 Objective

Prevent malformed, oversized, or deeply nested MCP requests from aborting the
server or mutating the active session. Enforce finite byte and JSON-depth limits
at every external request entry point, bound batch cardinality, and preserve
controlled recovery for rejected frames where the transport remains usable.

### 2.2 Scope and acceptance

- [x] Define documented MCP frame-byte, JSON-depth, and batch-count limits.
- [x] Enforce frame bytes while reading stdio input rather than allocating an
  unbounded line; oversized frames receive a controlled parse error and are
  drained through their newline so the next valid frame can be processed.
- [x] Route single requests, batches, and in-process `handle_request` calls
  through bounded JSON parsing; no external path uses unlimited recursion.
- [x] Reject over-limit batches without executing any member and preserve the
  session/lifecycle state.
- [x] Add shallow boundary, deep nesting, oversized-frame, recovery, and batch
  fixtures, including a subprocess regression against the shipped `--mcp`
  binary proving stack overflow cannot terminate the test runner.
- [x] Preserve valid MCP lifecycle, notification, batch ordering, and tool
  behavior; no gameplay, replay, RNG, observation, or wire-schema semantics
  change.

### 2.3 Transport decision and non-goals

A frame exceeding the byte limit is rejected with a JSON-RPC parse error using a
`null` ID, drained to its newline, and processing continues. An unterminated
oversized frame is drained to EOF and then processing ends. This slice does not
change JSON grammar, tool validation, lifecycle rules, session persistence,
MCP protocol version, or browser/native frontend behavior.

### 2.4 Delivery evidence

- `cargo test --locked -p drl-mcp --lib`: 89 passed, including depth, frame,
  recovery, batch, lifecycle, and existing tool/projection coverage.
- `cargo test --locked -p drl-app --tests`: 15 unit tests and 1 subprocess test
  passed; the shipped `--mcp` binary returned controlled errors for deep input
  and continued to a valid request.
- `cargo clippy --locked -p drl-mcp -p drl-app --all-targets --all-features
  -- -D warnings`: PASS.
- `cargo fmt --all -- --check`, `sh scripts/check-version.sh`,
  `sh scripts/check-spec-structure.sh`, `git diff --check`, and
  `sh scripts/test-mcp-stdio.sh`: PASS.
- `sh scripts/check-repository.sh`: INCOMPLETE; the local run passed all
  preceding contracts and workspace tests through `laser_rifle_chainfire`,
  then exceeded the 30-minute execution allowance. No full-suite pass is
  claimed.
- Read-focused determinism review: owner inspection passes for the producer /
  consumer transport boundary, parser limits, batch rejection, recovery, and
  session non-mutation; no gameplay, replay, RNG, observation, or wire
  semantics were changed. Delegated reviewer attempts were unavailable in the
  agent harness, so no independent sign-off is claimed.
- Unavailable native, browser-capture, controlled-legacy, audiovisual, and
  human acceptance remain `NOT_RUN`.

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
6. presentation timing, resize, scale factor, surface loss, and storage side
   effects do not advance gameplay;
7. no runtime Lua or generic callback recreation;
8. current-Rust, cross-version, legacy, browser, audiovisual, and performance
   evidence remain separately labeled.
