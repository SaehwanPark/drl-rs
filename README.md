# DRL-Rust

DRL-Rust is a ground-up Rust reimplementation of [Doom the Roguelike (DRL)](https://drl.chaosforge.org/),
originally created by Kornel Kisielewicz and [ChaosForge](https://chaosforge.org/).
The product direction is browser-first: a deterministic Rust/WASM game rendered
with WebGPU in desktop Chrome/Edge, with an accessible HTML shell and
gesture-unlocked Web Audio. Headless Rust and MCP remain supported for agents,
replays, and regression testing.

## Current capabilities

- Deterministic simulation:
  - Complete M4 headless game loop with combat, FOV/fog, AI, levels, replay,
    scenarios, bots, batches, inventory, and MCP tooling.
  - Stable tile, item, monster, and standard-level definitions, canonical item
    factories, and table-driven generated item/monster selection with preserved
    RNG boundaries.
  - `drl-core::validate_current_content()` checks current typed definitions,
    roll coverage, level bounds, and special-level catalog ordering without
    claiming legacy parity or balance targets.
  - Rejected ranged attacks validate target legality and range before consuming
    ammo or RNG, rejected equips validate slot eligibility before removing
    inventory items, inventory insertion stages ammunition merges before
    committing capacity changes, drops validate destinations before removing
    items, unequip rejections are covered for empty slots and full backpacks,
    and use rejects non-consumable or missing items atomically; reload rejects
    invalid weapon, full-clip, and no-ammo states atomically; descend rejects
    off-stairs, blocked-terrain, and out-of-bounds commands atomically; melee
    rejects invalid directions and empty targets atomically; missing-item
    Equip/Drop and no-ground-item Pickup reject atomically; focused tests assert
    exact `Game` state preservation for these paths; typed `Invoke` and
    `AltReload` rejections and late death-drop failures in melee, ranged, and
    Subtle Knife paths are covered with the same invariant; ranged out-of-bounds,
    empty-target, no-weapon, and empty-clip rejections are covered as well;
    phase-device failure with no destination and post-game-over commands are
    atomic too; pickup validates out-of-bounds positions before touching ground
    items.
  - Direct player diagonal movement validates the requested destination only:
    a walkable diagonal tile remains enterable when both adjacent cardinal
    tiles are walls. AI fallback movement stays a separate policy, and the
    pinned legacy runtime comparison remains `NOT_RUN`.
  - Monster AI movement follows the pinned bounded candidate order: smoothed
    preferred step, raw retry, horizontal fallback, then vertical fallback;
    all blocked candidates wait rather than searching all neighbors.
  - Fixed-seed cohort reports preserve sample definitions, policy identity,
    aggregate metrics, and per-seed replay evidence for evaluation.
  - Cohort regression math applies explicit win-rate and average-turn
    tolerances without mutating simulation or claiming balance parity.
  - Cohort report validation rejects inconsistent sample/evidence metadata
    before a regression comparison is used.
  - Cohort outcome distributions preserve distinct terminal counts and
    sample-normalized rates without interpreting balance or significance.
  - Compatible cohort comparisons report absolute per-outcome rate deltas
    after integrity validation without adding tolerance or significance claims.
  - Outcome comparisons accept one finite, non-negative per-category rate
    tolerance and expose a deterministic pass/fail gate.
  - Cohort telemetry projections and compatible comparisons expose validated
    shot accuracy, damage, kill, pickup, and item-use totals/rates without
    inferring balance conclusions.
  - Cohort depth projections group validated deepest-level metrics into sorted
    sample buckets and rates without asserting a canonical difficulty curve.
- Versioned delivery:
  - `VERSION` is the canonical `x.y.z` project value (currently `0.2.134`),
    projected into Cargo, MCP, and release manifests; the agent harness rejects
    invalid code-change transitions and ignores document/setting-only diffs.
  - `GameRng::gen_range` uses unbiased rejection sampling over the full `u32`
    domain, with raw, bounded, and probability golden vectors.
  - Core probability rules use exact integer ratios; the floating-point helper
    is an explicit outer conversion.
  - Replay metadata carries gameplay-semantics and ruleset identities, plus a
    separate generator-semantics version for procedural maps; imports reject
    unsupported values before simulation.
  - Rejected `Game::step` commands restore turn, world, and RNG state through a
    bounded transaction guard; focused command-atomicity tests cover the
    invariant.
  - `drl-protocol::ItemSpawnKind::ALL` is the stable representative family
    catalog; `drl-core::item_definition::CURRENT_ITEM_SPAWN_KINDS` aliases it
    for structural validation while balance remains core-owned. The
    core-owned `CURRENT_ITEM_DEFINITIONS` catalog now supplies definition
    lookup and coverage without a second spawn-kind match. The stable
    archetype catalog also owns the loose-ammo count shape used by replay JSON
    decoding.
- Browser and presentation slice:
  - M7 functional checks pass locally and in remote web CI.
  - M8 provides square pixel-grid letterboxing, shared lighting bands, measured
    atlas slots, normalized UVs, renderer-neutral layer/composite plans, fair
    observation-derived presentation, and validated texture-source loading.
  - Native-tested base/mask/emissive rendering includes evidence-backed Green
    Armor, Phase Device, and StairsDown tint boundaries, an emissive lighting
    floor, optional outline-mask compositing, and animation frame
    metadata/selection.
  - The renderer and browser shell expose a fair explored-topology minimap
    projection as a bounded, focusable semantic text grid with deterministic
    ordering and visible actor/player markers; exact legacy minimap parity
    remains staged work.
  - Pure contracts cover effect timing, low-health tone/pulse, explosion marks,
    movement and missile progress, screen-shake fade, particle origins,
    burst directions/range sampling, decal cell/placement/eligibility,
    caller-owned insertion requests, deterministic bounded decal storage,
    stored-pixel decal draw planning with opaque sprite-handle resolution, and
    BrowserSession-to-WebGPU decal consumption without claiming full backend
    fidelity.
- Staged work:
  - Full audiovisual equivalence, broader content migration, OS-level PWA
    installation, production deployment, and support for other browsers remain
    roadmap work; local desktop-Chromium offline navigation/startup and
    Save/Load evidence is recorded in
    [`docs/acceptance/browser-offline-2026-08-23.md`](docs/acceptance/browser-offline-2026-08-23.md).
  - Cohort projections reject impossible telemetry and never carry player
    observations; `drl-rust cohort` emits bounded deterministic,
    machine-readable single-policy reports or a three-policy matrix, while
    balance interpretation remains open.
  - Build-time legacy-content conversion extracts shallow scalar fields with
    pinned-source provenance and explicit nested/function migration gaps for
    beings, item families, terrain cells, and special-level metadata; the
    reviewed evidence-coverage gate checks representative IDs, all 26 indexed
    special levels, complete record ID catalogs, scalar-only fields, explicit
    migration gaps, exact pinned source digests, and a versioned crosswalk
    schema. It never ships a Lua runtime or silently infers behavior.
  - The native `drl-app --mcp` stdio transport has a deterministic JSON-lines
    contract with version-aware initialize negotiation, required object-shaped
    capabilities/client metadata, scalar/null request-ID, method-parameter,
    and typed tool-argument validation, plus an identified initialize/initialized
    lifecycle gate before discovery, gameplay,
    replay/metrics, reset, scenarios, resources, fairness denial, notification
    side effects without responses, and ordered JSON-RPC batches; reconnect and
    full external-client compatibility remain open.
  - Browser saves use a bounded V2 command-count token, accept the shipped V1
    token for transactional replay, and migrate successful V1 restores in place;
    offline Save/Load is verified on the supported local Chromium target, and
    Clear Save now requires an explicit accessible confirmation with a tested
    cancel path. Destructive confirmation remains an action-time acceptance
    step.
  - M9 now includes definition-backed rocket and power-cell ammo families,
    base/rocket/power-battery ammo packs, blue/red armor, pinned med-pack
    descriptions, typed double/combat shotguns, Blaster, Laser Rifle, Missile
    Launcher, Nuclear Plasma Rifle, Nuclear BFG 9000, BFG 10K, BFG 9000,
    Mega Buster, Grammaton Beretta, Frag Shotgun, chainsaw, chaingun,
    plasma-rifle, rocket-launcher, Revenant's Launcher, Railgun, Acid
    Spitter, Null Pointer, Combat Pistol, Assault Shotgun, Plasma Shotgun,
    Jackhammer, Super
    Shotgun, Tristar Blaster, Butcher's Cleaver, Mjollnir, Subtle Knife,
    Trigun, Anti-Freak Jackal, Minigun, Onyx Armor, Phaseshift Armor, and
    Gothic Armor, Malek's Armor, Cybernetic Armor, Necroarmor, Medical
    Powerarmor, Lava Armor, and Shielded Armor families with replay/atlas
    coverage;
    Medical Powerarmor's periodic repair, Subtle Knife's alternate invoke,
    Trigun's confirmed alternate reload/nuke transition, Grammaton's typed
    single/burst/auto fire-mode cycle, Jackhammer's typed burst/single
    fire-mode toggle, Lava Armor's five-tick Lava recharge, and Null Pointer's
    target-dependent on-hit score branch, plus baseline Acid/Lava entered-cell
    contact damage, are
    behavior-covered through typed
    deterministic transitions and events in the headless core;
    resistance, damage-type projection, movement, prepared-slot consumption,
    explosion/map effects,
    other dynamic healing, weapon callbacks/effects, and broader legacy item
    behavior remain staged.
  - The service worker reads only the current generated release cache, so stale
    or unrelated Cache Storage namespaces cannot satisfy offline requests;
    registration bypasses browser HTTP caching for worker updates and reports
    waiting updates without forcing takeover; install/control/reload acceptance
    is verified on the supported local Chromium target.
  - Dynamic browser interaction semantics qualify generated inventory actions,
    keep status announcements in one live region, associate canvas help, and
    provide visible focus/recovery behavior; supported-Chromium runtime DOM
    evidence is recorded in
    [`docs/acceptance/browser-dynamic-dom-2026-08-23.md`](docs/acceptance/browser-dynamic-dom-2026-08-23.md),
    while WCAG AA and screen-reader acceptance remain open.
  - Browser startup classifies insecure contexts and missing WebGPU before
    initialization, with stable recovery guidance; other browsers/backends
    remain explicitly unclaimed.
  - Release builds emit a hashed static-bundle manifest with graphics rights
    metadata; optional detached manifest signing now has ephemeral-key hosted
    CI smoke coverage plus local release-tree/symlink/permission hygiene, while
    production key custody, OS-level PWA installation, and cross-browser
    acceptance remain open.
  - `docs/release-rights.md` and `scripts/check-release-rights.sh` keep the
    bundled graphics evidence and excluded legacy code/audio/music/font/WAD
    boundary explicit; missing bundle builds are `NOT_RUN`, not inferred
    releases or legal clearance.
  - Placeholder M7 atlas rectangles are not a fidelity claim.

## Quick start

### Headless and MCP

```sh
cargo run -p drl-app
cargo run -p drl-app -- --mcp
cargo run -p drl-app -- cohort --seed 12 --episodes 1000 --max-turns 200 --bot greedy
```

The `cohort` command emits stable line-oriented outcome, telemetry, and
deepest-level fields for reproducible descriptive studies; it does not claim
balance, a canonical difficulty curve, or statistical significance.

The MCP semantic tool suite exports a complete deterministic in-memory V1
replay envelope through `game_save_replay`, including initial-state metadata
and typed semantic commands. `game_verify_replay` verifies either the active
session replay or a supplied canonical envelope read-only, while
`game_load_replay` transactionally restores that exact V1 envelope into a
session, retains it for appended commands, and reruns it deterministically on
reset. MCP-created envelopes use bounded session dimensions and generator
parameters enforced at `game_start`; replay-file IO, migrations, cross-version
schemas, and external replay interchange remain open.
Numeric `game_step_action` coordinates and item IDs are validated as exact
bounded values before simulation dispatch, and `tools/list` publishes the
canonical alias spellings, enum domains, numeric ranges, and conditional
action requirements without rejecting unknown fields; the
`command`-without-`action` compatibility form is included in the discriminator,
while external-client certification remains open. `tools/list` and
`resources/list` use deterministic fixed-size pages (4 tools, 2 resources)
with method-scoped opaque cursors; invalid cursors return `-32602`.
Recognized `tools/call` runtime failures use successful MCP results with
`isError: true` and deterministic `data.code`/`data.message` details, while
malformed envelopes/arguments and unknown methods/tools remain JSON-RPC errors.
After a terminal victory, death, turn limit, or stall, further actions are
rejected without changing metrics or replay; reset remains available.
The published `game_step_action` schema now conditionally describes its
direction, coordinate-alias, item, and slot requirements while retaining
unknown property tolerance. `game_list_actions` also advertises explicit
unequip and adjacent melee commands. Its fair-observation candidates are
filtered through cloned core probes, so `game_list_actions`, returned
`legal_actions`, and pre-dispatch admission agree on commands accepted by the
current simulation without exposing hidden state; the core still owns
geometry/LOS/range and all other rules. Hidden-state search, unbounded
candidate generation, and external-client certification remain open.

### Browser slice

Prerequisites: Rust, the `wasm32-unknown-unknown` target, and `wasm-pack
0.15.0`.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0 --locked
scripts/build-web.sh
scripts/serve-web.sh
```

Open `http://localhost:8080` over the local static server, press Start to
unlock audio/WebGPU, and focus the canvas. Arrows/WASD and numpad move;
Space/`.` waits; G picks up; R reloads; F selects the nearest visible enemy,
Enter fires, and Escape cancels; `>` descends. Numpad 7/9/1/3 are the
documented diagonal bindings. Inventory controls are exposed through the
semantic DOM region.

If WebGPU is unavailable, the page shows an explicit unsupported-device
status. Audio may remain suspended until a trusted user gesture; that state is
recoverable and never advances the game.

## Verification

```sh
sh scripts/check-repository.sh
sh scripts/check-assets.sh
scripts/check-version.sh
scripts/check-web.sh
scripts/check-browser-diagnostics.sh # also run by check-web.sh
scripts/check-browser-accessibility.sh # also run by check-web.sh
scripts/check-reference-capture.sh
scripts/check-release-manifest.sh  # after scripts/build-web.sh
```

`check-web.sh` compiles the WASM target and runs native contract tests. It runs
headless Chrome WASM tests when Chrome is installed; otherwise it reports the
browser run as `NOT_RUN`. Remote Ubuntu CI owns the required web-CI evidence.

## Architecture

```text
DOM / keyboard -> drl-protocol::Command -> drl-core::Game
               -> PlayerObservation + GameEvent
               -> drl-render / drl-audio -> WebGPU canvas / Web Audio
```

The core has no rendering, audio, browser, filesystem, network, or MCP
dependency. Frontends consume fair player observations, never `World`.
`drl-assets` contains platform-neutral semantic atlas descriptors and licensed
legacy graphics metadata; it is not a dependency of the core.

## Workspace layout

- `crates/drl-core`: deterministic simulation, combat, FOV, AI, levels, items,
  scenarios, bots, batches, and replays.
- `crates/drl-protocol`: commands, observations, events, identifiers, and
  compatibility-sensitive MCP/replay contracts.
- `crates/drl-assets`:
  - Atlas IDs/dimensions, measured rectangles, registered layers and shader
    roles, normalized UVs, texture-source bindings, and semantic asset mapping.
  - Pinned legacy revision identity and licensing metadata.
- `crates/drl-render`:
  - Pure scene construction, pixel viewport layout, layer/composite plans,
    lighting, animation selection, and observed tint mappings.
  - Source-derived contracts for health tone/pulse, effect and missile timing,
    screen shake, particle origins, decal placement/eligibility/insertion,
    bounded decal storage, and post-process glow/LUT math.
  - Renderer/backend and full audiovisual equivalence remain staged work.
- `crates/drl-audio`: semantic cues and WASM Web Audio mixer.
- `crates/drl-web`:
  - Browser session, fair observation boundary, validated texture loading,
    renderer-owned WebGPU uploads, and the partial textured pass.
  - Animation playback, bounded scheduling, fixed-session snapshots,
    best-effort localStorage, bounded rejected-save quarantine,
    generated-bundle service-worker cache, and the project-version/source-
    revision cache policy and manifest digest sidecar recorded by release
    manifests, with a mocked service-worker lifecycle contract and source-
    identity audit.
  - Local accessible browser-support/startup diagnostics with recovery guidance;
    pure startup capability classification for insecure contexts and missing
    WebGPU; no telemetry or untested-browser support claim.
  - Static shell accessibility audit for names, labels, focus, and live regions;
    dynamic WCAG/screen-reader acceptance remains open.
- `crates/drl-mcp`: JSON-RPC/MCP server and fairness boundary.
- `crates/drl-app`: native headless demo and MCP stdio runner.
- `docs/DRL-Rust_Project_Roadmap.md`: canonical milestones and gates.
- `SPEC.md`: the one active implementation slice.
- `ARCHITECTURE.md`, `CHANGELOG.md`, `docs/adr/`, `docs/legacy-behavior/`, and
  `docs/reference-captures/`, `docs/harness/`: verified structure, history,
  decisions, evidence, and agent workflow.

## Acknowledgements and credits

- **Original Game**: [DRL](https://drl.chaosforge.org/) (formerly *Doom, the Roguelike*)
  was created by Kornel Kisielewicz and developed by [ChaosForge](https://chaosforge.org/).
  The upstream open-source FreePascal codebase is hosted on GitHub at
  [ChaosForge/drl](https://github.com/chaosforgeorg/drl).
- **Art & Sprites**: Original sprite art and tiles (0.9.9.7) were created by
  Derek Yu (CC BY-SA 4.0). Additions and modifications (0.9.9.8+) were created
  by Łukasz Śliwiński (CC BY-SA 4.0).
- **Spiritual Successors**: ChaosForge has since created the modern 3D
  spiritual successor [Jupiter Hell](https://store.steampowered.com/app/811320/Jupiter_Hell/)
  and [Jupiter Hell Classic](https://store.steampowered.com/app/3126530/Jupiter_Hell_Classic/).

## Legacy assets and licensing

The imported graphics under `assets/legacy/drl/graphics/` come from the pinned
legacy Git revision recorded in `MANIFEST.txt` and `SHA256SUMS`, with the
upstream CC BY-SA 4.0 license and attribution. The repository's MIT license
does not relicense them. Legacy code is GPL; audio, music, and fonts are not
bundled until their separate redistribution rights are recorded. See
`docs/legacy-behavior/asset-provenance.md`, `docs/release-rights.md` for the
machine-checkable inventory and bundle gate, and
`docs/reference-captures/manifest.md`, which records checkout dirty-state and
evidence classification, rights, and media hashes while keeping capture
promotion gated on a clean controlled checkout with directly observed evidence.

### Downloading original assets

This repository tracks only the 32 CC BY-SA 4.0 graphics sprite sheets in
`assets/legacy/drl/graphics/`. Untracked assets such as sound effects, music,
fonts, and WAD packages can be downloaded from the original sources:

- **Official binary downloads (audio, music, WADs)**:
  1. Visit the ChaosForge downloads page at
     [https://drl.chaosforge.org/downloads](https://drl.chaosforge.org/downloads).
  2. Download the official game release archive for your platform (Windows, Linux,
     or macOS) along with the optional MP3 music pack and HQ sound pack.
  3. Extract the downloaded archives to locate:
     - Sound effects: `sound/` and `soundhq/` (or `data/drlhq/sounds/` and
       `data/drllq/sounds/`).
     - Music: `music/` (MIDI) and `mp3/` (HQ audio), or `data/drlhq/music/`.
     - Data packages: `drl.wad` and `core.wad`.
- **GitHub repository (source, data, and definitions)**:
  1. Clone the upstream repository:
     ```sh
     git clone https://github.com/chaosforgeorg/drl.git
     ```
  2. Lua gameplay scripts, definitions, and raw data are located under
     `bin/data/` (`drl/`, `drlhq/`, and `drllq/`).
  3. The Valkyrie engine source is available at
     [https://github.com/ChaosForge/fpcvalkyrie](https://github.com/ChaosForge/fpcvalkyrie).

## Contributing

Read `AGENTS.md`, `CONTRIBUTING.md`, the active `SPEC.md`, and the roadmap
before changing a milestone. Preserve deterministic headless behavior and run
`sh scripts/check-repository.sh`. Browser changes also need WASM/build
evidence, browser metadata, and an explicit statement of any unavailable
WebGPU/audio/reference-capture checks.
