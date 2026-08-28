# Architecture

Last reviewed: 2026-08-28
Current project version: `0.2.193`

Status: Verified for current deterministic headless core, MCP tooling, and
browser-playable WebGPU slice; full audiovisual parity remains planned.

Near-term architecture corrections and migration constraints are tracked in
[`docs/steering/`](docs/steering/README.md). Steering may identify a documented
invariant as a correction target when audit evidence shows the current
implementation does not yet satisfy it; such a target must not be read as a
verified implementation claim.

---

## 1. Core Architectural Principles

DRL-Rust reimplements *Doom the Roguelike* with modern software engineering
invariants:

- **Functional Core, Imperative Shell**: Pure, deterministic game logic in
  `drl-core`; all side effects (WebGPU, Web Audio, DOM, MCP, I/O) are confined
  to the outer boundary crates (`drl-web`, `drl-audio`, `drl-app`).
- **Strict Determinism & Replayability**: Seedable PRNG (`GameRng`), explicit
  command-driven turn execution, zero ambient state, and bit-exact replay
  verification within a declared compatible gameplay-semantics/ruleset
  boundary. Cross-version archival compatibility requires explicit versioning;
  see the replay/RNG steering decision.
- **Fair Information Boundaries**: Frontends and AI agents consume only fair
  `PlayerObservation` views (active FOV, explored fog memory, visible entities);
  internal `World` state is never exposed to clients.
- **Zero External Dependencies in Core**: `drl-core` and `drl-protocol` are pure
  Rust crates with zero dependencies on WebGPU, Web Audio, DOM, filesystem,
  network, or MCP libraries.
- **No Runtime Scripting**: Lua is treated as build-time reference and
  conversion evidence only; no Lua runtime exists in the WASM browser bundle.

---

## 2. System Boundaries & Data Flow

```text
HTML / DOM UI / Keyboard / MCP Tool Call
  │
  ▼
drl-protocol::Command (semantic input)
  │
  ▼
drl-core::Game::step (deterministic simulation authority)
  │
  ├─► drl-protocol::PlayerObservation (fair FOV/fog/entity state)
  ├─► drl-protocol::GameEvent (ordered simulation events)
  │
  ▼
Presentation Boundary
  ├─► drl-render::RenderScene ──► drl-web WebGPU Canvas
  └─► drl-audio::AudioCue   ──► drl-audio Web Audio Mixer
```

### Data Flow Invariants

1. **Client Input**: All clients (headless tests, MCP agents, browser DOM,
   future native apps) interact with the game exclusively by submitting
   `drl-protocol::Command` values.
2. **Simulation Authority**: `drl-core` is the sole authority for world state,
   action legality, action costs, energy scheduling, PRNG consumption, and
   event emission.
   Direct player diagonal movement validates the requested destination only;
   adjacent-cardinal corner checks are not part of that command path. Monster
   `MoveTowards` uses smoothed preferred, raw retry, horizontal, and vertical
   candidates in a bounded order, then waits when all are blocked; it does not
   perform broad pathfinding.
3. **One-Way Presentation**: Presentation layers (`drl-render`, `drl-audio`,
   `drl-web`) consume observations and events only. Rendering, animation, audio,
   tab visibility, viewport resize, or GPU device loss **never** advance the
   simulation or alter PRNG streams.
4. **Atomic Rejection Contract — Active Correction Gate**: A rejected command
   is required to leave the complete simulation state unchanged, including
   world, inventory/equipment, scheduler/turn state, counters, and RNG state.
   The ranged-attack path validates target legality and range before its
   prepare/commit mutation boundary. The equip path likewise validates item
   existence and equipment-slot eligibility before removing the item from
   inventory, and inventory insertion stages ammunition merges so a capacity
   rejection cannot partially mutate an existing stack. Drop validates its
   destination before removing the inventory item. These paths have focused
   equality tests; the repository-wide command invariant remains an active
   correction target until every command family has equivalent evidence. The
   unequip path rejects empty slots and full inventories before removing an
   equipped item. See
   [`docs/steering/decisions/atomic-command-transactions.md`](docs/steering/decisions/atomic-command-transactions.md).

---

## 3. Workspace Crates

### `drl-protocol` — Shared Semantic Contracts
- **Role**: Stable semantic boundary shared across core, renderers, MCP, and
  frontends. New gameplay balance, content policy, or behavior should not be
  added here merely because a semantic identifier crosses a boundary; see the
  content/behavior steering decision.
