# Project audit and feedback

## Executive assessment

**The deterministic foundation is strong, but the project is not yet release-ready.**
This audit confirmed three actionable issues: hidden item-state disclosure through
player observations, unbounded MCP JSON input that can abort the server, and a
version-policy gap for executable JavaScript modules. Fix these before expanding
content breadth. No Critical or High finding was established in this review.

The existing separation of simulation, protocol, presentation, and platform code
is worth preserving. The larger delivery risk is mistaking extensive per-weapon
regression coverage for complete gameplay, product, or legacy acceptance.

## Scope and confidence

- **Audit date:** 2026-09-07.
- **Revision:** `c9af78f2baa230798eb20cff1b71d30b7b42b6fc` on the initially clean checkout.
- **Project version:** `0.2.348`.
- **Environment:** Darwin arm64; Rust `1.98.0`; Node `26.8.1`.
- **Method:** Risk-based source and contract inspection, local checks, and isolated
  reproductions. This is a sampled project audit, not an exhaustive review of every
  file, a penetration test, or an independent legacy-parity certification.
- **Inspected areas:** Steering/specification and roadmap status; workspace/CI;
  core transactions, RNG, replay validation and observations; MCP parsing and
  transport; browser sessions, persistence and offline caching; version enforcement.
- **Independence:** Subagent delegation was attempted but unavailable
  (`Fabric root is not attached`). Findings were investigated directly, without a
  separate reviewer sign-off.
- **Changes:** This report only. No application code, version, roadmap, or steering
  status was changed. This document is audit evidence, not a replacement plan.

## Confirmed findings

### F1 — Medium: Player observations expose live ground items outside current FOV

**Locations:** `crates/drl-core/src/world.rs:621–629`;
`crates/drl-mcp/src/session.rs:720–727`.

**Trigger and impact:** A tile was explored, is no longer visible, and its ground
items change. `create_player_observation` filters current ground items by
`explored_tiles`, not current visibility or a last-seen item snapshot. Consequently,
new items, removals, and changed item data in fog become observable immediately.
MCP serializes these entries without another visibility filter. This violates the
fair-information boundary and can give an agent information unavailable visually.

**Reproduction:** An isolated Rust probe linked against the local built crates:

1. Created a `40×16` game with the player at `(4,8)`.
2. Moved the player through the fixture API to `(30,8)` and updated visibility.
3. Recorded the observation, then spawned a Small MedPack at `(4,8)`.
4. Recorded the observation again without revisiting that tile.

Observed output:

```text
hidden_tile: position=(4,8), is_visible=false
before_items=0
after_items=1, including the new Small MedPack at (4,8)
```

The fixture directly establishes the projection defect; it does not claim a
recorded human-play exploit. Gameplay already has splash/death-drop and ground-item
destruction routes that make off-screen changes relevant. The browser renderer
filters ground-item sprites by visible tiles (`crates/drl-render/src/lib.rs:1016–1022`),
but that does not repair the upstream observation or MCP disclosure.

**Smallest safe correction:** Return live ground items only for currently visible
positions. If remembered items are a product requirement, store last-seen item
views and update them only while visible; do not derive memory from live hidden
state. Review the analogous live terrain projection at `world.rs:605–608` when
choosing the memory contract.

**Acceptance:** Two worlds with identical visible state and observation memory,
but different hidden items, must yield identical fair observations and MCP JSON.
Cover addition, removal, and count changes, followed by legitimate reveal when
visibility returns. Assess gameplay/replay semantics impact explicitly because
agent decisions consume these observations.

### F2 — Medium: A small deeply nested MCP request aborts the server

**Locations:** `crates/drl-mcp/src/server.rs:309–330`;
`crates/drl-mcp/src/protocol.rs:49–50`;
`crates/drl-mcp/src/json.rs:129–138`.

**Trigger and impact:** Stdio reads an unlimited line and parses it recursively
through `JsonValue::parse`, which selects `usize::MAX` nesting depth. A client can
terminate the MCP process before lifecycle or tool validation. This loses the
active in-memory session. The observed exposure is local stdio, not a demonstrated
remote network service.

