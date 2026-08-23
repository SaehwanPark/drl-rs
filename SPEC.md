# Specification

Last reviewed: 2026-08-23
Current project version: `0.2.59`

The [Roadmap](docs/DRL-Rust_Project_Roadmap.md) owns overall milestone scope,
ordering, and delivery tracking. This file expands **exactly one active
implementation slice** into observable behavior, pure contracts, acceptance
criteria, and verification boundaries.

---

## 1. Status Vocabulary

- `[x]` — **Delivered and Verified**: Supported by checked repository evidence,
  tests, or artifacts.
- `[ ]` — **Present or Future Work**: Open implementation task or acceptance
  gate.
- `NOT_RUN` — **Environment Unavailable**: Required prerequisites (e.g., Linux
  x86-64 binary or browser WebGPU) were not present; not an inferred pass.
- `INCONCLUSIVE` — **Unresolved Evidence**: Output exists but cannot
  definitively satisfy criteria without additional evidence.

---

## 2. Active Implementation Slice: M6/M13 Tool-Execution Error Results

### 2.1 Scope & Objective

Normalize runtime failures from recognized `tools/call` tools into successful
MCP-shaped results with `isError: true`, while preserving JSON-RPC errors for
malformed envelopes/parameters, unsafe arguments, unknown methods/tools, and
malformed supplied replay JSON. This slice documents a deterministic local
error boundary and does not claim complete external-client MCP compatibility.

### 2.2 Predecessor Foundation (Delivered Slices)

1. **Graphics provenance**:
   - `assets/legacy/drl/graphics/` is pinned to revision
     `17d9be1204751899b2d69d8d3a2dde247bd0cc5c` with copied CC BY-SA 4.0
     license, source manifest, and SHA-256 checksums.
2. **Asset boundary**:
   - `assets/README.md` and `docs/legacy-behavior/asset-provenance.md` keep
     legacy code, audio/music, and fonts out of the browser bundle pending
     separate rights evidence.
3. **Release packaging**:
   - `build-web.sh` copies only the cleared graphics tree into `dist/` and
     emits a release manifest whose rights declaration names the graphics
     license; full artifact and service-worker validation remains separate.
4. **Evidence vocabulary**:
   - Rights and capture uncertainty are recorded as `NOT_RUN`, `INCONCLUSIVE`,
     or `NOT_CLEARED`, never inferred from repository presence alone.
5. **MCP semantic boundary**:
   - `game_list_actions` derives fair candidates from `PlayerObservation`, and
     the 0.2.55 slice filters them through cloned core probes before listing or
     dispatching; replay export and supplied verification remain separate
     deterministic contracts.

### 2.3 Present Slice Acceptance Criteria

- [x] **Runtime result boundary**: Inactive-session, invalid recognized action,
  terminal, permission, and replay execution failures return a successful
  JSON-RPC result with `content[0].type = "text"`, `isError: true`, and
  `data.code`/`data.message`.
- [x] **Protocol error preservation**: Malformed JSON-RPC, malformed or
  non-object `tools/call` params/arguments, unsafe numeric values, malformed
  supplied replay JSON, and unknown methods/tools retain `-32700`, `-32602`,
  or `-32601` JSON-RPC error responses as applicable.
- [x] **State and transport safety**: Runtime failures do not mutate session,
  metrics, or replay state; notifications suppress results, batches preserve
  response order/IDs, and repeated stdio output is byte-identical.
- [x] **Delivered predecessor retained**: Deterministic `tools/list` and
  `resources/list` pagination remains stable with fixed 4/2 pages and
  method-scoped cursors.
- [x] **Explicit non-goals**: No new tools, gameplay, lifecycle, replay
  import/load, transport, deployment, or external-client/schema certification
  claims are made.

### 2.4 Pure Contract

- **Input**: A validated `tools/call` envelope and object-shaped arguments.
- **Output**: Successful tool execution retains `{content, isError: false,
  data}`; recognized runtime failure returns `{content, isError: true, data}`
  where `data` contains the numeric project error code and stable message.
