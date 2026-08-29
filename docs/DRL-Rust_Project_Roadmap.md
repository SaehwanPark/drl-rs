# DRL-Rust Project Roadmap

Last reviewed: 2026-08-28
Current project version: `0.2.231`

---

## 1. Product Direction

DRL-Rust is a ground-up Rust reimplementation of *Doom the Roguelike* (DRL).
The project replaces the legacy Pascal and Lua codebase with a modern,
memory-safe, and deterministic architecture while faithfully preserving
canonical gameplay semantics.

### Primary Goals

- **Browser-First 1.0 Target**: Playable in desktop Chromium browsers
  (Chrome/Edge) via WebAssembly and WebGPU, packaged as a high-performance
  static HTTPS bundle with an accessible DOM shell and offline PWA support.
- **Deterministic Headless Core**: The simulation core (`drl-core`) is pure,
  reproducible, and completely decoupled from rendering, audio, browser, OS,
  and filesystem APIs.
- **First-Class Agent and Tooling Support**: Rich Model Context Protocol (MCP)
  interfaces, deterministic headless replay engines, automated bots, and
  statistical evaluation suites.
- **Attributable Asset Provenance**: Built-in tracking for licenses, checksums,
  and rights clearance. No runtime Lua engine in the browser bundle.

### Portability Scope

- **1.0 Scope**: Desktop Chromium (WebGPU), static web hosting, headless CLI,
  and stdio MCP server.
- **Post-1.0 Scope**: WebGL2 fallback, cross-browser support (Firefox/Safari),
  mobile/touch controls, gamepad navigation, and native desktop packaging.

---

## 2. Status Vocabulary

To maintain strict truthfulness in progress tracking, every milestone and
verification item uses explicit status semantics:

- `[x]` — **Delivered and Verified**: Fully implemented and validated by
  repository tests, CI runs, or checked artifacts.
- `[ ]` — **Planned or Open**: In progress, planned, or awaiting verification.
- `NOT_RUN` — **Environment Unavailable**: Required execution prerequisites
  were not present in the execution environment (e.g., Linux x86-64 binary
  probes on macOS arm64, or headless Chrome on minimal CI runners). This is
  recorded neutrally and is never treated as an inferred pass or failure.
- `INCONCLUSIVE` — **Unresolved Evidence**: Output exists but cannot
  definitively satisfy acceptance criteria without further evidence or rights.

---

## 3. Current Progress Summary (`VERSION` 0.2.231)

### Delivered Foundations

- **Core Simulation (M0–M2, M4)**: Pure deterministic grid maps, PRNG, turn
  economy, melee/ranged combat, armor mitigation, kinetic knockback, FOV/fog,
  inventory/equipment, tactical monster AI, and procedural level generation.
- **M1/M2 Correctness Slice (`0.2.89`–`0.2.110`)**: Ranged-command target
  legality and range are prepared before ammo/RNG mutation; equip commands
  validate item existence and slot eligibility before inventory mutation; and
  inventory insertion stages ammunition merges before committing capacity
  changes. Drop commands also validate the destination before removing items.
  Out-of-range, blocked-line, non-equippable-item, full-backpack pickup,
  out-of-bounds drop, empty-slot unequip, full-backpack unequip, non-consumable
  use, missing-item use, no-weapon/full-clip/no-ammo reload, and off-stairs
  descend, blocked-terrain movement, out-of-bounds movement, invalid-direction
  melee, empty-target melee, missing-item Equip/Drop, no-ground-item Pickup,
  out-of-bounds ranged, empty-target ranged, no-weapon ranged, and empty-clip
  ranged rejections and no-destination phase-device rejection are covered by
  exact `Game` equality tests; post-game-over command rejection is covered by
  exact `Game` equality as well. Pickup validates malformed out-of-bounds
  positions before ground-item removal. `GameRng::gen_range` now uses
  unbiased rejection sampling with pinned raw, bounded, and probability
  vectors. Replay metadata now records gameplay-semantics and ruleset identity
  and rejects incompatible values before simulation; migration remains open.
  Core procedural generation now uses exact integer-ratio probability sampling
  rather than a floating-point rule. Procedural replay metadata now carries a
  separate generator-semantics version and rejects incompatible generated-map
  policies while fixed-map replays remain independent of that identity.
  `Game::step` now restores the complete pre-command state on every rejection,
  preserving turn and RNG through a documented bounded rollback backstop.
  Accepted-only `Wait` and `Move(None)` branches are covered separately because
  they have no reachable rejection path.
- **M1/Gate A late-failure audit (`0.2.122`)**: Death-drop destinations are
  preflighted before player melee, ranged, and typed Subtle Knife effects;
  exact `Game` equality tests cover the resulting blocked-terrain rejection,
  including clip/RNG restoration for ranged attacks and typed-command costs.
  The current dispatch surface now has explicit rejection evidence for every
  reachable command family; future command additions must preserve the same
  prepare/commit boundary.
- **M9/Gate C routine projection (`0.2.123`)**: The stable protocol
  `ItemArchetype` catalog now owns loose-ammo count shape, and MCP replay JSON
  decoding consumes that projection instead of maintaining a second variant
  list. Gameplay balance and explicit definition mappings remain core-owned.
  The first Gate C slice centralizes stable representative item spawn families
  in protocol's `ItemSpawnKind::ALL`; core structural validation accesses that
  catalog through `CURRENT_ITEM_SPAWN_KINDS`, and definition coverage derives
  from the alias. Replay JSON stable item names, loose-ammo counts, and inverse
  decoding now use typed protocol projections. Stable item names now expose one
  typed projection used by display and inverse parsing, and routine atlas
  descriptor coverage tests iterate the protocol's stable `ItemArchetype::ALL`
  catalog. Replay completeness fixtures now derive their family coverage from
  `ItemSpawnKind::ALL` while keeping explicit normalized ammo counts; behavior
  and presentation mappings remain explicitly open. The
  current manual fan-out inventory is recorded in
  `docs/steering/decisions/item-registration-fanout-inventory.md`.
- **M9/Gate C definition lookup (`0.2.124`)**: Core item definitions now use a
  single `CURRENT_ITEM_DEFINITIONS` catalog for family lookup and coverage;
  the prior 56-arm spawn-kind registration match is removed while balance and
  behavior remain core-owned.
- **M9/Gate C inverse spawn projection (`0.2.125`)**: Ordinary replay spawn
  families now resolve from `ItemSpawnKind::ALL`; only loose-ammo count
  reconstruction remains explicit, preserving missing-count and unknown-family
  rejection semantics.
- **M9/Vertical fidelity movement (`0.2.126`)**: Pinned legacy source shows
  direct player movement validates only the requested destination, while AI
  `MoveTowards` fallback remains separate. A deterministic fixed-map test now
  protects diagonal corner cutting into a walkable destination when both
  adjacent cardinal tiles are walls; controlled legacy runtime comparison is
  `NOT_RUN`.
- **M9/Vertical fidelity AI movement (`0.2.127`)**: Pinned legacy source now
  drives a bounded `MonsterAi` candidate order: smoothed preferred step, raw
  retry, horizontal fallback, then vertical fallback. All candidates blocked
  produce `Wait` rather than broad pathfinding. Unit and scheduled-turn tests
  cover the order, including same-position `Wait`; gameplay-semantics replay
  identity advances to `6` and rejects version-5 envelopes until migration.
  Controlled legacy runtime comparison is `NOT_RUN`.
- **M9/Gate D Grammaton behavior (`0.2.128`)**: Pinned legacy source now
  drives a typed Grammaton fire-mode cycle (`Single -> Burst -> Auto`) with
  mode-specific damage/shot profiles, a 200 score-count cost, deterministic
  multi-shot ranged resolution, and an ordered mode-change event. Partial
  bursts reject before clip/RNG mutation; gameplay-semantics replay identity
  advances to `7` and rejects older envelopes until migration. Legacy runtime
  accuracy-equation and presentation comparison remain `NOT_RUN`.
- **M9/Gate D Jackhammer behavior (`0.2.129`)**: Pinned legacy source now
  drives a typed burst/single fire-mode toggle with the existing `8d3` shotgun
  profile, one score-count cost, deterministic selected-shell resolution, and
  an ordered mode-change event. Partial clips reject before clip/RNG mutation;
  gameplay-semantics replay identity advances to `8` and rejects older
  envelopes until migration. Legacy spread/falloff, timing, runtime, and
  presentation comparison remain `NOT_RUN`.
- **M9/Gate D Lava Armor behavior (`0.2.130`)**: Pinned legacy callback
  evidence now drives a typed five-accepted-command Lava recharge transition.
  A walkable `Lava` tile, +3 durability clamp, full-armor guard, non-Lava
  interval reset, replay semantics `9`, and deterministic event ordering are
  covered; hazard damage/resistance and runtime/presentation parity remain
  `NOT_RUN`.
- **M9/Gate D Null Pointer behavior (`0.2.131`)**: Pinned legacy on-hit
  evidence now drives a typed target score-count branch and deterministic
  explosion-schedule event for the catalog-backed weapon. Exact delayed
  explosion geometry, damage ordering, runtime, and audiovisual parity remain
  `NOT_RUN`.
- **M9/Gate D Acid Spitter behavior (`0.2.132`)**: Pinned legacy pre-reload
  evidence now drives a typed Acid-to-Water terrain transition that loads one
  rocket and spends 1000 score count. Acid hazard damage, runtime, and
  audiovisual parity remain `NOT_RUN`.
- **M9/Gate D terrain hazard behavior (`0.2.133`)**: Pinned legacy cell
  callbacks now drive typed baseline player-contact damage of 6 on Acid and 12
  on Lava after accepted movement, with deterministic environment damage/death
  events. Resistance, difficulty/running modifiers, fluid movement cost,
  runtime comparison, and audiovisual parity remain `NOT_RUN`.
- **M9/Gate D fluid movement-cost behavior (`0.2.134`)**: Pinned legacy cell
  movement costs now drive an integer 1250 action cost for direct player moves
  onto Acid/Lava while ordinary walkable movement remains 1000. Running/NORUN,
  fractional scheduler details, fluid flow, runtime comparison, and
  audiovisual parity remain `NOT_RUN`.
- **M9/Gate D Water movement-cost behavior (`0.2.135`)**: The pinned legacy
  `move_cost=1.25` policy now also drives 1250-unit direct player movement onto
  Water without contact damage; Mud's `1.65` movement cost remains deferred
  because no Rust Mud tile exists.
- **M9/Gate D damage-type projection (`0.2.136`)**: Environment Acid/Fire
  classifications now travel through typed `DamageApplied` events and MCP JSON
  while actor/unclassified damage remains explicitly absent; resistance and
  balance modifiers remain `NOT_RUN`.
- **M9/Gate D Mud terrain (`0.2.137`)**: Pinned Mud terrain is now a typed
  walkable tile with replay/scenario/MCP/browser projections and deterministic
  1650-unit direct movement cost. Flow, runtime, and exact presentation parity
  remain `NOT_RUN`.
- **M1/Gate A rejection matrix (`0.2.138`)**: Representative invalid commands
  across the current command surface now assert exact cloned `Game` and RNG
  identity on rejection; no gameplay semantics or protocol schema changed.
- **M2/Gate B replay compatibility (`0.2.139`)**: Replay metadata tests now
  reject stale gameplay, ruleset, and procedural-generator identities before
  execution while preserving fixed-map independence from generator metadata.
- **M2/Gate B RNG sampling semantics (`0.2.140`)**: Replay metadata now
  declares the bounded RNG sampling version; stale RNG identities reject before
  simulation, and the canonical MCP replay envelope is versioned as V2.
- **M9/Gate C item identity catalog (`0.2.141`)**: One protocol declaration
  now generates the stable `ItemArchetype` enum, ordered `ALL` view, and wire
  names; count-sensitive spawn payloads, gameplay definitions, and presentation
  mappings remain explicit.
- **M9/Gate C catalog order invariant (`0.2.142`)**: Protocol tests now assert
  normalized spawn-family order stays aligned with the stable archetype catalog.
- **M9/Gate C shared identity/spawn catalog (`0.2.143`)**: One protocol
  declaration now generates stable archetype and replay spawn variants,
  normalized `ALL` views, stable names, and archetype mapping. Count-sensitive
  loose-ammo stack handling, gameplay definitions, and presentation mappings
  remain explicit.
- **Gate D first behavior slice:** Medical Powerarmor now has a typed,
  deterministic periodic-repair transition in `drl-core`, accepted-turn
  integration, exact timer/durability/health edge tests, and a typed repair
  event; gameplay-semantics replay identity `3` rejects older envelopes until
  migration exists, while legacy runtime parity and later stress cases remain
  open.
- **Gate D second behavior slice:** Subtle Knife now has a typed alternate
  invoke command, HP/status/score costs, deterministic visible-target internal
  damage, typed invocation events, replay/persistence coverage, and gameplay-
  semantics replay identity `4`; runtime/presentation parity and Trigun remain
  open.
- **Gate D third behavior slice:** Trigun now has a typed, confirmation-gated
  alternate reload with explicit HP/max-HP/score costs, one-tick nuke state,
  terminal internal damage, replay/persistence coverage, and gameplay-semantics
  replay identity `5`; explosion/map effects and runtime/presentation parity
  remain open.
- **Gate D evidence:** A third callback-heavy stress case is now characterized
  in `docs/legacy-behavior/trigun.md`; the evidence set and typed
  implementation are complete for the initial three-case target, while runtime
  confirmation and presentation parity remain open.