**Reproduction:** Ran the local `target/debug/drl-rs --mcp` in a disposable child
process, with core dumps disabled, using this request construction:

```python
request = (
  '{"jsonrpc":"2.0","id":1,"method":"ping","params":'
  + '[' * 20000 + '0' + ']' * 20000 + '}\n'
)
```

The **40,052-byte** request exited with signal 6 (`returncode=-6`):

```text
thread 'main' ... has overflowed its stack
fatal runtime error: stack overflow, aborting
```

A depth-32 control request returned a normal ping response. The replay-file CLI
already demonstrates the safer boundary pattern: an input-size cap and depth 64
(`crates/drl-app/src/replay_cli.rs:9–10,40–65`).

**Smallest safe correction:** Establish documented byte and nesting limits for all
external MCP request entry points, including single requests and batches. Enforce
bytes while reading, not only after allocating the entire line. Reuse
`parse_with_limits`; avoid reparsing with an unlimited entry point. Define whether
an oversized frame is drained safely or closes the connection.

**Acceptance:** Boundary and over-limit fixtures must return controlled errors or
an intentional transport close, never abort or mutate the session. Include a
valid request after a rejected frame when recovery is supported, plus a bounded
batch policy. Add a subprocess regression test so stack aborts cannot kill the
entire test runner.

### F3 — Medium: Executable `.mjs` changes bypass required version increments

**Location:** `scripts/check-version.sh:49–64`.

**Trigger and impact:** A change confined to an executable `.mjs` file is classified
as non-code because the suffix allowlist includes `.js` but not `.mjs`. The
repository uses `.mjs` for shipped browser behavior, including
`web/browser-support.mjs` and `web/offline-cache.mjs`. Such changes can pass with no
version increment; conversely, adding the required increment can be rejected as a
documentation/settings-only bump.

**Reproduction:** Copied the actual checker into a temporary Git fixture with
version `0.1.0`, committed `web/module.mjs`, then changed its exported boolean
without changing the version. Running the checker against that base returned:

```text
Version contract: PASS (0.1.0)
```

The checker is invoked indirectly by `scripts/check-agent-harness.sh:229`, so this
is a classification defect, not a missing CI invocation.

**Smallest safe correction:** Add `.mjs` to the code-path classifier. Add positive
and negative fixtures for runtime JavaScript modules, ordinary Rust/shell code,
and documentation/settings-only changes. Validate both required and forbidden
version transitions.

**Acceptance:** `.mjs` behavior changes without a bump fail; exactly one permitted
version transition passes; documentation-only changes still require no bump.

## Strengths to preserve

- **Clear simulation ownership.** `Game::step` owns the rollback snapshot and
  restores it on `Err` (`crates/drl-core/src/game.rs:220–235`). Browser submission
  records accepted commands without another outer game snapshot. Do not remove
  that safety net solely to simplify code or improve an unmeasured benchmark.
- **Explicit reproducibility contracts.** RNG rejection sampling and golden
  vectors are present. Replay validation checks gameplay, RNG, ruleset, and
  applicable generator identities before execution. These are materially stronger
  guarantees than same-seed tests alone.
- **Safe browser restore structure.** V3 snapshots are bounded and semantics-bound;
  restore builds a replacement session and assigns it only after successful replay.
  Tests cover incompatible identities and late rejection.
- **Substantial automated evidence.** The focused core/MCP/browser library run
  passed 381 tests. Repository script contracts cover provenance, release rights,
  signing, MCP lifecycle, notifications, batches, and transaction benchmarks.
- **Platform boundaries and honest non-goals.** Ten workspace crates keep the
  native preview separate from browser productization. The roadmap explicitly
  retains native interactive, audiovisual, accessibility, and release acceptance
  gaps rather than treating compile success as product completion.

## Delivery and maintainability feedback

These are recommendations and known risks, not additional reproduced defects.