- **Key Modules & Types**:
  - Domain primitives: `Position`, `Direction`, `Turn`, `EntityId`, `ItemId`,
    `LevelId`.
  - Commands & Errors: `Command`, `CommandError`.
  - Observations: `PlayerObservation`, `TileView`, `ActorView`, `ItemView`.
  - Events: `GameEvent` stream (combat, movement, items, levels, and typed
    alternate-behavior transitions such as Grammaton fire-mode changes).
  - Stable item identity and normalized replay spawn families use one
    compile-time catalog for the `ItemArchetype`/`ItemSpawnKind` enums, ordered
    `ALL` views, and wire names; count-sensitive reconstruction, gameplay
    definitions, and presentation policy remain owned by their respective
    crates.
  - Current residual typed-content helpers include `MonsterKind::definition()`
    and `TileKind::definition()`; their gameplay-policy ownership is a tracked
    boundary-cleanup item, not a pattern for further expansion.
  - Replay contracts: `ReplayVersion::V2`, `ReplayLog`, and explicit RNG
    sampling semantics metadata.
- **Dependencies**: Pure `std` only; zero dependencies on any other workspace
  crate.

### `drl-core` — Deterministic Simulation Kernel
- **Role**: Pure simulation authority and headless test/evaluation engine.
- **Key Modules & Subsystems**:
  - Simulation & Maps: `Map`, `Tile`, `World`, `Game::step`.
  - PRNG: `GameRng` (deterministic SplitMix64 + Xoshiro256++).
  - Combat & Scheduling: `Scheduler`, `CombatResolver`, kinetic knockback.
  - Perception & AI: Field of View (`fov`), Line of Sight, `MonsterAi`.
  - Items & Inventory: `Inventory`, `Equipment`, `Item::from_spawn_kind`.
    Rocket and power-cell ammo families, their typed pack boundaries, and blue
    and red armor are definition-backed spawns; pinned pickup amounts are immutable
    definition metadata, while replay/scenario counts remain caller-owned.
    Chainsaw is a typed melee weapon; double/combat shotguns, Combat Pistol,
    Assault Shotgun, Plasma Shotgun, Jackhammer, Super Shotgun, Tristar Blaster,
    Blaster, Laser Rifle, Missile Launcher, Butcher's Cleaver, Mjollnir,
    Subtle Knife, Trigun, Anti-Freak Jackal, Minigun, Onyx Armor,
    Phaseshift Armor, Gothic Armor, Malek's Armor, Cybernetic Armor,
    Necroarmor, Medical Powerarmor, Lava Armor, Shielded Armor,
    Nuclear Plasma Rifle, Nuclear BFG 9000, BFG 10K,
    BFG 9000, Mega Buster, Grammaton Beretta, Frag Shotgun, Chaingun, Plasma
    Rifle, Rocket Launcher, Revenant's Launcher, Railgun, and Acid Spitter are
    typed ammo weapons with
    pinned clip and damage ranges;
    current range, accuracy, and timing are Rust policy.
    Blue and red armor preserve protection, descriptions, the shared sprite
    slot, and their presentation tints; med-pack definitions preserve pinned
    descriptions while fixed healing remains Rust policy. The explicit
    `behavior` and `subtle_knife` modules own typed callback-derived
    transitions: Medical Powerarmor keeps armor-owned timer state and emits
    `GameEvent::MedicalPowerarmorRepaired`, while Subtle Knife invoke applies
    actor cost/status and deterministic visible-target internal damage through
    `Command::Invoke` and `GameEvent::SubtleKnifeInvoked`. Trigun alternate
    reload uses the dedicated `trigun` transition and `NukeState`, preserving
    explicit confirmation, resource clamps, weapon retention, and terminal
    internal damage through `Command::AltReload` and typed nuke events. Legacy
    Lava Armor recharge uses the dedicated `behavior` transition and a
    typed `Tile::Lava` terrain check; successful intervals emit
    `GameEvent::LavaArmorRecharged`. The Blaster's periodic cell recharge uses
    the same module's explicit weapon-owned timer and emits
    `GameEvent::WeaponRecharged` without reserve-ammo or presentation policy.
    Nuclear Plasma Rifle recharge uses the same typed state with an explicit
    delay/cadence/amount policy; it emits the existing `WeaponRecharged` event
    and consumes no reserve ammunition.
    Nuclear BFG 9000 recharge uses the same policy with a zero delay and
    five-command cadence, preserving the same event and reserve boundary.
    The `behavior` module also exposes a compile-time `BehaviorSpec` vocabulary
    and immutable profiles for passive/equipment, attack/hit/kill,
    alternate-action, periodic, explicit-cost, and deterministic-target
    concepts. Profiles describe behavior without string keys or runtime
    callbacks; dedicated transition modules remain the execution authority.
    The exotic Missile Launcher uses the explicit single-shell reload policy,
    loading one rocket per accepted `Reload` while retaining the shared
    `WeaponReloaded` event and atomic rejection contract. Its alternate/full
    reload uses a dedicated typed planner that preflights the entire deficit,
    consumes exact loose-rocket reserve, and caps the aggregate action cost at
    2,500 units while retaining the same event contract.
    Malek’s Armor uses the focused `malek_armor` state machine with a typed
    delay-50/cadence-5 policy; accepted-command ticks restore durability below
    maximum, damage resets its timer, and successful repairs emit the neutral
    `GameEvent::MalekArmorRecharged` event. General armor degradation and
    resistance remain separate policy work.
    Nuclear Plasma alternate overload uses the focused `nuclear_overload`
    preflight: a confirmed full clip on a non-stairs tile arms the existing
    `NukeState`, removes the equipped weapon, spends score count, and emits
    `GameEvent::NuclearWeaponOverloaded`; Acid/Lava selects countdown 1 and a
    safe floor selects 100. Legacy map-wide `NukeRun` effects remain outside
    this transition.
    Nuclear BFG 9000 alternate overload reuses the same focused preflight and
    event boundary for its 40-cell weapon; map-wide `NukeRun` behavior remains
    explicitly deferred.
    Standard BFG 9000 exact-hit behavior is a typed weapon policy that skips
    only ranged to-hit sampling; its separate typed 40-cell shot-cost policy
    preflights and debits ammo before the ordinary single-projectile combat
    path. LOS, range, action cost, damage RNG, and existing attack/damage
    events remain unchanged. Projectile routing and explosion effects remain
    separate policy work.
    Nuclear BFG 9000 opts into the same typed exact-hit policy without changing
    its recharge or alternate-overload state; its shot-cost, projectile, and
    NukeRun effects remain separate policy work.
    Revenant’s Launcher opts into the same typed exact-hit policy without
    changing its one-rocket clip or damage policy; homing, projectile routing,
    delayed explosions, and timing remain separate policy work.
    Nuclear BFG 9000 shares the typed forty-cell shot-cost policy with the
    standard BFG while preserving its exact-hit, recharge, and overload state;
    projectile routing, explosions, and NukeRun remain separate policy work.
    BFG 10K opts into the typed exact-hit and five-cell shot-cost policies;
    its scatter, five-shot volley, chainfire, and explosion behavior remains
    separate policy work.
    The pinned `IF_NORELOAD` families use an explicit item policy that rejects
    ordinary `Reload` before mutation; this remains separate from alternate
    reload and automatic recharge behavior.
    Grammaton mode cycling uses the dedicated
    `grammaton` transition and a
    typed `WeaponFireMode`; mode-specific multi-shot resolution preflights
    clip capacity before consuming RNG and emits ordered shot events. Legacy
    Assault Shotgun alternate reload uses the dedicated `assault_shotgun`
    transition: it preflights the complete clip deficit against loose-shell
    reserve, fills atomically, emits the existing aggregate `WeaponReloaded`
    event, and caps the action cost at 2,500 units; ordinary reload remains the
    single-shell policy. Combat Shotgun alternate reload uses the dedicated
    `combat_shotgun` transition with the same atomic deficit/cost policy and
    directly resets the item-owned pump chamber; ordinary reload remains the
    one-shell/pump-only policy. Legacy
    Null Pointer target score branching uses the dedicated `null_pointer`
    transition and emits a typed hit plus deferred-explosion schedule event;
    exact area damage remains an explicit gap. Legacy
    Acid/Lava entered-cell contact uses the dedicated `environment` classifier
    and applies the bounded raw baseline through environment damage/death
    events; Acid/Lava/Water movement uses the typed 1250-unit terrain cost,
    Mud movement uses the typed 1650-unit terrain cost, and Acid/Fire damage
    types are optionally projected; resistance remains an explicit gap. Legacy
    resistance, running modifiers,
    prepared-slot consumption,
    map-cell explosions, and
    broader item behavior remain explicit gaps, as do exact legacy
    timing/accuracy semantics.
  - Level Generation: `generator` (BFS reachability, room connectivity).
  - Content Definitions: `item_definition`, `loot_definition`,
    `monster_roll_definition`, `level_definition`, and descriptive
    `special_level_definition` metadata.
  - Content Invariants: `content_validation::validate_current_content()`
    rejects malformed current tables before they are treated as valid content;
    it does not import legacy behavior or balance targets.
  - Evaluation & Cohorts: `CohortConfig`, `CohortReport`, `BatchRunner`,
    seed/summary/telemetry integrity validation, outcome distributions, and
    observation-free telemetry projections.
