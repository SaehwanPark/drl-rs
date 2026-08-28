# Specification

Last reviewed: 2026-08-28
Current project version: `0.2.192`

The [Roadmap](docs/DRL-Rust_Project_Roadmap.md) owns overall milestone scope,
ordering, and delivery tracking. The current steering constraints in
[`docs/steering/`](docs/steering/README.md) govern which candidate work is
eligible to become active while their stop gates remain open. This file expands
**exactly one active implementation slice** into observable behavior, pure
contracts, acceptance criteria, and verification boundaries.

---

## 1. Status Vocabulary

- `[x]` — **Delivered and Verified**: Supported by checked repository evidence,
  tests, or artifacts.
- `[ ]` — **Present or Future Work**: Open implementation task or acceptance
  gate.
- `NOT_RUN` — **Environment Unavailable**: Required prerequisites were not
  present; not an inferred pass.
- `INCONCLUSIVE` — **Unresolved Evidence**: Output exists but cannot
  definitively satisfy criteria without additional evidence.

---

## 2. Active Implementation Slice: M9 — Vertical Nuclear BFG Shot-Cost Encounter

### 2.1 Objective

Exercise the delivered typed Nuclear BFG 9000 `shotcost=40` behavior end-to-end
in one deterministic canonical encounter, preserving the direct-core command
result through ScenarioRunner, replay, MCP, and BrowserSession boundaries.

### 2.1a Scope and steering gate

- **Steering priority:** Vertical canonical fidelity and typed legacy behavior.
- **Steering gates:** Gate A rejected-input safety, Gate B explicit replay
  compatibility, and Gate D callback behavior evidence.
- **Observable outcome:** A fixed ASCII encounter equips a Nuclear BFG 9000,
  places a visible static target at a legal ranged position, and accepts one
  attack that consumes exactly forty cells while preserving exact-hit, damage,
  action cost, and event ordering. ScenarioRunner, replay, MCP, and
  BrowserSession all expose the same accepted result and final state.
- **Gameplay/replay impact:** Gameplay semantics remain `37`; replay wire
  schema, RNG, generator, and ruleset identities remain unchanged. Project
  version advances from `0.2.191` to `0.2.192` for the added executable
  vertical coverage.
- **Protocol/domain ownership:** `drl-core` continues to own typed exact-hit
  and shot-cost policies; protocol, MCP, render, audio, and browser layers
  project the existing stable command/event contracts without new gameplay
  balance.
- **Evidence boundary:** Pinned legacy source at revision
  `17d9be1204751899b2d69d8d3a2dde247bd0cc5c` plus core, scenario, replay, MCP,
  and browser-boundary tests are authoritative. Controlled legacy runtime and
  audiovisual comparisons remain `NOT_RUN`.
- **Non-goals:** Nuclear BFG alternate overload, projectile-path routing,
  radius and falloff, delayed explosions, recharge timing changes, mod
  behavior, additional shot-cost families, replay-file IO/migrations, exact
  legacy runtime behavior, browser capture, and audiovisual parity.

### 2.2 Why this slice is bounded

The pinned Nuclear BFG 9000 carries `shotcost=40` alongside exact-hit,
recharge, overload, and explosion behavior. This slice composes the already
delivered typed shot-cost policy with the existing vertical scenario/replay and
browser-boundary harnesses, preserving exact-hit, LOS/range/clip preflight,
damage RNG, and event flow. Alternate overload, projectile routing, explosions,
and audiovisual effects remain separate evidence and implementation slices.

Additional broad scalar-only family additions remain gated by the open behavior
and evidence criteria in Section 2.8.

### 2.3 Historical Gate C item identity and spawn contracts

The catalog declaration is the only routine source for `ItemArchetype` and
`ItemSpawnKind` variants, ordered `ALL` views, stable wire names, normalized
spawn values, and archetype mapping.
`from_stable_name` must round-trip every archetype, names must be unique, and
existing consumers must continue to iterate the same order. Count-sensitive
loose-ammo reconstruction and semantic definition/presentation mappings remain
explicit.

#### Legacy evidence boundary

No legacy source claim is needed for this registration slice. Protocol and asset
catalog tests are authoritative; external runtime and presentation comparisons
remain `NOT_RUN`.

### 2.4 Previous movement and historical correctness contracts

#### AI movement contract

For a monster at `(6,6)` and player at `(4,4)`, if the smoothed preferred
diagonal destination `(5,5)` is blocked and `(5,6)` is open, `MonsterAi` must
choose `Direction::West`; if horizontal is also blocked and `(6,5)` is open,
it must choose `Direction::North`. For a strongly skewed target, a blocked
smoothed cardinal must retry the raw diagonal. If all candidates are blocked,
it must return `MonsterAction::Wait` even when another neighboring tile is
open. A same-position target must also wait rather than emitting
`Direction::None`. The decision must not consume RNG. A fixed-map integration
test also verifies the selected movement event during a scheduled monster turn.

#### Previous direct-player movement contract

Direct player movement computes one target from the submitted direction and
validates that destination's bounds, terrain, and occupancy. It does not apply
an adjacent-cardinal corner check. For a fixed 5×5 map with the player at
`(2,2)`, walls at `(1,2)` and `(2,1)`, and a walkable destination at `(1,1)`,
`Command::Move(Direction::NorthWest)` must succeed, emit `EntityMoved`, place
the player at `(1,1)`, and advance exactly one accepted turn. The move consumes
no RNG. Monster `MoveTowards` fallback remains outside this direct-player
contract.

#### Historical Gate A/C evidence inventory

The following entries remain historical evidence for already-delivered
correctness and catalog work; they are retained here so the active slice does
not erase the project's verified contracts.

**Delivered in `0.2.122`:** Death-drop terrain preflight runs before player
melee/ranged and Subtle Knife mutations, with focused tests proving exact
rejection identity and ranged clip/RNG restoration. This remains part of the
verified Gate A baseline for the current command surface.

**Delivered in `0.2.123`:** The protocol archetype catalog owns the loose-ammo
count-shape projection consumed by MCP replay JSON decoding.

**Delivered in `0.2.124`:** Core definition lookup uses
`CURRENT_ITEM_DEFINITIONS` in stable spawn-family order, removing the duplicate
`definition_for_spawn_kind` registration match while preserving explicit
core-owned balance data.

**Previous bounded delivery target (`0.2.125`):** Ordinary
`ItemSpawnKind::from_archetype` lookup derives from `ItemSpawnKind::ALL`, with
explicit tests for all family round trips, missing loose-ammo counts, and
unknown-family rejection.

**Previous bounded delivery target (`0.2.126`):** The pinned legacy movement
source records destination-only direct player validation. A focused fixed-map
test proves that a walkable diagonal destination is accepted when both
adjacent cardinal tiles are walls; a controlled legacy runtime comparison is
`NOT_RUN`.

**Previous bounded delivery target (`0.2.127`):** The pinned legacy AI movement
source records smoothed preferred, raw retry, horizontal, and vertical
candidate order. Focused unit and scheduled-turn integration tests prove the
fallback order, strongly-skewed raw retry, same-position `Wait`,
blocked-candidate `Wait`, and unchanged RNG; a controlled legacy runtime
comparison is `NOT_RUN`. The strongly-skewed smoothing ratio is supporting
evidence from the dirty untracked helper rather than a clean pinned-source
fact.

**Previous bounded delivery target (`0.2.128`):** The pinned legacy Grammaton
source records the single/burst/auto cycle, `2d6`/`1d8`/`1d7` profiles,
three-/six-shot counts, and 200 score-count cost. Typed transition, ordered
multi-shot, partial-clip rejection, replay, and MCP projections are covered;
the legacy accuracy equation and controlled runtime/presentation comparison
remain `NOT_RUN`.

**Delivered in `0.2.90` on `codex/fix-equip-rejection-atomicity`:** Equipping a
non-equippable inventory item must return `CommandError::CannotEquip` without
removing, reordering, or otherwise changing that item or any other simulation
state. Equipment-slot eligibility must be validated through immutable state
before the inventory is mutated. The existing exact-`Game` equality helper
provides the executable rejection contract.

**Delivered in `0.2.91`:** Player pickup rejection is transactional when a
ground ammunition stack would partially merge into an existing inventory
stack and then encounter a full backpack. Inventory
insertion must validate or stage all fallible capacity work before committing
stack mutations. A rejected pickup must leave the complete `Game` state,
including the original ground item and every inventory stack, byte-for-byte
equivalent under `Game` equality. This closes only the pickup portion of Gate A;
drop, use, unequip, reload, descent, and command-wide audit coverage remain
follow-up work.

**Delivered in `0.2.92`:** Player drop validates the destination
position before removing an inventory item. If a malformed or externally
constructed world places the player outside the map, `Command::Drop` must return
`CommandError::OutOfBounds` with the complete `Game` state unchanged. This
closes only the drop portion of Gate A; use, unequip, reload, descent, and
command-wide audit coverage remain follow-up work.

The preceding equip target closed that portion of Gate A, and the ranged-command
portion was delivered in `0.2.89`; drop, unequip, use, reload, movement, melee,
descent, and command-wide audit coverage remain follow-up work. RNG/replay
semantics, content registration, protocol/domain ownership, legacy behavior,
browser behavior, and rights are unchanged and explicitly out of scope.

**Delivered in `0.2.93`:** Unequip rejection paths retain exact
`Game` identity. An empty equipment slot must return `CommandError::SlotEmpty`
without mutation, and a full inventory must return `CommandError::InventoryFull`
before removing the equipped item. This evidence-only target closes the
unequip portion of Gate A; use, reload, descent, and command-wide audit
coverage remain follow-up work.

**Delivered in `0.2.94`:** Use rejection paths retain exact `Game` identity. A
present non-consumable item must return `CommandError::CannotUse`, and a
missing item must return `CommandError::ItemNotFound`, without mutating the
inventory, health, RNG, or any other game state. Reload, descent, and
command-wide audit coverage remain follow-up work.

**Delivered in `0.2.95`:** Reload rejection paths retain exact `Game` identity.
No equipped ranged weapon, an already-full clip, and missing matching reserve
ammunition must return their documented `CommandError` without mutating
equipment, inventory, turn, RNG, or any other game state. Descent and
command-wide audit coverage remain follow-up work.

**Delivered in `0.2.96`:** Descend rejection retains exact `Game` identity.
Attempting `Command::Descend` away from a stairs tile must return
`CommandError::NotOnStairs` without changing the world, player, level, turn,
RNG, or any other game state. Command-wide audit coverage remains follow-up
work.

**Delivered in `0.2.97`:** Movement rejection retains exact `Game` identity.
Blocked-terrain and out-of-bounds movement targets must return their documented
`CommandError` without changing the player, visibility, turn, RNG, or any other
game state. Command-wide audit coverage remains follow-up work.

**Delivered in `0.2.98`:** Melee rejection retains exact `Game` identity. An
invalid direction and an empty adjacent target must return their documented
`CommandError` without changing the player, turn, RNG, or any other game state.
Command-wide audit coverage remains follow-up work.

**Delivered in `0.2.99`:** Inventory-command rejection retains exact `Game`
identity. Missing-item Equip/Drop and no-ground-item Pickup must return their
documented `CommandError` without changing inventory, ground items, turn, RNG,
or any other game state. Command-wide audit coverage remains follow-up work.

**Delivered in `0.2.100`:** Ranged target rejection retains exact `Game`
identity. Out-of-bounds and empty-target `AttackRanged` commands must return
their documented `CommandError` without changing ammunition, turn, RNG, or any
other game state. Command-wide audit coverage remains follow-up work.