- **Gate D selected-case acceptance reconciliation:** `SPEC.md` now records
  the initial Medical Powerarmor, Subtle Knife, and Trigun stress-case gate as
  verified from their evidence notes and deterministic Rust tests; broader
  behavior vocabulary and controlled legacy runtime parity remain open.
- **M9 exit-gate evidence reconciliation:** The active specification now records
  verified protocol/domain ownership, bounded module scope, and local plus
  hosted repository/WASM checks for the delivered foundation; legacy runtime
  and audiovisual parity remain outside the evidence boundary.
- **M9 vertical Subtle Knife encounter (`0.2.144`):** The delivered typed
  Subtle Knife transition now has a declarative ASCII scenario, replay
  determinism, and browser-session presentation-boundary parity covering
  target visibility, event order, observations, effects, and scene derivation.
  No gameplay-semantics or balance change is introduced; controlled legacy
  runtime, browser capture, audio, WebGPU, armor/resistance, and broad AI
  parity remain `NOT_RUN`.
- **M9 vertical Trigun encounter (`0.2.145`):** The delivered typed Trigun
  alternate-reload transition now has a declarative ASCII encounter, replay
  determinism, and browser-session presentation-boundary parity covering
  confirmation, HP/max-HP/score costs, one-tick nuke resolution, terminal
  event ordering, observations, effects, and scene derivation. No gameplay-
  semantics or balance change is introduced; explosion geometry, animation,
  confirmation UI, controlled legacy runtime, browser capture, audio, WebGPU,
  armor/resistance, and broad AI parity remain `NOT_RUN`.
- **M9 vertical Acid Spitter encounter (`0.2.146`):** The delivered typed Acid
  Spitter reload transition now has a declarative ASCII encounter, replay
  determinism, and browser-session presentation-boundary parity covering
  terrain-fed reload costs, Acid-to-Water conversion, event/effect ordering,
  observations, and scene derivation. No gameplay-semantics or balance change
  is introduced; hazard resistance/flow, explosion geometry, animation timing,
  controlled legacy runtime, browser capture, audio, WebGPU, and broad AI
  parity remain `NOT_RUN`.
- **M9 vertical Null Pointer encounter (`0.2.147`):** The delivered typed Null
  Pointer target-dependent hit transition now has a declarative ASCII encounter,
  replay determinism, and browser-session presentation-boundary parity covering
  the boss score floor, deferred explosion scheduling, stable identities,
  event/effect ordering, observations, and scene derivation. No gameplay-
  semantics or balance change is introduced; delayed blast geometry, controlled
  legacy runtime, browser capture, audio, WebGPU, and broad AI parity remain
  `NOT_RUN`.
- **M9 vertical Grammaton encounter (`0.2.148`):** The delivered typed
  Grammaton fire-mode transition now has a declarative ASCII encounter, replay
  determinism, and browser-session presentation-boundary parity covering Burst
  selection, score cost, three-shot clip consumption, stable identities,
  event/effect ordering, observations, and scene derivation. No gameplay-
  semantics or balance change is introduced; exact accuracy equations,
  controlled legacy runtime, browser capture, audio, WebGPU, and broad AI
  parity remain `NOT_RUN`.
- **M9 vertical Jackhammer encounter (`0.2.149`):** The delivered typed
  Jackhammer fire-mode transition now has a declarative ASCII encounter, replay
  determinism, and browser-session presentation-boundary parity covering Single
  selection, score cost, one-shell clip consumption, knockback, stable
  identities, event/effect ordering, observations, and scene derivation. No
  gameplay-semantics or balance change is introduced; exact spread/falloff,
  controlled legacy runtime, browser capture, audio, WebGPU, and broad AI
  parity remain `NOT_RUN`.
- **M9 vertical Lava Armor encounter (`0.2.150`):** The delivered typed Lava
  Armor periodic recharge transition now has a declarative ASCII encounter,
  replay determinism, and browser-session presentation-boundary parity covering
  five accepted waits, timer progression, the three-point durability clamp,
  stable identity, event ordering, observations, and scene derivation. No
  gameplay-semantics or balance change is introduced; Lava hazard/resistance,
  controlled legacy runtime, browser capture, audio, WebGPU, audiovisual, and
  broad armor/content parity remain `NOT_RUN`.
- **M9 vertical Medical Powerarmor encounter (`0.2.151`):** The delivered
  typed Medical Powerarmor periodic repair transition now has a declarative
  ASCII encounter, replay determinism, and browser-session presentation-
  boundary parity covering thirty accepted waits, timer progression, one-point
  healing, durability spend, stable identity, event ordering, observations, and
  scene derivation. No gameplay-semantics or balance change is introduced;
  controlled legacy runtime, browser capture, audio, WebGPU, audiovisual, and
  broad armor/content parity remain `NOT_RUN`.
- **M9 vertical Former Human-profile progression (`0.2.152`):** The delivered
  typed Pistol progression now has a declarative ASCII encounter, replay
  determinism, and browser-session presentation-boundary parity covering
  scheduled Former Human-profile responses, ranged combat, target defeat, dropped
  ammunition pickup, stairs descent, stable identities, event ordering,
  observations, and scene derivation. No gameplay-semantics or balance change
  is introduced; controlled legacy runtime, browser capture, audio, WebGPU,
  audiovisual, and broader monster/weapon parity remain `NOT_RUN`.
- **M9 vertical Phase Device escape (`0.2.153`):** The delivered typed Phase
  Device transition now has a declarative ASCII encounter, replay determinism,
  and browser-session presentation-boundary parity covering pickup,
  deterministic teleport destination, item consumption, exploration, stable
  identity, event ordering, observations, literal effects, and scene
  derivation. No gameplay-semantics or balance change is introduced;
  controlled legacy runtime, browser capture, audio, WebGPU, audiovisual, and
  broader item/teleport parity remain `NOT_RUN`.
- **M9 vertical Shotgun knockback (`0.2.154`):** The delivered typed Shotgun
  encounter now has a declarative ASCII scenario, replay determinism, and
  browser-session presentation-boundary parity covering the seeded ranged hit,
  one-tile displacement, scheduled Former Sergeant-profile response, stable
  identities, event ordering, observations, literal effects, and scene
  derivation. No gameplay-semantics or balance change is introduced;
  controlled legacy runtime, browser capture, audio, WebGPU, audiovisual, and
  broader weapon/monster knockback parity remain `NOT_RUN`.
- **M9 vertical Green Armor protection (`0.2.155`):** The delivered typed
  Green Armor encounter now has a declarative ASCII scenario, replay
  determinism, and browser-session presentation-boundary parity covering the
  seeded Former Sergeant-profile response, raw-versus-mitigated damage,
  observed armor protection, stable identities, event ordering, observations,
  literal effects, and scene derivation. No gameplay-semantics or balance
  change is introduced; controlled legacy runtime, browser capture, audio,
  WebGPU, audiovisual, durability/resistance, and broader armor/monster parity
  remain `NOT_RUN`.
- **M9 vertical Small MedPack recovery (`0.2.156`):** The delivered typed
  Small MedPack encounter now has a declarative ASCII scenario, replay
  determinism, and browser-session presentation-boundary parity covering
  capped healing, item consumption, action-cost ordering, stable identity,
  observations, literal use effects, and scene derivation. No
  gameplay-semantics or balance change is introduced; controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader consumable
  parity remain `NOT_RUN`.
- **M9 vertical Demon MedPack recovery (`0.2.157`):** The delivered typed
  Demon melee-pressure encounter now has a declarative ASCII scenario, replay
  determinism, and browser-session presentation-boundary parity covering two
  seeded AI responses around capped healing, item consumption, action-cost
  ordering, stable identities, observations, literal effects, and scene
  derivation. No gameplay-semantics or balance change is introduced;
  controlled legacy runtime, browser capture, audio, WebGPU, audiovisual, and
  broader monster/consumable parity remain `NOT_RUN`.
- **M9 vertical Pistol reload (`0.2.158`):** The delivered typed Pistol
  encounter now has a declarative ASCII scenario, replay determinism, and
  browser-session presentation-boundary parity covering ten seeded ranged
  attacks, hit/damage totals, ammunition consumption, reload state, stable
  identities, observations, literal effects, and scene derivation. No
  gameplay-semantics or balance change is introduced; controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/ammunition parity remain `NOT_RUN`.
- **M9 vertical Plasma Rifle cell reload (`0.2.159`):** The delivered typed
  Plasma Rifle encounter now has a declarative ASCII scenario, replay
  determinism, and browser-session presentation-boundary parity covering six
  seeded ranged attacks, cell-ammunition consumption, six-round clip state,
  stable identities, observations, literal effects, and scene derivation. No
  gameplay-semantics or balance change is introduced; controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/ammunition parity remain `NOT_RUN`.
- **M9 vertical Rocket Launcher one-shot reload (`0.2.160`):** The delivered
  typed Rocket Launcher encounter now has a declarative ASCII scenario, replay
  determinism, and browser-session presentation-boundary parity covering one
  seeded ranged hit, rocket-ammunition consumption, one-shot clip state,
  stable identities, observations, literal effects, and scene derivation. No
  gameplay-semantics or balance change is introduced; controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/ammunition parity remain `NOT_RUN`.
- **M9 vertical Chainsaw melee (`0.2.161`):** The delivered typed Chainsaw
  encounter now has a declarative ASCII scenario, replay determinism, and
  browser-session presentation-boundary parity covering one seeded melee hit,
  Demon-profile target damage, stable identities, observations, literal
  effects, and scene derivation. No gameplay-semantics or balance change is
  introduced; controlled legacy runtime, browser capture, audio, WebGPU,
  audiovisual, and broader weapon/melee parity remain `NOT_RUN`.
- **M9 vertical Shotgun shell reload (`0.2.162`):** The delivered typed
  standard Shotgun encounter now has a declarative ASCII scenario, replay
  determinism, and browser-session presentation-boundary parity covering eight
  seeded shell attacks, shell consumption, the distinct 1200-unit reload cost,
  stable identities, observations, literal effects, and scene derivation. No
  gameplay-semantics or balance change is introduced; controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/spread and alternate-reload parity remain `NOT_RUN`.
- **M9 vertical Assault Shotgun shell reload (`0.2.163`):** The delivered
  typed Assault Shotgun encounter now has a declarative ASCII scenario, replay
  determinism, and browser-session presentation-boundary parity covering six
  seeded shell attacks, blocked knockback, shell consumption, stable
  identities, observations, literal effects, and scene derivation. No
  gameplay-semantics or balance change is introduced; controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/spread and alternate-reload parity remain `NOT_RUN`.
- **M9 vertical Double Shotgun clip reload (`0.2.164`):** The delivered typed
  Double Shotgun encounter now has a declarative ASCII scenario, replay
  determinism, and browser-session presentation-boundary parity covering two
  seeded shell attacks, blocked knockback, shell consumption, stable
  identities, observations, literal effects, and scene derivation. No
  gameplay-semantics or balance change is introduced; controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/spread parity remain `NOT_RUN`.
- **M9 vertical Assault Shotgun single-shell reload (`0.2.165`):** The
  delivered typed reload correction now honors the pinned `IF_SINGLERELOAD`
  behavior with one-shell loading, atomic full/no-reserve rejection coverage,
  deterministic scenario/replay evidence, and browser-session presentation
  parity. Gameplay semantics advance to `17`; alternate reload, controlled
  legacy runtime, browser capture, audio/WebGPU, audiovisual, and broader
  weapon/spread parity remain `NOT_RUN`.
- **M9 vertical Combat Shotgun clip reload (`0.2.166`):** The delivered typed
  Combat Shotgun encounter now has a declarative ASCII scenario, replay
  determinism, and browser-session presentation-boundary parity covering five
  seeded shell attacks, blocked knockback, shell consumption, stable
  identities, observations, literal effects, scene derivation, and the
  standard reload ordering. Gameplay semantics remain unchanged; controlled
  legacy runtime, browser capture, audio/WebGPU, audiovisual, and broader
  weapon/spread parity remain `NOT_RUN`.
- **M9 vertical Combat Shotgun single-shell reload (`0.2.167`):** The
  delivered typed reload correction honors the pinned `IF_SINGLERELOAD`
  behavior with one-shell loading, atomic full/no-reserve rejection coverage,
  deterministic scenario/replay evidence, and browser-session presentation
  parity. Gameplay semantics advance to `18`; pump-action, alternate reload,
  controlled legacy runtime, browser capture, audio/WebGPU, audiovisual, and
  broader weapon/spread parity remain `NOT_RUN`.
- **M5/M6 direct-core replay custom-tile bounds (`0.2.168`):** Direct replay
  validation now rejects out-of-bounds custom tile overrides before map
  construction, matching the MCP decoder's spatial contract while preserving
  valid replay execution. Gameplay and replay semantics remain unchanged.
- **M5/M6 direct-core replay header consistency (`0.2.169`):** Direct replay
  validation now rejects unsupported schema versions and mismatched top-level
  and metadata headers before execution, matching the V2 MCP envelope contract.
  Gameplay and replay semantics remain unchanged.
- **M5 direct-core replay dimension bounds (`0.2.173`):** Direct replay
  validation now rejects map dimensions outside the MCP decoder's bounded
  `3..=512` range before map construction, while valid replay execution and
  all gameplay/replay semantics remain unchanged.
- **M5 direct-core replay structural bounds (`0.2.174`):** Direct replay
  validation now rejects oversized initial-state, custom-tile, command, and
  player-item arrays plus unsafe procedural room/content parameters before map
  construction, matching the MCP decoder's safety caps. Gameplay semantics,
  replay wire, RNG, generator, and ruleset identities remain unchanged;
  replay-file I/O, migrations, and external interchange remain open.