- **Dependencies**: Depends only on `drl-protocol`.

### `drl-assets` — Atlas Descriptors & Provenance
- **Role**: Platform-neutral graphics atlas descriptors, geometry, and license
  metadata.
- **Key Responsibilities**:
  - Measured 16-column / 32-pixel sprite sheet cell coordinates.
  - Normalized UV math with top-left origin.
  - Registered source-layer metadata and legacy shader roles (`base`,
    `colorization`, `outline`, `emissive`).
  - CC BY-SA 4.0 licensing records and SHA-256 asset checksums.
- **Dependencies**: No image decoders or platform libraries; core does not
  depend on it.

### `drl-render` — Pure Presentation Planning
- **Role**: Deterministic renderer-neutral scene construction, layout, and
  timing math.
- **Key Responsibilities**:
  - Scene Construction: `PresentationStep`, `RenderScene`, target selection.
  - Fair minimap projection: `MinimapState` includes explored topology and
    currently visible actor/player markers only.
  - Viewport Layout: `PixelViewport`, `PixelRect` integer square-cell scaling.
  - Shading & Tone: `LightingBand` (FOV vs fog), `SceneTone` (player health),
    `low_health_pulse_target_alpha`, `LowHealthPulseState`.
  - Draw Plans: `layer_draw_plan`, `sprite_composite_plan`, `AtlasTextureSource`.
  - Animation & Effects: `active_effect_frames`, elapsed-time frame selection,
    pure math for explosion marks, cell effects, kill segments, FX, movement,
    missile steps/rays, and screen shake.
  - Particles & Decals: Burst origins, directions, range sampling, decal cell
    mapping, decal placement/eligibility, `ParticleDecalInsertion`, and
    caller-bounded `ParticleDecalStore`.