**Delivered in `0.2.101`:** Ranged weapon-state rejection retains exact `Game`
identity. `AttackRanged` with no equipped ranged weapon or an empty clip must
return its documented `CommandError` without changing ammunition, turn, RNG, or
any other game state. Command-wide audit coverage remains follow-up work.

**Delivered in `0.2.102`:** Phase-device rejection retains exact `Game`
identity. Using a phase device when no unoccupied walkable destination exists
must return its documented `CommandError` without removing the item or changing
RNG, turn, or any other game state. Command-wide audit coverage remains
follow-up work.

**Delivered in `0.2.103`:** Terminal-state rejection retains exact `Game`
identity. After a normal combat death sets the game-over flag, any subsequent
command must return the documented terminal `CommandError` without changing the
terminal state, turn, RNG, or any other game state. Command-wide audit coverage
remains follow-up work.

**Delivered in `0.2.104`:** Pickup validates the player's map position before
ground-item removal. An out-of-bounds player position must return
`CommandError::OutOfBounds` with exact `Game` identity, preventing a malformed
pickup path from removing an item before rollback can fail. Command-wide audit
coverage remains follow-up work.

**Delivered in `0.2.105`:** `GameRng::gen_range` uses rejection sampling over
the complete `2^32` output domain, avoiding modulo bias while preserving the
explicit raw PRNG stream. Raw output, bounded samples (including rejection),
and current probability conversion have fixed golden vectors. Replay metadata
does not yet carry the sampler identifier; replay semantics versioning remains
follow-up work.

**Delivered in `0.2.106`:** Replay metadata now carries a gameplay-semantics
version and ruleset/content identity independently from historical wire schema
V1. Core and MCP replay validation reject unsupported values before simulation,
avoiding silent reinterpretation through current rules.

**Delivered in `0.2.107`:** Core probability rules use `GameRng::gen_bool_ratio`
with unbiased integer-domain sampling. Procedural room decoration now expresses
its 1/2 branch as an exact ratio; the floating-point helper documents its role
as an outer convenience conversion and has golden coverage.

**Delivered in `0.2.108`:** Procedural replay metadata carries a separate
generator-semantics version. Core and MCP validation require that version only
when a replay reconstructs a procedural map, while fixed-map replays continue
to use gameplay and ruleset identities without a generator-policy dependency.

**Delivered in `0.2.109`:** `Game::step` now snapshots and restores the complete
state when any command rejects, preserving turn, world, and RNG even if a later
fallible substep fails after an earlier mutation. Existing prepare/commit
handlers remain the preferred local pattern; the bounded rollback guard is an
interim command-boundary backstop.

**Delivered in `0.2.110`:** The command audit records accepted-only coverage for
`Wait` and `Move(None)`, whose dispatch branches have no reachable rejection;
all other current command classes have reachable rejection cases protected by
the transactional boundary.

**Delivered in `0.2.111`:** Structural item validation now consumes the
  canonical `ItemSpawnKind::ALL` catalog owned by the stable protocol spawn
  contract and exposed to core through `CURRENT_ITEM_SPAWN_KINDS`;
the catalog has uniqueness and definition-backed coverage tests. Behavioral and
presentation projections remain explicit until their own fan-out slices migrate.

Verification passed the focused and full `drl-core` suites, locked workspace
format/Clippy/tests, the base-relative version contract, and the repository
consistency script. Native/WASM compile and web contract checks also passed for
the behavior-preserving renderer lint adaptation required by the local Rust
1.97.1 toolchain; the real browser runner and controlled legacy runtime were
`NOT_RUN`.

For every `Game::step(command)` invocation:

- if the result is `Ok(events)`, all state changes must correspond to the
  accepted command and emitted simulation events;
- if the result is `Err(error)`, the complete `Game` value after the call must
  compare equal to the complete `Game` value before the call.

State identity on rejection includes, at minimum:

- world and actor state;
- inventory and equipment;
- item/entity counters;
- map and explored/visibility state;
- scheduler energy;
- turn and terminal state;
- RNG internal state;
- any future simulation-owned counters or caches included in `Game` equality.

#### Acceptance criteria

- [x] Add a reusable invariant test helper asserting `Err => before == after`
  for the bounded ranged-command slice.
- [x] Cover every current command class with a reachable rejection scenario
  where one exists; `Wait` and `Move(None)` have accepted-path coverage because
  their dispatch branches have no reachable rejection.
- [x] Fix ranged attacks so target geometry/range/legality is validated before
  ammunition, RNG, or other mutable state is consumed.
- [x] Validate equip eligibility before removing the inventory item, with an
  exact-state rejection test for a reachable non-equippable item.
- [x] Make pickup rejection atomic when inventory insertion would partially
  merge ammunition before returning `InventoryFull`.
- [x] Validate the drop destination before removing an inventory item, with an
  exact-state rejection test for an out-of-bounds player position.
- [x] Cover unequip rejection for an empty slot and a full inventory with exact
  `Game` equality tests.
- [x] Cover use rejection for a present non-consumable and a missing item with
  exact `Game` equality tests.
- [x] Cover reload rejection for no equipped ranged weapon, a full clip, and
  missing matching reserve ammunition with exact `Game` equality tests.
- [x] Cover off-stairs descend rejection with an exact `Game` equality test.
- [x] Cover blocked-terrain and out-of-bounds movement rejection with exact
  `Game` equality tests.
- [x] Cover invalid-direction and empty-target melee rejection with exact
  `Game` equality tests.
- [x] Record pinned legacy movement evidence and protect destination-only
  diagonal corner cutting with a deterministic integration test.
- [x] Record pinned legacy AI movement evidence and protect the smoothed/raw,
  horizontal, vertical, and bounded `Wait` candidate policy with deterministic
  decision and scheduled-turn tests.
- [x] Cover missing-item Equip/Drop and no-ground-item Pickup rejection with
  exact `Game` equality tests.
- [x] Cover out-of-bounds and empty-target ranged rejection with exact `Game`
  equality tests.
- [x] Cover no-equipped-weapon and empty-clip ranged rejection with exact
  `Game` equality tests.
- [x] Cover phase-device use with no valid destination using an exact `Game`
  equality test.
- [x] Cover a command after normal game-over termination with an exact `Game`
  equality test.
- [x] Validate pickup position before ground-item removal with an exact
  out-of-bounds `Game` equality test.
- [x] Fix pickup/use/drop/reload and other multi-step commands so expected
  validation failures cannot lose or partially mutate items through the
  transactional command boundary.
- [x] Audit all current `Game::step` command paths: handlers retain
  prepare/commit validation where practical and the shared rollback guard covers
  every dispatch branch.
- [x] Preserve turn and RNG state exactly on rejection.
- [x] Document the chosen implementation pattern: prepare/commit is preferred;
  a bounded rollback guard is used as an interim correctness backstop.
- [x] Preflight death-drop destinations before player melee, ranged, and
  Subtle Knife mutations so expected terrain failures do not occur after
  combat, ammo, RNG, or typed behavior effects commit.
- [x] Cover invalid `Invoke` and `AltReload` commands plus blocked late
  death-drop failures with exact `Game` equality and no ground-item creation.

### 2.5 RNG and Replay Semantics Contract

Determinism means more than running the current implementation twice. A replay
must state which gameplay semantics it expects, and random sampling behavior
must be intentional because changing it changes deterministic histories.

#### Acceptance criteria

- [x] Replace modulo-based bounded integer sampling with an unbiased algorithm
  whose output contract is documented and tested.
- [x] Define boolean/probability sampling with an explicit integer-domain
  contract; core rules prefer rational ratios and the outer float conversion is
  bounded and tested.
- [x] Add fixed raw, bounded, rejection, and probability RNG vectors for the
  supported sampler semantics version.
- [x] Distinguish replay wire-schema version from engine/gameplay semantics
  version.
- [x] Record a ruleset/content semantics identifier sufficient to reject
  incompatible replays (migration remains future work).
- [x] Define procedural-generation semantics with a separate generator version
  in replay metadata; validate it only for procedural replays.
- [x] Reject incompatible replay semantics explicitly until a migration path is
  implemented; do not silently reinterpret an old replay through current item
  definitions.

### 2.6 Typed Content and Behavior Architecture Contract

The project must retain compile-time exhaustiveness without requiring every new
ordinary content archetype to be manually synchronized across many independent
registries.

Legacy Lua callbacks are evidence of required behavior, not an architecture to
recreate. The Rust model should use a small number of explicit typed effect and
action concepts, with bespoke state machines only where composition is
insufficient.

#### Acceptance criteria

- [x] Inventory all current manual fan-out points for adding an item archetype;
  see [the verified inventory](docs/steering/decisions/item-registration-fanout-inventory.md).
- [x] Establish one authoritative compile-time item/content catalog or
  equivalent source of truth for routine stable identity and normalized spawn
  registration. Stable spawn-family identity, definition coverage, replay JSON
  encode/decode and completeness fixtures, stable-name parsing/display, and
  routine descriptor coverage tests derive from typed catalogs/projections;
  core structural validation uses the protocol catalog through
  `CURRENT_ITEM_SPAWN_KINDS`; behavioral and presentation mappings remain
  explicit.
- [x] Generate or mechanically derive routine projections such as stable IDs,
  display strings, validation coverage, replay names, and normalized spawn
  lookup where doing so does not weaken type safety. Count-sensitive payload
  reconstruction remains explicit.
- [x] Derive the replay loose-ammo count requirement from the protocol
  archetype catalog and remove the duplicate MCP decoder variant list.
- [x] Resolve core item definitions through a single catalog ordered to the
  protocol spawn-family catalog, with length, uniqueness, and order coverage.
- [x] Derive ordinary inverse spawn lookup from `ItemSpawnKind::ALL` while
  retaining explicit loose-ammo count reconstruction and rejection coverage.
- [x] Keep genuinely behavioral code explicit and reviewable rather than
  embedding arbitrary callbacks in the catalog.
- [ ] Define a typed behavior vocabulary that can represent at least:
  **Partial in `0.2.120`:** explicit Medical Powerarmor periodic repair and
  Subtle Knife alternate invoke transitions are delivered; the Trigun
  alternate-reload transition is delivered in `0.2.121` and the broader
  vocabulary remains open.
  - passive stat/resistance modifiers;
  - equip/unequip effects and item-set membership;
  - attack/hit/kill effects;
  - alternate fire/reload/use actions;
  - recharge or periodic effects;
  - explicit costs such as HP, energy, ammo, or status;
  - deterministic target selection over fair/current simulation state.
- [x] Avoid a generic string-keyed event bus or unconstrained dynamic callback
  registry in `drl-core`.
- [x] Keep runtime Lua absent from the shipped game.

### 2.7 Behavior Stress Cases

The behavior model is not accepted based on toy examples alone. Demonstrate it
using a small set of deliberately difficult legacy cases selected to exercise
different mechanisms.

The initial target set should include at least three of the following or
similarly difficult equivalents, with source evidence recorded from the pinned
legacy revision:

The first selected case is Medical Powerarmor; its source and callback
decomposition are recorded in
[`docs/legacy-behavior/medical-powerarmor.md`](docs/legacy-behavior/medical-powerarmor.md).
Its typed Rust implementation is delivered in `0.2.119`; controlled legacy
runtime cadence and presentation parity remain unclaimed.