- **M9 vertical Blaster periodic recharge (`0.2.175`):** The equipped Blaster
  now owns a typed, deterministic cell-recharge timer: it restores one cell
  after 40 accepted commands and every 10 commands while below capacity,
  resets on successful fire, and emits a presentation-neutral
  `WeaponRecharged` event. Pure behavior, scenario/replay, MCP, and browser
  boundary parity are covered; exact legacy runtime cadence, manual reload
  denial for other `IF_NORELOAD` families, and audiovisual parity remain open.
- **M9 typed `IF_NORELOAD` manual-reload denial (`0.2.176`):** Ordinary
  `Reload` now rejects the pinned Blaster, Nuclear Plasma Rifle, and Nuclear
  BFG 9000 families before any pump, clip, reserve, timer, turn, or RNG
  mutation, using the explicit `CannotReload` error. Replay diagnostics,
  MCP-session safety, and browser-boundary parity are covered; `IF_NOUNLOAD`,
  alternate reload, and recharge behavior remain separate concerns.
- **M9 typed Nuclear Plasma periodic recharge (`0.2.177`):** The equipped
  Nuclear Plasma Rifle now owns an explicit delay-40/cadence-2/amount-1 cell
  recharge policy: it restores one cell at accepted-command tick 42 and every
  two ticks below its 24-cell capacity. Scenario/replay and browser-boundary
  parity are covered through the existing `WeaponRecharged` event; alternate
  nuke/chainfire, exact legacy runtime cadence, and audiovisual parity remain
  open.
- **M9 typed Nuclear BFG 9000 periodic recharge (`0.2.178`):** The equipped
  Nuclear BFG 9000 now owns an explicit delay-0/cadence-5/amount-1 cell
  recharge policy: it restores one cell at accepted-command tick 5 and every
  five ticks below its 40-cell capacity. Scenario/replay and browser-boundary
  parity use the existing `WeaponRecharged` event; alternate nuke,
  exact-hit/explosion, runtime, and audiovisual parity remain open.
- **M9 vertical Missile Launcher single-shell reload (`0.2.179`):** The exotic
  Missile Launcher now honors its pinned `IF_SINGLERELOAD` policy, loading one
  rocket per accepted ordinary reload while preserving atomic full/no-reserve
  rejection and shared `WeaponReloaded` event ordering. Scenario/replay and
  BrowserSession boundary parity are covered; rocket-jump, explosion, runtime,
  and audiovisual parity remain open.
- **M9 vertical Missile Launcher alternate/full reload (`0.2.180`):** The
  exotic Missile Launcher now honors its pinned `perk_altreload_full` callback,
  filling a complete sufficiently supplied deficit in one accepted
  `AltReload`, consuming exact loose-rocket reserve, emitting one aggregate
  `WeaponReloaded`, and capping cost at 2,500 units. Full and under-supplied
  rejections are atomic, with scenario/replay, MCP legal-action, and
  BrowserSession boundary parity covered; rocket-jump, explosion, runtime, and
  audiovisual parity remain open.
- **M9 Malek’s Armor periodic recharge (`0.2.181`):** Equipped Malek’s Armor
  now owns a typed delay-50/cadence-5 timer, restoring one durability at
  accepted command tick 55 and every five ticks below maximum. Full armor
  preserves its timer, received damage resets it, and the neutral
  `MalekArmorRecharged` event is covered by pure, scenario/replay, MCP, and
  BrowserSession/direct-core parity tests. General armor degradation,
  resistance, exact legacy scheduler cadence, runtime, and audiovisual parity
  remain open.
- **M9 Nuclear Plasma alternate overload (`0.2.182`):** A confirmed full-clip
  Nuclear Plasma Rifle overload now preflights stairs and pending-nuke state,
  removes the equipped weapon, spends 1,000 score count, and arms the existing
  typed nuke countdown (1 on Acid/Lava, 100 elsewhere). Typed overload and
  nuke events, atomic rejection, scenario/replay, MCP, and BrowserSession
  parity are covered; Nuclear BFG, legacy `NukeRun` map effects, runtime, and
  audiovisual parity remain open.
- **M9 Nuclear BFG 9000 alternate overload (`0.2.183`):** A confirmed
  full-clip Nuclear BFG 9000 now reuses the typed overload preflight, destroys
  the equipped weapon, spends 1,000 score count, and arms countdown 1 on
  Acid/Lava or 100 elsewhere. Atomic rejection, typed events, scenario/replay,
  MCP legal-action, and BrowserSession parity are covered; legacy `NukeRun`
  map effects, runtime, and audiovisual parity remain open.
- **M9 standard BFG 9000 exact-hit behavior (`0.2.184`):** The standard BFG
  now bypasses only the ranged to-hit sample while retaining LOS, range,
  clip, action-cost, damage RNG, and existing attack/damage events. Invalid
  target, LOS, range, and empty-clip commands remain atomic; pure combat,
  scenario/replay, and BrowserSession boundary coverage is included. Other
  exact-hit families, projectile routing, explosions, runtime, and
  audiovisual parity remain open.
- **M9 Nuclear BFG 9000 exact-hit behavior (`0.2.185`):** The Nuclear BFG
  now shares the typed exact-hit policy, bypassing only its ranged to-hit
  sample while retaining LOS, range, clip, action-cost, damage RNG, and
  existing attack/damage events. Atomic rejection, pure combat, scenario/
  replay, MCP, and BrowserSession boundary coverage is included. Shot-cost,
  projectile routing, explosions, NukeRun, runtime, and audiovisual parity
  remain open.
- **M9 standard BFG 9000 shot-cost behavior (`0.2.186`):** The standard BFG
  now preflights and consumes exactly 40 cells for each valid one-shot attack,
  rejecting clips below 40 atomically while preserving its exact-hit resolver,
  action cost, damage RNG, and existing attack/damage events. Scenario/replay,
  MCP, and BrowserSession boundary parity are covered; Nuclear BFG and other
  shot costs, projectile routing, explosions, NukeRun, runtime, and
  audiovisual parity remain open.
- **M9 Revenant’s Launcher exact-hit behavior (`0.2.187`):** Revenant’s
  Launcher now shares the typed exact-hit policy, bypassing only its ranged
  to-hit sample while preserving LOS, range, clip, action cost, damage RNG,
  and existing attack/damage events. Atomic rejection, scenario/replay, MCP,
  and BrowserSession boundary parity are covered; homing, projectile routing,
  delayed explosions, runtime, and audiovisual parity remain open.
- **M9 Nuclear BFG 9000 shot-cost behavior (`0.2.188`):** Nuclear BFG 9000
  now preflights and consumes exactly 40 cells for each valid one-shot attack,
  rejecting clips below 40 atomically while preserving exact-hit, recharge,
  overload, action-cost, damage RNG, and existing attack/damage events.
  Scenario/replay, MCP, and BrowserSession boundary parity are covered; other
  shot costs, projectile routing, explosions, NukeRun, runtime, and
  audiovisual parity remain open.
- **M9 BFG 10K exact-hit behavior (`0.2.189`):** BFG 10K now shares the typed
  exact-hit policy, bypassing only its ranged to-hit sample while preserving
  LOS, range, clip, action cost, damage RNG, and existing attack/damage events.
  Atomic rejection, scenario/replay, MCP, and BrowserSession boundary parity
  are covered; scatter, multi-shot, chainfire, projectile routing, explosions,
  runtime, and audiovisual parity remain open.
- **M9 BFG 10K shot-cost behavior (`0.2.190`):** BFG 10K now preflights and
  consumes exactly five cells for each valid Rust one-shot attack, rejecting
  clips below five atomically while preserving exact-hit, action cost, damage
  RNG, and existing attack/damage events. At that revision its legacy
  five-shot volley, scatter, chainfire, projectile routing, explosions, runtime,
  and audiovisual parity remained open; the direct-target five-projectile
  volley was delivered subsequently in `0.2.200`.
- **M9 vertical BFG 10K shot-cost encounter (`0.2.191`):** A deterministic
  `Bfg10kShotCostVertical` fixture now carries the same five-cell attack through
  ScenarioRunner, replay, MCP, and BrowserSession/direct-core boundary checks,
  preserving the 45-cell post-shot clip, hit/action/turn event ordering, and
  final-state equality. At that revision the legacy volley, scatter,
  projectile/explosion routing, controlled runtime, browser capture, and
  audiovisual parity remained open; the direct-target five-projectile volley
  was delivered subsequently in `0.2.200`.
- **M9 vertical Nuclear BFG 9000 shot-cost encounter (`0.2.192`):** A
  deterministic `NuclearBfgShotCostVertical` fixture now carries the same
  forty-cell attack through ScenarioRunner, replay, MCP, and
  BrowserSession/direct-core boundary checks, preserving the zero-cell
  post-shot clip, hit/action/turn event ordering, and final-state equality.
  Alternate overload, recharge timing, projectile/explosion routing,
  controlled runtime, browser capture, and audiovisual parity remain open.
- **M9 typed behavior vocabulary contract (`0.2.193`):** `drl-core` now exposes
  compiler-checked behavior fragments and immutable profiles for passive,
  equip/unequip, attack/hit/kill, alternate action, periodic, explicit-cost,
  and deterministic-target concepts. Medical Powerarmor, Subtle Knife, and
  Trigun profiles compose the existing dedicated transitions without runtime
  callbacks; controlled legacy runtime and presentation parity remain open.
- **M9 Null Pointer behavior profile (`0.2.194`):** The typed vocabulary now
  includes an immutable Null Pointer profile with deterministic single-target
  selection, an explicit boss/non-boss score-count branch and floor, and the
  deferred range-1/delay-50 explosion schedule. The existing dedicated
  transition and all gameplay/replay semantics remain unchanged.
- **M9 BFG 10K behavior profile (`0.2.195`):** The typed vocabulary now
  included an immutable BFG 10K profile for the exact-hit, one-projectile,
  five-cell one-shot boundary. The typed five-projectile direct-target volley
  was added in `0.2.200`; scatter, projectile routing, runtime, and
  audiovisual parity remain explicitly open.
- **M9 BFG-family behavior profiles (`0.2.196`):** The typed vocabulary now
  includes immutable standard and Nuclear BFG 9000 profiles for exact-hit,
  one-projectile, and forty-cell one-shot boundaries; Nuclear BFG also records
  its typed recharge and overload actions. Legacy projectile routing, NukeRun,
  runtime, and audiovisual parity remain explicitly open.
- **M9 vertical standard BFG 9000 shot-cost encounter (`0.2.197`):** A
  deterministic `StandardBfgShotCostVertical` fixture now carries the existing
  forty-cell one-shot policy through ScenarioRunner, replay, MCP, and
  BrowserSession/direct-core boundaries, preserving the 60-cell post-shot clip,
  hit/action/turn event ordering, and final-state equality. Projectile routing,
  explosions, controlled legacy runtime, browser capture, and audiovisual parity
  remain explicitly open.
- **M9 Nuclear BFG 9000 recharge MCP boundary (`0.2.198`):** The existing
  delay-0/cadence-5/amount-1 recharge encounter now has MCP parity coverage
  across one accepted shot and four waits, including per-step event,
  observation, state, replay, and determinism equality. Wall-clock legacy
  cadence, controlled runtime, browser capture, and audiovisual parity remain
  explicitly open.
- **M9 BFG exact-hit MCP vertical boundaries (`0.2.199`):** Standard and
  Nuclear BFG exact-hit shots now match direct-core events, observations, state,
  clip results, replay output, and determinism through the MCP session boundary.
  Projectile routing, controlled legacy runtime, and audiovisual parity remain
  explicitly open.
- **M9 BFG 10K five-projectile volley (`0.2.200`):** The typed BFG 10K path now
  resolves five ordered exact-hit direct-target shots and charges five cells per
  projectile, consuming twenty-five cells from a full clip. Scenario/replay,
  MCP, and BrowserSession/direct-core evidence preserve state, observations,
  event ordering, and determinism; scatter, projectile routing, explosions,
  chainfire, runtime, and audiovisual parity remain explicitly open.
- **M9 BFG 10K explosion schedule metadata (`0.2.201`):** Each direct-target
  volley hit now emits one ordered `Bfg10kExplosionScheduled` event carrying the
  pinned delay `25`, radius `2`, and knockback `16` payload. Scenario/replay,
  MCP, and BrowserSession boundary evidence preserve event ordering and
  determinism; explosion geometry, splash damage, knockback application,
  routing, runtime, and audiovisual parity remain explicitly open.
- **M9 standard BFG 9000 explosion schedule metadata (`0.2.202`):** Each
  direct-target standard BFG hit now emits one ordered
  `Bfg9000ExplosionScheduled` event carrying the pinned delay `33`, radius `8`,
  and knockback `16` payload. Scenario/replay, MCP, and BrowserSession
  boundary evidence preserve event ordering and determinism; explosion
  geometry, splash damage, knockback application, routing, runtime, and
  audiovisual parity remain explicitly open.
- **M9 Nuclear BFG 9000 explosion schedule metadata (`0.2.203`):** Each
  direct-target Nuclear BFG hit now emits one ordered
  `NuclearBfg9000ExplosionScheduled` event carrying the pinned delay `33`,
  radius `8`, and knockback `16` payload. Scenario/replay, MCP, and
  BrowserSession boundary evidence preserve event ordering and determinism;
  recharge, alternate overload, NukeRun, explosion geometry, splash damage,
  knockback application, routing, runtime, and audiovisual parity remain
  explicitly open.