- **Dependencies**: Depends on `drl-protocol` and `drl-assets`. No GPU or window
  dependencies.

### `drl-audio` — Semantic Audio Engine
- **Role**: Deterministic event-to-audio mapping and Web Audio mixer.
- **Key Responsibilities**:
  - Pure mapping from `GameEvent` to semantic `AudioCue`.
  - WASM Web Audio synthesizer with gesture unlock, volume, and mute controls.
- **Dependencies**: Depends on `drl-protocol`.

### `drl-web` — Browser Shell & WebGPU Presentation
- **Role**: WASM `cdylib` / `rlib` browser host, WebGPU renderer, and PWA shell.
- **Key Responsibilities**:
  - Browser session management (`BrowserSession`) and DOM/keyboard mapping.
  - The browser session submits commands to the same authoritative `Game` as
    headless callers; native vertical-fidelity tests may wrap a replay-built
    game to compare events, fair observations, pure effects, and scenes without
    introducing a browser-side simulation model. The Trigun alternate-reload
    Acid Spitter terrain-reload, Null Pointer target-hit, Grammaton burst mode,
    Jackhammer single-mode, Lava Armor recharge, Medical Powerarmor repair,
    Former Human-profile progression, Phase Device escape, Shotgun knockback,
    Green Armor protection, Small MedPack recovery, Demon melee-pressure
    recovery, Pistol reload, Plasma Rifle cell-reload, Rocket Launcher
    one-shot reload, Chainsaw melee, standard Shotgun shell-reload, Assault
    Shotgun single-shell and alternate full reload, Double Shotgun clip-reload,
    Combat Shotgun pump-action and alternate-reload encounters are covered
    by this same
    cross-boundary comparison.
  - Accessible semantic minimap text grid fed by the fair `MinimapState`
    projection; it is bounded and never queries hidden world state.
  - WebGPU pipeline: texture cache, linear `Rgba8Unorm` storage, nearest base
    sampling, emissive lighting floor, `0.1` alpha cutoff, colorization tints,
    and outline-mask straight-alpha compositing.
  - Browser animation loop: `requestAnimationFrame` driving elapsed rendering
    with `visibilitychange` clock rebasing.
  - State Persistence: `SessionSnapshot` codec with localStorage save/load.
    Rejected values are quarantined in a bounded browser-owned slot before
    active storage cleanup; future version migration is explicit and gated.
    The DOM shell requires an explicit accessible Clear Save confirmation before
    calling the Rust-owned storage removal export; cancel and Escape do not
    mutate the save or active simulation.
  - Release Packaging: Bootstrap-independent service worker registration,
    service worker caching with no-HTTP-cache update checks and waiting-update
    status, release manifest validation, digest sidecars, checkout-identity
    verification, and optional detached OpenSSL signature verification. The
    signing boundary rejects release-tree, symlinked, and group/world-readable
    private-key inputs before OpenSSL.
  - Accessibility and Support: Accessible DOM shell, keyboard/numpad
    navigation, focused diagnostics panel, and a pure browser-environment
    classifier that rejects insecure contexts or missing WebGPU before WASM
    startup.
- **Dependencies**: Depends on `drl-protocol`, `drl-render`, `drl-assets`,
  `drl-audio`, and web-sys/wasm-bindgen.

### `drl-mcp` — Model Context Protocol Server
- **Role**: Zero-dependency JSON-RPC 2.0 MCP server for AI agents and test
  automation.