The second selected case is Subtle Knife alternate fire; its source and
callback decomposition are recorded in
[`docs/legacy-behavior/subtle-knife.md`](docs/legacy-behavior/subtle-knife.md).
Its typed Rust implementation is delivered in `0.2.120`; runtime and
presentation parity remain intentionally unclaimed.

The third selected case is Trigun alternate reload; its source and callback
decomposition are recorded in
[`docs/legacy-behavior/trigun.md`](docs/legacy-behavior/trigun.md). Its typed
Rust implementation was delivered in `0.2.121`; controlled legacy runtime and
presentation parity remain intentionally unclaimed.

- an equipment set with reversible equip/unequip modifiers;
- Subtle Knife-style alternate action with health/status cost and visible-target
  iteration;
- Trigun-style alternate reload with destructive special action;
- Null Pointer-style on-hit behavior with target-dependent branching;
- a recharge/healing armor with periodic behavior.

#### Acceptance criteria

- [x] Each selected stress case has a legacy evidence note identifying scalar
  fields, callback behavior, ordering assumptions, and unresolved ambiguity.
- [x] Each behavior is represented using the typed model without adding a
  one-off global hook framework.
- [x] Deterministic scenario/tests exercise successful behavior and at least
  one rejected or edge path for each selected case.
- [x] Behavior-complete migration is distinguished from scalar-definition
  coverage in roadmap/status language; the selected three-case model is
  covered while broader vocabulary and parity remain open.

### 2.7a Current Medical Powerarmor delivery target

The bounded implementation target for this revision is the first stress case,
Medical Powerarmor. Its typed transition must:

- [x] keep an armor-owned repair timer in deterministic core state;
- [x] tick only while the armor is equipped and the owner is below the
  evidence-backed half-health threshold with durability above `20`;
- [x] increment the timer once per accepted player command, heal exactly one
  HP at timer `30`, set the timer to `20`, and spend one armor durability;
- [x] reset the timer to `0` at or above half health while preserving the
  source-nested durability guard behavior;
- [x] leave gameplay state unchanged when no behavior is selected or the
  durability guard blocks, and emit a typed
  `GameEvent::MedicalPowerarmorRepaired` only for an actual repair;
- [x] cover the pure transition, edge cases, accepted-turn integration, and
  deterministic replay/event ordering with focused tests.

### 2.7b Current Subtle Knife delivery target

The bounded implementation target for this revision is the second stress case,
Subtle Knife alternate invoke. Its typed transition must:

- [x] expose an explicit `Command::Invoke(ItemId)` path for the equipped knife;
- [x] apply a five-HP cost clamped to one, set the typed tired condition, and
  spend 1000 score count without consuming RNG;
- [x] select living non-player actors in the player's current field of view in
  deterministic EntityId order and apply 15 points of internal damage;
- [x] emit a typed invocation event plus ordered damage/death events while
  leaving player and hidden actors excluded;
- [x] reject invalid or tired invocations atomically without spending a turn;
- [x] cover pure transition, visibility boundaries, accepted integration,
  rejection rollback, replay determinism, and command/protocol persistence.

### 2.7c Previous Trigun delivery target

The bounded implementation target for this revision is the third stress case,
Trigun alternate reload. Its typed transition must:

- [x] expose an explicit `Command::AltReload { item_id, confirmed }` path for
  the equipped Trigun;
- [x] reject a missing/non-Trigun item, low maximum HP, and declined
  confirmation atomically without spending a turn, RNG, or mutating player
  resources;
- [x] on success, reduce maximum HP by five but clamp it to ten, reduce current
  HP by five but clamp it to one, subtract 1000 score count with explicit
  signed saturation, and preserve the equipped weapon;
- [x] schedule a one-tick nuke and resolve it at the accepted-turn boundary,
  emitting typed activation/level-nuked/damage/death events and applying 6000
  internal environment damage to the player;
- [x] cover the pure transition, confirmation and low-HP rejection paths,
  exact command rollback, nuke event ordering, terminal game-over behavior,
  replay determinism, and command/protocol persistence;
- [x] record that explosion geometry, animation timing, confirmation UI, and
  controlled legacy runtime parity remain `NOT_RUN` or out of scope.

### 2.7d Previous Grammaton delivery target

The bounded implementation target for this revision is the fourth stress case,
Grammaton Cleric Beretta fire-mode behavior. Its typed transition must:

- [x] expose `WeaponFireMode` and cycle the equipped weapon through single,
  burst, and full-auto profiles in the pinned order;
- [x] apply the evidence-backed `2d6`/`1d8`/`1d7` damage profiles, one/three/
  six-shot counts, and a bounded 200 score-count cost;
- [x] preflight the selected shot count before clip or RNG mutation, emit
  ordered mode/shot/damage events, and stop at the first actual lethal result;
- [x] cover partial-clip rollback, pure transition behavior, replay
  determinism, and MCP legal-action/event projection;
- [x] record that the legacy accuracy equation and controlled runtime or
  presentation parity remain `NOT_RUN`.

### 2.7e Previous Jackhammer delivery target

The bounded implementation target for this revision is the fifth stress case,
Jackhammer alternate fire-mode behavior. Its typed transition must:

- [x] expose the stable `WeaponFireMode` contract while keeping Jackhammer's
  default mode as three-shot burst and toggling `Burst <-> Single`;
- [x] preserve the pinned `8d3` shotgun profile and range while resolving one
  or three shells according to the selected mode;
- [x] subtract one score count with saturating core policy and emit an ordered
  `JackhammerFireModeChanged` event;
- [x] preflight the selected shot count before clip or RNG mutation, stop after
  the first actual lethal result, and preserve one death drop at most;
- [x] cover pure transition behavior, partial-clip rollback, replay
  determinism, and MCP legal-action/event projection;
- [x] record that spread/falloff, exact timing, UI text, and controlled runtime
  or presentation parity remain `NOT_RUN`.

### 2.7f Previous Lava Armor delivery target

The bounded implementation target for this revision is the sixth stress case,
Lava Armor periodic recharge. Its typed transition must:

- [x] expose a walkable `Tile::Lava`/`TileKind::Lava` terrain contract without
  claiming hazard damage parity;
- [x] keep an armor-owned recharge timer and tick it once per accepted player
  command only while durability is below maximum;
- [x] restore up to three durability points on the fifth tick while standing
  on Lava, clamp to maximum, and reset the timer;
- [x] reset the timer on a non-Lava fifth tick while preserving durability, and
  preserve the timer when armor is already full;
- [x] emit `GameEvent::LavaArmorRecharged` only for an actual durability
  increase, with deterministic event ordering and no RNG consumption;
- [x] cover pure transition, accepted-turn integration, non-Lava/full/clamp
  edges, custom-tile replay determinism, and MCP event projection;
- [x] record that fire/acid hazard damage, resistance equations, controlled
  runtime comparison, and exact presentation parity remain `NOT_RUN`.

### 2.7g Previous Null Pointer delivery target

The bounded implementation target for this revision is the seventh stress case,
Charch's Null Pointer on-hit behavior. Its typed transition must:

- [x] add the catalog-backed zero-direct-damage plasma weapon with the pinned
  60-cell capacity and accuracy scalars; ordinary Rust action-cost policy
  remains explicit rather than claiming legacy shot-cost parity;
- [x] apply a pure boss/non-boss target score-count branch with 1000/2000 costs
  and a 1000 floor, preserving the explicit actor boss property;
- [x] emit ordered `NullPointerHit` and
  `NullPointerExplosionScheduled` events after a successful ranged hit;
- [x] cover pure branch behavior, accepted ranged integration, replay
  determinism, and MCP JSON/event projection;
- [x] record that delayed area damage, exact explosion geometry/order, runtime,
  and audiovisual parity remain `NOT_RUN`.

### 2.7h Previous Acid Spitter delivery target

The bounded implementation target for this revision is the eighth stress case,
Acid Spitter terrain-fed reload. Its typed transition must:

- [x] expose walkable `Tile::Acid`/`TileKind::Acid` and `Tile::Water`/
  `TileKind::Water` terrain contracts without claiming hazard parity;
- [x] reject a full clip or a non-Acid player tile atomically without changing
  ammunition, score, terrain, turn, or RNG;
- [x] on Acid, load one rocket up to the ten-round clip cap, convert the tile to
  Water, spend 1000 score count with saturating core policy, and emit an ordered
  `AcidSpitterReloaded` event;
- [x] cover pure transition behavior, accepted reload integration, replay and
  scenario tile persistence, deterministic no-RNG behavior, and MCP event
  projection;
- [x] record that Acid hazard damage/resistance, fluid movement cost, runtime
  comparison, and exact presentation parity remain `NOT_RUN`.

### 2.7i Previous terrain hazard delivery target

The bounded implementation target for this revision is the ninth stress case,
baseline Lava and Acid entered-cell contact damage. Its typed transition must:

- [x] expose pure Acid `6` and Lava `12` baseline damage outcomes while leaving
  non-hazard tiles unchanged;
- [x] apply damage only after an accepted player move onto the hazard, reusing
  environment `DamageApplied` and `ActorDied` contracts without consuming RNG;
- [x] preserve exact game/replay state on rejected movement and end the game
  deterministically on lethal contact;
- [x] cover pure transition behavior, Acid/Lava movement integration, lethal and
  non-contact edges, custom-tile replay/scenario determinism, and existing
  render/audio/MCP projections;
- [x] record that difficulty/running modifiers, resistance/avoidance, fluid
  movement cost, monster contact, runtime comparison, and exact presentation
  parity remain `NOT_RUN`.

### 2.7j Previous Acid/Lava fluid movement-cost delivery target

The bounded implementation target for the previous revision was the tenth
stress case, Acid and Lava fluid movement cost. Its typed transition must:

- [x] expose pure `ActionCost::new(1250)` outcomes for Acid/Lava and retain
  `ActionCost::MOVE` for ordinary walkable tiles;
- [x] apply the cost only after an accepted direct player move, preserving the
  previously delivered hazard events and consuming no RNG;
- [x] preserve exact game/replay state on rejected movement and keep ordinary
  floor movement at 1000 units;
- [x] cover pure transition behavior, accepted Acid/Lava/floor scheduling,
  rejected-state rollback, custom-tile replay/scenario determinism, and retain
  the existing render/audio/MCP projection contracts;
- [x] record that running/NORUN restrictions, fractional scheduler details,
  Mud movement-cost parity, fluid flow, resistance/avoidance, monster
  movement, runtime comparison, and exact presentation parity remain `NOT_RUN`.

### 2.7k Previous Water fluid movement-cost delivery target

The bounded implementation target for the previous revision was the eleventh
stress case, Water fluid movement cost. Its typed transition must:

- [x] expose pure `ActionCost::new(1250)` outcomes for Acid/Lava/Water and
  retain `ActionCost::MOVE` for ordinary walkable tiles;
- [x] apply the cost only after an accepted direct player move, preserving the
  previously delivered hazard/no-damage events and consuming no RNG;
- [x] preserve exact game/replay state on rejected movement and keep Mud
  movement-cost parity explicitly deferred until a Rust Mud tile exists;
- [x] cover pure transition behavior, accepted Water/no-damage scheduling,
  rejected-state rollback, custom-tile replay/scenario determinism, and retain
  the existing render/audio/MCP projection contracts;
- [x] record that running/NORUN restrictions, fractional scheduler details,
  Mud movement cost, fluid flow, resistance/avoidance, monster movement,
  runtime comparison, and exact presentation parity remain `NOT_RUN`.