- **M9 Nuclear Plasma Rifle behavior profile (`0.2.204`):** The immutable
  `NUCLEAR_PLASMA_BEHAVIOR` profile now records the delivered alternate
  overload and delay-40/cadence-2/amount-1 recharge fragments in declaration
  order. Dedicated transition modules remain the execution authority;
  chainfire, runtime, and audiovisual parity remain explicitly open.
- **M9 Blaster behavior profile (`0.2.205`):** The immutable
  `BLASTER_BEHAVIOR` profile now records the delivered delay-30/cadence-10/
  amount-1 recharge fragment. The dedicated weapon-recharge transition remains
  authoritative; aimed fire, runtime, and audiovisual parity remain explicitly
  open.
- **M9 Malek's Armor behavior profile (`0.2.206`):** The immutable
  `MALEK_ARMOR_BEHAVIOR` profile now records the delivered
  delay-50/cadence-5/amount-1 durability-recharge fragment. The dedicated
  armor transition remains authoritative; resistance/degradation, runtime, and
  audiovisual parity remain explicitly open.
- **M9 Lava Armor behavior profile (`0.2.207`):** The immutable
  `LAVA_ARMOR_BEHAVIOR` profile now records the delivered `TileKind::Lava`
  terrain-gated interval-5/amount-3 durability-recharge fragment. The
  dedicated armor transition remains authoritative; hazard damage/resistance,
  runtime, and audiovisual parity remain explicitly open.
- **M9 Jackhammer behavior profile (`0.2.208`):** The immutable
  `JACKHAMMER_BEHAVIOR` profile now records ordered Single/Burst mode fragments
  and the one-point score-count cost. The dedicated mode transition remains
  authoritative; spread/falloff, exact timing/accuracy, runtime, and
  audiovisual parity remain explicitly open.
- **M9 Grammaton behavior profile (`0.2.209`):** The immutable
  `GRAMMATON_BEHAVIOR` profile now records ordered Single/Burst/Auto mode
  fragments and the 200-point score-count cost. The dedicated mode transition
  remains authoritative; legacy accuracy equations, exact timing, runtime, and
  audiovisual parity remain explicitly open.
- **M9 Acid Spitter behavior profile (`0.2.210`):** The immutable
  `ACID_SPITTER_BEHAVIOR` profile now records the Acid-to-Water one-round
  terrain reload and 1,000-point score-count cost. The dedicated transition
  remains authoritative; hazard damage/resistance, runtime, and audiovisual
  parity remain explicitly open.
- **M9 Missile Launcher behavior profile (`0.2.211`):** The immutable
  `MISSILE_LAUNCHER_BEHAVIOR` profile now records ordinary single-rocket
  reload and capped full-deficit reload (`2,500` score-count units). The
  dedicated planner remains authoritative; rocket-jump, explosion, runtime,
  and audiovisual parity remain explicitly open.
- **M9 Combat Shotgun behavior profile (`0.2.212`):** The immutable
  `COMBAT_SHOTGUN_BEHAVIOR` profile now records ordinary single-shell reload
  and capped full-deficit reload (`2,500` score-count units). Dedicated
  planner and pump-action state remain authoritative; runtime and audiovisual
  parity remain explicitly open.
- **M9 Revenant's Launcher behavior profile (`0.2.213`):** The immutable
  `REVENANTS_LAUNCHER_BEHAVIOR` profile now records the pinned exact-hit
  attack policy. Dedicated combat execution remains authoritative; homing,
  projectile routing, delayed explosions, runtime, and audiovisual parity
  remain explicitly open.
- **M9 Assault Shotgun behavior profile (`0.2.214`):** The immutable
  `ASSAULT_SHOTGUN_BEHAVIOR` profile now records ordinary single-shell reload
  and capped full-deficit reload (`2,500` score-count units). The dedicated
  planner remains authoritative; runtime and audiovisual parity remain
  explicitly open.
- **M9 Combat Shotgun pump-action profile (`0.2.215`):** The immutable
  `COMBAT_SHOTGUN_BEHAVIOR` profile now also records the pump-only chamber
  action at cost `200`, alongside ordinary and capped full-deficit reload
  fragments. Dedicated chamber/reload execution remains authoritative; exact
  timing, partial-reserve, runtime, and presentation parity remain open.
- **M9 Double Shotgun dual-shot behavior (`0.2.216`):** Double Shotgun fire now
  resolves two ordered projectiles and consumes two shells per accepted command,
  with an immutable `DOUBLE_SHOTGUN_BEHAVIOR` profile and deterministic
  scenario/replay/MCP/BrowserSession parity. Spread/falloff, exact timing,
  runtime, and audiovisual parity remain open.
- **M9 Standard Shotgun knockback profile (`0.2.217`):** The immutable
  `SHOTGUN_BEHAVIOR` profile now records the current one-cell knockback hit and
  one-shell cost while generic ranged execution remains authoritative for
  collision-aware displacement and transaction safety. Exact legacy force,
  timing, spread/falloff, runtime, and audiovisual parity remain open.
- **M9 Pistol ordinary-fire profile (`0.2.218`):** The immutable
  `PISTOL_BEHAVIOR` profile now records one ordered projectile and one 9mm
  round cost while generic ranged execution remains authoritative for legality,
  damage RNG, event ordering, and transaction safety. Aimed-fire callback,
  exact legacy timing/accuracy, runtime, and audiovisual parity remain open.
- **M9 Rocket Launcher ordinary-fire profile (`0.2.219`):** The immutable
  `ROCKET_LAUNCHER_BEHAVIOR` profile now records one ordered projectile and one
  rocket cost while generic ranged execution remains authoritative for legality,
  damage RNG, event ordering, and transaction safety. Rocket-jump/explosion
  callbacks, exact legacy timing/accuracy, runtime, and audiovisual parity
  remain open.
- **M9 Combat Pistol ordinary-fire profile (`0.2.220`):** The immutable
  `COMBAT_PISTOL_BEHAVIOR` profile now records one ordered projectile and one
  9mm round cost while generic ranged execution remains authoritative for
  legality, damage RNG, event ordering, and transaction safety. Aimed-fire
  callback, exact legacy timing/accuracy, runtime, and audiovisual parity
  remain open.
- **M9 Plasma Shotgun ordinary-fire cost (`0.2.221`):** The immutable
  `PLASMA_SHOTGUN_BEHAVIOR` profile records one ordered projectile and a
  three-cell clip cost, and generic ranged execution now preflights that cost
  before mutation. Gameplay semantics advance to `43`, so stale semantics-42
  replay metadata is rejected before execution. Full spread/falloff/knockback
  semantics, exact legacy timing/accuracy, runtime, and audiovisual parity
  remain open.
- **M9 Frag Shotgun ordinary-fire cost (`0.2.222`):** The immutable
  `FRAG_SHOTGUN_BEHAVIOR` profile records one ordered projectile and a
  two-round 9mm cost, and generic ranged execution now preflights that cost
  before mutation. Gameplay semantics advance to `44`, so stale semantics-43
  replay metadata is rejected before execution. Full spread/falloff/knockback
  semantics, exact legacy timing/accuracy, runtime, and audiovisual parity
  remain open.
- **M9 Railgun ordinary-fire cost (`0.2.223`):** The immutable
  `RAILGUN_BEHAVIOR` profile records one ordered projectile and a five-cell
  cost, and generic ranged execution now preflights that cost before mutation.
  Gameplay semantics advance to `45`, so stale semantics-44 replay metadata is
  rejected before execution. Ray/piercing routing, spread/falloff semantics,
  exact legacy timing/accuracy, runtime, and audiovisual parity remain open.
- **M9 Null Pointer ordinary-fire cost (`0.2.224`):** The immutable
  `NULL_POINTER_BEHAVIOR` profile records one ordered projectile and a
  ten-cell cost, and generic ranged execution now preflights that cost before
  mutation. Gameplay semantics advance to `46`, so stale semantics-45 replay
  metadata is rejected before execution. The existing target-score branch and
  deferred explosion schedule remain authoritative; delayed explosion geometry,
  full callback parity, runtime, and audiovisual parity remain open.
- **M9 Tristar Blaster ordinary-fire volley (`0.2.225`):** The immutable
  `TRISTAR_BLASTER_BEHAVIOR` profile records three ordered projectiles and a
  five-cell per-projectile cost, and generic ranged execution now preflights
  the fifteen-cell volley cost before mutation. Gameplay semantics advance to
  `47`, so stale semantics-46 replay metadata is rejected before execution.
  Spread routing, delayed explosion geometry, callback parity, runtime, and
  audiovisual parity remain open.
- **M9 Acid Spitter ordinary-fire cost (`0.2.226`):** The immutable
  `ACID_SPITTER_BEHAVIOR` profile now records one ordered projectile and a
  ten-rocket clip cost alongside the existing terrain-reload fragments.
  Generic ranged execution preflights the ten-rocket cost before mutation;
  gameplay semantics advance to `48`, so stale semantics-47 replay metadata is
  rejected before execution. The existing Acid-to-Water reload remains
  authoritative; explosion geometry/content, spread/falloff, callback parity,
  runtime, and audiovisual parity remain open.
- **M9 Mega Buster ordinary-fire volley (`0.2.227`):** The immutable
  `MEGA_BUSTER_BEHAVIOR` profile now records three ordered projectiles and a
  three-round per-projectile cost, and generic ranged execution preflights the
  nine-round volley before mutation. Gameplay semantics advance to `49`, so
  stale semantics-48 replay metadata is rejected before execution. The kill
  morph callback, spread/falloff, runtime, and audiovisual parity remain open.
- **M9 Super Shotgun ordinary-fire volley (`0.2.228`):** The immutable
  `SUPER_SHOTGUN_BEHAVIOR` profile records two ordered projectiles and the
  default one-shell-per-projectile cost. Generic ranged execution preflights
  the two-shell volley before mutation; gameplay semantics advance to `50`, so
  stale semantics-49 replay metadata is rejected before execution. Spread/
  falloff, exact timing, runtime, and audiovisual parity remain open.
- **M9 Minigun ordinary-fire volley (`0.2.229`):** The immutable
  `MINIGUN_BEHAVIOR` profile records eight ordered projectiles and the default
  one-round-per-projectile cost. Generic ranged execution preflights the
  eight-round volley before mutation; gameplay semantics advance to `51`, so
  stale semantics-50 replay metadata is rejected before execution. Alternate
  chainfire, exact timing/accuracy, runtime, and audiovisual parity remain open.
- **M9 Chaingun ordinary-fire volley (`0.2.230`):** The immutable
  `CHAINGUN_BEHAVIOR` profile records four ordered projectiles and the default
  one-round-per-projectile cost. Generic ranged execution preflights the
  four-round volley before mutation; gameplay semantics advance to `52`, so
  stale semantics-51 replay metadata is rejected before execution. Alternate
  chainfire, exact timing/accuracy, runtime, and audiovisual parity remain open.
- **M9 Laser Rifle ordinary-fire volley (`0.2.231`):** The immutable
  `LASER_RIFLE_BEHAVIOR` profile records five ordered projectiles and the
  default one-cell-per-projectile cost. Generic ranged execution preflights the
  five-cell volley before mutation; gameplay semantics advance to `53`, so
  stale semantics-52 replay metadata is rejected before execution. Alternate
  chainfire, exact timing/accuracy, runtime, and audiovisual parity remain open.
- **M9 vertical Combat Shotgun pump action (`0.2.170`):** The deterministic
  `CombatPumpVertical` encounter now carries typed chamber state: successful
  fire empties it, empty-chamber fire rejects atomically, accepted movement and
  pump-only reload chamber it, and an empty clip follows the one-shell reload
  path. Scenario/replay and browser-boundary parity are verified; chamber
  presentation, alternate reload, and controlled legacy runtime comparison
  remain `NOT_RUN`.
- **M9 vertical Assault Shotgun alternate reload (`0.2.171`):** The
  `AssaultShotgunAltReloadVertical` encounter now carries the pinned typed
  full-reload transition: a sufficiently supplied partial clip fills to six
  shells, consumes exactly the deficit, emits one `WeaponReloaded` event, and
  pays the capped 2,500-unit cost. Full and under-supplied clips reject
  atomically; ordinary reload remains single-shell. Scenario/replay and
  browser-boundary parity are verified; partial-reserve policy, Combat
  variants, controlled legacy runtime, and presentation parity remain
  `NOT_RUN`.
- **M9 vertical Combat Shotgun alternate reload (`0.2.172`):** The
  `CombatShotgunAltReloadVertical` encounter now performs the typed full
  deficit reload, caps cost at 2,500 units, consumes exact loose-shell reserve,
  and directly resets an empty pump chamber. Full and under-supplied clips
  reject atomically; ordinary reload remains one-shell/pump-only. Scenario,
  replay, MCP, and browser-boundary parity are verified; partial-reserve policy,
  controlled legacy runtime, and presentation parity remain `NOT_RUN`.
- **Tooling & Replays (M5, M6)**: Versioned replay engine (`V2`), declarative
  ASCII scenario runners, scripted bots, batch sweep runners, and a pure Rust
  zero-dependency MCP server.
- **Asset Pipeline (M3)**: Tracked CC BY-SA 4.0 legacy graphics import from
  pinned Git revision with SHA-256 validation; audio/music/fonts remain gated.
- **Browser Playable Slice (M7)**: WASM/WebGPU shell with square-cell layout,
  DOM accessibility shell, synthesized Web Audio cues, keyboard/numpad input,
  and remote web CI acceptance.