- **Key Responsibilities**:
  - Full MCP method suite (`initialize`, `tools/*`, `resources/*`).
  - Semantic tools for game control, observation, action enumeration, and
    replays, including state-safe deterministic replay verification.
  - `game_verify_replay` verifies either the complete in-memory replay or a
    supplied canonical V2 JSON envelope without mutating session state,
    including recorded procedural generator parameters. This establishes
    repeatability under the current compatible engine semantics; replay wire
    acceptance alone is not a cross-version gameplay-compatibility promise.
    MCP session creation enforces bounded dimensions and generator parameters
    before export; replay file IO, migration, and cross-version interchange
    remain outside this boundary.
  - `game_load_replay` decodes the exact canonical V2 envelope, executes it in
    a temporary `ReplayEngine` state, and commits the game/metrics/replay log
    only after success. The imported log and optional MCP turn limit remain the
    reset source and accept later commands when non-terminal. Existing
    `ReplayEngine` terminal-prefix behavior remains authoritative for supplied
    logs; filesystem or network replay IO and migrations remain outside this
    boundary.
  - `game_save_replay` projects every V2 `ReplayLog` field through the
    deterministic `replay_json` envelope (`drl-rust-replay-v2`) with structured
    semantic command objects, complete initial-state containers, and explicit
    nulls for absent optional values. `replay_json` also decodes this exact V2
    envelope for read-only verification; it does not activate sessions, migrate
    versions, perform file IO, or claim external replay interchange.
  - `tools/list` and `resources/list` slice stable registries into fixed-size
    pages with method-scoped opaque cursors; malformed or stale cursors fail
    before session access, and list pagination does not alter tool/resource
    definitions or lifecycle state.
  - `tools/call` keeps malformed envelopes, unsafe arguments, malformed replay
    input, and unknown methods/tools as JSON-RPC errors, but wraps recognized
    runtime failures in successful MCP results with `isError: true`, stable
    text, and numeric `data.code`/`data.message`; this boundary does not change
    session mutation or the lower-level `execute_tool` contract.
  - Strict observation boundaries with explicit `dev_mode` flag for omniscient
    inspection.
  - `initialize` validates a string `protocolVersion`, echoes supported
    `2024-11-05`, and returns that version as a deterministic fallback for
    unsupported strings; missing/non-string values return `-32602`.
  - Initialize params also require object `capabilities` and `clientInfo`
    fields with string `name` and `version`; unknown nested fields remain
    tolerated and malformed required fields return `-32602`.
  - JSON-RPC request IDs are limited to strings, numbers, or explicit `null`;
    boolean, array, and object IDs return `-32600` before dispatch, while
    omitted IDs remain notifications.
  - Stateful `tools/call` and `resources/read` methods require object params;
    `tools/call.arguments` must be an object when present, with malformed
    envelopes returning `-32602` before session or resource execution.
  - `game_start` and `game_load_scenario` reject wrong-typed optional integer
    arguments and values outside the accepted finite JSON-safe integer range
    (`0..=2^53`, with `u32` for dimensions) with `-32602` before session
    mutation; omitted fields retain their existing defaults.
  - `game_step_action` validates ranged coordinates as exact `i32` values and
    use/equip/drop item IDs as exact non-negative JSON-safe integers before
    dispatching to the simulation.
  - `tools/list` publishes canonical action, direction, slot, and field alias
    spellings with enum domains and exact numeric bounds; unknown properties
    stay tolerated, and the action-or-command discriminator preserves the
    runtime compatibility alias.
  - The `game_step_action` schema adds deterministic action/command conditions
    for direction, ranged coordinate aliases, item IDs, and equipment slots;
    unknown properties remain tolerated and runtime parsing stays authoritative.
  - `McpSession` derives fair legal-action candidates from
    `PlayerObservation`, probes each candidate on a cloned `drl_core::Game`,
    and uses the filtered catalog for `game_list_actions`, tool response
    payloads, and pre-dispatch admission. The live core remains authoritative;
    hidden-state search and unbounded candidate generation are not exposed.
    Unknown/malformed input remains parser-owned `-32602`, while recognized
    commands not in the filtered catalog return `-32001`. It gates actions
    after terminal outcomes, reports level
    transitions as victories, and leaves reset/replay/metrics inspection
    available after termination without modifying `drl-core::Game`.
  - A private `Uninitialized → AwaitingInitialized → Ready` phase gate requires
    an identified initialize request followed by `notifications/initialized`
    before tools and resources are available; premature/duplicate transitions
    return `-32003` without changing game state.
  - Stdio transport suppresses responses for omitted-ID notifications and emits
    ordered response arrays for nonempty batches while preserving identified,
    explicit-null, malformed-request, and empty-batch boundaries; lifecycle
    enforcement and external-client support remain open.