- **Ownership Boundary**:
  - `McpServer` classifies only runtime codes (`SESSION_NOT_ACTIVE`,
    `INVALID_ACTION`, `PERMISSION_DENIED`, `INTERNAL_ERROR`) as MCP tool
    results; `execute_tool` remains the semantic implementation boundary.
  - Tool calls, resource reads, session state, lifecycle transitions, replay
    behavior, registry definitions, and delivered list pagination remain
    unchanged.

---

## 3. Recent Delivered Slices

### M13/M6 — Deterministic List Pagination (`VERSION` 0.2.58)

- [x] `tools/list` and `resources/list` expose stable fixed-size pages (4 and
  2 entries) with method-scoped cursors, deterministic continuation, final-page
  omission, and fail-closed invalid-cursor handling.
- [x] No-params behavior, session state, tool/resource content, lifecycle, and
  broader external MCP compatibility remain unchanged/open.

### M13/M6 — Read-Only Canonical V1 Replay Verification (`VERSION` 0.2.57)

- [x] `game_verify_replay` decodes and verifies supplied canonical V1 replay
  JSON without an active session or mutation; malformed, unsafe, out-of-bounds,
  and type-invalid input returns `-32602`, while replay execution failures use
  the recognized runtime-error result boundary.
- [x] MCP session dimensions and generator parameters are bounded before export;
  session loading, replay-file IO, migration, and external interchange remain
  open.

### M13 — Canonical Complete V1 MCP Replay Export (`VERSION` 0.2.56)

- [x] `game_save_replay` exports every in-memory V1 field and semantic command
  through the deterministic `drl-rust-replay-v1` envelope; supplied replay
  verification now consumes this exact contract without loading a session.
- [x] Import/load, replay-file IO, migration, and external interchange remain
  explicitly open.

### M13 — Exact Fair-Observation Legal-Action Enumeration (`VERSION` 0.2.55)

- [x] Fair candidates are filtered through cloned core probes and the exact
  catalog drives MCP listing, response payloads, and pre-dispatch admission.
- [x] Terminal catalogs become empty, rejected commands remain state-safe, and
  hidden-state search, unbounded generation, and external compatibility remain
  open.

### M13 — Legal-Action Catalog Coherence (`VERSION` 0.2.54)

- [x] Added explicit adjacent `attack_melee` and equipped-slot `unequip`
  entries, then rejected recognized commands not currently advertised before
  live simulation with `-32001`; unknown/malformed input remains `-32602`.
- [x] Preserved state safety, terminal/reset precedence, core geometry/LOS/range
  authority, virtual-player behavior, and deterministic stdio output.

### M13 — Public Release-Rights Inventory (`VERSION` 0.2.53)

- [x] `docs/release-rights.md` records included, excluded, notice-only, and
  unavailable evidence categories without claiming legal clearance.
- [x] Source and optional bundle gates validate pinned graphics provenance,
  exact manifest rights, recursive checksums, symlink/non-regular rejection,
  and excluded legacy code/audio/music/font/WAD boundaries; absent bundles are
  `NOT_RUN`.

### M13 — Conditional `game_step_action` Schema (`VERSION` 0.2.52)

- [x] Added deterministic action/command discriminator and conditional
  direction, ranged-coordinate-alias, item-ID, slot, and no-argument branches.
- [x] Preserved runtime aliases, malformed-input behavior, unknown-property
  tolerance, action precedence, mixed coordinate aliases, and repeated stdio
  output; the catalog/pre-dispatch coherence contract was delivered in 0.2.54,
  while exact fair-observation filtering is delivered in the active 0.2.55
  slice.

### M13 — MCP Terminal-Outcome Gate (`VERSION` 0.2.51)

- [x] Terminal victory, death, turn-limit, and stalled sessions reject later
  actions with `-32001` without mutating metrics or replay; reset remains
  available.