- **Audiovisual Contracts (M8)**: Measured 32px atlas slots, normalized UVs,
  layer draw plans, emissive floor sampling, `0.1` alpha cutoff, evidence-backed
  tints (Green Armor, Phase Device, StairsDown), outline-mask compositing,
  elapsed-time animation scheduling, pure effect/missile math, bounded
  particle-decal insertion and storage contracts, and a fair explored-topology
  minimap projection with visible actor markers plus a bounded accessible DOM
  text-grid surface.
- **Typed Content & Persistence (M9, M10)**: Rust-owned definitions for current
  monsters, items, tiles, and levels, including pinned rocket, power-cell, and
  blue/red-armor families, pinned med-pack descriptions, and a typed plasma
  rifle, rocket launcher, chaingun, chainsaw, BFG 9000, shotgun variants, and
  exotic Blaster/Laser Rifle/Missile Launcher, Nuclear Plasma Rifle, Nuclear
  BFG 9000, BFG 10K, Mega Buster, Grammaton Beretta, Frag Shotgun, Revenant's
  Launcher, Railgun, Acid Spitter, Combat Pistol, Assault Shotgun, Plasma
  Shotgun, Jackhammer, Super Shotgun, Tristar Blaster, Butcher's Cleaver,
  Mjollnir, Subtle Knife, Trigun, Anti-Freak Jackal, Minigun, Onyx Armor,
  Phaseshift Armor, Gothic Armor, Malek's Armor, Cybernetic Armor, and
  Necroarmor, Medical Powerarmor, Lava Armor, and Shielded Armor variants;
  versioned
  fixed-session snapshot codec with localStorage
  persistence, bounded rejected-save quarantine, and static service-worker
  cache.
- **Evaluation & Release Hardening (M11, M12)**: Fixed-seed cohort reports with
  seed, summary, and telemetry integrity validation plus descriptive
  outcome/telemetry projections and a deterministic cohort-study CLI; release
  manifests with SHA-256 sidecars,
  optional detached signatures, cache invalidation, and checkout binding.

### Active & Open Work

- **Latest Delivered Milestone Slice (M5/M6/M13)**: `game_load_replay` transactionally
  restores the exact canonical V2 replay envelope by executing it in temporary
  core state before replacing the session. The imported replay remains the
  reset source, optional turn limits preserve terminal outcomes, subsequent
  valid actions append to its log, and terminal loads expose no legal actions.
  Malformed input remains `-32602`; simulation failure uses the delivered
  `tools/call` runtime-result boundary, and active sessions remain byte-
  identical on rejected loads. The preceding `tools/list`/
  `resources/list` pagination and tool-execution error-result contracts remain
  delivered. Replay-file IO, migrations, cross-version/external schemas,
  reconnect/resume, and broader MCP compatibility remain open.
  `game_verify_replay` accepts an exact `drl-rust-replay-v2` JSON envelope and
  verifies it read-only, including without an active session. The preceding
  0.2.56 export emits every in-memory V1 `ReplayLog` field with deterministic
  typed commands and byte-identical repeats. The fair
  legal-action catalog derives candidates from
  `PlayerObservation`, probes each candidate on a cloned `drl_core::Game`, and
  uses the filtered set for listing, response payloads, and pre-dispatch
  admission without mutating live state or exposing hidden search. Unknown/
  malformed action inputs remain `-32602`, recognized commands omitted from the
  filtered catalog return `-32001`, and the core remains authoritative. The delivered
  release-rights inventory and source/optional-bundle gate make the pinned
  CC BY-SA 4.0 graphics import, excluded legacy code/audio/music/fonts/WADs,
  and unavailable capture/media evidence explicit without claiming legal
  clearance. The preceding MCP `tools/list` conditional `allOf`/`if`/`then`
  branches for move/melee
  directions, ranged coordinate aliases, item IDs, unequip slots, and
  no-argument actions remain delivered; unknown properties remain tolerated and
  runtime dispatch is unchanged. Terminal sessions reject post-outcome actions
  with `-32001`, stair transitions report `Victory`, and reset/replay/metrics
  inspection remain available. MCP `initialize` also validates object
  `capabilities` and string `clientInfo.name`/`version` fields while retaining
  identified lifecycle gating before discovery, tools, and resources; JSON-RPC
  request IDs now reject non-scalar values before dispatch, and stateful method
  envelopes reject non-object params/arguments before execution; stateful tool
  arguments now reject wrong-typed optional integers before mutation, and
  `game_verify_replay` exposes deterministic in-memory and supplied canonical
  V2 replay verification without mutating sessions; `game_step_action` now rejects unsafe numeric
  coordinates and item IDs before mutation. Hidden-state search, unbounded
  candidate generation, and external-client compatibility remain open.
- **M9 Content Evidence**: Base, expansion, user-item, being, terrain-cell,
  and special-level evidence slices are delivered without runtime Lua or
  gameplay overclaims.
- **M11 Evaluation**: Deterministic cohort reports support a three-policy
  matrix plus descriptive depth distributions; canonical difficulty targets
  remain open.
- **M13 Tooling**: The actual `drl-app --mcp` stdio transport now has a fixed,
  repeatable JSON-RPC contract with version-aware initialize negotiation,
  identified lifecycle gating, notification side effects, ordered batch
  responses, explicit null-ID responses, and malformed-input errors; reconnect,
  full external-client compatibility, and deployment remain open.
- **M12 Accessibility**: Static and native contracts cover generated names,
  escaping, live-channel boundaries, help association, focus styling, and
  diagnostic recovery; startup now classifies insecure contexts and missing
  WebGPU before WASM initialization. Supported-Chromium runtime DOM evidence
  now covers focus transfer, keyboard progression, and generated inventory
  names; real assistive-technology and broad browser acceptance remain open.
- **M12 Release Hardening**: Optional detached release-manifest signing has
  repository and hosted ephemeral-key CI smoke coverage plus local private-key
  path/permission hygiene; production key governance and trust-root policy
  remain open.
- **Open Audiovisual Parity (M8)**: Exact legacy outline/glow and lighting/LUT
  equations from reference captures, HUD typography, and replacement audio.
- **Controlled Reference Captures (M3, M7, M8)**: Runtime captures are `NOT_RUN`
  on macOS arm64; pending execution in a controlled Linux x86-64 environment.
- **Content Breadth & Balance (M9, M11)**: Full typed migration, expanded
  content tables, and canonical difficulty target studies.
- **PWA & Release Hardening (M10, M12, M13)**: Bootstrap-independent offline
  cache registration, no-HTTP-cache worker update checks, waiting-update
  status, current-cache-only reads, and signed releases are delivered; full
  offline-after-first-load acceptance and 1.0 desktop Chromium deployment
  remain open.

---

## 4. Milestone Checklists

### M0 — Truthful Steering, Documentation, and Harness

Establish repository structure, documentation governance, and deterministic
agent workflow.

- [x] Align project proposal, roadmap, SPEC, architecture, README, and ADRs to
  a browser-first product direction.
- [x] Establish repository-local agent harness and skills
  (`drl-milestone-delivery`, `drl-test-play`, `drl-determinism-review`,
  `drl-legacy-archaeology`).
- [x] Enforce single active slice in `SPEC.md` and serialized canonical writes.
- [x] Establish evidence-based testing vocabulary (`PASS`, `FAIL`,
  `INCONCLUSIVE`, `NOT_RUN`).
- [x] Record legacy behavior shells in `docs/legacy-behavior/` (`combat.md`,
  `movement.md`, `turn-economy.md`).
- [x] Enforce 2-space formatting, tab prohibition, and automated checks via
  `sh scripts/check-repository.sh`.
- [x] Implement strict `VERSION` tracking and transition checks via
  `scripts/check-version.sh`.

---

### M1 — Deterministic Simulation Kernel

Build the standalone, pure Rust game state and turn execution loop.

- [x] Pure Rust 2D grid maps (`Map`, `Tile`, `Position`, `Direction`).
- [x] Explicit, seedable PRNG (`GameRng` wrapping SplitMix64 + Xoshiro256++).
- [x] Semantic player commands (`Command::Move`, `Command::Wait`) and typed
  errors (`CommandError`).
- [x] Deterministic turn step execution (`Game::step`) with collision checks.
- [x] Ordered simulation event stream (`GameEvent`).
- [x] Replay execution engine (`ReplayEngine`) with bit-exact reproducibility.

---

### M2 — Turn Economy and Combat

Implement energy-based scheduling and deterministic tactical combat mechanics.

- [x] Energy-based actor scheduler (`Scheduler`) supporting variable actor
  speeds.
- [x] Pure combat calculation module (`CombatResolver`) for melee and ranged
  attacks.
- [x] Combat domain models (`HitPoints`, `Speed`, `ActionCost`, `DamageAmount`,
  `DamageType`, `DeathCause`).
- [x] Combat and turn events (`AttackResolved`, `DamageApplied`, `ActorDied`,
  `ActionCostPaid`).
- [x] Health management, armor mitigation, damage clamping, and death
  transitions.
- [x] Headless combat demonstration and deterministic replay verification in
  `drl-app`.

---

### M3 — Browser-Compatible Assets, Provenance, and Fidelity Evidence

Establish asset pipelines, licensing boundaries, and legacy capture manifests.

- [x] Dedicated `drl-assets` crate for platform-neutral atlas descriptors and
  geometry.
- [x] Import tracked legacy graphics from pinned Git revision
  (`17d9be1204751899b2d69d8d3a2dde247bd0cc5c`).
- [x] Complete CC BY-SA 4.0 licensing records, attribution, and SHA-256
  checksums in `MANIFEST.txt`.
- [x] Reference capture manifest tooling (`scripts/check-reference-capture.sh`).
- [x] Automated capture manifest preflight fixture tests
  (`scripts/test-reference-capture.sh`).
- [ ] Controlled legacy runtime captures in a rights-cleared Linux x86-64
  environment (currently `NOT_RUN` on macOS arm64).
- [ ] Validated capture-to-game fidelity comparison matrix.
- [ ] Rights clearance and asset tracking for audio, music, and fonts.

---

### M4 — Perception and Content Foundations

Implement perception rules, core items, monsters, and procedural level
progression.

- [x] Deterministic Field of View (FOV) and Line of Sight (LOS) raycasting.
- [x] Fog-of-war exploration memory for revealed tiles.
- [x] Fair observation filtering in `PlayerObservation` hiding unseen entities.
- [x] Bounded player inventory (`Inventory`) and equipment slots (`Equipment`).
- [x] Weapon mechanics: magazine clips, ammo consumption, reloading, and firing.
- [x] Consumable items: Small/Large MedPacks and emergency Phase Device.
- [x] Kinetic knockback pushing targets along firing vectors without obstacle
  clipping.
- [x] Tactical monster archetypes (`FormerHuman`, `FormerSergeant`, `Imp`,
  `Demon`) with AI and loot drops.
- [x] Procedural dungeon generator with BFS reachability and room connectivity.
- [x] Exit stairs interaction (`Command::Descend`) and multi-level player
  persistence.

---

### M5 — Replays, Scenarios, and Test Agents

Build headless infrastructure for testing, bot exploration, and scenario
validation.

- [x] Versioned replay log schema (`ReplayVersion::V2`) with diagnostic error
  locations.
- [x] Replay consistency validation (`ReplayEngine::validate`), including
  V2/top-level metadata header checks and preflight rejection of out-of-bounds
  custom tile overrides and dimensions outside `3..=512` before map
  construction.
- [x] Declarative ASCII scenario fixture framework (`Scenario`,
  `ScenarioRunner`).
- [x] Observation-only test bot policies (`RandomBot`, `GreedyCombatBot`,
  `ExplorerBot`).
- [x] Headless batch simulation runner (`BatchRunner`) with statistical metric
  aggregation.
- [x] Deterministic batch sweep test suites and multi-turn scenario tests.

---

### M6 — MCP Semantic Interface

Provide standard Model Context Protocol interfaces for autonomous agents and
tooling.

- [x] Zero-dependency pure Rust JSON-RPC 2.0 protocol engine (`crates/drl-mcp`).
- [x] Standard MCP lifecycle methods (`initialize`, `ping`, `tools/list`,
  `tools/call`, `resources/list`, `resources/read`).
- [x] Complete semantic tool suite (`game_start`, `game_load_scenario`,
  `game_get_observation`, `game_list_actions`, `game_step_action`, `game_reset`,
  `game_get_metrics`, `game_save_replay`, `game_verify_replay`,
  `game_load_replay`).
- [x] `game_step_action` validates ranged coordinates and item IDs as exact
  bounded numbers before dispatch while preserving valid action aliases.
- [x] `tools/list` publishes accepted action/direction/slot and field aliases
  with JSON-safe, `u32`, and `i32` numeric bounds, plus conditional action
  requirements, while retaining unknown-field tolerance.
- [x] `game_verify_replay` verifies complete in-memory procedural and scenario
  replays without mutating sessions.
- [x] `game_save_replay` exports complete V2 replay metadata, initial-state
  containers, and typed command variants through a deterministic JSON envelope;
  replay-file IO, migration, and external interchange remain open.
- [x] `game_verify_replay` decodes and verifies a supplied canonical V2 replay
  read-only, including inactive-session verification and fail-closed malformed
  input handling. MCP session creation enforces bounded dimensions and
  procedural parameters before export; same-version object loading is
  delivered while replay-file IO, migration, and external interchange remain
  open.
- [x] `tools/list` and `resources/list` provide deterministic fixed-size pages
  with method-scoped cursors, stable reconstruction, final-page omission, and
  fail-closed invalid-cursor handling; broader MCP compatibility remains open.