### 2.7l Previous damage-type projection delivery target

The bounded implementation target for the previous revision was the twelfth
stress case, typed Acid/Fire damage classification. Its typed transition must:

- [x] carry optional `DamageType` on `DamageApplied` without changing amount,
  source, HP, or event ordering contracts;
- [x] emit `Some(Acid)` for Acid contact and `Some(Fire)` for Lava contact,
  while actor and unclassified environment damage remain `None`;
- [x] project the optional type to MCP JSON and preserve replay/scenario
  determinism, rollback identity, and existing render/audio behavior;
- [x] record that resistance/avoidance, running/difficulty modifiers, Mud
  movement, runtime comparison, and exact audiovisual parity remain `NOT_RUN`.

### 2.7m Previous Mud terrain delivery target

The bounded implementation target for the previous revision was the thirteenth
stress case, typed Mud terrain and movement cost. Its typed transition did:

- [x] expose walkable `Tile::Mud`/`TileKind::Mud` through core, replay,
  scenario, MCP, web, render, and asset projections;
- [x] return `ActionCost::new(1650)` for direct player movement onto Mud while
  leaving contact damage and ordinary movement policy unchanged;
- [x] preserve exact rejected-command state/RNG identity and deterministic
  custom-tile replay/scenario behavior;
- [x] record that flow, fractional scheduler details, modifiers, runtime
  comparison, and exact audiovisual parity remain `NOT_RUN`.

### 2.7n Previous Gate A rejection matrix delivery target

The bounded implementation target for the previous revision was explicit
rejection coverage across the current command surface. Its invariant did:

- [x] exercise representative invalid Move, melee/ranged combat, pickup/drop,
  equip/unequip/use/invoke, alternate reload, reload, and descend commands;
- [x] assert `Game::step` errors preserve exact cloned `Game` state and RNG for
  every matrix entry;
- [x] keep gameplay semantics at `16` and introduce no accepted transition or
  protocol/schema change;
- [x] record runtime, browser, audio/visual, and external capture comparisons as
  `NOT_RUN`.

### 2.7o Previous Gate B replay compatibility delivery target

The bounded implementation target for the previous revision was replay
metadata compatibility coverage. Its matrix did:

- [x] accept current gameplay semantics and ruleset metadata;
- [x] reject stale gameplay semantics and ruleset values before execution;
- [x] reject stale generator semantics for procedural replays while allowing
  fixed-map replays to ignore that unused identity;
- [x] preserve gameplay semantics `16` and record migration/runtime/browser/
  audiovisual comparisons as `NOT_RUN`.

### 2.7p Previous Gate B RNG sampling semantics delivery target

The bounded implementation target for the previous revision was explicit RNG
sampling identity in replay metadata. Its transition did:

- [x] declare the current RNG sampling semantics version in protocol metadata
  and the canonical MCP V2 envelope;
- [x] reject stale RNG sampling versions before map construction or command
  execution in core and MCP replay validation;
- [x] retain raw/bounded/probability golden vectors and keep gameplay semantics
  at `16` without changing accepted command behavior;
- [x] record migration/runtime/browser/audiovisual comparisons as `NOT_RUN`.

### 2.7q Previous Gate C item identity catalog delivery target

The bounded implementation target for this revision is one compile-time source
for routine stable item identity projections. Its transition must:

- [x] generate `ItemArchetype` and `ItemSpawnKind`, ordered `ALL` views, stable
  wire names, normalized spawn values, and archetype mapping from one protocol
  declaration;
- [x] preserve stable-name uniqueness, round-trip parsing, spawn-family
  round-trips, and catalog order;
- [x] keep count-sensitive spawn reconstruction, gameplay definitions, and
  presentation mappings explicit;
- [x] record legacy runtime, behavior, browser, and audiovisual comparisons as
  `NOT_RUN`.

### 2.7r Previous vertical Subtle Knife encounter delivery target

The bounded implementation target for this revision is evidence that the
delivered Subtle Knife behavior remains coherent across the public Rust
boundaries. Its vertical slice must:

- [x] construct a deterministic declarative encounter with a configured knife,
  a visible target, and an occluded target;
- [x] run the same `Command::Invoke` through `ScenarioRunner` and preserve
  target ordering, hidden-target exclusion, player cost, and event ordering;
- [x] verify the resulting replay remains deterministic and retains the stable
  item identity/command payload;
- [x] compare `BrowserSession::submit` with direct `Game::step` for events,
  player observations, pure effect timelines, and scene derivation;
- [x] keep accepted gameplay semantics unchanged while recording controlled
  legacy runtime, browser capture, audio, WebGPU, armor/resistance, and broad
  monster/AI parity as `NOT_RUN`.

### 2.7s Previous vertical Trigun encounter delivery target

The bounded implementation target for this revision is a canonical Trigun
alternate-reload encounter across the declarative scenario, replay, and
browser presentation boundaries. Its vertical slice must:

- [x] construct a deterministic ASCII encounter with a confirmed Trigun,
  visible and occluded actors, and explicit player configuration;
- [x] run the same `Command::AltReload` through `ScenarioRunner`, preserving
  confirmation, HP/max-HP/score costs, one-tick nuke resolution, terminal
  ordering, and stable item identity;
- [x] verify replay determinism and retain the typed alternate-reload command
  payload without introducing a browser-specific replay format;
- [x] compare `BrowserSession::submit` with direct `Game::step` for events,
  player observations, pure effect timelines, terminal state, and scene
  derivation;
- [x] keep gameplay semantics unchanged while recording explosion geometry,
  animation timing, confirmation UI, controlled legacy runtime, browser
  capture, audio, WebGPU, armor/resistance, and broader monster/AI parity as
  `NOT_RUN`.

### 2.7t Current vertical Acid Spitter encounter delivery target

The bounded implementation target for this revision is a canonical Acid
Spitter reload encounter across the declarative scenario, replay, and browser
presentation boundaries. Its vertical slice must:

- [x] construct a deterministic ASCII encounter with a configured Acid Spitter
  on an Acid tile and an adjacent Water destination;
- [x] run the same `Command::Reload` through `ScenarioRunner`, preserving
  one-round clip loading, Acid-to-Water conversion, score-count cost, action
  ordering, and stable item identity;
- [x] verify replay determinism and retain the typed reload command payload
  without introducing a browser-specific replay format;
- [x] compare `BrowserSession::submit` with direct `Game::step` for events,
  player observations, pure effect timelines, terrain projection, and scene
  derivation;
- [x] keep gameplay semantics unchanged while recording hazard resistance/flow,
  explosion geometry, animation timing, controlled legacy runtime, browser
  capture, audio, WebGPU, and broader monster/AI parity as `NOT_RUN`.

### 2.7u Current vertical Null Pointer encounter delivery target

The bounded implementation target for this revision is a canonical Null Pointer
ranged-hit encounter across the declarative scenario, replay, and browser
presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic ASCII encounter with a configured Null
  Pointer and an explicit boss target;
- [x] run the same `Command::AttackRanged` through `ScenarioRunner`, preserving
  the target-dependent score floor, stable item/target identity, deferred
  explosion payload, and action ordering;
- [x] verify replay determinism and compare replayed events and final game state
  with the direct command result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for events,
  player observations, concrete ranged-attack effects, and scene derivation;
- [x] keep gameplay semantics unchanged while recording delayed blast geometry,
  legacy runtime, browser capture, audio, WebGPU, and broader monster/AI parity
  as `NOT_RUN`.

### 2.7v Current vertical Grammaton encounter delivery target

The bounded implementation target for this revision is a canonical Grammaton
Cleric Beretta fire-mode encounter across the declarative scenario, replay, and
browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic ASCII encounter with a configured
  Grammaton and a visible target;
- [x] run the same `Command::AltReload` and `Command::AttackRanged` sequence
  through `ScenarioRunner`, preserving Burst selection, score cost, three-shot
  clip consumption, stable identities, and action ordering;
- [x] verify replay determinism and compare replayed events and final game state
  with the direct command result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for events,
  player observations, concrete mode/attack effects, and scene derivation;
- [x] keep gameplay semantics unchanged while recording exact accuracy
  equations, legacy runtime, browser capture, audio, WebGPU, and broader
  monster/AI parity as `NOT_RUN`.

### 2.7w Current vertical Jackhammer encounter delivery target

The bounded implementation target for this revision is a canonical Jackhammer
fire-mode encounter across the declarative scenario, replay, and browser
presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic ASCII encounter with a configured
  Jackhammer and a visible target;
- [x] run the same `Command::AltReload` and `Command::AttackRanged` sequence
  through `ScenarioRunner`, preserving Single selection, score cost,
  one-shell clip consumption, knockback, stable identities, and action order;
- [x] verify replay determinism and compare replayed events and final game state
  with the direct command result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for events,
  player observations, concrete mode/attack/hit/knockback effects, and scene
  derivation;
- [x] keep gameplay semantics unchanged while recording exact spread/falloff,
  legacy runtime, browser capture, audio, WebGPU, and broader monster/AI parity
  as `NOT_RUN`.

### 2.7x Current vertical Lava Armor encounter delivery target

The bounded implementation target for this revision is a canonical Lava Armor
periodic-recharge encounter across the declarative scenario, replay, and
browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic ASCII encounter with explicit Lava under
  the player and beside the spawn, plus configured Lava Armor at `97/100`
  durability;
- [x] run the same five `Command::Wait` commands through `ScenarioRunner`,
  preserving timer progression, one three-point recharge clamp, stable item
  identity, and action order;
- [x] verify replay determinism and compare replayed events and final game state
  with the direct command result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for each wait's
  events, player observations, empty pure effect timelines, and scene derivation;
- [x] keep gameplay semantics unchanged while recording Lava hazard/resistance,
  legacy runtime, browser capture, audio, WebGPU, audiovisual, and broader
  armor/content parity as `NOT_RUN`.

### 2.7y Previous vertical Medical Powerarmor encounter delivery target

The bounded implementation target for this revision is a canonical Medical
Powerarmor periodic-repair encounter across the declarative scenario, replay,
and browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 8x4 ASCII encounter with a low-health
  player and configured Medical Powerarmor at `100/100` durability;
- [x] run the same thirty `Command::Wait` commands through `ScenarioRunner`,
  preserving timer progression, one-point healing, one durability spend, the
  post-repair timer value, stable item identity, and action order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every wait's
  events, player observations, empty pure effect timelines, and scene derivation;
- [x] keep gameplay semantics unchanged while recording repair-threshold
  variants, controlled legacy runtime, browser capture, audio, WebGPU,
  audiovisual, and broader armor/content parity as `NOT_RUN`.

### 2.7z Previous vertical Former Human-profile progression delivery target

The bounded implementation target for this revision is a canonical Pistol
progression through a Former Human-profile encounter, dropped ammunition
pickup, and stairs descent across the declarative scenario, replay, and browser
presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 8x4 ASCII arena with a configured Pistol,
  one Former Human-profile target (using an explicit non-catalog identity to
  keep scenario and replay metadata equal), and a down-stairs exit;
- [x] run the same movement, ranged-attack, pickup, and `Descend` commands
  through `ScenarioRunner`, preserving scheduled Former Human-profile
  responses, damage and
  death events, dropped-ammunition identity, pickup state, action order, and
  level transition;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  command's events, player observations, pure effect timeline, and scene
  derivation;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  monster/weapon parity as `NOT_RUN`.

### 2.7aa Previous vertical Phase Device escape delivery target