1. **Resolve the review-policy operating model before relying on it as enforcement.**
   `docs/steering/current-priorities.md` records that PR #467's Review policy check
   failed closed and the change merged using `enforce_admins=false`. The documented
   exception is transparent, but a required check that is routinely bypassed is not
   an effective independent acceptance gate. Establish an eligible reviewer path or
   an explicit, bounded exception process with attributable evidence and an expiry.
   Live GitHub settings were not queried during this audit.
2. **Prefer complete behavior branches and acceptance closure to further counters.**
   Preserve the existing steering rule. Select the next fidelity branch by player
   impact and available evidence, not by the number of small weapon-specific PRs.
   Keep definition-covered, behavior-covered, and legacy-compared claims separate.
3. **Reduce test navigation cost without weakening coverage.**
   `crates/drl-mcp/src/session.rs` is 10,369 lines, with its test module beginning at
   line 1,841; it is not a 10,000-line production implementation. Move tests into
   focused modules and consolidate genuinely identical plateau vectors while
   retaining independent cross-boundary assertions. Use the existing `drl-web`
   split as a local precedent, not a reason for a broad rewrite.
4. **Make release gaps visible at the product entry point.**
   M12/M13 still list production signing-key custody, dynamic screen-reader/WCAG
   acceptance, production HTTPS/offline-install acceptance, audiovisual parity,
   and external MCP compatibility as open. Keep a compact, linked release-readiness
   summary instead of duplicating another checklist. Also correct README's
   “9 focused crates” description: the manifest and diagram contain ten.
5. **Measure verification turnaround.**
   The full repository check exceeded this audit's 20-minute allowance, despite
   the later focused library run completing successfully. Record build, script,
   and test durations separately before introducing sharding or removing tests.
   This single run does not establish a CI performance regression.

## Verification ledger

| Check | Audit result |
| --- | --- |
| `verify_code(mode="all")` | Tool/configuration mismatch: selected `go test ./...` in this Rust workspace; not a Rust failure |
| `sh scripts/check-repository.sh` | **INCOMPLETE**: timed out at 1,200 seconds during workspace integration tests |
| Script checks preceding compilation | PASS, except reference-capture preflight explicitly `NOT_RUN` |
| Repository formatting and all-target/all-feature Clippy | PASS before the full-check timeout |
| `cargo test --locked -p drl-core -p drl-mcp -p drl-web --lib` | PASS: core 197, MCP 83, browser 101; 381 total |
| `sh scripts/check-version.sh` | PASS for current manifest consistency; transition hole reproduced separately |
| Service-worker mocked lifecycle/fetch tests | PASS |
| Browser-support classifier tests | PASS |
| Browser accessibility shell and diagnostics scripts | PASS; not screen-reader or WCAG runtime acceptance |
| Hidden-item observation probe | Confirmed F1 |
| MCP nesting probe | Confirmed F2; normal shallow control passed |
| Temporary Git version-classification fixture | Confirmed F3 |
| Full WASM/browser, interactive native, human play, controlled legacy and audiovisual acceptance | `NOT_RUN` in this audit |
| Remote CI, branch protection, dependency vulnerability database, production deployment | Not independently verified in this audit |

Local detailed logs are `/tmp/drl-project-audit-checks.log` and
`/tmp/drl-project-audit-focused.log`; they are ephemeral, not committed acceptance
artifacts. The full-check log showed no failing test before timeout, but that does
not establish a full-suite pass. Reproduction probes used temporary files/processes
and did not modify application sources.

## Recommended order of work

1. Close **F1** with an explicit fair observation/memory contract and non-disclosure tests.
2. Close **F2** with bounded MCP framing/parsing and subprocess regressions.
3. Close **F3** with executable-module version fixtures.
4. Obtain independent review, rerun the complete repository and applicable browser
   checks, then select one evidence-backed vertical fidelity or named 1.0 acceptance
   slice under the existing steering policy.

Each implementation should follow the repository's version and semantics rules.
Do not reopen broad content/platform scope merely because the focused tests pass.