- [x] Recognized `tools/call` runtime failures are successful MCP results with
  `isError: true`, deterministic text, and numeric error details, while
  malformed envelopes/arguments, malformed supplied replay input, and unknown
  methods/tools retain JSON-RPC errors; notifications, batches, and state safety
  remain deterministic.
- [x] `game_load_replay` restores a required canonical V2 replay object only
  after bounded decode and complete `ReplayEngine` execution succeed; it
  exposes `ReplayLoaded`, preserves the imported log and optional turn limit
  for appended commands/reset, restores turn-limit terminal state, and leaves
  prior active sessions unchanged on malformed or simulation-invalid input.
  Replay-file IO and migrations remain open.
- [x] Terminal `game_step_action` calls are rejected after victory, death,
  turn-limit, or stalled outcomes; stair transitions report victory while
  reset and replay/metrics inspection remain available.
- [x] Conditional `game_step_action` schema branches publish the
  action/command discriminator and action-specific required fields without
  rejecting unknown properties or changing runtime dispatch.
- [x] Strict information boundaries with explicit `dev_mode` flag for omniscient
  inspection.
- [x] Stdio transport runner and CLI integration (`drl-rust --mcp`).
- [x] Virtual AI player integration tests verifying determinism and safety.

---

### M7 — Browser-Playable M4 Slice

Deliver the initial interactive WebAssembly and WebGPU browser presentation.

- [x] `drl-web` WASM/WebGPU crate compiled via pinned `wasm-pack 0.15.0`.
- [x] Host-agnostic build, check, and serve scripts (`build-web.sh`,
  `serve-web.sh`, `check-web.sh`).
- [x] Pure `PresentationStep` and `RenderScene` transformations from simulation
  data.
- [x] Square-cell integer grid layout and viewport centering.
- [x] Accessible HTML/DOM shell with keyboard/numpad bindings and inventory
  controls.
- [x] Synthesized semantic audio cues and gesture-unlocked Web Audio mixer.
- [x] Graceful error recovery for unsupported WebGPU and suspended audio.
- [x] Remote web CI acceptance in Ubuntu headless Chrome.
- [ ] Capture-backed reference-scene comparison (pending reference captures).

---

### M8 — Audiovisual Parity

Achieve visual and acoustic equivalence with legacy presentation through
rigorous contracts.

#### Delivered Contracts

- [x] Pure `PixelViewport` integer cell layout and deterministic letterboxing.
- [x] Visibility-derived `LightingBand` (full light vs explored fog factor).
- [x] Fair health-derived `SceneTone` and pure low-health pulse alpha
  equations.
- [x] Event-ordered `EffectSpan` timing with visibility filtering.
- [x] Measured 16-column/32-pixel atlas slots and normalized UV conversion.
- [x] Registered layer metadata and shader input roles (base, colorization,
  outline, emissive).
- [x] Renderer-neutral `layer_draw_plan` and `sprite_composite_plan`.
- [x] Subpath-safe same-origin texture loading and manifest validation.
- [x] WebGPU linear `Rgba8Unorm` texture cache and external-image copy uploads.
- [x] Nearest-filtered base texture WGSL pass with emissive floor sampling.
- [x] Verified legacy `0.1` fragment alpha cutoff in textured shader.
- [x] Colorization-mask pass with evidence-backed tints (Green Armor, Phase
  Device, StairsDown).
- [x] Outline-mask GPU resource transport and straight-alpha compositing pass.
- [x] Pure sprite animation metadata (player, actors, Phase Device) and elapsed
  UV selection.
- [x] Browser `requestAnimationFrame` loop with visibility-lifecycle clock
  rebasing.
- [x] Pure post-process glow/LUT coordinate math, blur taps, blur reduction,
  and pass planning.
- [x] Pure animation arithmetic: explosion marks, cell effects, kill segments,
  FX frames, movement progress, missile steps/rays, and screen shake fade.
- [x] Pure particle contracts: burst origins, burst direction normalization/arc,
  range sampling, decal cell mapping, decal placement, and map eligibility.
- [x] Pure particle-decal insertion request (`ParticleDecalInsertion`).
- [x] Deterministic, caller-bounded `ParticleDecalStore` with explicit capacity
  enforcement.
- [x] Renderer-neutral particle-decal draw planning with opaque caller-resolved
  handles, stored-pixel placement, stable ordering, viewport filtering, and
  floor-level insertion into scene plans.
- [x] Renderer-neutral `MinimapState` projection from explored tiles and
  currently visible actor/player markers, with deterministic ordering,
  duplicate resolution, and malformed-position rejection.
- [x] Bounded semantic DOM minimap text grid with focusable, labelled markup,
  stable glyph mapping, and fail-closed oversized-dimension handling.

#### Present Slice (Expanded in `SPEC.md`)

- [x] Project explored minimap topology and fair visible actor markers without
  hidden-world access, then expose it through the browser's semantic DOM.
  Exact legacy minimap geometry and capture-backed visual parity remain open.

#### Open Work

- [ ] Exact legacy outline/glow and lighting/LUT equations from approved
  reference captures.
- [ ] Broader tint sources and content animation/effect timing.
- [ ] Capture-backed particle decal visual regressions.
- [ ] HUD typography, layout, and minimap parity.
- [ ] Rights-cleared replacement audio and music tracks.
- [ ] Automated pixel-level and audio regression test harness.

---

### M9 — Typed Content Migration and Gameplay Breadth

Migrate legacy content into typed, immutable Rust definitions without runtime
scripting.

- [x] Rust-owned definitions for current monster archetypes
  (`MonsterKind::definition()`).
- [x] Immutable definition table for item spawn families and death drops.
- [x] Immutable roll-bound tables for procedural room loot and monster spawns.
- [x] Protocol-owned immutable tile definitions (`TileKind::definition()`).
- [x] Rust-owned standard procedural level generation policy.
- [x] Pure current-content invariant validation rejects malformed typed
  definitions, roll-bound tables, level bounds, and special-level ordering;
  legacy parity, fairness targets, and dynamic migration remain open.
- [x] Build-time conversion evidence tooling for pinned shallow `register_being`
  and `register_item` Lua records, with provenance and explicit migration gaps.
- [x] Build-time conversion evidence for pinned shallow `register_cell` records,
  including deterministic simple-string decoding and explicit migration gaps.
- [x] Multi-source build-time evidence bundle for base, expansion, and user-item
  records with per-source provenance, sorted merge, and duplicate rejection.
- [x] Build-time special-level evidence index across pinned level files,
  including long-bracket map strings and explicit dynamic gaps.
- [x] Immutable Rust special-level metadata catalog for 26 active IDs, scalar
  names/text, optional depth, and deterministic lookup.
- [x] Reviewed pinned evidence-coverage gate for representative beings,
  item families, terrain cells, and all 26 indexed special-level IDs; source
  provenance, SHA-256 shape, ordering, uniqueness, and dynamic gaps remain
  explicit.
- [x] Evidence-level IDs are checked against the Rust
  `SPECIAL_LEVEL_DEFINITIONS` catalog to prevent descriptive metadata drift.
- [x] Names, optional legacy depths, entry strings, and welcome strings are
  checked against the pinned level evidence records.
- [x] Each reviewed legacy source is checked against its exact pinned SHA-256
  digest, not only digest shape.
- [x] Being, item-family, and terrain-cell bundles are checked against their
  complete reviewed record ID catalogs, not only representatives.
- [x] Every converted record is checked for scalar-only fields, structured
  migration gaps, and positive source line metadata.
- [x] The evidence crosswalk uses schema version 2 for its exact-digest and
  complete-catalog contract; obsolete schema versions fail closed.
- [x] Typed rocket and power-cell ammo families preserve pinned scalar fields,
  immutable initial amounts, stack policies, replay JSON names, and atlas slots
  without Lua runtime code.
- [x] Typed rocket-box ammo-pack boundary preserves pinned amount/capacity,
  replay kind, and atlas slot without implementing prepared-slot consumption.
- [x] Typed power-battery ammo-pack boundary preserves pinned amount/capacity,
  replay kind, two-frame marker, and atlas slot without prepared-slot use.
- [x] Typed base 10mm-chain and shell-box ammo-pack boundaries preserve pinned
  amount/capacity, replay kinds, descriptions, and atlas slots.
- [x] Typed blue armor preserves pinned protection, description, replay kind,
  shared armor atlas slot, and pure blue presentation tint; legacy resistance
  and movement modifiers remain explicit migration gaps.
- [x] Typed red armor preserves pinned protection, description, replay kind,
  shared armor atlas slot, and pure red presentation tint; legacy resistance
  and movement modifiers remain explicit migration gaps.
- [x] Pinned `smed`/`lmed` descriptions and existing med-pack atlas slots are
  preserved; dynamic difficulty/perk healing formulas remain open.
- [x] Typed plasma rifle preserves pinned cell relation, six-shot clip, `1d7`
  damage range, description, replay kind, and `SPRITE_PLASMA` slot; exact
  accuracy/timing and dynamic callbacks remain open.
- [x] Typed rocket launcher preserves pinned rocket relation, one-shot clip,
  `6d6` damage range, description, replay kind, and `SPRITE_BAZOOKA` slot;
  blast/effect callbacks and exact accuracy/timing remain open.
- [x] Typed chaingun preserves pinned 9mm relation, 40-round clip, `1d6`
  damage range, description, replay kind, and `SPRITE_CHAINGUN` slot;
  chainfire/burst effects and exact timing/accuracy remain open.
- [x] Typed chainsaw preserves pinned melee shape, `4d6` damage range,
  description, replay kind, and `SPRITE_CHAINSAW` slot; first-pickup callbacks
  and exact timing remain open.
- [x] Typed BFG 9000 preserves pinned cell relation, 100-cell clip, `10d6`
  damage range, description, replay kind, and `SPRITE_BFG9000` slot;
  standard and Nuclear BFG exact-hit behavior is covered in `0.2.184` and
  `0.2.185`; standard BFG's 40-cell shot cost is covered in `0.2.186`, while
  other shot-cost, projectile, and explosion semantics remain open.
- [x] Typed double/combat shotguns preserve pinned shell relation, clips,
  damage/range scalars, descriptions, replay kinds, and `SPRITE_DSHOTGUN`/
  `SPRITE_CSHOTGUN` slots; callbacks and spread/falloff remain open.
- [x] Typed `ublaster`, `ulaser`, and `umbazooka` preserve pinned exotic
  descriptions, ammo relations, clip/damage scalars, replay kinds, and
  measured reuse of pistol/plasma/bazooka atlas slots; Blaster recharge is
  behavior-covered in `0.2.175`, while other recharge, chainfire, rocket-jump,
  and explosion callbacks remain open.
- [x] Typed `unplasma`, `unbfg9000`, and `ubfg10k` preserve pinned heavy-energy
  descriptions, cell relations, clips/damage scalars, replay kinds, and
  measured plasma/BFG/BFG10K atlas slots; Nuclear Plasma recharge is
  behavior-covered in `0.2.177`, Nuclear BFG exact-hit in `0.2.185`, Nuclear
  BFG's 40-cell shot cost in `0.2.188`, BFG10K exact-hit in `0.2.189`, and
  BFG10K's typed five-cell shot cost in `0.2.190`, and its typed
  five-projectile direct-target volley in `0.2.200`; Nuclear Plasma exact-hit,
  chainfire, explosion, and mod callbacks remain open.
- [x] Vertical BFG 10K shot-cost encounter preserves the typed five-cell
  one-shot policy through deterministic scenario/replay, MCP, and
  BrowserSession/direct-core parity. The five-projectile direct-target volley
  is covered by the `Bfg10kExactHitVertical` and `Bfg10kShotCostVertical`
  fixtures in `0.2.200`; scatter, projectile/explosion routing, runtime, and
  audiovisual parity remain open.
- [x] BFG 10K five-projectile direct-target volley resolves five ordered
  exact-hit attack/damage pairs, consumes twenty-five cells from a full clip,
  and preserves ScenarioRunner/replay, MCP, BrowserSession, and deterministic
  RNG parity; scatter, projectile routing, explosions, chainfire, runtime, and
  audiovisual parity remain open.
- [x] BFG 10K direct-target volley emits five ordered delayed-explosion schedule
  events carrying delay `25`, radius `2`, and knockback `16`, with
  ScenarioRunner/replay, MCP, BrowserSession, and deterministic parity
  evidence; explosion geometry, splash damage, knockback application, scatter,
  routing, chainfire, runtime, and audiovisual parity remain open.
- [x] Standard BFG 9000 direct-target fire emits one ordered delayed-explosion
  schedule event carrying delay `33`, radius `8`, and knockback `16`, with
  ScenarioRunner/replay, MCP, BrowserSession, and deterministic parity
  evidence; explosion geometry, splash damage, knockback application, routing,
  runtime, and audiovisual parity remain open.
- [x] Nuclear BFG 9000 direct-target fire emits one ordered delayed-explosion
  schedule event carrying delay `33`, radius `8`, and knockback `16`, with
  ScenarioRunner/replay, MCP, BrowserSession, and deterministic parity
  evidence; recharge, alternate overload, NukeRun, explosion geometry, splash
  damage, knockback application, routing, runtime, and audiovisual parity
  remain open.
- [x] Nuclear Plasma Rifle has an immutable behavior profile for its ordered
  alternate-overload and delay-40/cadence-2/amount-1 recharge fragments;
  dedicated transitions remain authoritative and chainfire, runtime, and
  audiovisual parity remain open.