- [x] Stair transitions report `Victory`, and terminal replays remain
  deterministic; core gameplay, replay format, and external compatibility
  claims remain unchanged.

### M13 — Truthful MCP Tool Schemas (`VERSION` 0.2.50)

- [x] `tools/list` publishes action, direction, slot, `command`, and `x`/`y`
  aliases with enum domains and exact numeric bounds for stateful arguments.
- [x] Unknown properties remained tolerated; conditional action schemas,
  gameplay changes, and external-client certification remained open at that
  slice.

### M13 — Typed `game_step_action` Numbers (`VERSION` 0.2.49)

- [x] Ranged coordinates and item IDs reject unsafe or wrong-typed numeric
  arguments before state mutation while preserving valid aliases and actions.
- [x] Added state-safety, direct parser, virtual gameplay, and repeated stdio
  coverage; complete action-schema validation remains open.

### M13 — MCP Replay Verification (`VERSION` 0.2.48)

- [x] Added `game_verify_replay` for state-safe deterministic verification of
  procedural and custom-scenario in-memory replays with command-count/version
  metadata.
- [x] Added inactive-session, repeated-response, virtual-player, and repeated
  real-stdio coverage; replay import/load and external interchange remain open.

### M13 — Tool-Argument Type Validation (`VERSION` 0.2.47)

- [x] Optional integer fields for `game_start` and `game_load_scenario` reject
  wrong types and dimensions outside the accepted 32-bit range with `-32602`
  before state mutation.
- [x] Valid defaults and predecessor method-envelope/request-ID/lifecycle
  contracts remain deterministic; full MCP schema/client compatibility remains
  open.

### M13 — Method-Parameter Envelope Validation (`VERSION` 0.2.46)

- [x] `tools/call` and `resources/read` require object params; tool arguments
  must be an object when present, with malformed calls returning `-32602`
  without state mutation.
- [x] Request-ID, initialize, lifecycle, notification, and batch contracts
  remain deterministic; full MCP schema/client compatibility remains open.

### M13 — JSON-RPC Request-ID Validation (`VERSION` 0.2.45)

- [x] Request IDs accept strings, numbers, and explicit `null`; non-scalar IDs
  return `-32600` without dispatch or state mutation.
- [x] Real stdio notification and explicit-null response behavior remains
  deterministic; initialize envelope and lifecycle contracts remain delivered.

### M13 — Required Initialize Envelope Validation (`VERSION` 0.2.44)

- [x] Initialize requires object params, object capabilities, and clientInfo
  with string name/version; malformed fields return `-32602` without unlock.
- [x] Version negotiation and the predecessor identified lifecycle gate remain
  deterministic; full MCP schema/client compatibility remains open.

### M13 — MCP Lifecycle State Gate (Phase 1) (`VERSION` 0.2.43)

- [x] Identified initialize → initialized transitions gate tools and resources
  behind `Ready`, with deterministic `-32003` pre-ready errors.
- [x] Premature/duplicate/omitted-ID transitions do not advance state; ping,
  malformed input, batch order, and notification suppression remain bounded.
- [x] Game reset remains separate from protocol lifecycle; full external-client
  compatibility and reconnect/resume remain open.

### M13 — Version-Aware JSON-RPC Initialize (`VERSION` 0.2.42)

- [x] Supported `2024-11-05` requests echo the requested version.
- [x] Unsupported strings receive the deterministic supported fallback;
  missing/non-string versions return `-32602`.
- [x] Existing capabilities, server metadata, stdio framing, batch ordering,
  notifications, and tool semantics remain unchanged.
- [ ] Full MCP lifecycle enforcement, external-client compatibility, and
  production deployment remain open.

### M13 — JSON-RPC Batch Stdio Transport (`VERSION` 0.2.41)

- [x] Batch arrays preserve response order, omit notification members, and
  reject empty batches.
- [x] Existing notification, null-ID, malformed-input, and identified-request
  contracts remain covered; full external-client compatibility remains open.

### M10 — Browser Offline Lifecycle Acceptance (evidence baseline)