The bounded implementation target for this revision is a Phase Device
pickup-and-teleport encounter across the declarative scenario, replay, and
browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 8x4 ASCII arena with a ground Phase
  Device and the default player loadout;
- [x] run the same `Move`, `Pickup`, and `Use` commands through
  `ScenarioRunner`, preserving item identity, deterministic unoccupied-cell
  selection, item consumption, exploration, and event order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  command's events, player observations, literal teleport/use effect spans,
  and scene derivation;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  item/teleport parity as `NOT_RUN`.

### 2.7ab Previous vertical Shotgun knockback delivery target

The bounded implementation target for this revision is a standard Shotgun
knockback encounter against a Former Sergeant profile across the declarative
scenario, replay, and browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 8x4 ASCII arena with a configured
  Shotgun, a Former Sergeant-profile target, and one open knockback destination;
- [x] run the same ranged-attack command through `ScenarioRunner`, preserving
  seeded hit/damage, one-tile target displacement, scheduled target response,
  stable identities, and event order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for the attack's
  events, player observations, literal ranged/hit/knockback effects, and scene
  derivation;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/monster knockback parity as `NOT_RUN`.

### 2.7ac Previous vertical Green Armor protection delivery target

The bounded implementation target for this revision is a Green Armor
mitigation encounter against a Former Sergeant profile across the declarative
scenario, replay, and browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 8x4 ASCII arena with configured Green
  Armor, a Former Sergeant-profile target, and a visible ranged line;
- [x] run the same wait command through `ScenarioRunner`, preserving the seeded
  target hit, raw-versus-mitigated damage, player HP, stable identities, and
  event order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for the response
  events, player observations (including armor protection), literal ranged/hit
  effects, and scene derivation;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, durability/resistance,
  and broader armor/monster parity as `NOT_RUN`.

### 2.7ad Previous vertical Small MedPack recovery delivery target

The bounded implementation target for this revision is a Small MedPack
recovery encounter across the declarative scenario, replay, and browser
presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 8x4 ASCII arena with HP `45/50` and a
  configured Small MedPack;
- [x] run `Use(ItemId(4))` through `ScenarioRunner`, preserving capped health
  recovery to `50`, item consumption, stable identity, and event order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for the use
  event, player observations, literal `Use` effect, scene derivation, and
  retained replay command;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  consumable parity as `NOT_RUN`.

### 2.7ae Previous vertical Demon MedPack recovery delivery target

The bounded implementation target for this revision is a Demon melee-pressure
and Small MedPack recovery encounter across the declarative scenario, replay,
and browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 8x4 ASCII arena with a configured
  `Rush Demon`, player HP `46/50`, and a Small MedPack;
- [x] run `Wait` then `Use(ItemId(4))` through `ScenarioRunner`, preserving the
  two seeded Demon melee responses, capped healing/consumption, stable
  identities, and event order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for melee,
  damage, use, and action-cost events, player observations, literal effect
  timing, scene derivation, and retained replay commands;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  monster/consumable parity as `NOT_RUN`.

### 2.7af Previous vertical Pistol reload delivery target

The bounded implementation target for this revision is a Pistol clip depletion
and reload encounter across the declarative scenario, replay, and browser
presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 8x4 ASCII arena with a configured
  Pistol, twenty reserve 9mm rounds, and a static Former Human-profile target;
- [x] run ten seeded ranged attacks then `Reload` through `ScenarioRunner`,
  preserving shot/hit totals, ammunition consumption, reload state, stable
  identities, and event order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  command's events, observations, literal effects, scene derivation, and
  retained replay commands;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/ammunition parity as `NOT_RUN`.

### 2.7ag Previous vertical Plasma Rifle cell-reload delivery target

The bounded implementation target for this revision is a Plasma Rifle
six-cell clip depletion and reload encounter across the declarative scenario,
replay, and browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 8x4 ASCII arena with a configured
  Plasma Rifle, twelve reserve cells, and a static Former Human-profile
  target;
- [x] run six seeded ranged attacks then `Reload` through `ScenarioRunner`,
  preserving shot/hit totals, ammunition consumption, reload state, stable
  identities, and event order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  command's events, observations, literal effects, scene derivation, and
  retained replay commands;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/ammunition parity as `NOT_RUN`.

### 2.7ah Previous vertical Rocket Launcher one-shot reload delivery target

The bounded implementation target for this revision is a Rocket Launcher
one-shot clip depletion and reload encounter across the declarative scenario,
replay, and browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 8x4 ASCII arena with a configured
  Rocket Launcher, two reserve rockets, and a static Former Human-profile
  target;
- [x] run one seeded ranged attack then `Reload` through `ScenarioRunner`,
  preserving the hit/damage total, rocket consumption, reload state, stable
  identities, and event order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  command's events, observations, literal effects, scene derivation, and
  retained replay commands;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/ammunition parity as `NOT_RUN`.

### 2.7ai Previous vertical Chainsaw melee delivery target

The bounded implementation target for this revision is a Chainsaw close-range
melee encounter across the declarative scenario, replay, and browser
presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 8x4 ASCII arena with a configured
  Chainsaw and a static Demon-profile target;
- [x] run one seeded melee attack through `ScenarioRunner`, preserving the
  hit/damage total, stable identities, and event order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for the
  command's events, observations, literal effects, scene derivation, and
  retained replay command;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/melee parity as `NOT_RUN`.

### 2.7aj Previous vertical Shotgun shell-reload delivery target

The bounded implementation target for this revision is a standard Shotgun
shell-clip depletion and reload encounter across the declarative scenario,
replay, and browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 9x4 ASCII arena with a configured
  Shotgun, ten reserve shells, and a static Former Human-profile target whose
  east knockback destination is blocked;
- [x] run eight seeded ranged attacks then `Reload` through `ScenarioRunner`,
  preserving shot/hit totals, shell consumption, reload state, stable
  identities, and the distinct `1200` action cost/order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  command's events, observations, literal effects, scene derivation, and
  retained replay commands;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/spread and alternate-reload parity as `NOT_RUN`.

### 2.7ak Previous vertical Assault Shotgun shell-reload delivery target

The bounded implementation target for this revision is an Assault Shotgun
shell-clip depletion and reload encounter across the declarative scenario,
replay, and browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 9x4 ASCII arena with a configured
  Assault Shotgun, eight reserve shells, and a static Former Human-profile
  target whose east knockback destination is blocked;
- [x] run six seeded ranged attacks then `Reload` through `ScenarioRunner`,
  preserving shot/hit totals, shell consumption, reload state, stable
  identities, and the standard `1000` action cost/order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  command's events, observations, literal effects, scene derivation, and
  retained replay commands;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/spread and alternate-reload parity as `NOT_RUN`.

### 2.7al Previous vertical Double Shotgun clip-reload delivery target

The bounded implementation target for this revision is a Double Shotgun
two-shell clip depletion and reload encounter across the declarative scenario,
replay, and browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 9x4 ASCII arena with a configured
  Double Shotgun, four reserve shells, and a static Former Human-profile target
  whose east knockback destination is blocked;
- [x] run two seeded ranged attacks then `Reload` through `ScenarioRunner`,
  preserving shot/hit totals, shell consumption, reload state, stable
  identities, and action-cost order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  command's events, observations, literal effects, scene derivation, and
  retained replay commands;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/spread parity as `NOT_RUN`.

### 2.7am Previous vertical Assault Shotgun single-shell reload delivery target

The bounded implementation target for this revision is a callback-derived
Assault Shotgun single-shell reload correction across the declarative scenario,
replay, and browser presentation boundaries. Its vertical slice must:

- [x] use the exact deterministic 9x4 ASCII arena with a configured Assault
  Shotgun, eight reserve shells, and a static Former Human-profile target whose
  east knockback destination is blocked;
- [x] run six seeded ranged attacks then `Reload` through `ScenarioRunner`,
  preserving the six-shot totals, shell consumption, one-shell reload state,
  stable identities, and the standard `1000` action cost/order;
- [x] reject full-clip and no-reserve reload attempts atomically for the typed
  Assault Shotgun transition;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  command's events, observations, literal effects, scene derivation, and
  retained replay commands;
- [x] advance gameplay semantics from `16` to `17` while preserving replay
  wire schema, RNG sampling, generator, and ruleset identities; controlled
  legacy runtime, browser capture, audio, WebGPU, audiovisual, alternate
  reload, and broader weapon/spread parity remain `NOT_RUN`.

### 2.7an Previous vertical Combat Shotgun clip-reload delivery target

The bounded implementation target for this revision is a Combat Shotgun
five-shell clip depletion and reload encounter across the declarative scenario,
replay, and browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic 9x4 ASCII arena with a configured
  Combat Shotgun, ten reserve shells, and a static Former Human-profile target
  whose east knockback destination is blocked;
- [x] run five seeded ranged attacks then `Reload` through `ScenarioRunner`,
  preserving three hits for `46` damage, shell consumption, five-shell reload
  state, stable identities, and the standard `1000` action cost/order;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  command's events, observations, literal effects, scene derivation, and
  retained replay commands;
- [x] keep gameplay semantics unchanged while recording controlled legacy
  runtime, browser capture, audio, WebGPU, audiovisual, and broader
  weapon/spread parity as `NOT_RUN`.

### 2.7ao Previous vertical Combat Shotgun single-shell reload delivery target

The bounded implementation target for this revision is the callback-derived
Combat Shotgun single-shell reload correction across the declarative scenario,
replay, and browser presentation boundaries. Its vertical slice must:

- [x] use the exact deterministic `CombatPumpVertical` 9x4 ASCII arena with a
  configured Combat Shotgun, five depleted rounds, and ten reserve shells;
- [x] apply the final `Command::Reload` after the explicit pump-action sequence
  so it loads exactly one shell, consumes one reserve shell, and preserves the
  standard 1,000-unit action cost;
- [x] reject full-clip and missing-ammo reload attempts atomically with exact
  `Game` equality;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct scenario result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  command's events, observations, literal effects, scene derivation, and
  retained replay commands;
- [x] advance gameplay semantics from `17` to `18` while preserving replay
  wire schema, RNG sampling, generator, and ruleset identities; controlled
  legacy runtime, browser capture, audio, WebGPU, audiovisual, pump-action,
  alternate-reload, and broader weapon/spread comparisons remain `NOT_RUN`.

### 2.7ap Previous direct-core replay custom-tile bounds delivery target

The bounded implementation target for this revision is structural replay
validation parity for explicit custom tile overrides. Its slice must:

- [x] reject every custom tile position outside replay dimensions with a
  deterministic validation error before map construction or mutation;
- [x] accept an in-bounds custom tile and preserve existing replay execution;
- [x] cover both rejection and acceptance in the core replay-versioning tests,
  retaining the MCP decoder's matching bounds contract;
- [x] keep gameplay semantics, replay wire schema, RNG sampling, generator,
  and ruleset identities unchanged while advancing the patch version from
  `0.2.167` to `0.2.168`.

### 2.7aq Previous direct-core replay header consistency delivery target

The bounded implementation target for this revision is structural replay
schema-header validation. Its slice must:

- [x] reject a top-level replay schema other than V2 before map construction;
- [x] reject a metadata schema version that differs from the top-level replay
  version before execution;
- [x] retain acceptance, map application, and deterministic reproduction for a
  canonical V2 replay;
- [x] keep gameplay semantics, replay wire schema, RNG sampling, generator,
  and ruleset identities unchanged while advancing the patch version from
  `0.2.168` to `0.2.169`.