- [x] Blaster has an immutable behavior profile for its delay-30/cadence-10/
  amount-1 recharge fragment; the dedicated transition remains authoritative
  and aimed fire, runtime, and audiovisual parity remain open.
- [x] Malek's Armor has an immutable behavior profile for its
  delay-50/cadence-5/amount-1 durability-recharge fragment; the dedicated
  transition remains authoritative and armor resistance/degradation, runtime,
  and audiovisual parity remain open.
- [x] Lava Armor has an immutable behavior profile for its
  `TileKind::Lava` terrain-gated interval-5/amount-3 durability-recharge
  fragment; the dedicated transition remains authoritative and hazard
  damage/resistance, runtime, and audiovisual parity remain open.
- [x] Jackhammer has an immutable behavior profile for its ordered Single/
  Burst mode fragments and one-point score-count cost; the dedicated
  transition remains authoritative and spread/falloff, exact timing/accuracy,
  runtime, and audiovisual parity remain open.
- [x] Grammaton Cleric Beretta has an immutable behavior profile for its
  ordered Single/Burst/Auto mode fragments and 200-point score-count cost; the
  dedicated transition remains authoritative and legacy accuracy equations,
  exact timing, runtime, and audiovisual parity remain open.
- [x] Acid Spitter has an immutable behavior profile for its one-projectile
  ordinary fire and ten-rocket cost alongside the Acid-to-Water one-round
  terrain reload and 1,000-point score-count cost; generic ranged execution
  preflights the ordinary cost while dedicated reload execution remains
  authoritative. Hazard damage/resistance, fluid movement cost, explosion
  geometry/content, runtime, and audiovisual parity remain open.
- [x] Mega Buster has an immutable behavior profile for its three-projectile
  ordinary fire and three-round per-projectile cost; generic ranged execution
  resolves the ordered volley, preflights its nine-round total cost, and
  preserves atomic below-cost rejection while the kill morph callback,
  spread/falloff, runtime, and audiovisual parity remain open.
- [x] Super Shotgun has an immutable behavior profile for its two-projectile
  ordinary fire and one-shell per-projectile cost; generic ranged execution
  resolves the ordered two-shell volley, preflights its aggregate cost, and
  preserves atomic below-cost rejection while spread/falloff, exact timing,
  runtime, and audiovisual parity remain open.
- [x] Minigun has an immutable behavior profile for its eight-projectile
  ordinary fire and one-round per-projectile cost; generic ranged execution
  resolves the ordered eight-round volley, preflights its aggregate cost, and
  preserves atomic below-cost rejection while alternate chainfire, exact
  timing/accuracy, runtime, and audiovisual parity remain open.
- [x] Chaingun has an immutable behavior profile for its four-projectile
  ordinary fire and one-round per-projectile cost; generic ranged execution
  resolves the ordered four-round volley, preflights its aggregate cost, and
  preserves atomic below-cost rejection while alternate chainfire, exact
  timing/accuracy, runtime, and audiovisual parity remain open.
- [x] Laser Rifle has an immutable behavior profile for its five-projectile
  ordinary fire and one-cell per-projectile cost; generic ranged execution
  resolves the ordered five-cell volley, preflights its aggregate cost, and
  preserves atomic below-cost rejection while alternate chainfire, exact
  timing/accuracy, runtime, and audiovisual parity remain open.
- [x] Missile Launcher has an immutable behavior profile for ordinary
  single-rocket reload and capped full-deficit reload (`2,500` score-count
  units); dedicated reload/planner paths remain authoritative and rocket-jump,
  explosion, runtime, and audiovisual parity remain open.
- [x] Combat Shotgun has an immutable behavior profile for ordinary
  pump-only chamber action (`200` action units), single-shell reload, and capped
  full-deficit reload (`2,500` score-count units); dedicated reload/planner and
  pump-action paths remain authoritative and runtime, chamber presentation, and
  audiovisual parity remain open.
- [x] Double Shotgun dual-shot fire resolves two ordered projectiles and
  consumes two shells per accepted command through deterministic
  ScenarioRunner/replay/MCP/BrowserSession parity; spread/falloff, exact timing,
  runtime, and audiovisual parity remain open.
- [x] Standard Shotgun has an immutable behavior profile for its current
  one-cell knockback hit and one-shell cost; generic ranged execution remains
  authoritative while exact legacy force/timing, spread/falloff, runtime, and
  audiovisual parity remain open.
- [x] Pistol has an immutable behavior profile for its current one-projectile
  ordinary fire and one-9mm-round cost; generic ranged execution remains
  authoritative while aimed-fire callback semantics, exact legacy timing/
  accuracy, runtime, and audiovisual parity remain open.
- [x] Rocket Launcher has an immutable behavior profile for its current
  one-projectile ordinary fire and one-rocket cost; generic ranged execution
  remains authoritative while rocket-jump/explosion callback semantics, exact
  legacy timing/accuracy, runtime, and audiovisual parity remain open.
- [x] Combat Pistol has an immutable behavior profile for its current
  one-projectile ordinary fire and one-9mm-round cost; generic ranged execution
  remains authoritative while aimed-fire callback semantics, exact legacy
  timing/accuracy, runtime, and audiovisual parity remain open.
- [x] Plasma Shotgun has an immutable behavior profile for its current
  one-projectile ordinary fire and three-cell clip cost; generic ranged
  execution preflights the cost and preserves atomic below-cost rejection while
  full spread/falloff/knockback semantics, exact legacy timing/accuracy,
  runtime, and audiovisual parity remain open.
- [x] Frag Shotgun has an immutable behavior profile for its current
  one-projectile ordinary fire and two-round 9mm cost; generic ranged
  execution preflights the cost and preserves atomic below-cost rejection while
  full spread/falloff/knockback semantics, exact legacy timing/accuracy,
  runtime, and audiovisual parity remain open.
- [x] Railgun has an immutable behavior profile for its current one-projectile
  ordinary fire and five-cell cost; generic ranged execution preflights the
  cost and preserves atomic below-cost rejection while ray/piercing routing,
  spread/falloff semantics, exact legacy timing/accuracy, runtime, and
  audiovisual parity remain open.
- [x] Null Pointer has an immutable behavior profile for its current
  one-projectile ordinary fire and ten-cell cost; generic ranged execution
  preflights the cost and preserves atomic below-cost rejection while its
  target-score branch, deferred explosion geometry, full callback parity,
  exact legacy timing/accuracy, runtime, and audiovisual parity remain open.
- [x] Tristar Blaster has an immutable behavior profile for its current
  three-projectile ordinary fire and five-cell per-projectile cost; generic
  ranged execution resolves the ordered volley, preflights its fifteen-cell
  total cost, and preserves atomic below-cost rejection while spread routing,
  delayed explosion geometry, callback parity, exact legacy timing/accuracy,
  runtime, and audiovisual parity remain open.
- [x] Revenant's Launcher has an immutable behavior profile for its pinned
  exact-hit attack policy; dedicated combat execution remains authoritative and
  homing, projectile routing, delayed explosions, runtime, and audiovisual
  parity remain open.
- [x] Assault Shotgun has an immutable behavior profile for ordinary
  single-shell reload and capped full-deficit reload (`2,500` score-count
  units); its dedicated planner remains authoritative and runtime, exact timing,
  and audiovisual parity remain open.
- [x] Vertical Nuclear BFG 9000 shot-cost encounter preserves the typed
  forty-cell one-shot policy through deterministic scenario/replay, MCP, and
  BrowserSession/direct-core parity; alternate overload, recharge timing,
  projectile/explosion routing, runtime, and audiovisual parity remain open.
- [x] Nuclear BFG 9000 recharge MCP boundary preserves the typed
  delay-0/cadence-5/amount-1 policy across one accepted shot and four waits,
  with per-step event/observation/state equality and deterministic replay;
  wall-clock legacy cadence, controlled runtime, browser capture, and
  audiovisual parity remain open.
- [x] Vertical standard BFG 9000 shot-cost encounter preserves the typed
  forty-cell one-shot policy through deterministic scenario/replay, MCP, and
  BrowserSession/direct-core parity; projectile/explosion routing, runtime,
  browser capture, and audiovisual parity remain open.
- [x] Typed `umega`, `uberetta`, and `ufshotgun` preserve pinned unique-firearm
  descriptions, 9mm relations, clips/damage/range scalars, replay kinds, and
  measured plasma/pistol/combat-shotgun atlas slots; mode switching, kill
  medals, mods, spread, and timing callbacks remain open.
- [x] Typed `urbazooka`, `urailgun`, and `uacid` preserve pinned
  special-projectile descriptions, ammo relations, clips/damage/range scalars,
  replay kinds, and measured bazooka/plasma atlas slots; Revenant’s Launcher
  exact-hit behavior is covered in `0.2.187`, while homing, piercing, acid-map,
  explosion, and timing callbacks remain open.
- [x] Typed `ucpistol`, `uashotgun`, and `upshotgun` preserve pinned eitems
  descriptions, ammo relations, clips/damage/range scalars, replay kinds, and
  measured pistol/combat-shotgun/shotgun atlas slots; aimed-fire,
  alternate-reload, spread/falloff, shot-cost, and callback semantics remain
  open.
- [x] Typed `usjack`, `udshotgun`, and `utrigun` preserve pinned
  heavy-shotgun descriptions, shell/cell relations, clips/damage/range
  scalars, replay kinds, and measured combat-shotgun/double-shotgun atlas
  slots; Double Shotgun dual-shot behavior is covered in `0.2.216` and
  Standard Shotgun knockback in `0.2.217`; alternate reload, spread, chainfire,
  shot-cost, explosions, and callbacks remain open.
- [x] Typed `ubutcher`, `umjoll`, and `usubtle` preserve pinned unique-melee
  descriptions, damage/range scalars, replay kinds, and measured cleaver/knife
  atlas slots; blade/throw/alt-fire perks, callbacks, sound/UI, and exact
  timing remain open.
- [x] Typed `utrigun`, `ujackal`, and `uminigun` preserve pinned user/eitems
  descriptions, 9mm relations, clips/damage scalars, replay kinds, and measured
  pistol/chaingun atlas slots; aimed/chainfire perks, fire-rate, explosions,
  and callbacks remain open.
- [x] Typed `uoarmor`, `uparmor`, and `ugarmor` preserve pinned descriptions,
  armor values, Gothic durability, replay kinds, and shared armor atlas
  geometry; resistance, movement/knockback, no-durability, set effects, and
  callbacks remain open.
- [x] Typed `umarmor`, `ucarmor`, and `unarmor` preserve pinned descriptions,
  armor values, replay kinds, and shared armor atlas geometry; resistance,
  movement/knockback, no-destroy, item-set, and callbacks remain open.
- [x] Typed `umedparmor`, `ulavaarmor`, and `ushieldarmor` preserve pinned
  descriptions, armor values, replay kinds, and shared armor atlas geometry;
  resistance, movement/knockback, no-durability, and callbacks remain open.
- [x] Medical Powerarmor periodic repair is behavior-covered by an explicit
  core timer transition: an equipped item ticks once per accepted player
  command, heals one HP at the evidence-backed interval, spends one
  durability point, and emits a deterministic repair event. Legacy runtime
  cadence and exact presentation remain `NOT_RUN`/open.
- [x] Subtle Knife alternate invoke is behavior-covered by an explicit typed
  command: it applies the evidence-backed HP/status/score costs, damages living
  visible targets in deterministic EntityId order, emits invocation/damage
  events, and rejects tired/invalid invocations atomically. Legacy runtime and
  presentation parity remain `NOT_RUN`/open.
- [x] Trigun alternate reload is behavior-covered by an explicit typed command:
  it applies the evidence-backed confirmation, health/score costs, one-tick
  nuke transition, terminal internal damage, and ordered events without
  destroying the weapon. Legacy runtime, explosion/map effects, and
  presentation parity remain `NOT_RUN`/open.
- [x] A compile-time typed behavior vocabulary and immutable profiles represent
  passive/equipment, attack/hit/kill, alternate-action, periodic, explicit
  resource/status-cost, and deterministic-target concepts without a callback
  registry; existing stress-case transitions remain explicit and parity work is
  still open.
- [x] Null Pointer's target-dependent on-hit branch is represented by an
  immutable typed profile with deterministic target ordering, an explicit
  boss/non-boss score floor, and a deferred explosion schedule; its dedicated
  runtime transition remains the execution authority.
- [x] Grammaton Cleric Beretta alternate reload is behavior-covered by a typed
  fire-mode cycle with deterministic single/burst/auto profiles, a bounded
  200 score-count cost, ordered multi-shot resolution, and replay/MCP event
  projection. Legacy accuracy-equation and presentation parity remain
  `NOT_RUN`/open.
- [x] Jackhammer alternate reload is behavior-covered by a typed burst/single
  fire-mode toggle with one score-count cost, ordered selected-shell
  resolution, replay coverage, and MCP event projection. Legacy spread/falloff,
  timing, runtime, and presentation parity remain `NOT_RUN`/open.
- [x] Lava Armor periodic recharge is behavior-covered by a typed armor-owned
  timer: after five accepted commands on a `Lava` tile it restores up to three
  durability points and emits `LavaArmorRecharged`; full armor and non-Lava
  edge behavior are tested. Hazard damage/resistance and runtime/presentation
  parity remain `NOT_RUN`/open.
- [x] Null Pointer on-hit behavior is behavior-covered by a typed target
  score-count transition with boss/non-boss floors, deterministic hit and
  explosion-schedule events, and replay/MCP projections. Exact delayed area
  damage, geometry, runtime, and presentation parity remain `NOT_RUN`/open.