- [x] A real desktop Chromium run installed the generated service worker,
  reloaded from the current release cache with network requests disabled, and
  started the cached WASM game successfully. The recorded environment and
  boundary are in
  [`docs/acceptance/browser-offline-2026-08-23.md`](docs/acceptance/browser-offline-2026-08-23.md).
- [ ] The test-plan-required offline Clear Save action is awaiting explicit
  action-time confirmation; OS-level PWA installation prompts, production
  HTTPS deployment, other browsers/backends, WCAG/screen-reader behavior,
  audible output, and capture-backed parity remain open.

### M12 — Supported-Chromium Dynamic DOM Acceptance (evidence baseline)

- [x] Runtime evidence covers supported-start focus transfer to the canvas,
  authored focus styling, live status updates after keyboard input, static
  keyboard help, and item-qualified inventory action names. See
  [`docs/acceptance/browser-dynamic-dom-2026-08-23.md`](docs/acceptance/browser-dynamic-dom-2026-08-23.md).
- [ ] WCAG 2.1 AA, screen-reader/assistive-technology, contrast, complete
  keyboard traversal, and broad-browser acceptance remain `NOT_RUN`.

### M13 — JSON-RPC Notification-Correct Stdio (`VERSION` 0.2.40)

- [x] Valid omitted-ID requests mutate session state without emitting a response
  from the stdio transport.
- [x] Identified requests, explicit `id: null`, and malformed-input parse errors
  retain one response each.

### M9 — Rust-Owned Content Invariant Validation (`VERSION` 0.2.39)

- [x] Added a pure validator for current monster, item, loot, level, and
  descriptive special-level table invariants.
- [x] Focused negative tests reject invalid damage ranges, weapon shape,
  roll-bound coverage, and level dimensions/room bounds.
- [ ] Legacy parity, fairness targets, and full dynamic content migration
  remain open.

### M12 — Signing-Key Boundary Hygiene (`VERSION` 0.2.38)

- [x] Signing rejects release-tree keys, symlinks, and group/world-readable
  private keys before OpenSSL writes artifacts.
- [x] Existing valid signing, detached verification, and mutation rejection
  remain covered.
- [ ] Production custody, provisioning, rotation, and trust-root governance
  remain open.

### M13 — Stdio MCP Lifecycle (`VERSION` 0.2.37)

- [x] Added a fixed JSON-lines subprocess fixture covering initialization,
  discovery, gameplay, replay/metrics, reset, scenario loading, and resources.
- [x] Repeated runs are byte-identical and preserve permission denial for the
  default omniscient dev-state request.
- [ ] Full external MCP-client compatibility and production deployment remain
  open.

### M10 — PWA Update Freshness (`VERSION` 0.2.36)

- [x] Registration freshness and waiting-update status are implemented and
  covered by deterministic Node fixtures.
- [x] Real browser offline installation and reload are covered by the later
  evidence baseline; update activation remains open.

### M12 — Browser-Environment Diagnostics (`VERSION` 0.2.35)

- [x] Pure classifier distinguishes insecure contexts from missing WebGPU.
- [x] Start-up guard routes unsupported environments through the focused
  diagnostic panel before WASM initialization.
- [x] Node tests cover both failure classes, supported startup, and stable
  recovery text.
- [x] The supported desktop Chromium target and offline installation are
  covered by the later evidence baseline; broader browser, WCAG, and
  screen-reader acceptance remain open.

### M9 — Representative Content-Evidence Coverage (`VERSION` 0.2.34)

- [x] Added a reviewed crosswalk and validator for pinned being, item-family,
  terrain-cell, and special-level evidence bundles.
- [x] Added provenance, digest-shape, ordering, uniqueness, representative,
  and complete 26-level coverage checks plus fixture rejection cases.
- [x] Synchronized the reviewed 26-level ID list with the Rust descriptive
  `SPECIAL_LEVEL_DEFINITIONS` catalog and rejected fixture drift.