### 2.7ar Previous vertical Combat Shotgun pump-action delivery target

The bounded implementation target for this revision is the pinned Combat
Shotgun pump-action chamber transition across the declarative scenario, replay,
and browser presentation boundaries. Its vertical slice must:

- [x] construct the deterministic `CombatPumpVertical` 9x4 arena with a
  configured five-shell Combat Shotgun, ten reserve shells, and a static target;
- [x] mark the chamber empty after a successful shot and reject the next shot
  with typed `CommandError::ChamberEmpty` before clip/RNG/turn mutation;
- [x] chamber on an accepted walk only, while blocked movement and waiting leave
  the chamber empty;
- [x] make empty-chamber reload with clip ammo pump-only at cost `200`, without
  reserve consumption or `WeaponReloaded`; empty clip then uses the regular
  one-shell reload at cost `1000`;
- [x] verify replay determinism and compare generated replay events/final game
  state with the direct scenario result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  accepted command's events, observations, effects, scenes, and replay log;
- [x] advance gameplay semantics from `18` to `19` while preserving replay wire
  schema, RNG sampling, generator, and ruleset identities; chamber UI/audio,
  alternate reload, controlled legacy runtime, and audiovisual parity remain
  `NOT_RUN`.

### 2.7as Previous vertical Assault Shotgun alternate-reload delivery target

The bounded implementation target for this revision is the pinned Assault
Shotgun alternate/full-reload callback across the declarative scenario, replay,
and browser presentation boundaries. Its vertical slice must:

- [x] construct the deterministic `AssaultShotgunAltReloadVertical` 9x4 arena
  with a configured six-shell Assault Shotgun, eight reserve shells, and a
  static target;
- [x] run six seeded ranged attacks followed by
  `Command::AltReload { confirmed: false }`, preserving the six-shot totals,
  shell consumption, full six-shell clip, and stable identities;
- [x] preflight the complete clip deficit so full clips and insufficient/no
  reserve reject atomically, while a successful transition emits one existing
  `WeaponReloaded` event and consumes exactly the deficit;
- [x] pay `min(deficit * reload_cost, 2500)` for accepted alternate reloads;
  ordinary `Command::Reload` remains the one-shell `IF_SINGLERELOAD` path;
- [x] verify replay determinism and compare generated replay/final game state
  with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  command's events, observations, effects, scene, and retained replay log;
- [x] advance gameplay semantics from `19` to `20` while preserving replay
  wire schema, RNG sampling, generator, and ruleset identities; Combat Shotgun
  full reload, partial-reserve policy, controlled legacy runtime, audio,
  WebGPU, and audiovisual parity remain `NOT_RUN`.

### 2.7at Previous vertical Combat Shotgun alternate-reload delivery target

The bounded implementation target for this revision is the pinned Combat
Shotgun alternate/full-reload callback across the declarative scenario, replay,
and browser presentation boundaries. Its vertical slice must:

- [x] construct the deterministic `CombatShotgunAltReloadVertical` 9x4 arena
  with a configured five-shell Combat Shotgun, ten reserve shells, and a static
  target;
- [x] deplete the five-shell clip through explicit pump cycles, then run
  `Command::AltReload { confirmed: false }` with a complete five-shell deficit
  and enough loose reserve;
- [x] preflight the complete clip deficit so full clips and insufficient/no
  reserve reject atomically, while a successful transition emits one existing
  `WeaponReloaded` event and consumes exactly the deficit;
- [x] reset an empty pump chamber as part of successful alternate reload, so an
  immediate ranged attack succeeds without an additional 200-unit pump cost;
- [x] pay `min(deficit * reload_cost, 2500)` for accepted alternate reloads;
  ordinary `Command::Reload` remains the one-shell/pump-only path;
- [x] verify replay determinism and compare generated replay/final game state
  with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for every
  command's events, observations, effects, scene, and retained replay log;
- [x] advance gameplay semantics from `20` to `21` while preserving replay
  wire schema, RNG sampling, generator, and ruleset identities; partial-reserve
  policy, ammo packs, controlled legacy runtime, audio, WebGPU, and audiovisual
  parity remain `NOT_RUN`.

### 2.7au Previous direct replay dimension-bound delivery target

The bounded implementation target for this revision is direct-core replay
dimension validation parity with the existing MCP decoder. Its acceptance
criteria are:

- [x] reject zero, one, and two-cell widths or heights before map construction;
- [x] reject dimensions above the bounded maximum of `512` before map
  construction;
- [x] preserve acceptance of valid `3..=512` dimensions and all existing replay
  headers, initial-state containers, commands, and deterministic execution;
- [x] verify the lower and upper bound rejections in the replay-versioning test
  suite without mutating a game or consuming RNG;
- [x] advance project version from `0.2.172` to `0.2.173` without changing
  gameplay, replay wire, RNG, generator, or ruleset semantics.

### 2.7av Previous direct replay structural-bound delivery target

The bounded implementation target for this revision is direct-core replay
structural validation parity with the existing MCP decoder. Its acceptance
criteria are:

- [x] reject more than `4,096` initial monsters or ground items before map
  construction;
- [x] reject more than `65,536` custom tile overrides or `100,000` commands
  before map construction;
- [x] reject more than `4,096` configured player initial items before map
  construction;
- [x] reject procedural configurations with more than `64` rooms, zero or
  inverted room-size bounds, room dimensions above `64`, or more than `64`
  monsters/items per room;
- [x] verify every structural rejection and a malformed replay diagnostic at
  command index `0`, proving no command execution or RNG consumption;
- [x] advance project version from `0.2.173` to `0.2.174` without changing
  gameplay, replay wire, RNG, generator, or ruleset semantics; replay-file IO,
  migrations, and external interchange remain open.

### 2.7aw Previous Blaster periodic-recharge delivery target

The bounded implementation target for this revision is the pinned Blaster
recharge callback across typed core behavior, scenario/replay determinism, and
the MCP/browser presentation boundaries. Its acceptance criteria are:

- [x] construct an equipped Blaster with its ten-cell clip and no reserve-ammo
  dependency;
- [x] tick the equipped weapon once after each accepted player command, restore
  one cell at timer `40`, then restore one cell every `10` ticks while below
  capacity, clamping at ten cells;
- [x] leave a full clip's timer unchanged and reset the timer on a successful
  ranged fire; rejected commands restore the timer and clip through the
  existing transaction guard;
- [x] emit one typed `WeaponRecharged` event per restored cell with entity,
  item, restored-amount, current/max-clip, and retained-timer fields; metrics
  remain unchanged and render/audio remain presentation-neutral;
- [x] verify pure state-machine boundaries, a deterministic scenario/replay,
  MCP event JSON, and BrowserSession parity with direct `Game::step`;
- [x] advance gameplay semantics from `21` to `22` and project version from
  `0.2.174` to `0.2.175` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; manual-reload denial, other rechargeable weapons,
  partial-reserve behavior, controlled legacy runtime, and audiovisual parity
  remain open.

### 2.7ax Previous `IF_NORELOAD` manual-reload delivery target

The bounded implementation target for this revision is the pinned
`IF_NORELOAD` denial at the ordinary `Reload` command boundary. Its acceptance
criteria are:

- [x] deny manual reload for exactly Blaster, Nuclear Plasma Rifle, and Nuclear
  BFG 9000 with typed `CannotReload(item_id)` before pump, clip, reserve,
  recharge-timer, turn, or RNG mutation;
- [x] preserve the ordinary reload path for all other ranged weapons and keep
  `IF_NOUNLOAD` behavior, alternate reload, and automatic recharge separate;
- [x] verify exact core game equality for each rejected family, including a
  partially loaded clip and reserve ammunition;
- [x] verify replay diagnostics at command index `0`, valid replay determinism,
  MCP-session non-mutation/error behavior, and BrowserSession parity;
- [x] advance gameplay semantics from `22` to `23` and project version from
  `0.2.175` to `0.2.176` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; runtime, audio, and other family behavior remain open.

### 2.7ay Previous Nuclear Plasma periodic-recharge delivery target

The bounded implementation target for this revision is the pinned Nuclear
Plasma Rifle recharge callback across typed core behavior, scenario/replay
determinism, and BrowserSession parity. Its acceptance criteria are:

- [x] construct an equipped Nuclear Plasma Rifle with its 24-cell clip and a
  typed recharge policy of delay `40`, cadence `2`, and amount `1`;
- [x] tick the equipped weapon once after each accepted player command, restore
  one cell on tick `42`, then restore one cell every 2 ticks while below
  capacity, clamping at 24 cells;
- [x] leave a full clip's timer unchanged and reset the timer on a successful
  ranged fire; rejected commands restore timer and clip through the existing
  transaction guard, with no reserve-ammo mutation;
- [x] emit the existing `WeaponRecharged` event with the resulting clip and
  retained timer, and verify pure policy boundaries, ScenarioRunner/replay
  determinism, and BrowserSession/direct-core parity;
- [x] advance gameplay semantics from `23` to `24` and project version from
  `0.2.176` to `0.2.177` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; alternate/nuke, chainfire, runtime, and audiovisual
  parity remain open.

### 2.7az Previous Nuclear BFG 9000 periodic-recharge delivery target

The bounded implementation target for this revision is the pinned Nuclear BFG
9000 recharge callback across typed core behavior, scenario/replay
determinism, and BrowserSession parity. Its acceptance criteria are:

- [x] construct an equipped Nuclear BFG 9000 with its 40-cell clip and a typed
  recharge policy of delay `0`, cadence `5`, and amount `1`;
- [x] tick the equipped weapon once after each accepted player command, restore
  one cell on tick `5`, then restore one cell every 5 ticks while below
  capacity, clamping at 40 cells;
- [x] leave a full clip's timer unchanged and reset the timer on a successful
  ranged fire; rejected commands restore timer and clip through the existing
  transaction guard, with no reserve-ammo mutation;
- [x] emit the existing `WeaponRecharged` event with the resulting clip and
  retained timer, and verify pure policy boundaries, ScenarioRunner/replay
  determinism, and BrowserSession/direct-core parity;
- [x] advance gameplay semantics from `24` to `25` and project version from
  `0.2.177` to `0.2.178` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; alternate nuke, exact-hit/explosion, runtime, and
  audiovisual parity remain open.

### 2.7ba Previous Missile Launcher single-shell reload delivery target

The bounded implementation target for this revision is the pinned exotic
Missile Launcher's ordinary `IF_SINGLERELOAD` path across typed core behavior,
scenario/replay determinism, and BrowserSession parity. Its acceptance criteria
are:

- [x] construct an equipped Missile Launcher with a four-rocket clip and
  ordinary reload cost;
- [x] load exactly one loose rocket on each accepted `Reload` while the clip is
  below capacity, preserving the shared `WeaponReloaded` event and action-cost
  ordering;
- [x] reject full clips and empty reserve atomically, preserving clip,
  inventory, turn, and RNG state;
- [x] verify pure command atomicity, ScenarioRunner/replay determinism,
  existing MCP legal-action filtering, and BrowserSession/direct-core parity;
- [x] advance gameplay semantics from `25` to `26` and project version from
  `0.2.178` to `0.2.179` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; alternate/full reload, rocket-jump, explosion, runtime,
  and audiovisual parity remain open.

### 2.7bb Previous Missile Launcher alternate/full reload delivery target

The bounded implementation target for this revision is the pinned exotic
Missile Launcher's `perk_altreload_full` path across typed core behavior,
scenario/replay determinism, existing MCP legal-action filtering, and
BrowserSession parity. Its acceptance criteria are:

- [x] construct an equipped Missile Launcher with a four-rocket clip and
  partial deficit plus sufficient loose rockets;
- [x] load the complete deficit on one accepted `AltReload`, consume exactly
  that reserve, emit one `WeaponReloaded`, and pay
  `min(deficit * reload_cost, 2500)`;
- [x] reject full clips and insufficient reserve atomically, preserving clip,
  inventory, turn, and RNG state;
- [x] preserve ordinary one-shell `Reload`, verify ScenarioRunner/replay
  determinism and BrowserSession/direct-core parity, and confirm existing MCP
  legal-action filtering exposes the typed command;
- [x] advance gameplay semantics from `26` to `27` and project version from
  `0.2.179` to `0.2.180` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; rocket-jump, explosion, runtime, and audiovisual parity
  remain open.

### 2.7bc Previous Malek’s Armor periodic-recharge delivery target

The bounded implementation target for this revision is the pinned Malek’s
Armor recharge callback across typed core behavior, deterministic damage-reset
semantics, scenario/replay determinism, MCP event projection, and
BrowserSession parity. Its acceptance criteria are:

- [x] construct an equipped Malek’s Armor item with a typed policy of delay
  `50`, cadence `5`, amount `1`, and an explicit timer state;
- [x] tick the equipped armor once after each accepted player command, restore
  one durability at accepted tick `55`, then restore one durability every 5
  accepted ticks while below maximum, clamping at maximum;
- [x] leave full durability’s timer unchanged and reset the timer when the
  actor receives damage; rejected commands restore timer and durability
  through the existing transaction guard;
- [x] emit `MalekArmorRecharged` with the restored amount, remaining
  durability, and retained timer, and verify pure policy boundaries,
  ScenarioRunner/replay determinism, MCP JSON projection, and
  BrowserSession/direct-core parity;
- [x] advance gameplay semantics from `27` to `28` and project version from
  `0.2.180` to `0.2.181` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; general armor degradation/resistance, other recharge
  families, runtime, and audiovisual parity remain open.

### 2.7bd Previous Nuclear Plasma alternate-overload delivery target

The bounded implementation target for this revision is the pinned Nuclear
Plasma Rifle alternate-nuke callback across typed core behavior,
scenario/replay determinism, MCP legal-action and event projection, and
BrowserSession parity. Its acceptance criteria are:

- [x] preflight an equipped Nuclear Plasma Rifle with a full 24-cell clip,
  explicit confirmation, a non-stairs tile, and no pending nuke before any
  score, equipment, or nuke-state mutation;
- [x] remove the equipped weapon, spend 1,000 score count, emit one
  `NuclearWeaponOverloaded`, and arm countdown `1` on Acid/Lava or `100` on a
  safe floor using the existing `NukeState` transition;
- [x] reject unconfirmed, partial-clip, stairs, and pending-nuke commands
  atomically, preserving complete `Game` and RNG state;
- [x] verify pure planner boundaries, hazard and floor scenarios, typed nuke
  event ordering, replay determinism, MCP legal-action/JSON projections, and
  BrowserSession/direct-core parity;
- [x] advance gameplay semantics from `28` to `29` and project version from
  `0.2.181` to `0.2.182` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; Nuclear BFG, `NukeRun` map-wide effects, runtime, and
  audiovisual parity remain open.

### 2.7be Previous Nuclear BFG 9000 alternate-overload delivery target

The bounded implementation target for this revision is the pinned Nuclear BFG
9000 alternate-nuke callback across typed core behavior, scenario/replay
determinism, MCP legal-action and event projection, and BrowserSession parity.
Its acceptance criteria are:

- [x] preflight an equipped Nuclear BFG 9000 with a full 40-cell clip,
  explicit confirmation, a non-stairs tile, and no pending nuke before any
  score, equipment, or nuke-state mutation;
- [x] remove the equipped weapon, spend 1,000 score count, emit one
  `NuclearWeaponOverloaded`, and arm countdown `1` on Acid/Lava or `100` on a
  safe floor using the existing `NukeState` transition;
- [x] reject unconfirmed, partial-clip, stairs, and pending-nuke commands
  atomically, preserving complete `Game` and RNG state;
- [x] verify the shared pure planner boundaries, hazard and floor scenarios,
  typed nuke event ordering, replay determinism, MCP legal-action/JSON
  projections, and BrowserSession/direct-core parity for the BFG archetype;
- [x] advance gameplay semantics from `29` to `30` and project version from
  `0.2.182` to `0.2.183` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; Nuclear Plasma remains delivered, while `NukeRun`
  map-wide effects, runtime, and audiovisual parity remain open.

### 2.7bf Previous standard BFG 9000 exact-hit delivery target

The bounded implementation target for this revision is the pinned standard
BFG 9000 `IF_EXACTHIT` behavior across typed combat resolution,
scenario/replay determinism, MCP boundary behavior, and BrowserSession parity.
Its acceptance criteria are:

- [x] mark only the standard BFG 9000 weapon as exact-hit in the typed weapon
  policy, preserving ordinary accuracy behavior for every other weapon;
- [x] bypass only the to-hit RNG for a valid standard BFG 9000 shot while
  retaining line-of-sight, range, clip, action-cost, damage RNG, and existing
  attack/damage event contracts;
- [x] reject invalid target, blocked line-of-sight, out-of-range, and empty-clip
  commands atomically, preserving complete `Game` and RNG state;
- [x] verify pure resolver boundaries, deterministic standard-BFG scenarios,
  replay equality/determinism, MCP projections, and BrowserSession/direct-core
  parity;
- [x] advance gameplay semantics from `30` to `31` and project version from
  `0.2.183` to `0.2.184` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; other exact-hit families, projectile paths, explosions,
  runtime, and audiovisual parity remain open.

### 2.7bg Previous Nuclear BFG 9000 exact-hit delivery target

The bounded implementation target for this revision is the pinned Nuclear BFG
9000 `IF_EXACTHIT` behavior, reusing the delivered standard-BFG typed resolver
path across combat, scenario/replay determinism, MCP behavior, and
BrowserSession parity. Its acceptance criteria are:

- [x] mark only the Nuclear BFG 9000 weapon as exact-hit in addition to the
  delivered standard BFG policy, preserving ordinary accuracy behavior for
  every other weapon;
- [x] bypass only the to-hit RNG for a valid Nuclear BFG 9000 shot while
  retaining line-of-sight, range, clip, action-cost, damage RNG, and existing
  attack/damage event contracts;
- [x] reject invalid target, blocked line-of-sight, out-of-range, and empty-clip
  commands atomically, preserving complete `Game` and RNG state;
- [x] verify pure resolver boundaries, deterministic Nuclear-BFG scenarios,
  replay equality/determinism, MCP projections, and BrowserSession/direct-core
  parity;
- [x] advance gameplay semantics from `31` to `32` and project version from
  `0.2.184` to `0.2.185` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; BFG shot cost, projectile paths, explosions, NukeRun,
  other exact-hit families, runtime, and audiovisual parity remain open.

### 2.7bh Previous standard BFG 9000 shot-cost delivery target

The bounded implementation target for this revision is the pinned standard
BFG 9000 `shotcost=40` behavior, reusing the delivered exact-hit resolver path
across combat, scenario/replay determinism, MCP behavior, and BrowserSession
parity. Its acceptance criteria are:

- [x] represent the standard BFG 9000's forty-cell per-shot ammo cost in a
  typed core policy while leaving other weapon costs unchanged;
- [x] accept a valid visible, in-range standard BFG 9000 shot with at least
  forty cells, decrementing exactly forty cells once while preserving its
  exact-hit resolution, damage RNG, action cost, and existing events;
- [x] reject clips containing fewer than forty cells atomically, preserving
  complete `Game` and RNG state and avoiding partial shot/event execution;
- [x] verify pure shot-cost boundaries, clip 40/39 integration, deterministic
  scenario/replay equality, MCP projections, and BrowserSession/direct-core
  parity;
- [x] advance gameplay semantics from `32` to `33` and project version from
  `0.2.185` to `0.2.186` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; Nuclear BFG/other shot costs, projectile paths,
  explosions, NukeRun, runtime, and audiovisual parity remain open.

### 2.7bi Previous Revenant’s Launcher exact-hit delivery target

The bounded implementation target for this revision is the pinned Revenant’s
Launcher `IF_EXACTHIT` behavior, reusing the delivered typed resolver path
across combat, scenario/replay determinism, MCP behavior, and BrowserSession
parity. Its acceptance criteria are:

- [x] mark only Revenant’s Launcher as exact-hit in addition to the delivered
  BFG-family policy, preserving ordinary accuracy behavior for every other
  weapon;
- [x] bypass only the to-hit RNG for a valid Revenant’s Launcher shot while
  retaining line-of-sight, range, clip, action-cost, damage RNG, and existing
  attack/damage event contracts;
- [x] reject invalid target, blocked line-of-sight, out-of-range, and empty-clip
  commands atomically, preserving complete `Game` and RNG state;
- [x] verify pure resolver boundaries, deterministic Revenant scenarios,
  replay equality/determinism, MCP projections, and BrowserSession/direct-core
  parity;
- [x] advance gameplay semantics from `33` to `34` and project version from
  `0.2.186` to `0.2.187` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; homing, projectile routing, explosions, other exact-hit
  families, runtime, and audiovisual parity remain open.

### 2.7bj Previous Nuclear BFG 9000 shot-cost delivery target

The bounded implementation target for this revision is the pinned Nuclear BFG
9000 `shotcost=40` behavior, extending the delivered typed standard-BFG
shot-cost policy across combat, scenario/replay determinism, MCP behavior, and
BrowserSession parity. Its acceptance criteria are:

- [x] represent the Nuclear BFG 9000's forty-cell per-shot ammo cost in a typed
  core policy while leaving other weapon costs unchanged;
- [x] accept a valid visible, in-range Nuclear BFG 9000 shot with at least
  forty cells, decrementing exactly forty cells once while preserving its
  exact-hit resolution, damage RNG, action cost, recharge state, and existing
  events;
- [x] reject clips containing fewer than forty cells, invalid targets, blocked
  line-of-sight, and out-of-range targets atomically, preserving complete
  `Game` and RNG state and avoiding partial shot/event execution;
- [x] verify pure shot-cost boundaries, clip 40/39 integration, deterministic
  scenario/replay equality, MCP projections, and BrowserSession/direct-core
  parity;
- [x] advance gameplay semantics from `34` to `35` and project version from
  `0.2.187` to `0.2.188` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; explosions, projectile paths, NukeRun, alternate
  overload/recharge changes, other shot-cost families, runtime, and
  audiovisual parity remain open.

### 2.7bk Previous BFG 10K exact-hit delivery target

The bounded implementation target for this revision is the pinned BFG 10K
`IF_EXACTHIT` behavior, reusing the delivered typed resolver path across
combat, scenario/replay determinism, MCP behavior, and BrowserSession parity.
Its acceptance criteria are:

- [x] mark only BFG 10K as exact-hit in addition to the delivered BFG-family
  policy, preserving ordinary accuracy behavior for every other weapon;
- [x] bypass only the to-hit RNG for a valid BFG 10K shot while retaining
  line-of-sight, range, clip, action-cost, damage RNG, and existing attack and
  damage event contracts;
- [x] reject invalid target, blocked line-of-sight, out-of-range, and empty-clip
  commands atomically, preserving complete `Game` and RNG state;