- [x] Direct player diagonal movement preserves the pinned destination-only
  validation rule, including corner cutting around two blocked cardinal
  neighbors; AI fallback remains a separate policy and legacy runtime
  comparison is `NOT_RUN`.
- [x] Monster AI movement preserves the pinned bounded candidate order:
  smoothed preferred step, raw retry, horizontal fallback, then vertical
  fallback; all blocked candidates produce `Wait` instead of broad pathfinding;
  runtime comparison is `NOT_RUN`.
- [x] Combat Shotgun normal reload preserves the pinned `IF_SINGLERELOAD`
  policy: one shell loads per accepted command, full/no-reserve rejection is
  atomic, and scenario/replay/browser-boundary parity is verified; pump-action
  is delivered in the `0.2.170` vertical slice, Assault Shotgun alternate full
  reload is delivered in `0.2.171`, and Combat Shotgun alternate full reload
  with chamber reset is delivered in `0.2.172`; partial reserve policy and
  controlled legacy runtime comparison remain open.
- [x] Missile Launcher ordinary reload preserves the pinned `IF_SINGLERELOAD`
  policy: one rocket loads per accepted command, full/no-reserve rejection is
  atomic, and scenario/replay/browser-boundary parity is verified; rocket-jump,
  explosion, and controlled legacy runtime comparison remain open.
- [x] Missile Launcher alternate/full reload preserves the pinned
  `perk_altreload_full` policy: one accepted command fills a complete,
  sufficiently supplied deficit, consumes exact loose-rocket reserve, caps the
  action cost at 2,500 units, rejects full/under-supplied clips atomically, and
  has scenario/replay/MCP/BrowserSession boundary parity; rocket-jump,
  explosion, and controlled legacy runtime comparison remain open.
- [x] Nuclear Plasma Rifle periodic recharge is behavior-covered by an
  explicit delay-40/cadence-2/amount-1 policy: one cell returns at accepted
  command tick 42, then every two ticks below capacity, with scenario/replay
  and BrowserSession parity. Alternate nuke/chainfire and exact runtime/
  presentation parity remain open.
- [x] Malek’s Armor periodic recharge is behavior-covered by an explicit
  delay-50/cadence-5/amount-1 armor-owned timer: one durability returns at
  accepted command tick 55, then every five ticks below maximum; full armor
  preserves its timer, received damage resets it, and
  `MalekArmorRecharged` is covered by pure, replay, MCP, and browser-boundary
  tests. General armor degradation/resistance and exact runtime cadence remain
  open.
- [x] Nuclear Plasma Rifle alternate overload is behavior-covered by a typed
  full-clip/confirmation/stairs/pending-nuke preflight: it destroys the
  equipped weapon, spends 1,000 score count, selects countdown 1 on Acid/Lava
  or 100 elsewhere, and emits `NuclearWeaponOverloaded` plus the existing
  nuke events. Atomicity, scenario/replay, MCP, and BrowserSession parity are
  covered; Nuclear BFG, legacy `NukeRun` map effects, runtime, and audiovisual
  parity remain open.
- [x] Nuclear BFG 9000 alternate overload reuses the typed
  full-clip/confirmation/stairs/pending-nuke preflight, destroys the equipped
  weapon, spends 1,000 score count, selects countdown 1 on Acid/Lava or 100
  elsewhere, and emits `NuclearWeaponOverloaded` plus the existing nuke events.
  Atomicity, scenario/replay, MCP, and BrowserSession parity are covered;
  legacy `NukeRun` map effects, runtime, and audiovisual parity remain open.
- [x] Standard BFG 9000 exact-hit behavior bypasses only its ranged to-hit
  sample while preserving LOS, range, clip, action cost, damage RNG, and
  existing attack/damage events; atomic rejection, pure combat, replay, and
  BrowserSession boundary parity are covered. Other exact-hit families,
  projectile routing, explosions, runtime, and audiovisual parity remain open.
- [x] Nuclear BFG 9000 exact-hit behavior reuses the typed exact-hit policy,
  bypassing only its ranged to-hit sample while preserving LOS, range, clip,
  action cost, damage RNG, existing attack/damage events, atomic rejection,
  replay determinism, MCP, and BrowserSession boundary parity. Shot-cost,
  projectile routing, explosions, NukeRun, runtime, and audiovisual parity
  remain open.
- [ ] Full migration of legacy monsters, weapons, armor, mods, and consumable
  items.
- [ ] Full migration of special levels, vaults, and dungeon branches.
- [ ] Validation gates for content fairness, determinism, replayability, and
  asset mappings.

---

### M10 — Browser Persistence and PWA State

Implement robust client-side save state and offline browser capabilities.

- [x] Versioned fixed-session command snapshot codec (`SessionSnapshot`).
- [x] Deterministic transactional replay restore from snapshots.
- [x] Best-effort browser `localStorage` save/load controls in `drl-web`.
- [x] Versioned static service-worker caching boundary.
- [x] Service-worker registration begins during page bootstrap independently of
  WebGPU startup, with explicit capability, installing, ready, and failure
  diagnostics.
- [x] Registration bypasses browser HTTP caching for worker updates and reports
  waiting updates without forcing active-client takeover.
- [x] Service-worker navigation and asset reads are isolated to the current
  generated release cache; stale/unrelated cache namespaces fail closed.
- [x] Clear Save uses an explicit accessible confirmation dialog; cancel and
  Escape preserve the saved session and active simulation without mutation.
- [ ] Full offline-after-first-load PWA lifecycle acceptance remains open while
  destructive confirmation remains an action-time acceptance step. Offline
  navigation/startup and Save/Load are evidenced on the local desktop
  Chromium target in
  [`docs/acceptance/browser-offline-2026-08-23.md`](acceptance/browser-offline-2026-08-23.md).
- [x] Corruption recovery policy: fail-closed restore, bounded quarantine, and
  playable boot/load when storage cleanup is unavailable.
- [x] Replay-compatible save migration for the shipped V1 token into strict V2
  after successful transactional restore; storage-write failures remain
  playable warnings.
- [x] Explicit non-goal: No online accounts or centralized backend services.

---

### M11 — Balance and Evaluation

Provide statistical evaluation tools to benchmark bot performance and gameplay
balance.

- [x] Headless fixed-seed cohort configuration (`CohortConfig`) and report
  generation (`CohortReport`).
- [x] Cohort report integrity validation (record count, seed order, summary
  coherence).
- [x] Descriptive cohort outcome distributions (victory, death, turn limit,
  stalled, in progress).
- [x] Pure compatible cohort comparisons and caller-owned outcome-rate tolerance
  gates.
- [x] Telemetry distributions (shot accuracy, damage, kills, pickups, items
  used) and delta comparisons.
- [x] Telemetry invariant gate rejects impossible shot counts, level zero, and
  records beyond the configured turn budget before projection.
- [x] Bounded deterministic `drl-rust cohort` study command emits stable
  machine-readable outcome and telemetry reports for greedy, random, and
  explorer policies.
- [x] Deterministic `drl-rust cohort --bot all` matrix output runs the declared
  greedy, random, and explorer policies over shared cohort configuration.
- [x] Descriptive depth distribution projects validated `level_reached` metrics
  into ascending buckets with stable per-depth sample rates.
- [ ] Difficulty curve validation against canonical target metrics.
- [x] Strict isolation between player observations and evaluation telemetry is
  enforced by the observation-free `EpisodeRecord` boundary.

---

### M12 — Static Web Productization and Release Hardening

Harden the browser deployment for production hosting, accessibility, and
diagnostics.

- [x] Deterministic release manifest generation (`release-manifest.json`)
  recording version, source revision, sorted SHA-256 hashes, and license
  metadata.
- [x] Manifest SHA-256 digest sidecar (`release-manifest.sha256`).
- [x] Source-derived service worker cache naming and invalidation policy.
- [x] Mocked service-worker lifecycle and fetch contract test harness.
- [x] Git checkout-identity binding and source-identity validation.
- [x] Static HTML shell accessibility audit (landmarks, named controls,
  labels, focus, live regions).
- [x] Dynamic interaction accessibility contract: escaped item-qualified
  inventory actions, one live status channel, canvas help association,
  focus-visible styling, and diagnostic focus recovery.
- [x] Local accessible browser-support and startup diagnostics panel.
- [x] Supported-Chromium runtime DOM evidence covers canvas focus, live status,
  keyboard turn progression, and item-qualified inventory names; see
  [`docs/acceptance/browser-dynamic-dom-2026-08-23.md`](acceptance/browser-dynamic-dom-2026-08-23.md).
- [x] Pure startup capability classification rejects insecure contexts and
  missing WebGPU with stable recovery guidance before WASM initialization.
- [x] Optional detached cryptographic release signing and fail-closed manifest
  verification using an externally supplied OpenSSL key.
- [x] Repository and hosted CI smoke coverage signs with an ephemeral
  runner-local key, verifies the manifest, and keeps the private key outside
  `dist`.
- [x] Local signing rejects release-tree keys, symlinks, and group/world-
  readable private-key permissions before OpenSSL runs.
- [ ] Production key custody, secret provisioning, rotation, and trust-root
  policy.
- [ ] Dynamic WCAG 2.1 AA and screen-reader accessibility acceptance.
- [x] Real-world browser service-worker installation, offline navigation, and
  reload acceptance on the local desktop Chromium target; OS-level install
  prompts and production HTTPS deployment remain M13 scope.
- [x] Graceful fallback and diagnostics for known unsupported startup
  environments; untested browsers and backends remain unclaimed.

---

### M13 — Browser-First 1.0 Release

Final release readiness, documentation, and static distribution.

- [ ] Production static HTTPS deployment for desktop Chromium (Chrome/Edge) with
  WebGPU.
- [ ] Approved audiovisual parity matrix verified against reference captures.
- [ ] Fully functional offline PWA installation.
- [x] Deterministic `drl-app --mcp` stdio lifecycle contract covers discovery,
  gameplay, replay/metrics, reset, scenario, resources, and fairness denial.
- [x] Stdio notifications apply side effects without responses while identified,
  explicit null-ID, and malformed requests retain response contracts.
- [x] Nonempty stdio batch arrays preserve response order, omit notification
  members, and reject empty batches deterministically.
- [x] `initialize` echoes supported `2024-11-05`, falls back deterministically
  for unsupported strings, and rejects missing/non-string versions with
  `-32602`; full lifecycle and external-client compatibility remain open.
- [x] Lifecycle phase gate requires identified `initialize` followed by
  `notifications/initialized` before discovery, tools, or resources; premature
  and duplicate transitions return `-32003` without resetting game state.
- [x] Initialize envelope validation requires object `capabilities` and
  `clientInfo` with string `name`/`version`, returning `-32602` without
  advancing lifecycle for malformed fields; full schema/client compatibility
  remains open.
- [x] JSON-RPC request IDs accept strings, numbers, and explicit `null`, while
  boolean/array/object IDs return `-32600` before dispatch; notification and
  batch response boundaries remain deterministic.
- [x] `tools/call` and `resources/read` reject non-object params, and
  `tools/call.arguments` rejects non-object values with `-32602` before stateful
  execution; full MCP schema compatibility remains open.
- [x] `game_start` and `game_load_scenario` reject wrong-typed optional integer
  arguments and values outside the accepted finite JSON-safe integer range
  (`0..=2^53`, with `u32` for dimensions) with `-32602` before session
  mutation; valid defaults remain unchanged.
- [x] `game_list_actions` derives fair candidates, filters them through cloned
  core probes, and uses the exact filtered set for discovery, response
  payloads, and pre-dispatch admission; recognized omitted commands return
  `-32001` without mutating turn, metrics, recent events, replay, or
  observation, while malformed input remains `-32602`.
- [x] `game_save_replay` exports every in-memory V2 `ReplayLog` field through a
  deterministic `drl-rust-replay-v2` envelope with complete initial-state
  containers and typed semantic command objects; import/load, validation,
  migration, and external replay interchange remain open.
- [ ] Complete deterministic headless/MCP agent tooling suite and external
  client compatibility.
- [x] Comprehensive public rights inventory and release documentation is
  machine-checked for source provenance, exact manifest rights, and optional
  bundle exclusions; unresolved legal clearance remains open.

---

## 5. Post-1.0 Portability

Future work expanding platform and input support without compromising core
invariants:

- [ ] WebGL2 fallback renderer for older devices and unsupported browsers.
- [ ] Cross-browser validation for Firefox and Safari.
- [ ] Mobile/touch interface and responsive on-screen controls.
- [ ] Gamepad / controller input support.
- [ ] Native desktop application packaging for Linux, macOS, and Windows.

---

## 6. Delivery Gates & Verification

Every milestone and pull request must satisfy these automated gates:

- **Repository Integrity**: `sh scripts/check-repository.sh` (formatting,
  clippy, unit/integration tests, agent harness checks).
- **Asset Integrity**: `sh scripts/check-assets.sh` (manifest checksums and
  licensing).
- **Version Contract**: `scripts/check-version.sh` (valid `x.y.z` transition;
  no bumps on doc-only changes).
- **Web Contracts**: `scripts/check-web.sh` (WASM target build and native/WASM
  contract tests).
- **Release Manifest**: `sh scripts/build-web.sh && sh scripts/check-release-manifest.sh`
  (static artifact bundle validation).
- **Evidence**: `PASS`, `FAIL`, `INCONCLUSIVE`, and `NOT_RUN` are explicit; remote criteria are never marked complete from local
  inference.