- **Dependencies**: Pure `std` + `drl-protocol` + `drl-core`.

### `drl-app` — Headless CLI & MCP Runner
- **Role**: Native executable for running headless demos, deterministic cohort
  study reports, batch sweeps, and stdio MCP sessions. A deterministic
  subprocess lifecycle contract checks this transport separately from the
  in-process MCP semantics.
- **Dependencies**: Depends on `drl-core`, `drl-protocol`, `drl-mcp`.

### `drl-script` — Content Conversion Boundary
- **Role**: Build-time conversion boundary for legacy content. The current
  extraction tool emits provenance-bearing scalar tables and explicit gaps;
  `scripts/check-content-evidence.py` validates reviewed source coverage and
  validates version-2 crosswalks with complete record ID catalogs, scalar-only
  evidence fields, structured migration gaps, and exact pinned source digests;
  runtime Lua remains prohibited. Because runtime scripting is no longer a
  responsibility, the crate name/existence is an active cleanup question; do
  not grow it into a runtime script host. See the content/behavior steering
  decision.

---

## 4. Subsystem Architecture & Rules

### 4.1 Simulation & Turn Economy
- **Energy-Based Scheduler**: Actors accumulate energy based on their `Speed`.
  When an actor reaches the action threshold, it executes one action costing
  the standard 1000 energy units by default; typed terrain movement rules may
  override that cost (currently Acid/Lava/Water direct movement uses 1250 and
  Mud direct movement uses 1650).
- **Transactional command boundary**: `Game::step` snapshots and restores the
  complete state on any rejection, including turn, world, and RNG. Command
  handlers still use prepare/commit validation where practical; the bounded
  rollback guard protects later fallible substeps. A representative rejection
  matrix covers every current command family and compares exact cloned state.
- **Deterministic PRNG**: All randomness flows through `GameRng`. No ambient or
  thread-local RNG is permitted. Bounded integer sampling uses documented
  rejection sampling under `RNG_SAMPLING_SEMANTICS_VERSION`; core rules use
  exact integer probability ratios, and replay metadata separately records and
  validates gameplay-semantics, RNG-sampling, generator-semantics (for
  procedural replays), and ruleset identities.
- **Combat Resolution**: `CombatResolver` evaluates melee bump attacks and
  targeted ranged attacks with explicit distance accuracy scaling, uniform
  damage rolls, armor protection mitigation, and health clamping.
- **Kinetic Knockback**: Shotgun blasts push surviving targets along firing
  vectors with collision checks against map borders, solid walls, and other
  actors.

### 4.2 Content Tables & Definitions
- **Monster Definitions**: `MonsterKind::definition()` in `drl-protocol`
  currently owns immutable stats, speeds, attack ranges, accuracies, and drop
  tables for current archetypes. This is existing structure, not the desired
  ownership for new gameplay policy; migrate gameplay definitions toward the
  core/domain side while stable semantic kind IDs remain protocol-safe.
- **Item Definitions**: `drl-core::item_definition` owns item definitions;
  `ItemSpawnKind::ALL` is the stable representative family catalog,
  `CURRENT_ITEM_SPAWN_KINDS` is the core-facing alias, and
  `CURRENT_ITEM_DEFINITIONS` is the core-owned balance table used for lookup
  and coverage. `Item::from_spawn_kind` serves as canonical item factory. New
  routine content should converge on one authoritative compile-time catalog
  whose projections supply enums/lookup/display/replay/validation coverage
  instead of manual cross-crate registry duplication.
- **Behavior Definitions**: Callback-heavy legacy semantics should be expressed
  through a bounded typed Rust vocabulary (modifiers, equip/use/attack/kill
  effects, alternate actions, recharge/periodic policy, set membership, and
  explicit typed state machines for exceptional cases), not a generic runtime
  callback/event bus. The Combat Shotgun pump-action chamber is one such
  item-owned state machine; it remains hidden from `ItemView` until a later
  presentation slice.
- **Loot & Monster Rolls**: Pure roll-bound tables map caller-supplied PRNG
  rolls to procedural room loot and monster spawns.
- **Tile Definitions**: `TileKind::definition()` in `drl-protocol` currently
  defines physical walkability, transparency, and liquid properties. As with
  monster balance, current ownership does not justify adding unrelated
  gameplay policy to the protocol boundary.
- **Level Policy**: `drl-core::level_definition` provides standard procedural
  generation parameters.
- **Special-Level Metadata**: `drl-core::special_level_definition` exposes
  pinned scalar names, text, and optional legacy depths only. It never selects,
  generates, or executes legacy maps or callbacks.

### 4.3 Rendering Pipeline & Viewport
- **Square-Cell Integer Viewport**: `PixelViewport` computes integer square
  cell dimensions from canvas dimensions and applies deterministic centered
  letterboxing. Non-uniform axis stretching is prohibited.