- [x] verify pure resolver boundaries, deterministic BFG-10K scenarios,
  replay equality/determinism, MCP projections, and BrowserSession/direct-core
  parity;
- [x] advance gameplay semantics from `35` to `36` and project version from
  `0.2.188` to `0.2.189` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; scatter, multi-shot, chainfire, shot cost, explosions,
  other exact-hit families, runtime, and audiovisual parity remain open.

### 2.7bl Previous BFG 10K shot-cost delivery target

The bounded implementation target for this revision is the pinned BFG 10K
`shotcost=5` policy, extending the delivered typed shot-cost seam without
claiming the separate legacy five-shot/scatter path. Its acceptance criteria
are:

- [x] represent only BFG 10K's five-cell per-shot clip cost in a typed policy;
- [x] accept a valid visible, in-range one-shot attack with at least five
  cells, consuming exactly five cells while retaining exact-hit, action-cost,
  damage RNG, and existing attack/damage events;
- [x] reject clips below five, invalid targets, blocked line-of-sight, and
  out-of-range targets atomically, preserving complete `Game` and RNG state;
- [x] verify pure policy boundaries, deterministic BFG-10K scenarios,
  replay equality/determinism, MCP projections, and BrowserSession/direct-core
  parity;
- [x] advance gameplay semantics from `36` to `37` and project version from
  `0.2.189` to `0.2.190` while preserving replay V2 wire, RNG, generator, and
  ruleset identities; five-shot volley, scatter, chainfire, projectile
  routing, explosions, mods, runtime, and audiovisual parity remain open.

### 2.7bm Previous vertical BFG 10K shot-cost delivery target

The bounded implementation target for this revision is a canonical BFG 10K
shot-cost encounter across the declarative scenario, replay, MCP, and browser
presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic ASCII encounter with a configured BFG
  10K and a visible static target at a legal ranged position;
- [x] run the same `Command::AttackRanged` through `ScenarioRunner`, preserving
  exact-hit resolution, five-cell clip consumption, action cost, stable item
  and target identities, and ordered events;
- [x] verify replay determinism and compare replayed events/final game state
  with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for events,
  player observations, pure effect timelines, and scene derivation;
- [x] keep gameplay semantics unchanged while recording the five-shot volley,
  scatter, projectile routing, explosion geometry, controlled legacy runtime,
  browser capture, audio, WebGPU, and audiovisual parity as `NOT_RUN`.

### 2.7bn Current vertical Nuclear BFG 9000 shot-cost delivery target

The bounded implementation target for this revision is a canonical Nuclear BFG
9000 shot-cost encounter across the declarative scenario, replay, MCP, and
browser presentation boundaries. Its vertical slice must:

- [x] construct an exact deterministic ASCII encounter with a configured
  Nuclear BFG 9000 and a visible static target at a legal ranged position;
- [x] run the same `Command::AttackRanged` through `ScenarioRunner`, preserving
  exact-hit resolution, forty-cell clip consumption, action cost, stable item
  and target identities, and ordered events;
- [x] verify replay determinism and compare replayed events/final game state
  with the direct `ScenarioRunner` result;
- [x] compare `BrowserSession::submit` with direct `Game::step` for events,
  player observations, pure effect timelines, and scene derivation;
- [x] keep gameplay semantics unchanged while recording alternate overload,
  recharge timing, projectile routing, explosion geometry, controlled legacy
  runtime, browser capture, audio, WebGPU, and audiovisual parity as `NOT_RUN`.

### 2.8 Exit Gates Before Broad Content Migration Resumes

All of the following are required:

- [x] Rejected commands are state-identical across the audited command surface,
  including late death-drop terrain failures and the typed alternate commands.
- [x] RNG sampling semantics are unbiased, golden-tested, versioned, and
  declared in replay metadata.
- [x] Replays declare gameplay/RNG semantics compatibility and reject
  incompatible interpretation.
- [x] Routine stable identity and normalized spawn registration have one
  authoritative catalog path with materially reduced manual fan-out; gameplay
  balance, count-sensitive reconstruction, behavior, and presentation remain
  explicitly owned elsewhere.
- [x] The typed behavior model passes the selected legacy stress cases
  (Medical Powerarmor, Subtle Knife, and Trigun); broader behavior vocabulary
  and controlled legacy runtime parity remain open.
- [x] `drl-protocol` contains stable semantic contracts but no longer owns
  mutable gameplay balance merely because a type crosses a boundary; the
  catalog slice keeps balance and typed behavior in `drl-core`.
- [x] Large implementation modules touched by this work are split only where
  there are clear independent reasons to change; the catalog slice required no
  new crate or unrelated file-size refactor.
- [x] Repository, deterministic scenario, replay, and supported browser checks
  pass for the resulting revision, with local harness evidence and merged PR
  #243 repository/WASM checks.

Once these gates pass, the next active slice should be a **vertical canonical
fidelity slice**, not another scalar-only family batch.

### 2.9 Vertical Fidelity Successor Slice

The successor slice should select one bounded canonical progression or encounter
and migrate it end-to-end, including relevant interactions among:

- canonical turn economy;
- representative monsters and AI;
- weapon behavior and timing;
- armor/resistance or traits where relevant;
- one or more callback-derived special behaviors;
- deterministic replay/scenario evidence;
- browser presentation required to play the slice.

The `0.2.152` Former Human-profile progression was a foundational progression
slice: it exercised the canonical turn, combat, AI, inventory, and
level-transition path, while callback-derived behavior was already covered by
the preceding typed vertical slices. The `0.2.153` Phase Device escape added a
callback-bearing special-item transition, and the `0.2.154` Shotgun knockback
encounter extends the chain with a scheduled ranged response. The `0.2.155`
Green Armor protection encounter adds the relevant armor/resistance boundary
without changing callback semantics. The `0.2.156` Small MedPack recovery
encounter adds the consumable inventory/health boundary without changing
callback semantics. The `0.2.157` Demon MedPack recovery encounter adds a
seeded melee-AI response around that consumable boundary. Future composite
successors that select callback-bearing behavior must include that behavior
explicitly. The `0.2.158` Pistol reload encounter adds the ranged ammunition
and clip-timing boundary without changing callback semantics. The `0.2.159`
Plasma Rifle cell-reload encounter adds a distinct cell-ammunition and
six-round clip boundary without changing callback semantics. The `0.2.160`
Rocket Launcher one-shot reload encounter adds a one-round rocket clip
boundary without changing callback semantics. The `0.2.161` Chainsaw melee
encounter adds a canonical close-range melee boundary without changing
callback semantics. The `0.2.162` Shotgun shell-reload encounter adds the
standard shell-ammunition and distinct 1200-unit reload boundary without
changing callback semantics. The `0.2.163` Assault Shotgun shell-reload
encounter adds a six-shell clip/reload boundary without changing callback
semantics. The `0.2.164` Double Shotgun clip-reload encounter adds a
two-shell shotgun boundary without changing callback semantics.
The `0.2.165` Assault Shotgun single-shell reload correction makes the
callback-derived one-shell policy explicit, advances gameplay semantics to
`17`, and leaves alternate reload and broader spread/falloff behavior open.
The `0.2.166` Combat Shotgun clip-reload encounter adds a five-shell
ammunition boundary without changing gameplay semantics. The `0.2.167`
successor adds the pinned Combat Shotgun single-shell policy, `0.2.170` adds
the typed pump-action chamber transition, and `0.2.171` adds the typed Assault
Shotgun alternate/full reload, `0.2.172` adds the typed Combat Shotgun
alternate/full reload with chamber reset, and `0.2.173` aligns direct replay
dimension validation with the MCP `3..=512` bound. The `0.2.174` successor
aligns direct replay structural caps for arrays and procedural parameters with
the MCP decoder. Partial-reserve behavior, broader callbacks, replay-file
IO/migrations, and presentation parity remain open.

The `0.2.175` successor adds the bounded, Blaster-only periodic recharge
transition described above. Exact legacy runtime cadence remains `NOT_RUN`; the
accepted-command tick is the explicit deterministic abstraction used by the
headless core and its boundary tests.

The `0.2.176` successor adds typed manual-reload denial for the three pinned
`IF_NORELOAD` families. `IF_NOUNLOAD`, alternate actions, and broader runtime
parity remain open.

The `0.2.177` successor extends the explicit recharge policy to the pinned
Nuclear Plasma Rifle: one cell returns at accepted-command tick `42`, then
every two ticks below capacity. Exact legacy runtime cadence remains `NOT_RUN`.

The `0.2.178` successor extends the explicit recharge policy to the pinned
Nuclear BFG 9000: one cell returns at accepted-command tick `5`, then every
five ticks below capacity. Exact legacy runtime cadence remains `NOT_RUN`.

The `0.2.187` successor extends the typed exact-hit policy to the pinned
Revenant’s Launcher: valid visible, in-range shots bypass only the to-hit
sample while preserving damage RNG and existing command/event contracts.
Homing, projectile routing, delayed explosions, and exact legacy runtime or
audiovisual parity remain open.

The `0.2.188` successor extends the typed shot-cost policy to the pinned Nuclear
BFG 9000: valid ordinary one-shot attacks consume exactly forty cells, while
clips below that threshold reject atomically. Projectile routing, explosions,
NukeRun, alternate overload/recharge changes, and other shot-cost families
remain open.

The `0.2.189` successor extends the typed exact-hit policy to the pinned BFG
10K: valid visible, in-range shots bypass only the to-hit sample while
preserving damage RNG and existing command/event contracts. Scatter,
multi-shot, chainfire, projectile routing, explosions, and shot-cost behavior
remain open.

The `0.2.190` successor extends the typed shot-cost policy to the pinned BFG
10K: valid ordinary one-shot attacks consume exactly five clip cells, while
clips below that threshold reject atomically. The legacy five-shot volley,
scatter, chainfire, projectile routing, explosions, and audiovisual parity
remain open.

The `0.2.191` successor exercises that typed policy in a canonical vertical
BFG 10K encounter across ScenarioRunner, replay, MCP, and BrowserSession
boundaries. Gameplay semantics remain `37`; controlled legacy runtime,
browser capture, and audiovisual parity remain `NOT_RUN`.

The `0.2.192` successor exercises the typed Nuclear BFG 9000 forty-cell
shot-cost policy in the same canonical vertical boundary sequence. Gameplay
semantics remain `37`; controlled legacy runtime, browser capture, and
audiovisual parity remain `NOT_RUN`.

Reference-runtime comparison remains `NOT_RUN` when the controlled legacy
execution environment is unavailable. Source similarity alone is not parity
proof.

### 2.10 Explicit Non-Goals

This active slice does **not** include:

- broad addition of more scalar-only item families;
- full DRL content completion;
- new MCP surface area unrelated to the correctness changes;
- new release-signing or deployment features;
- new browser/platform targets;
- runtime Lua;
- an ECS/plugin framework;
- full audiovisual parity;
- unsupported legal-clearance claims.

---

## 3. Verification Requirements

At minimum, the final delivery must run:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace
sh scripts/check-repository.sh
```

Add focused tests for command rejection atomicity, RNG golden vectors, replay
compatibility rejection, catalog exhaustiveness, and the selected behavior
stress cases.

Browser checks remain required if the slice changes WASM-visible contracts or
presentation behavior. Controlled legacy runtime comparison remains separately
reported as `PASS`, `FAIL`, `INCONCLUSIVE`, or `NOT_RUN`; it must never be
inferred from Rust-only tests.