- [x] Compared all descriptive scalar fields against the pinned level evidence
  records without importing maps, callbacks, or level behavior.
- [x] Locked every reviewed source to its exact pinned SHA-256 digest and added
  wrong-digest rejection coverage.
- [x] Pinned complete being, item-family, and terrain-cell record ID catalogs
  and added converter-output catalog drift rejection coverage.
- [x] Enforced scalar-only evidence fields, structured migration gaps, and
  positive source lines without importing nested Lua data.
- [x] Versioned the expanded crosswalk as schema 2 and added obsolete-schema
  rejection coverage.
- [ ] Full typed migration, assets, callbacks, and gameplay parity remain open.

### M10 — Same-Release Offline-Cache Isolation (`VERSION` 0.2.27)

- [x] Service-worker reads now match only the current generated release cache;
  stale/unrelated namespaces cannot satisfy offline requests.
- [x] Node worker contracts cover current-cache hits and fail-closed stale
  isolation while preserving routing and activation behavior.
- [x] Real browser offline install/control/reload acceptance is recorded in
  the later evidence baseline.

### M12 — Dynamic Interaction Accessibility Contract (`VERSION` 0.2.26)

- [x] Added item-qualified escaped inventory actions, one live status channel,
  canvas help association, focus-visible styling, and diagnostic focus parity.
- [x] Added static shell and native markup contract coverage.
- [ ] WCAG 2.1 AA, screen-reader, contrast, and broad browser acceptance remain
  open.

### M11 — Descriptive Cohort Depth Distribution (`VERSION` 0.2.25)

- [x] Added validated, sorted deepest-level buckets and sample rates to cohort
  reports and single-policy/matrix CLI output.
- [x] Added empty-cohort, invalid-evidence, deterministic-order, and stable
  formatting coverage.
- [ ] Canonical difficulty targets and externally approved progression metrics
  remain open.

### M10 — Replay-Compatible Save Migration (`VERSION` 0.2.24)

- [x] Added strict V2 command-count encoding and V1 decode compatibility.
- [x] Migrated successful V1 restores in place while preserving fail-closed
  quarantine and transactional replay.
- [x] Real offline browser lifecycle acceptance is recorded in the later
  evidence baseline.

### M9 — Special-Level Identity Catalog (`VERSION` 0.2.23)

- [x] Added immutable Rust metadata for 26 active level IDs, names, and texts.
- [x] Preserved optional legacy depth values and missing scalar fields.
- [ ] Full typed special-level migration, assets, and behavior validation remain
  open.

### M9 — Special-Level Evidence Index (`VERSION` 0.2.22)

- [x] Added pinned index coverage across 24 level files and 26 active records.
- [x] Added long-bracket map-string handling and explicit dynamic gaps.

### M9 — Item-Family Evidence Bundle (`VERSION` 0.2.21)

- [x] Added multi-source base/expansion/user-item evidence bundling.
- [x] Added source provenance, source indices, sorted merge, duplicate rejection,
  and fixture/pinned coverage.
- [ ] Full typed item-family migration and behavior validation remain open.

### M11 — Cohort Policy Matrix (`VERSION` 0.2.20)

- [x] Added deterministic `--bot all` output for greedy, random, and explorer.
- [x] Preserved shared seed/episode/turn configuration and prefixed report
  fields.
- [x] Added native repeatability and repository contract coverage.
- [ ] Canonical difficulty targets and statistical interpretation remain open.

### M9 — Terrain-Cell Evidence Converter (`VERSION` 0.2.19)

- [x] Added pinned-source extraction for shallow `register_cell` scalar data.
- [x] Added deterministic byte/single-quoted string handling and explicit
  nested/function/symbolic migration gaps.
- [x] Added fixture repeatability and pinned `cells.lua` coverage.
- [ ] Full typed terrain migration, asset mapping, and parity validation remain
  open.

### M12 — Signed-Release CI Smoke (`VERSION` 0.2.18)