- **Visibility-Derived Lighting**: `LightingBand` assigns full light (`1.0`) to
  active FOV tiles and a fixed fog factor (`0.3`) to explored memory tiles.
  Hidden tiles are omitted.
- **Scene Tone & Low-Health Pulse**: `SceneTone` applies a low-health clear tone
  below 25% HP; `low_health_pulse_target_alpha` and `LowHealthPulseState`
  provide smooth, bounded alpha modulation.
- **Layer & Composite Plans**: `layer_draw_plan` generates ordered scene draws
  (tiles, items, actors); `sprite_composite_plan` groups all registered layer
  roles per sprite for WebGPU shader bind groups.

### 4.4 Animation & Visual Timing
- **Decoupled Effect Timing**: `EffectSpan` assigns fixed logical durations to
  events. Presentation timing never drives or advances gameplay turns.
- **Elapsed-Time Frame Selection**: `animation_frame_index_at_elapsed` selects
  animation frames based on elapsed milliseconds and explicit loop/clamp
  policies.
- **Pure Effect Arithmetic**: Explosion marks, cell animations, kill segments,
  FX frames, movement interpolation, missile steps/rays, and screen shake fade
  are pure mathematical functions without GPU or simulation dependencies.
- **Visibility Lifecycle Clock**: Browser animation loop listens to
  `visibilitychange` and rebases presentation clocks when background tabs resume.

### 4.5 Particles & Decals
- **Pure Arithmetic Contracts**: Burst origins, direction normalization, arc
  adjustments, range interpolation, decal cell mapping, and placement are pure
  functions.
- **Eligibility Filter**: `particle_decal_cell_is_eligible` enforces map bounds,
  non-liquid, and non-blocking rules.
- **Insertion Request**: `ParticleDecalInsertion` packages placement with the
  caller-provided sprite ID.
- **Bounded Decal Store**: `ParticleDecalStore` retains requests in strict
  insertion order with caller-configured capacity, reporting overflow without
  dropping prior entries.
- **Decal Draw Planning**: `particle_decal_draw_plan` resolves opaque,
  caller-provided sprite handles to complete atlas layer groups, uses stored
  pixel placement for sub-cell offsets, carries caller-resolved lighting,
  omits unknown/out-of-viewport entries, and leaves the store unchanged.
  Combined scene plans place decals between terrain and ordinary objects;
  WebGPU owns batching and resource binding.

### 4.6 Browser & WebGPU Integration
- **Texture Cache**: Imported atlas PNGs are loaded same-origin, dimension-
  checked, and uploaded once into linear `Rgba8Unorm` WebGPU 2D textures.
- **Nearest Base Filtering**: Base sprite pixels are sampled with nearest
  filtering.
- **Emissive Floor**: Emissive mask red channel acts as a minimum lighting
  floor.
- **Alpha Cutoff**: WGSL textured shader discards fragments below the legacy
  `0.1` alpha threshold.
- **Colorization Tints**: Pinned vertex tints apply to Green Armor, Phase
  Device, and StairsDown.
- **Outline Straight-Alpha**: Optional outline-mask shadow layers composite
  behind base pixels with tested straight-alpha weights.

### 4.7 Replays, Cohorts & Evaluation
- **Replay V2 Engine**: `ReplayEngine` records and executes versioned replay
  logs with exact initial spawn metadata and command streams. Current
  verification proves deterministic reproduction under the implementation and
  semantics that interpret that log; it does not by itself define archival
  compatibility across future gameplay/content/RNG changes. Structural
  validation rejects unsupported schema headers, out-of-bounds custom tile
  overrides, dimensions outside `3..=512`, oversized replay containers, and
  unsafe procedural parameters before map construction, matching the MCP
  decoder's spatial and structural contracts.
- **Replay Compatibility Layers**: Wire/schema version, engine/gameplay
  semantics version, and ruleset/content identity must be distinguished before
  cross-version compatibility is claimed. Generator semantics should be
  versioned separately when they can change replay outcomes.
- **Cohort Reports**: `CohortConfig` / `CohortReport` execute multi-seed sweeps,
  validating seed order, record counts, and summary metrics.
- **Descriptive Telemetry**: Cohort outcome distributions and telemetry
  comparisons report exact win/loss rates, accuracy, damage, and kills with
  caller-owned tolerances.
- **Descriptive Depth**: `CohortDepthDistribution` validates reports before
  grouping `level_reached` into ascending buckets and sample rates; it carries
  no canonical difficulty or balance interpretation.

### 4.8 Browser Interaction Accessibility
- **Qualified Inventory Actions**: Generated controls use escaped item names in
  group labels and action-specific accessible names; item IDs remain data-only
  command handles.
- **Announcement Boundary**: `#game-status` is the sole dynamic polite status
  channel; keyboard help remains static and is associated with the focusable
  canvas through `aria-describedby`.
- **Recovery Focus**: Browser and Rust-originated diagnostics reveal and focus
  the same alert panel. Focus-visible styling is authored in the HTML shell.
- **Acceptance Boundary**: These are structural/native/browser-contract
  guarantees, not WCAG 2.1 AA, screen-reader, or broad browser claims.

### 4.9 Persistence & Release Packaging
- **Session Snapshots**: `drl-web::persistence` encodes complete command
  histories as strict V2 command-count tokens, accepts only the shipped V1
  token for transactional replay, and migrates successful V1 restores in the
  existing storage slot.
- **Service Worker Cache**: Versioned same-origin worker caches static bundles
  keyed by project version and commit hash; reads open only the current release
  cache and fail closed when its shell/assets are absent.
- **Release Manifests**: Build tooling generates `release-manifest.json` with
  sorted artifact SHA-256 hashes and a `.sha256` sidecar digest.
- **Release-rights Boundary**: `docs/release-rights.md` records the bundled
  graphics license/provenance and excluded legacy material; the source and
  optional bundle checks in `scripts/check-release-rights.sh` reject legacy
  code, audio/music, fonts, WADs, and other excluded media without making a
  legal-clearance claim. Legacy-derived creative text in Rust/content
  definitions is tracked as a separate provenance review question rather than
  silently grouped with numeric mechanics.

---

## 5. Architectural Invariants

Every change to the codebase must preserve these design invariants. Where the
current audit has identified an implementation gap, the invariant remains the
target contract but is not reported as already verified:

1. **No Ambient State in Core**: No global variables, thread-local state,
   ambient RNG, wall clocks, or non-deterministic hash iteration in `drl-core`.
2. **Deterministic Replay**: Given identical seed, command stream, declared
   gameplay semantics, and ruleset/content identity, the simulation must
   produce bit-exact identical observations, events, and final world state on
   supported platforms. Cross-version compatibility is an explicit contract,
   not an implication of using the same wire schema.
3. **Observation Decoupling**: Renderers, bots, and MCP agents consume only
   fair `PlayerObservation` views; they must never access `World` or inspect
   unexplored tiles.
4. **Presentation Decoupling**: Audio cues, rendering frames, WebGPU shaders,
   canvas resizing, and tab visibility transitions must never mutate simulation
   state or advance game turns.
5. **No Runtime Scripting**: No Lua VM or JavaScript gameplay interpreters.
6. **Atomic Rejection**: Illegal/rejected commands and corrupt saves must fail
   without partial simulation mutation. Ranged-command, non-equippable-item,
   full-backpack pickup, out-of-bounds drop, empty-slot unequip, and
   full-backpack unequip, non-consumable use, missing-item use, reload,
   off-stairs descend, blocked-terrain movement, out-of-bounds movement,
   invalid-direction melee, empty-target melee, missing-item Equip/Drop, and
   no-ground-item Pickup, out-of-bounds/empty-target ranged, and
   no-equipped-weapon/empty-clip ranged, no-destination phase-device,
   `Invoke`/`AltReload`, post-game-over command rejection, and out-of-bounds
   pickup validation cases are covered by
   `crates/drl-core/tests/command_atomicity.rs`. Death-drop destinations are
   preflighted before combat mutation, so expected terrain failures cannot
   occur after ammo, RNG, or typed behavior effects have committed.

---

## 6. Verification & Automated Boundary Enforcement

The repository enforces architectural boundaries via automated test suites:

- **Boundary Enforcement**: `crates/drl-core/tests/boundaries.rs` validates
  dependency direction and ensures core remains free of presentation or platform
  crates.
- **Steering Gates**: `docs/steering/current-priorities.md` and the repo-local
  milestone-delivery skill define temporary stop gates. A gate is retired only
  after its acceptance evidence exists and enduring documentation is reconciled.
- **Repository Health**: `sh scripts/check-repository.sh` runs formatting,
  clippy, unit tests, integration tests, and harness checks.
- **Asset Manifest**: `sh scripts/check-assets.sh` verifies graphics licensing
  and SHA-256 checksums.
- **Release-rights Inventory**: `sh scripts/check-release-rights.sh` validates
  the rights inventory, graphics evidence, exact manifest rights declaration,
  and optional bundle exclusions; an absent bundle is reported `NOT_RUN`.
- **Web Contracts**: `sh scripts/check-web.sh` checks WASM compilation and web
  contracts.
- **Release Validation**: `scripts/check-release-manifest.sh` validates release
  artifacts and service worker coverage.
- **Version Projections**: `scripts/check-version.sh` enforces valid `x.y.z`
  transitions.