- [x] Added repository contract coverage for signing and mutation rejection.
- [x] Added hosted WASM CI coverage with a runner-local ephemeral RSA key.
- [x] Asserted signed artifacts are present and the private key stays outside
  `dist`.
- [ ] Production key custody, rotation, and trust-root governance remain open.

### M9 — Legacy Content Evidence Converter (`VERSION` 0.2.17)

- [x] Added pinned-source extraction for shallow being/item scalar records.
- [x] Added provenance fields and explicit nested/function migration gaps.
- [x] Added deterministic fixture coverage without a Lua runtime.
- [ ] Full content migration and behavior validation remain open.

### M11 — Cohort Study CLI (`VERSION` 0.2.16)

- [x] Added bounded deterministic `drl-rust cohort` reports for three bots.
- [x] Added stable metadata, outcome, and telemetry line fields with validation.
- [x] Added repeatability and invalid-option contract tests.
- [ ] Canonical difficulty targets and statistical interpretation remain open.

### M10 — Offline-Cache Readiness (`VERSION` 0.2.15)

- [x] Started service-worker registration independently of WebGPU startup.
- [x] Added explicit unavailable/installing/ready/failure diagnostics.
- [x] Added injected-capability tests and bootstrap ordering checks.
- [x] Real browser offline installation and reload are recorded in the later
  evidence baseline.

### M12 — Detached Release Signing (`VERSION` 0.2.14)

- [x] Added optional detached manifest signing and public-key derivation.
- [x] Added fail-closed verification when signature artifacts are present.
- [x] Added ephemeral-key shell coverage for tamper rejection.
- [ ] Key custody, CI enforcement, and production trust-root governance remain
  open.

### M11 — Cohort Telemetry Integrity (`VERSION` 0.2.13)

- [x] Added typed telemetry-invariant diagnostics for shot counts, level
  identity, and configured turn budgets.
- [x] Enforced the invariants before outcome or telemetry projection without
  mutating reports or accessing player observations.
- [x] Added focused integration coverage for each rejected metric shape.
- [ ] Automated large-scale balance and canonical difficulty studies remain
  open.

### M10 — Browser-Save Corruption Recovery (`VERSION` 0.2.12)

- [x] Quarantined malformed, unsupported, oversized, and replay-invalid save
  values in a bounded browser-owned diagnostic slot.
- [x] Cleared rejected active values when storage permits and kept boot/load
  playable when storage access or cleanup fails.
- [x] Preserved transactional restore and tested active-session immutability.
- [ ] Explicit replay-compatible migration for recognized older formats.

### M8 — Particle-Decal Renderer Integration (`VERSION` 0.2.11)

- [x] Resolved opaque caller-provided sprite handles through presentation-only
  descriptor tables without guessing a legacy blood atlas or slot.
- [x] Preserved stored-pixel sub-cell placement and inserted decals between
  terrain and ordinary objects in renderer-neutral plans.
- [x] Added browser WebGPU entry points and native ordering/immutability tests;
  capture-backed visual parity remains `NOT_RUN`.

### M8 — Particle-Decal Storage Boundary (`VERSION` 0.2.10)

- [x] Implemented `ParticleDecalStore` with caller-configured maximum capacity.
- [x] Preserved insertion order and duplicate requests deterministically.
- [x] Explicit `CapacityExceeded` error reporting without dropping prior
  entries.
- [x] Comprehensive test coverage for empty, append, duplicate, and capacity
  limits.

### M8 — Particle-Decal Insertion Request (`VERSION` 0.2.9)

- [x] Packaged accepted placement coordinates with caller-provided sprite ID.
- [x] Reused checked placement and eligibility math without duplicate logic.
- [x] Preserved legacy cell, pixel offset, and sprite ID representations.

### M12 — Release Hardening & Integrity (`VERSION` 0.2.1)

- [x] Git checkout-identity binding in release manifest generator.
- [x] Release manifest SHA-256 sidecar (`release-manifest.sha256`).
- [x] Versioned service-worker cache invalidation based on version and commit.
- [x] Mocked service worker lifecycle and fetch contract test harness.
- [x] Static HTML shell accessibility audit (landmarks, focus, live regions).

### M11 — Cohort Telemetry & Outcome Projections (`VERSION` 0.1.1–0.2.3)

- [x] Fixed-seed cohort reports (`CohortConfig`, `CohortReport`) with integrity
  validation.
- [x] Cohort outcome distributions (victory, death, turn limit, stalled, in
  progress).
- [x] Pure outcome-rate and telemetry tolerance gates for regression detection.
- [x] Descriptive telemetry metrics: shot accuracy, damage dealt/taken, kills,
  pickups, and item usage.

---

## 4. Shared Observable Behavior & Invariants

### 4.1 Simulation & Fairness

- **Information Decoupling**: `BrowserSession` and MCP tools expose only fair
  `PlayerObservation` and `GameEvent` streams. Hidden world state is never
  accessible.
- **Replay Determinism**: Identical seed and command sequences produce bit-exact
  identical events and end states across headless, MCP, and browser runners.
- **Transactional Rollback**: Rejected or illegal commands roll back session
  state without advancing the simulation turn or consuming energy.

### 4.2 Browser & WebGPU Presentation

- **Input Translation**: Browser shell maps keyboard/numpad, wait, pickup,
  reload, descend, ranged targeting, and DOM inventory clicks directly to
  protocol `Command`s.
- **No Simulation Side Effects**: Canvas resize, DPR change, animation tick,
  audio playback, tab visibility change, or WebGPU device loss never advance
  simulation time.
- **Square-Cell Viewport**: Integer scaling and centering ensure square game
  tiles without non-uniform axis stretching.
- **Visibility-Derived Lighting**: Full light for tiles in active FOV; fixed fog
  factor for remembered explored tiles. Presentation never queries hidden
  tiles.

### 4.3 Evaluation & Telemetry

- **Contiguous Seeds**: Cohort reports record contiguous seed ranges and
  preserve per-seed replay logs.
- **Descriptive Metrics**: Projections report descriptive counts and rates; they
  do not assert balance conclusions or statistical significance.

### 4.4 Release Packaging

- **Manifest Validation**: Release scripts verify manifest schema, sorted
  SHA-256 hashes, license metadata, worker coverage, sidecar digests, and source
  commit identity.

---

## 5. Public Contracts & Boundaries

### 5.1 Protocol & Simulation (`drl-protocol`, `drl-core`)

- **Domain Types**: `Position`, `Direction`, `Turn`, `EntityId`, `ItemId`,
  `LevelId`, `HitPoints`, `Speed`, `ActionCost`.
- **Commands**: `Command::Move`, `Command::Wait`, `Command::Pickup`,
  `Command::Drop`, `Command::Equip`, `Command::Unequip`, `Command::Reload`,
  `Command::Use`, `Command::AttackMelee`, `Command::AttackRanged`,
  `Command::Descend`.
- **Observations**: `PlayerObservation` (FOV tiles, visible actors, ground
  items, inventory, equipment, player HP, map dimensions).
- **Events**: `GameEvent::AttackResolved`, `GameEvent::DamageApplied`,
  `GameEvent::ActorDied`, `GameEvent::ActorKnockedBack`,
  `GameEvent::PlayerTeleported`, `GameEvent::LevelTransitioned`, etc.

### 5.2 Presentation & Assets (`drl-render`, `drl-assets`, `drl-web`)

- **Layout**: `PixelViewport`, `PixelRect`, integer cell scaling.
- **Shading & Tone**: `LightingBand`, `shade_color`, `SceneTone`,
  `low_health_pulse_target_alpha`, `LowHealthPulseState`.
- **Draw Plans**: `layer_draw_plan`, `sprite_composite_plan`, `LayerRole`,
  `AtlasTextureSource`.
- **Decals & Particles**: `ParticleDecalPlacement`, `ParticleDecalInsertion`,
  `ParticleDecalStore`, burst origins/directions/ranges.

### 5.3 Decoupling Invariants

- `drl-core` and `drl-protocol` have **zero** dependencies on WebGPU, Web
  Audio, DOM, filesystem, network, or MCP crates.

---

## 6. Verification Gates

### Verified Baseline (`VERSION` 0.2.45)

9c0eccf feat(mcp): support deterministic stdio batches (#132)
eeb246d feat(mcp): handle stdio notifications correctly (#131)
1441667 feat(content): validate typed definition invariants (#130)
fbd7eee feat(release): harden signing key inputs (#129)
a127868 test(mcp): cover stdio lifecycle (#128)
cf597d3 feat(web): report fresh service-worker updates (#127)
47f3526 feat(web): classify unsupported browser environments (#126)
d9d78ea feat(content): version evidence crosswalk schema (#125)
a8cda3e feat(content): validate evidence record shape (#124)
e3583de feat(content): pin complete evidence ID catalogs (#123)
75888b4 feat(content): lock pinned evidence source digests (#122)
6aade20 feat(content): synchronize special-level scalar evidence (#121)
3c8d618 docs(spec): record catalog sync baseline
872914c feat(content): synchronize special-level evidence (#120)
4263241 docs(spec): record merged evidence baseline
a3d8fd2 feat(content): add evidence coverage gate (#119)
0d919ec docs(spec): record merged cache isolation baseline
9f0d601 feat(web): isolate service-worker cache reads (#118)
afdc01f feat(web): harden dynamic accessibility semantics (#117)
6024a15 feat(eval): add cohort depth distribution (#116)
cae3b09 feat(web): migrate browser saves to v2 (#115)
d7e5602 feat(content): add special-level metadata catalog (#114)
1fd09d5 feat(content): add pinned special-level evidence index (#113)
d1a8903 feat(content): bundle legacy item family evidence (#112)
34a9578 feat(eval): add deterministic cohort policy matrix (#111)
8486e15 feat(content): add pinned terrain cell evidence conversion (#110)
2a5e149 ci(release): smoke-test ephemeral manifest signing (#109)
3cfe62e feat(content): add pinned legacy content evidence converter (#108)
f305503 feat(eval): add deterministic cohort study CLI (#107)
22e0e8b feat(web): start offline cache registration at bootstrap (#106)
af0fcf8 feat(release): add optional manifest signing (#105)
- [x] `sh scripts/check-repository.sh` — Full repository test suite, formatting,
  clippy, and harness checks.
- [x] `sh scripts/check-assets.sh` — 32 imported legacy PNGs, licensing, and
  SHA-256 checksums.
- [x] `scripts/test-reference-capture.sh` — Preflight fixture test suite for
  capture manifests.
- [x] `sh scripts/check-web.sh` — WASM target compilation and native/WASM web
  contract tests.
- [x] `sh scripts/build-web.sh && sh scripts/check-release-manifest.sh` — Static
  bundle build and 44-artifact manifest validation.
- [x] `scripts/check-version.sh` — Project version consistency and transition
  rules.

### Environment-Gated Verification

- [ ] Controlled Linux x86-64 reference captures (`NOT_RUN` on macOS arm64).
- [ ] Direct visual/audio diff against legacy reference captures (`NOT_RUN`).

---

## 7. Explicit Non-Goals

- **Pre-1.0 Multiplatform**: No WebGL2 fallback, mobile/touch UI, gamepad
  navigation, or native desktop packaging before the 1.0 desktop Chromium
  release.
- **No Runtime Scripting**: No Lua VM or JavaScript gameplay logic. All
  simulation is pure compiled Rust.
- **No Online Services**: No player accounts, multiplayer networking, or
  remote backend servers.
- **No Unattributed Assets**: No bundling of legacy audio, music, or fonts
  without documented redistribution rights.
- **No False Parity Claims**: Pure mathematical contracts and generated audio
  tones are not claimed as full legacy audiovisual equivalence until validated
  against reference captures.
