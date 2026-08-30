# Specification

Last reviewed: 2026-08-29
Current project version: `0.2.268`

The [Roadmap](docs/DRL-RS_Project_Roadmap.md) owns overall milestone scope,
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

## 2. Active Implementation Slice: M9 — Nuclear BFG 9000 Radius-8 Actor Fanout

### 2.1 Objective

Extend the delivered typed Nuclear BFG 9000 schedule boundary with its
legacy-pinned actor-only radius-8 fanout. A successful direct-target hit must
preserve the existing schedule event, roll one `8d6` Plasma result per clear
blast cell without distance falloff, skip the firing actor, and apply the
bounded radial knockback and deterministic death/drop follow-up without
changing the direct-hit or replay contract.

### 2.1a Scope and steering gate

- **Steering priority:** Vertical canonical fidelity with typed legacy behavior.
- **Steering gates:** Gate A rejected-input safety and Gate B explicit replay
  compatibility remain active acceptance constraints; Gate C catalog ownership
  and Gate D typed behavior evidence remain closed for this bounded extension.
- **Observable outcome:** Each successful Nuclear BFG 9000 direct-target hit
  emits the existing delay-33/radius-8/knockback-16 schedule metadata and
  immediately resolves a bounded actor-only radius-8 fanout in deterministic
  center-then-ring order. Every clear blast cell rolls one `8d6` Plasma
  result with no distance falloff; the firing actor is splash-safe, while
  other actors move by `damage / 16` tiles along the radial direction when
  possible and then receive the rolled environmental damage. Living actors are
  processed once; lethal victims emit normal death/drop follow-up and a player
  death marks game over. Ground items, terrain, secondary chain explosions, and
  delayed timing remain unchanged.
- **Gameplay/replay impact:** Gameplay semantics advance from `76` to `77`;
  replay wire/schema, RNG sampling, generator, and ruleset identities remain
  unchanged. Project version advances from `0.2.267` to `0.2.268`.
- **Protocol/domain ownership:** `drl-core` owns the typed behavior vocabulary,
  typed projectile-count/cost policy and generic execution; `drl-protocol` owns
  the semantic `AttackRanged`/`AttackRangedAimed` commands and typed event
  projection, while replay/MCP and browser boundaries
  serialize and route it without duplicating gameplay rules.
- **Evidence boundary:** Pinned Nuclear BFG 9000 item, exact-hit, shot-cost,
  and delayed-explosion evidence in
  `docs/legacy-behavior/nuclear-bfg9000-explosion.md`, together with the
  existing typed schedule contract and focused
  direct-core/replay/MCP/browser tests, are authoritative. Controlled legacy
  runtime, browser capture, and audiovisual comparisons remain `NOT_RUN`.
- **Non-goals:** EFCHAIN secondary explosions, higher chainfire levels,
  scatter/target rotation or spread, projectile routing,
  delayed timing/state-machine parity, terrain/content mutation, ground-item
  destruction, splash-immunity traits, exact callback timing/accuracy, new
  command variants or callback registries, unrelated gameplay balance, replay
  migrations, runtime Lua, and browser/audio/WebGPU capture parity.

### 2.2 Why this slice is bounded

The immutable profile, semantic command, and schedule event already exist for
the Nuclear BFG 9000. This extension adds only radius-8 geometry, `8d6`
damage, source self-safety, actor de-duplication, and deterministic
knockback/death ordering while reusing the existing replay, MCP, browser, and
transactional boundaries without adding a pending queue, new dispatcher, or
callback system.

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
- [x] Define a typed behavior vocabulary that can represent at least:
  **Delivered in `0.2.193`:** explicit `BehaviorSpec` fragments and immutable
  `BehaviorProfile` compositions cover passive stat/resistance modifiers,
  reversible equip/unequip effects and item-set membership, attack/hit/kill
  effects, alternate fire/reload/use actions, periodic/recharge effects,
  explicit resource/status costs, and deterministic target selection. Existing
  Medical Powerarmor, Subtle Knife, and Trigun transitions remain dedicated
  typed state machines; runtime and presentation parity remain open.
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

### 2.7bn Previous vertical Nuclear BFG 9000 shot-cost delivery target

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

### 2.7bo Previous typed behavior vocabulary delivery target

The bounded implementation target for this revision is an explicit
compile-time vocabulary and immutable profiles for the selected callback-heavy
stress cases. Its contract must:

- [x] represent passive stat/resistance modifiers and reversible equip/unequip
  effects, including typed item-set membership;
- [x] represent attack, hit, and kill effects plus alternate fire/reload/use
  actions without string-keyed dispatch;
- [x] represent periodic/recharge effects and explicit HP, energy, ammo, and
  status costs with typed payloads;
- [x] represent deterministic target-selection policies over fair observation
  or current simulation state;
- [x] compose the Medical Powerarmor, Subtle Knife, and Trigun profiles while
  preserving their existing runtime transitions and recording runtime,
  audiovisual, and controlled-legacy comparisons as `NOT_RUN`.

### 2.7bp Previous Null Pointer behavior-profile delivery target

The bounded implementation target for this revision is an immutable typed
profile for Charch's Null Pointer on-hit behavior. Its contract must:

- [x] select one deterministic target from current simulation state using the
  stable entity-ID order;
- [x] represent the explicit boss/non-boss score-count branch with its 1000
  floor as a typed target property and payload;
- [x] represent the deferred range-1, delay-50 explosion schedule without
  claiming delayed area-damage geometry or runtime parity;
- [x] assert exact profile content and declaration order while preserving the
  existing dedicated `NullPointerHitTransition` runtime state machine;
- [x] keep gameplay semantics, replay schema, RNG, protocol, MCP, browser,
  audio, WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN` where
  comparison evidence is unavailable.

### 2.7bq Previous BFG 10K behavior-profile delivery target

The bounded implementation target for this revision is an immutable typed
profile for the current BFG 10K one-shot path. Its contract must:

- [x] represent the delivered exact-hit policy as a typed attack effect;
- [x] represent the current Rust one-projectile boundary while leaving the
  legacy five-projectile volley explicitly deferred;
- [x] represent the delivered five-cell cell-ammunition cost as a typed
  resource cost;
- [x] assert exact profile content and declaration order while preserving the
  existing dedicated ranged-command runtime path;
- [x] keep gameplay semantics, replay schema, RNG, protocol, MCP, browser,
  audio, WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN` where
  comparison evidence is unavailable.

### 2.7br Previous BFG-family behavior-profile delivery target

The bounded implementation target for this revision is immutable typed
profiles for the standard and Nuclear BFG 9000 current one-shot paths. Their
contracts must:

- [x] represent exact-hit as a typed attack effect;
- [x] represent the current Rust one-projectile boundary while leaving legacy
  projectile volleys/routing explicitly deferred;
- [x] represent the forty-cell Cell ammunition cost as a typed resource cost;
- [x] represent Nuclear BFG's already-delivered periodic recharge and
  alternate overload as typed fragments;
- [x] assert exact profile content/declaration order while preserving the
  dedicated ranged/overload runtime paths;
- [x] keep gameplay semantics, replay schema, RNG, protocol, MCP, browser,
  audio, WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN` where
  comparison evidence is unavailable.

### 2.7bs Previous vertical standard BFG shot-cost delivery target

The bounded implementation target for this revision is a deterministic vertical
standard BFG 9000 encounter that carries the existing forty-cell shot-cost
policy through every supported boundary. Its contract must:

- [x] instantiate a standard BFG 9000 with a visible static target in a legal
  ranged position;
- [x] accept one ordinary shot, debit exactly forty clip cells, and preserve
  the existing exact-hit and one-projectile Rust path;
- [x] assert attack, action-cost, and turn-end event ordering in the scenario;
- [x] prove replay determinism and final-state/event equality through
  `ReplayEngine`;
- [x] prove MCP and BrowserSession/direct-core boundary equality, including
  observations, effects, scene projection, and replay logs;
- [x] keep gameplay semantics, replay schema, RNG, protocol, MCP, browser,
  audio, WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN` where
  comparison evidence is unavailable.

### 2.7bt Previous Nuclear BFG recharge MCP-boundary delivery target

The bounded implementation target for this revision is a deterministic MCP
vertical boundary for the existing Nuclear BFG 9000 recharge policy. Its
contract must:

- [x] load a Nuclear BFG 9000 with a visible static target and a forty-cell
  clip through the canonical replay setup;
- [x] carry one accepted ranged shot followed by four accepted waits, with no
  recharge event before the fifth accepted command and one cell restored on
  the fifth;
- [x] assert MCP events, observations, outcomes, and core state equality for
  every command;
- [x] assert the exported replay contains the command sequence and reproduces
  the direct final state and event stream deterministically;
- [x] keep gameplay semantics, replay schema, RNG, protocol, browser, audio,
  WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN` where
  comparison evidence is unavailable.

### 2.7bu Previous BFG exact-hit MCP-boundary delivery target

The bounded implementation target for this revision is a deterministic MCP
vertical boundary for the existing standard and Nuclear BFG 9000 exact-hit
policies. Its contract must:

- [x] load each BFG family with a visible static target in a legal ranged
  position through the canonical replay setup;
- [x] accept one ordinary shot for each family, preserve the exact-hit and
  one-projectile Rust paths, and assert the expected post-shot clip;
- [x] assert MCP events, observations, outcomes, and full core-state equality;
- [x] assert attack, action-cost, and turn-end event ordering;
- [x] assert the exported replay command sequence, final state, event stream,
  and deterministic verification for both families;
- [x] keep gameplay semantics, replay schema, RNG, protocol, browser, audio,
  WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN` where
  comparison evidence is unavailable.

### 2.7bv Historical BFG 10K five-projectile volley delivery target

The bounded implementation target for this revision is the pinned BFG 10K
five-projectile count and five-cell per-projectile cost on the existing direct
target path. Its contract must:

- [x] expose a typed BFG 10K projectile count of five while leaving ordinary
  weapon fire modes unchanged;
- [x] preflight and consume exactly twenty-five cells for a valid full-clip
  volley, rejecting clips below twenty-five atomically;
- [x] resolve five ordered exact-hit attack/damage pairs against the selected
  target, preserving deterministic RNG consumption and one action-cost/turn-end
  sequence;
- [x] assert ScenarioRunner/replay, MCP, and BrowserSession/direct-core state,
  observation, event, clip, and determinism equality;
- [x] advance gameplay semantics to `38` and reject stale replay metadata before
  simulation;
- [x] keep scatter, projectile routing, explosions, chainfire, mods, protocol,
  audio, WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN` where
  comparison evidence is unavailable.

### 2.7bw Historical BFG 10K explosion-schedule metadata delivery target

The bounded implementation target for this revision is the pinned BFG 10K
delayed explosion payload on the existing direct-target volley path. Its
contract must:

- [x] emit one typed schedule event after each of the five direct-target hit
  pairs;
- [x] preserve delay `25`, radius `2`, and knockback `16` from the pinned
  legacy item payload;
- [x] preserve the existing 25-cell preflight, five ordered attack/damage
  pairs, one action-cost/turn-end sequence, and five-draw RNG advancement;
- [x] assert ScenarioRunner/replay, MCP, and BrowserSession/direct-core state,
  observation, event, and determinism equality;
- [x] advance gameplay semantics to `39` and reject stale replay metadata before
  simulation;
- [x] keep explosion geometry, splash damage, knockback application, scatter,
  projectile routing, chainfire, mods, generic protocol registries, audio,
  WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN` where
  comparison evidence is unavailable.

### 2.7bx Historical standard BFG 9000 explosion-schedule metadata delivery target

The bounded implementation target for this revision is the pinned standard BFG
9000 delayed explosion payload on the existing direct-target shot path. Its
contract must:

- [x] emit one typed schedule event after the direct-target attack/damage pair;
- [x] preserve delay `33`, radius `8`, and knockback `16` from the pinned legacy
  item payload and use the item radius assigned by the legacy ranged path;
- [x] preserve the existing forty-cell preflight, exact-hit damage RNG, one
  action-cost/turn-end sequence, and no additional RNG draws;
- [x] assert ScenarioRunner/replay, MCP, and BrowserSession/direct-core state,
  observation, event, and determinism equality;
- [x] advance gameplay semantics to `40` and reject stale replay metadata before
  simulation;
- [x] keep explosion geometry, splash damage, knockback application, projectile
  routing, alternate overload, mods, generic protocol registries, audio,
  WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN` where
  comparison evidence is unavailable.

### 2.7by Historical Nuclear BFG 9000 explosion-schedule metadata delivery target

The bounded implementation target for this revision is the pinned Nuclear BFG
9000 delayed explosion payload on the existing direct-target shot path. Its
contract must:

- [x] emit one typed schedule event after the direct-target attack/damage pair;
- [x] preserve delay `33`, radius `8`, and knockback `16` from the pinned legacy
  item payload and use the item radius assigned by the legacy ranged path;
- [x] preserve the existing forty-cell preflight, exact-hit damage RNG, one
  action-cost/turn-end sequence, recharge state, alternate-overload state, and
  no additional RNG draws;
- [x] assert ScenarioRunner/replay, MCP, and BrowserSession/direct-core state,
  observation, event, and determinism equality;
- [x] advance gameplay semantics to `41` and reject stale replay metadata before
  simulation;
- [x] keep recharge timing, alternate overload, NukeRun, explosion geometry,
  splash damage, knockback application, projectile routing, mods, generic
  protocol registries, audio, WebGPU, and controlled-legacy behavior unchanged
  or `NOT_RUN` where comparison evidence is unavailable.

### 2.7bz Historical Nuclear Plasma Rifle behavior-profile delivery target

The bounded implementation target for this revision is an immutable profile for
the already-delivered Nuclear Plasma Rifle recharge and alternate-overload
transitions. Its contract must:

- [x] expose ordered `AlternateAction::Overload` and
  `PeriodicEffect::Recharge` fragments;
- [x] preserve delay `40`, cadence `2`, and amount `1` from the pinned legacy
  recharge callback while retaining dedicated transition ownership;
- [x] assert exact profile declaration order without adding a runtime command,
  callback registry, or replay-wire field;
- [x] keep chainfire, dynamic target rotation, gameplay balance, runtime, audio,
  WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN` where
  comparison evidence is unavailable;
- [x] advance project version from `0.2.203` to `0.2.204` while keeping
  gameplay semantics `41`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7ca Historical Blaster behavior-profile delivery target

The bounded implementation target for this revision is an immutable profile for
the already-delivered Blaster recharge transition. Its contract must:

- [x] expose one typed `PeriodicEffect::Recharge` fragment;
- [x] preserve delay `30`, cadence `10`, and amount `1` from the pinned legacy
  recharge callback while retaining dedicated transition ownership;
- [x] assert exact profile declaration order without adding a runtime command,
  callback registry, or replay-wire field;
- [x] keep aimed fire, gameplay balance, runtime, audio, WebGPU, and
  controlled-legacy behavior unchanged or `NOT_RUN` where comparison evidence
  is unavailable;
- [x] advance project version from `0.2.204` to `0.2.205` while keeping
  gameplay semantics `41`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7cb Historical Malek's Armor behavior-profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Malek's Armor durability-recharge transition. Its
contract must:

- [x] expose one typed `PeriodicEffect::DurabilityRecharge` fragment;
- [x] preserve delay `50`, cadence `5`, and amount `1` from the pinned legacy
  callback while retaining dedicated transition ownership;
- [x] assert exact profile declaration order without adding a runtime command,
  callback registry, or replay-wire field;
- [x] keep armor resistance/degradation, gameplay balance, runtime, audio,
  WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN` where
  comparison evidence is unavailable;
- [x] advance project version from `0.2.205` to `0.2.206` while keeping
  gameplay semantics `41`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7cc Historical Lava Armor behavior-profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Lava Armor terrain-gated durability-recharge
transition. Its contract must:

- [x] expose one typed `PeriodicEffect::TerrainRecharge` fragment for
  `TileKind::Lava`;
- [x] preserve interval `5` and amount `3` from the pinned legacy callback
  while retaining dedicated armor-owned transition ownership;
- [x] assert exact profile declaration order without adding a runtime command,
  callback registry, or replay-wire field;
- [x] keep hazard damage/resistance, movement/knockback modifiers, gameplay
  balance, runtime, audio, WebGPU, and controlled-legacy behavior unchanged or
  `NOT_RUN` where comparison evidence is unavailable;
- [x] advance project version from `0.2.206` to `0.2.207` while keeping
  gameplay semantics `41`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7cd Historical Jackhammer behavior-profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Jackhammer burst/single fire-mode transition. Its
contract must:

- [x] expose ordered typed `AlternateAction::Fire` fragments for
  `WeaponFireMode::Single` and `WeaponFireMode::Burst`;
- [x] preserve the one-point score-count cost from the pinned callback while
  retaining dedicated `JackhammerTransition` ownership;
- [x] assert exact profile declaration order without adding a runtime command,
  callback registry, or replay-wire field;
- [x] keep spread/falloff, exact timing/accuracy, gameplay balance, runtime,
  audio, WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN` where
  comparison evidence is unavailable;
- [x] advance project version from `0.2.207` to `0.2.208` while keeping
  gameplay semantics `41`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7ce Historical Grammaton behavior-profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Grammaton Single/Burst/Auto fire-mode transition. Its
contract must:

- [x] expose ordered typed `AlternateAction::Fire` fragments for
  `WeaponFireMode::Single`, `Burst`, and `Auto`;
- [x] preserve the 200-point score-count cost from the pinned callback while
  retaining dedicated `GrammatonTransition` ownership;
- [x] assert exact profile declaration order without adding a runtime command,
  callback registry, or replay-wire field;
- [x] keep legacy accuracy equations, exact timing, gameplay balance, runtime,
  audio, WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN` where
  comparison evidence is unavailable;
- [x] advance project version from `0.2.208` to `0.2.209` while keeping
  gameplay semantics `41`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7cf Historical Acid Spitter behavior-profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Acid Spitter terrain-reload transition. Its contract
must:

- [x] expose one typed `AlternateAction::TerrainReload` fragment requiring
  `TileKind::Acid`, producing `TileKind::Water`, and loading one round;
- [x] preserve the 1,000-point score-count cost from the pinned callback while
  retaining dedicated `acid_spitter` transition ownership;
- [x] assert exact profile declaration order without adding a runtime command,
  callback registry, or replay-wire field;
- [x] keep Acid hazard/resistance, fluid movement cost, gameplay balance,
  runtime, audio, WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN`
  where comparison evidence is unavailable;
- [x] advance project version from `0.2.209` to `0.2.210` while keeping
  gameplay semantics `41`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7cg Historical Missile Launcher behavior-profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Missile Launcher ordinary and alternate/full reload
transitions. Its contract must:

- [x] expose ordered typed `AlternateAction::Reload` and
  `AlternateAction::FullReload { cost_cap: 2500 }` fragments;
- [x] retain dedicated ordinary reload and `MissileLauncherTransition`
  planner ownership for one-rocket, full-deficit, reserve, and capped-cost
  execution;
- [x] assert exact profile declaration order without adding a runtime command,
  callback registry, or replay-wire field;
- [x] keep rocket-jump, explosion, gameplay balance, runtime, audio, WebGPU,
  and controlled-legacy behavior unchanged or `NOT_RUN` where comparison
  evidence is unavailable;
- [x] advance project version from `0.2.210` to `0.2.211` while keeping
  gameplay semantics `41`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7ch Historical Combat Shotgun behavior-profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Combat Shotgun ordinary and alternate/full reload
transitions. Its contract must:

- [x] expose ordered typed `AlternateAction::Reload` and
  `AlternateAction::FullReload { cost_cap: 2500 }` fragments;
- [x] retain dedicated normal reload, `CombatShotgunTransition` planner, and
  pump-action state ownership for one-shell, full-deficit, reserve, chamber,
  and capped-cost execution;
- [x] assert exact profile declaration order without adding a runtime command,
  callback registry, or replay-wire field;
- [x] keep gameplay balance, exact legacy timing, runtime, audio, WebGPU,
  chamber presentation, and controlled-legacy behavior unchanged or `NOT_RUN`
  where comparison evidence is unavailable;
- [x] advance project version from `0.2.211` to `0.2.212` while keeping
  gameplay semantics `41`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7ci Historical Revenant's Launcher behavior-profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Revenant's Launcher exact-hit transition. Its
contract must:

- [x] expose one typed `AttackEffect::ExactHit` fragment;
- [x] retain dedicated combat ownership for LOS/range/clip/action-cost
  validation, damage RNG, and ordered attack/damage events;
- [x] assert exact profile declaration order without adding a runtime command,
  callback registry, or replay-wire field;
- [x] keep homing, projectile routing, delayed explosions, gameplay balance,
  runtime, audio, WebGPU, and controlled-legacy behavior unchanged or `NOT_RUN`
  where comparison evidence is unavailable;
- [x] advance project version from `0.2.212` to `0.2.213` while keeping
  gameplay semantics `41`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7cj Historical Assault Shotgun behavior-profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Assault Shotgun ordinary and alternate/full reload
transitions. Its contract must:

- [x] expose ordered typed `AlternateAction::Reload` and
  `AlternateAction::FullReload { cost_cap: 2500 }` fragments;
- [x] retain dedicated normal reload and `AssaultShotgunTransition` planner
  ownership for one-shell, full-deficit, reserve, and capped-cost execution;
- [x] assert exact profile declaration order without adding a runtime command,
  callback registry, or replay-wire field;
- [x] keep gameplay balance unchanged, while exact legacy timing,
  partial-reserve policy, runtime, audio, WebGPU, and controlled-legacy behavior
  remain `NOT_RUN` where comparison evidence is unavailable;
- [x] advance project version from `0.2.213` to `0.2.214` while keeping
  gameplay semantics `41`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7ck Historical Combat Shotgun pump-action profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Combat Shotgun pump-only chamber transition alongside
its ordinary and alternate/full reload transitions. Its contract must:

- [x] expose ordered typed `ActionEffect::Pump { cost: 200 }`,
  `AlternateAction::Reload`, and
  `AlternateAction::FullReload { cost_cap: 2500 }` fragments;
- [x] retain dedicated pump-action state, normal reload, and
  `CombatShotgunTransition` planner ownership for chamber, reserve, deficit,
  and capped-cost execution;
- [x] assert exact profile declaration order without adding a runtime command,
  callback registry, or replay-wire field;
- [x] keep gameplay balance unchanged, while exact legacy timing,
  partial-reserve policy, chamber presentation, runtime, audio, WebGPU, and
  controlled-legacy behavior remain `NOT_RUN` where comparison evidence is
  unavailable;
- [x] advance project version from `0.2.214` to `0.2.215` while keeping
  gameplay semantics `41`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7cl Historical Double Shotgun dual-shot delivery target

The bounded implementation target for this revision is deterministic
two-projectile Double Shotgun fire plus its immutable typed behavior profile.
Its contract must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(2)` and
  `ResourceCost::Ammo { ammo_type: Shells, amount: 2 }` fragments;
- [x] resolve two ordered `AttackResolved` outcomes and consume two clip shells
  only after complete target/LOS/range/death-drop preflight succeeds;
- [x] preserve existing ranged damage RNG order, lethal/death-drop handling,
  replay/scenario/MCP/BrowserSession boundaries, and rejection atomicity;
- [x] assert exact profile declaration order without adding a command, callback
  registry, or wire field; spread/falloff, exact timing, runtime, and
  audiovisual parity remain `NOT_RUN` where comparison evidence is unavailable;
- [x] advance project version from `0.2.215` to `0.2.216` and gameplay
  semantics from `41` to `42`, preserving replay schema, RNG, generator, and
  ruleset identities.

### 2.7cm Historical Standard Shotgun knockback profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered standard Shotgun one-cell knockback hit and shell
cost. Its contract must:

- [x] expose ordered typed `HitEffect::Knockback { distance: 1 }` and
  `ResourceCost::Ammo { ammo_type: Shells, amount: 1 }` fragments;
- [x] retain generic ranged execution ownership for target validation, damage,
  collision-aware displacement, and transactional rejection behavior;
- [x] assert exact profile declaration order without adding a command, callback
  registry, replay-wire field, or gameplay semantics change;
- [x] keep exact legacy force/timing, spread/falloff, controlled runtime, and
  audiovisual parity `NOT_RUN` where comparison evidence is unavailable;
- [x] advance project version from `0.2.216` to `0.2.217` while keeping
  gameplay semantics `42`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7cn Historical Pistol ordinary-fire profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Pistol ordinary ranged action. Its contract must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(1)` and
  `ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 1 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, and transactional clip consumption;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, replay-wire field, or gameplay semantics change;
- [x] keep aimed-fire callback semantics, exact legacy timing/accuracy,
  controlled runtime, and audiovisual parity `NOT_RUN` where comparison
  evidence is unavailable;
- [x] advance project version from `0.2.217` to `0.2.218` while keeping
  gameplay semantics `42`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7co Historical Rocket Launcher ordinary-fire profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Rocket Launcher ordinary ranged action. Its contract
must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(1)` and
  `ResourceCost::Ammo { ammo_type: Rocket, amount: 1 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, and transactional clip consumption;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, replay-wire field, or gameplay semantics change;
- [x] keep rocket-jump/explosion callback semantics, exact legacy timing/
  accuracy, controlled runtime, and audiovisual parity `NOT_RUN` where
  comparison evidence is unavailable;
- [x] advance project version from `0.2.218` to `0.2.219` while keeping
  gameplay semantics `42`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7cp Historical Combat Pistol ordinary-fire profile delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Combat Pistol ordinary ranged action. Its contract
must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(1)` and
  `ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 1 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, and transactional clip consumption;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, replay-wire field, or gameplay semantics change;
- [x] keep aimed-fire callback semantics, exact legacy timing/accuracy,
  controlled runtime, and audiovisual parity `NOT_RUN` where comparison
  evidence is unavailable;
- [x] advance project version from `0.2.219` to `0.2.220` while keeping
  gameplay semantics `42`, replay schema, RNG, generator, and ruleset
  identities unchanged.

### 2.7cq Historical Plasma Shotgun ordinary-fire cost delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Plasma Shotgun ordinary ranged action. Its contract
must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(1)` and
  `ResourceCost::Ammo { ammo_type: Cell, amount: 3 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, and transactional clip consumption;
- [x] enforce the three-cell cost before mutation, reject clips below the cost
  atomically, and preserve the existing one-projectile event contract;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, or replay-wire field; stale gameplay semantics
  `42` replays are rejected before execution;
- [x] keep spread/falloff/knockback callback semantics, exact legacy
  timing/accuracy, controlled runtime, and audiovisual parity `NOT_RUN` where
  comparison evidence is unavailable;
- [x] advance project version from `0.2.220` to `0.2.221` and gameplay
  semantics from `42` to `43`, preserving replay schema, RNG, generator, and
  ruleset identities.

### 2.7cr Historical Frag Shotgun ordinary-fire cost delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Frag Shotgun ordinary ranged action. Its contract
must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(1)` and
  `ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 2 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, and transactional clip consumption;
- [x] enforce the two-round cost before mutation, reject clips below the cost
  atomically, and preserve the existing one-projectile event contract;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, or replay-wire field; stale gameplay semantics
  `43` replays are rejected before execution;
- [x] keep spread/falloff/knockback callback semantics, exact legacy
  timing/accuracy, controlled runtime, and audiovisual parity `NOT_RUN` where
  comparison evidence is unavailable;
- [x] advance project version from `0.2.221` to `0.2.222` and gameplay
  semantics from `43` to `44`, preserving replay schema, RNG, generator, and
  ruleset identities.

### 2.7cs Historical Railgun ordinary-fire cost delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Railgun ordinary ranged action. Its contract must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(1)` and
  `ResourceCost::Ammo { ammo_type: Cell, amount: 5 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, and transactional clip consumption;
- [x] enforce the five-cell cost before mutation, reject clips below the cost
  atomically, and preserve the existing one-projectile event contract;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, or replay-wire field; stale gameplay semantics
  `44` replays are rejected before execution;
- [x] keep ray/piercing routing, spread/falloff callback semantics, exact legacy
  timing/accuracy, controlled runtime, and audiovisual parity `NOT_RUN` where
  comparison evidence is unavailable;
- [x] advance project version from `0.2.222` to `0.2.223` and gameplay
  semantics from `44` to `45`, preserving replay schema, RNG, generator, and
  ruleset identities.

### 2.7ct Historical Null Pointer ordinary-fire cost delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Charch's Null Pointer ordinary ranged action. Its
contract must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(1)` and
  `ResourceCost::Ammo { ammo_type: Cell, amount: 10 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, the existing target-score branch,
  deferred explosion scheduling, and transactional clip consumption;
- [x] enforce the ten-cell cost before mutation, reject clips below the cost
  atomically, and preserve the existing one-projectile event contract;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, or replay-wire field; stale gameplay semantics
  `45` replays are rejected before execution;
- [x] keep delayed explosion geometry, full callback parity, exact legacy
  timing/accuracy, controlled runtime, and audiovisual parity `NOT_RUN` where
  comparison evidence is unavailable;
- [x] advance project version from `0.2.223` to `0.2.224` and gameplay
  semantics from `45` to `46`, preserving replay schema, RNG, generator, and
  ruleset identities.

### 2.7cu Historical Tristar Blaster ordinary-fire volley delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Tristar Blaster ordinary ranged action. Its contract
must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(3)` and
  `ResourceCost::Ammo { ammo_type: Cell, amount: 5 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, and transactional clip consumption;
- [x] enforce the fifteen-cell total cost before mutation, reject clips below
  the cost atomically, and preserve the existing three-projectile event
  contract;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, or replay-wire field; stale gameplay semantics
  `46` replays are rejected before execution;
- [x] keep spread routing, delayed explosion geometry, exact legacy
  timing/accuracy, controlled runtime, and audiovisual parity `NOT_RUN` where
  comparison evidence is unavailable;
- [x] advance project version from `0.2.224` to `0.2.225` and gameplay
  semantics from `46` to `47`, preserving replay schema, RNG, generator, and
  ruleset identities.

### 2.7cv Historical Acid Spitter ordinary-fire cost delivery target

The bounded implementation target for this revision is an immutable profile
for the already-delivered Acid Spitter ordinary ranged action. Its contract
must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(1)` and
  `ResourceCost::Ammo { ammo_type: Rocket, amount: 10 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, and transactional clip consumption;
- [x] enforce the ten-rocket cost before mutation, reject clips below the cost
  atomically, and preserve the existing one-projectile event contract;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, or replay-wire field; stale gameplay semantics
  `47` replays are rejected before execution;
- [x] keep the existing Acid-to-Water reload transition authoritative while
  leaving explosion geometry/content, spread/falloff, exact legacy
  timing/accuracy, controlled runtime, and audiovisual parity `NOT_RUN` where
  comparison evidence is unavailable;
- [x] advance project version from `0.2.225` to `0.2.226` and gameplay
  semantics from `47` to `48`, preserving replay schema, RNG, generator, and
  ruleset identities.

### 2.7cw Historical Mega Buster ordinary-fire volley delivery target

The bounded implementation target for this revision is an immutable profile
for the pinned Mega Buster ordinary ranged action. Its contract must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(3)` and
  `ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 3 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, and transactional clip consumption;
- [x] enforce the nine-round total cost before mutation, reject clips below the
  cost atomically, and preserve the existing three-projectile event contract;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, or replay-wire field; stale gameplay semantics
  `48` replays are rejected before execution;
- [x] keep the Mega Buster kill callback, spread/falloff, exact legacy
  timing/accuracy, controlled runtime, and audiovisual parity `NOT_RUN` where
  comparison evidence is unavailable;
- [x] advance project version from `0.2.226` to `0.2.227` and gameplay
  semantics from `48` to `49`, preserving replay schema, RNG, generator, and
  ruleset identities.

### 2.7cx Historical Super Shotgun ordinary-fire volley delivery target

The bounded implementation target for this revision is an immutable profile
for the pinned Super Shotgun ordinary ranged action. Its contract must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(2)` and
  `ResourceCost::Ammo { ammo_type: Shells, amount: 1 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, and transactional clip consumption;
- [x] enforce the two-shell aggregate cost before mutation, reject clips below
  the cost atomically, and preserve the existing two-projectile event contract;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, or replay-wire field; stale gameplay semantics
  `49` replays are rejected before execution;
- [x] keep spread/falloff, exact legacy timing/accuracy, controlled runtime,
  and audiovisual parity `NOT_RUN` where comparison evidence is unavailable;
- [x] advance project version from `0.2.227` to `0.2.228` and gameplay
  semantics from `49` to `50`, preserving replay schema, RNG, generator, and
  ruleset identities.

### 2.7cy Historical Minigun ordinary-fire volley delivery target

The bounded implementation target for this revision is an immutable profile
for the pinned Minigun ordinary ranged action. Its contract must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(8)` and
  `ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 1 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, and transactional clip consumption;
- [x] enforce the eight-round aggregate cost before mutation, reject clips
  below the cost atomically, and preserve the existing eight-projectile event
  contract;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, or replay-wire field; stale gameplay semantics
  `50` replays are rejected before execution;
- [x] keep alternate chainfire, spread/falloff, exact legacy timing/accuracy,
  controlled runtime, and audiovisual parity `NOT_RUN` where comparison
  evidence is unavailable;
- [x] advance project version from `0.2.228` to `0.2.229` and gameplay
  semantics from `50` to `51`, preserving replay schema, RNG, generator, and
  ruleset identities.

### 2.7cz Historical Chaingun ordinary-fire volley delivery target

The bounded implementation target for this revision is an immutable profile
for the pinned Chaingun ordinary ranged action. Its contract must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(4)` and
  `ResourceCost::Ammo { ammo_type: Ammo9mm, amount: 1 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, and transactional clip consumption;
- [x] enforce the four-round aggregate cost before mutation, reject clips
  below the cost atomically, and preserve the existing four-projectile event
  contract;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, or replay-wire field; stale gameplay semantics
  `51` replays are rejected before execution;
- [x] keep higher chainfire levels, spread/falloff, exact legacy
  timing/accuracy, controlled runtime, and audiovisual parity `NOT_RUN` where
  comparison evidence is unavailable; first-level chainfire is covered by the
  successor `2.7ea` target;
- [x] advance project version from `0.2.229` to `0.2.230` and gameplay
  semantics from `51` to `52`, preserving replay schema, RNG, generator, and
  ruleset identities.

### 2.7da Historical Laser Rifle ordinary-fire volley delivery target

The bounded implementation target for this revision is an immutable profile
for the pinned Laser Rifle ordinary ranged action. Its contract must:

- [x] expose ordered typed `AttackEffect::ProjectileCount(5)` and
  `ResourceCost::Ammo { ammo_type: Cell, amount: 1 }` fragments;
- [x] retain generic ranged execution ownership for target/LOS/range
  validation, damage RNG, event ordering, and transactional clip consumption;
- [x] enforce the five-cell aggregate cost before mutation, reject clips below
  the cost atomically, and preserve the existing five-projectile event
  contract;
- [x] assert exact profile declaration order without adding an alternate-fire
  command, callback registry, or replay-wire field; stale gameplay semantics
  `52` replays are rejected before execution;
- [x] keep alternate chainfire, spread/falloff, exact legacy timing/accuracy,
  controlled runtime, and audiovisual parity `NOT_RUN` where comparison
  evidence is unavailable;
- [x] advance project version from `0.2.230` to `0.2.231` and gameplay
  semantics from `52` to `53`, preserving replay schema, RNG, generator, and
  ruleset identities.

### 2.7db Historical Laser Rifle browser-boundary verification target

The bounded implementation target for this revision is a direct-core and
`BrowserSession` parity test for the delivered Laser Rifle ordinary-fire
contract. Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Laser Rifle, cell reserve, and high-HP target;
- [x] submit one ranged command through both paths and preserve identical
  `GameEvent` sequences, fair player observations, render effects, and scene
  projections;
- [x] verify the accepted command emits five ordered ranged-hit events and
  consumes five cells from the 40-cell clip without changing gameplay policy;
- [x] append the command to a replay, reproduce the direct events/state, and
  pass deterministic replay verification without adding protocol or runtime
  dispatch surfaces;
- [x] keep controlled legacy runtime, browser capture, exact timing/accuracy,
  alternate chainfire, and audiovisual parity `NOT_RUN` where evidence is
  unavailable;
- [x] advance project version from `0.2.231` to `0.2.232` while preserving
  gameplay semantics `53`, replay schema, RNG, generator, and ruleset
  identities.

### 2.7dc Historical Minigun browser-boundary verification target

The bounded implementation target for this revision is a direct-core and
`BrowserSession` parity test for the delivered Minigun ordinary-fire contract.
Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Minigun, 9mm reserve, and high-HP target;
- [x] submit one ranged command through both paths and preserve identical
  `GameEvent` sequences, fair player observations, render effects, and scene
  projections;
- [x] verify the accepted command emits eight ordered ranged-hit events and
  consumes eight rounds from the 200-round clip without changing gameplay
  policy;
- [x] append the command to a replay, reproduce the direct events/state, and
  pass deterministic replay verification without adding protocol or runtime
  dispatch surfaces;
- [x] keep controlled legacy runtime, browser capture, exact timing/accuracy,
  alternate chainfire, and audiovisual parity `NOT_RUN` where evidence is
  unavailable;
- [x] advance project version from `0.2.232` to `0.2.233` while preserving
  gameplay semantics `53`, replay schema, RNG, generator, and ruleset
  identities.

### 2.7dd Historical Chaingun browser-boundary verification target

The bounded implementation target for this revision is a direct-core and
`BrowserSession` parity test for the delivered Chaingun ordinary-fire contract.
Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Chaingun, 9mm reserve, and high-HP target;
- [x] submit one ranged command through both paths and preserve identical
  `GameEvent` sequences, fair player observations, render effects, and scene
  projections;
- [x] verify the accepted command emits four ordered ranged-hit events and
  consumes four rounds from the 40-round clip without changing gameplay policy;
- [x] append the command to a replay, reproduce the direct events/state, and
  pass deterministic replay verification without adding protocol or runtime
  dispatch surfaces;
- [x] keep controlled legacy runtime, browser capture, exact timing/accuracy,
  alternate chainfire, and audiovisual parity `NOT_RUN` where evidence is
  unavailable;
- [x] advance project version from `0.2.233` to `0.2.234` while preserving
  gameplay semantics `53`, replay schema, RNG, generator, and ruleset
  identities.

### 2.7de Historical Mega Buster browser-boundary verification target

The bounded implementation target for this revision is a direct-core and
`BrowserSession` parity test for the delivered Mega Buster ordinary-fire
contract. Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Mega Buster, 9mm reserve, and high-HP target;
- [x] submit one ranged command through both paths and preserve identical
  `GameEvent` sequences, fair player observations, render effects, and scene
  projections;
- [x] verify the accepted command emits three ordered ranged-hit events and
  consumes nine rounds from the 60-round clip without changing gameplay policy;
- [x] append the command to a replay, reproduce the direct events/state, and
  pass deterministic replay verification without adding protocol or runtime
  dispatch surfaces;
- [x] keep controlled legacy runtime, browser capture, exact timing/accuracy,
  kill callback, and audiovisual parity `NOT_RUN` where evidence is unavailable;
- [x] advance project version from `0.2.234` to `0.2.235` while preserving
  gameplay semantics `53`, replay schema, RNG, generator, and ruleset
  identities.

### 2.7df Historical Super Shotgun browser-boundary verification target

The bounded implementation target for this revision is a direct-core and
`BrowserSession` parity test for the delivered Super Shotgun ordinary-fire
contract. Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Super Shotgun, shell reserve, and high-HP target;
- [x] submit one ranged command through both paths and preserve identical
  `GameEvent` sequences, fair player observations, render effects, and scene
  projections;
- [x] verify the accepted command emits two ordered ranged-hit events and
  consumes two shells from the two-shell clip without changing gameplay policy;
- [x] append the command to a replay, reproduce the direct events/state, and
  pass deterministic replay verification without adding protocol or runtime
  dispatch surfaces;
- [x] keep controlled legacy runtime, browser capture, exact timing/accuracy,
  spread/falloff, and audiovisual parity `NOT_RUN` where evidence is unavailable;
- [x] advance project version from `0.2.235` to `0.2.236` while preserving
  gameplay semantics `53`, replay schema, RNG, generator, and ruleset
  identities.

### 2.7dg Historical Tristar Blaster browser-boundary verification target

The bounded implementation target for this revision is a direct-core and
`BrowserSession` parity test for the delivered Tristar Blaster ordinary-fire
contract. Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Tristar Blaster, cell reserve, and high-HP target;
- [x] submit one ranged command through both paths and preserve identical
  `GameEvent` sequences, fair player observations, render effects, and scene
  projections;
- [x] verify the accepted command emits three ordered ranged-hit events and
  consumes fifteen cells from the 45-cell clip without changing gameplay policy;
- [x] append the command to a replay, reproduce the direct events/state, and
  pass deterministic replay verification without adding protocol or runtime
  dispatch surfaces;
- [x] keep controlled legacy runtime, browser capture, exact timing/accuracy,
  spread/falloff, delayed effects, and audiovisual parity `NOT_RUN` where
  evidence is unavailable;
- [x] advance project version from `0.2.236` to `0.2.237` while preserving
  gameplay semantics `53`, replay schema, RNG, generator, and ruleset
  identities.

### 2.7dh Historical Railgun browser-boundary verification target

The bounded implementation target for this revision is a direct-core and
`BrowserSession` parity test for the delivered Railgun ordinary-fire contract.
Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Railgun, cell reserve, and high-HP target;
- [x] submit one ranged command through both paths and preserve identical
  `GameEvent` sequences, fair player observations, render effects, and scene
  projections;
- [x] verify the accepted command emits one ordered ranged-hit event and
  consumes five cells from the 40-cell clip without changing gameplay policy;
- [x] append the command to a replay, reproduce the direct events/state, and
  pass deterministic replay verification without adding protocol or runtime
  dispatch surfaces;
- [x] keep controlled legacy runtime, browser capture, exact timing/accuracy,
  piercing, spread/falloff, and audiovisual parity `NOT_RUN` where evidence is
  unavailable;
- [x] advance project version from `0.2.237` to `0.2.238` while preserving
  gameplay semantics `53`, replay schema, RNG, generator, and ruleset
  identities.

### 2.7di Historical Frag Shotgun browser-boundary verification target

The bounded implementation target for this revision is a direct-core and
`BrowserSession` parity test for the delivered Frag Shotgun ordinary-fire
contract. Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Frag Shotgun, 9mm reserve, and high-HP target;
- [x] submit one ranged command through both paths and preserve identical
  `GameEvent` sequences, fair player observations, render effects, and scene
  projections;
- [x] verify the accepted command emits one ordered ranged-hit event and
  consumes two rounds from the 16-round clip without changing gameplay policy;
- [x] append the command to a replay, reproduce the direct events/state, and
  pass deterministic replay verification without adding protocol or runtime
  dispatch surfaces;
- [x] keep controlled legacy runtime, browser capture, exact timing/accuracy,
  spread/falloff, knockback, and audiovisual parity `NOT_RUN` where evidence is
  unavailable;
- [x] advance project version from `0.2.238` to `0.2.239` while preserving
  gameplay semantics `53`, replay schema, RNG, generator, and ruleset
  identities.

### 2.7dj Historical Combat Pistol browser-boundary verification target

The bounded implementation target for this revision is a direct-core and
`BrowserSession` parity test for the delivered Combat Pistol ordinary-fire
contract. Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Combat Pistol, 9mm reserve, and high-HP target;
- [x] submit one ranged command through both paths and preserve identical
  `GameEvent` sequences, fair player observations, render effects, and scene
  projections;
- [x] verify the accepted command emits one ordered ranged-hit event and
  consumes one round from the 15-round clip without changing gameplay policy;
- [x] append the command to a replay, reproduce the direct events/state, and
  pass deterministic replay verification without adding protocol or runtime
  dispatch surfaces;
- [x] keep controlled legacy runtime, browser capture, aimed callback,
  exact timing/accuracy, and audiovisual parity `NOT_RUN` where evidence is
  unavailable;
- [x] advance project version from `0.2.239` to `0.2.240` while preserving
  gameplay semantics `53`, replay schema, RNG, generator, and ruleset
  identities.

### 2.7dk Historical Pistol browser-boundary verification target

The bounded implementation target for this revision is a direct-core and
`BrowserSession` parity test for the delivered Pistol ordinary-fire contract.
Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Pistol, 9mm reserve, and high-HP target;
- [x] submit one ranged command through both paths and preserve identical
  `GameEvent` sequences, fair player observations, render effects, and scene
  projections;
- [x] verify the accepted command emits one ordered ranged-hit event and
  consumes one round from the 10-round clip without changing gameplay policy;
- [x] append the command to a replay, reproduce the direct events/state, and
  pass deterministic replay verification without adding protocol or runtime
  dispatch surfaces;
- [x] keep controlled legacy runtime, browser capture, aimed behavior, exact
  timing/accuracy, and audiovisual parity `NOT_RUN` where evidence is unavailable;
- [x] advance project version from `0.2.240` to `0.2.241` while preserving
  gameplay semantics `53`, replay schema, RNG, generator, and ruleset
  identities.

### 2.7dl Historical Plasma Shotgun browser-boundary verification target

The bounded implementation target for this revision is a direct-core and
`BrowserSession` parity test for the delivered Plasma Shotgun ordinary-fire
contract. Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Plasma Shotgun, cell reserve, and high-HP target;
- [x] submit one ranged command through both paths and preserve identical
  `GameEvent` sequences, fair player observations, render effects, and scene
  projections;
- [x] verify the accepted command emits one ordered ranged-hit event and
  consumes three cells from the 30-cell clip without changing gameplay policy;
- [x] append the command to a replay, reproduce the direct events/state, and
  pass deterministic replay verification without adding protocol or runtime
  dispatch surfaces;
- [x] keep controlled legacy runtime, browser capture, spread/falloff,
  knockback, exact timing/accuracy, and audiovisual parity `NOT_RUN` where
  evidence is unavailable;
- [x] advance project version from `0.2.241` to `0.2.242` while preserving
  gameplay semantics `53`, replay schema, RNG, generator, and ruleset
  identities.

### 2.7dm Historical Blaster browser-boundary verification target

The bounded implementation target for this revision is a direct-core and
`BrowserSession` parity test for the delivered Blaster ordinary-fire contract.
Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Blaster, cell reserve, and high-HP target;
- [x] submit one ranged command through both paths and preserve identical
  `GameEvent` sequences, fair player observations, render effects, and scene
  projections;
- [x] verify the accepted command emits one ordered ranged-hit event and
  consumes one cell from the 10-cell clip without changing recharge or
  no-manual-reload policy;
- [x] append the command to a replay, reproduce the direct events/state, and
  pass deterministic replay verification without adding protocol or runtime
  dispatch surfaces;
- [x] keep controlled legacy runtime, browser capture, aimed callback,
  exact timing/accuracy, and audiovisual parity `NOT_RUN` where evidence is
  unavailable;
- [x] advance project version from `0.2.242` to `0.2.243` while preserving
  gameplay semantics `53`, replay schema, RNG, generator, and ruleset
  identities.

### 2.7dn Historical Pistol aimed-fire vertical fidelity target

The bounded implementation target for this revision is a typed Pistol-only
aimed-fire command exercised through direct core, replay/MCP JSON, and
`BrowserSession`. Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Pistol's ten-round clip, six reserve 9mm rounds, and a high-HP target;
- [x] submit `Command::AttackRangedAimed` through both paths and preserve
  identical `GameEvent` sequences, fair player observations, render effects,
  and scene projections;
- [x] apply the typed +3 accuracy bonus and doubled fire cost, paying
  `ActionCost(2_000)`, while emitting one ordered ranged-hit event and
  consuming exactly one 9mm round;
- [x] preserve deterministic replay verification, advertise/execute the aimed
  command through the fair MCP action catalog, and encode/decode it at the
  replay and MCP JSON boundaries without duplicating gameplay policy;
- [x] reject aimed fire atomically for non-Pistol weapons and empty Pistol
  clips, preserving the complete pre-command game state;
- [x] keep exact legacy callback state/timing, controlled runtime, browser
  capture, audiovisual parity, and presentation comparison `NOT_RUN` where
  evidence is unavailable;
- [x] advance project version from `0.2.243` to `0.2.244` and gameplay
  semantics from `53` to `54` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7do Historical Combat Pistol aimed-fire vertical fidelity target

The bounded implementation target for this revision is the shared typed
aimed-fire command exercised with Combat Pistol through direct core,
replay/MCP JSON, MCP action catalog, and `BrowserSession`. Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Combat Pistol's fifteen-round clip, six reserve 9mm rounds, and a
  high-HP target;
- [x] submit `Command::AttackRangedAimed` through both paths and preserve
  identical `GameEvent` sequences, fair player observations, render effects,
  and scene projections;
- [x] apply the shared typed +3 accuracy bonus and doubled fire cost, paying
  `ActionCost(2_000)`, while emitting one ordered ranged-hit event and
  consuming exactly one 9mm round;
- [x] advertise and execute the aimed command through the fair MCP action
  catalog, and encode/decode it at replay/MCP JSON boundaries without
  duplicating gameplay policy;
- [x] preserve deterministic replay verification and exact game-state
  rejection behavior for unsupported weapons and empty clips;
- [x] keep exact legacy callback state/timing, controlled runtime, browser
  capture, audiovisual parity, and presentation comparison `NOT_RUN` where
  evidence is unavailable;
- [x] advance project version from `0.2.244` to `0.2.245` and gameplay
  semantics from `54` to `55` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7dp Historical Blaster aimed-fire vertical fidelity target

The bounded implementation target for this revision is the shared typed
aimed-fire command exercised with Blaster through direct core, replay/MCP JSON,
MCP action catalog, and `BrowserSession`. Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Blaster's ten-cell clip, six reserve cells, and a high-HP target;
- [x] submit `Command::AttackRangedAimed` through both paths and preserve
  identical `GameEvent` sequences, fair player observations, render effects,
  and scene projections;
- [x] apply the shared typed +3 accuracy bonus and doubled fire cost, paying
  `ActionCost(2_000)`, while emitting one ordered ranged-hit event and
  consuming exactly one cell;
- [x] advertise and execute the aimed command through the fair MCP action
  catalog, and encode/decode it at replay/MCP JSON boundaries without
  duplicating gameplay policy;
- [x] preserve the Blaster's typed recharge reset and `IF_NORELOAD` behavior,
  deterministic replay verification, and exact rejection behavior for an
  empty clip;
- [x] keep exact legacy callback state/timing, controlled runtime, browser
  capture, audiovisual parity, and presentation comparison `NOT_RUN` where
  evidence is unavailable;
- [x] advance project version from `0.2.245` to `0.2.246` and gameplay
  semantics from `55` to `56` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7dq Historical Plasma Rifle ordinary-fire volley target

The bounded implementation target for this revision is the typed Plasma Rifle
ordinary-fire volley exercised through direct core, replay/MCP JSON, MCP action
catalog, and `BrowserSession`. Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Plasma Rifle's six-cell clip, six reserve cells, and a high-HP target;
- [x] submit one ordinary `Command::AttackRanged` through both paths and
  preserve identical `GameEvent` sequences, fair player observations, render
  effects, and scene projections;
- [x] emit six ordered ranged-hit events, consume one cell per projectile (six
  cells total), and pay the ordinary ranged action cost;
- [x] advertise and execute the ordinary command through the fair MCP action
  catalog, and encode/decode it at replay/MCP JSON boundaries without
  duplicating gameplay policy;
- [x] preserve deterministic replay verification and reject clips below six
  cells atomically, while leaving the existing reload path authoritative;
- [x] keep chainfire, overcharge, spread/falloff, projectile routing, exact
  callback timing, controlled runtime, browser capture, and audiovisual parity
  `NOT_RUN` where evidence is unavailable;
- [x] advance project version from `0.2.246` to `0.2.247` and gameplay
  semantics from `56` to `57` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7dr Historical Trigun aimed-fire vertical fidelity target

The bounded implementation target for this revision is the typed Trigun
aimed-fire command exercised through direct core, replay/MCP JSON, MCP action
catalog, and `BrowserSession`. Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Trigun's six-round clip, six reserve 9mm rounds, and a high-HP target;
- [x] submit `Command::AttackRangedAimed` through both paths and preserve
  identical `GameEvent` sequences, fair player observations, render effects,
  and scene projections;
- [x] apply the typed +3 accuracy bonus and doubled fire cost, paying
  `ActionCost(2_000)`, while emitting one ordered ranged-hit event and
  consuming exactly one 9mm round;
- [x] advertise and execute the aimed command through the fair MCP action
  catalog, and encode/decode it at replay/MCP JSON boundaries without
  duplicating gameplay policy;
- [x] preserve deterministic replay verification and reject empty Trigun clips
  atomically, while leaving the existing alternate reload/nuke transition
  authoritative;
- [x] keep exact legacy callback state/timing, alternate-target UI semantics,
  controlled runtime, browser capture, audiovisual parity, and presentation
  comparison `NOT_RUN` where evidence is unavailable;
- [x] advance project version from `0.2.247` to `0.2.248` and gameplay
  semantics from `57` to `58` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7ds Historical Nuclear Plasma Rifle ordinary-fire volley target

The bounded implementation target for this revision is the typed Nuclear
Plasma Rifle ordinary-fire volley exercised through direct core,
ScenarioRunner/replay, MCP JSON and action catalog, and `BrowserSession`. Its
contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with a Nuclear Plasma Rifle's 24-cell clip, six reserve cells, and a high-HP
  target;
- [x] submit one ordinary `Command::AttackRanged` through both paths and
  preserve identical `GameEvent` sequences, fair player observations, render
  effects, and scene projections;
- [x] emit six ordered ranged-hit events, consume one cell per projectile (six
  cells total), and pay the ordinary ranged action cost while resetting the
  typed recharge timer;
- [x] advertise and execute the ordinary command through the fair MCP action
  catalog, and encode/decode it at replay/MCP JSON boundaries without
  duplicating gameplay policy;
- [x] preserve deterministic ScenarioRunner/replay verification and reject
  clips below six cells atomically, while leaving alternate overload and
  periodic recharge transitions authoritative;
- [x] keep chainfire callback state, spread/falloff, exact callback timing,
  controlled runtime, browser capture, and audiovisual parity `NOT_RUN` where
  evidence is unavailable;
- [x] advance project version from `0.2.248` to `0.2.249` and gameplay
  semantics from `58` to `59` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7dt Historical Anti-Freak Jackal aimed-fire vertical fidelity target

The bounded implementation target for this revision is the typed Anti-Freak
Jackal aimed-fire command exercised through direct core, replay/MCP JSON,
MCP action catalog, and `BrowserSession`. Its contract must:

- [x] construct the same fixed replay setup in direct core and `BrowserSession`
  with an Anti-Freak Jackal's six-round clip, six reserve 9mm rounds, and a
  high-HP target;
- [x] submit `Command::AttackRangedAimed` through both paths and preserve
  identical `GameEvent` sequences, fair player observations, render effects,
  and scene projections;
- [x] apply the shared +3 accuracy bonus and doubled fire cost, paying
  `ActionCost(2_000)`, while emitting one ordered ranged-hit event and
  consuming exactly one 9mm round;
- [x] advertise and execute aimed fire through the fair MCP action catalog,
  and encode/decode it at replay/MCP JSON boundaries without duplicating
  gameplay policy;
- [x] preserve deterministic replay verification and reject empty Anti-Freak
  Jackal clips atomically, while leaving the delayed explosion callback
  outside this contract;
- [x] keep legacy explosion delay/radius presentation, callback state/timing,
  controlled runtime, browser capture, and audiovisual parity `NOT_RUN` where
  evidence is unavailable;
- [x] advance project version from `0.2.249` to `0.2.250` and gameplay
  semantics from `59` to `60` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7du Historical Anti-Freak Jackal delayed-explosion schedule target

The bounded implementation target for this revision is the typed Anti-Freak
Jackal on-hit explosion schedule exercised through direct core, replay/MCP JSON,
MCP event projection, and `BrowserSession`. Its contract must:

- [x] preserve the existing aimed-fire setup and replay identity while
  resolving a deterministic six-command sequence with at least one hit;
- [x] emit a typed `AntiFreakJackalExplosionScheduled` event after each
  successful damage application, carrying delay `40`, radius `1`, and default
  knockback `8`;
- [x] preserve identical event ordering and player-visible observations,
  render effects, scene projections, and final state between direct core and
  `BrowserSession`;
- [x] project the schedule event through MCP JSON with stable type and numeric
  fields without adding a second gameplay implementation;
- [x] preserve deterministic replay verification and transactional command
  behavior while leaving splash geometry, damage fanout, and callback state
  outside this contract;
- [x] keep controlled runtime, browser capture, and audiovisual parity
  `NOT_RUN` where evidence is unavailable;
- [x] advance project version from `0.2.250` to `0.2.251` and gameplay
  semantics from `60` to `61` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7dv Historical Anti-Freak Jackal radius-1 splash fanout target

The bounded implementation target for this revision is the typed Anti-Freak
Jackal splash fanout after its existing successful-hit schedule. Its contract
must:

- [x] preserve the existing six-command aimed-fire setup, schedule event, and
  replay identity while resolving a deterministic radius-1 blast;
- [x] consider the impact center followed by the eight neighboring cells
  clockwise from north in stable order, clamp at map edges, and exclude farther
  cells;
- [x] apply one deterministic fire-damage result per eligible living actor,
  including the impact target once, while preserving existing damage-event and
  observation boundaries;
- [x] preserve identical event ordering, fair player observations, render
  effects, scene projections, and final state between direct core and
  `BrowserSession`;
- [x] project the resulting damage through the existing generic MCP JSON
  `DamageApplied` boundary (covered by the environment-damage serializer
  contract) and preserve deterministic replay verification without adding a
  generic explosion registry or delayed command queue;
- [x] keep blast knockback, terrain/item destruction, callback state, exact
  legacy timing, controlled runtime, browser capture, and audiovisual parity
  `NOT_RUN` where evidence is unavailable;
- [x] advance project version from `0.2.251` to `0.2.252` and gameplay
  semantics from `61` to `62` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7dw Historical Anti-Freak Jackal splash knockback target

The bounded implementation target for this revision extends the delivered
radius-1 splash with typed radial knockback. Its contract must:

- [x] preserve the delay-40/radius-1 schedule, center-plus-eight-neighbor
  order, and one deterministic `5d3` roll per eligible living actor;
- [x] derive integer displacement as `damage / 8`, omit center-actor movement,
  and push each non-center actor away from the impact center along its radial
  direction;
- [x] stop at map edges, terrain, or living-actor occupancy, emit one
  `ActorKnockedBack` event only when a position changes, and order it before the
  corresponding environmental `DamageApplied` event;
- [x] preserve direct-core, replay, and `BrowserSession` event, observation,
  effect, scene, and final-state parity with deterministic replay, while the
  existing generic MCP JSON serializer covers the typed schedule, knockback,
  and damage events;
- [x] keep terrain/item destruction, spread/falloff, callback state, exact
  legacy timing, controlled runtime, browser capture, and audiovisual parity
  `NOT_RUN` where evidence is unavailable;
- [x] advance project version from `0.2.252` to `0.2.253` and gameplay
  semantics from `62` to `63` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7dx Historical Anti-Freak Jackal ground-item destruction target

The bounded implementation target for this revision extends the delivered
radius-1 splash and knockback with typed destruction of representable ground
items. Its contract must:

- [x] preserve the delay-40/radius-1 schedule, center-plus-eight-neighbor
  order, radial knockback, and one deterministic `5d3` roll per blast cell;
- [x] remove the lowest-ID ordinary loose-ammo stack at a blast cell only when
  that cell's rolled damage is strictly greater than `10`, while preserving
  non-ammunition items, prepared ammo packs, and cells without items;
- [x] emit one `GroundItemDestroyed` event after the corresponding damage
  resolution and before any lethal actor's death/drop follow-up;
- [x] preserve direct-core, replay, and `BrowserSession` event, observation,
  effect, scene, and final-state parity with deterministic replay, while the
  existing generic MCP JSON serializer covers the typed destruction event;
- [x] keep terrain/cell destruction, spread/falloff, callback state, exact
  legacy timing, controlled runtime, browser capture, and audiovisual parity
  `NOT_RUN` where evidence is unavailable;
- [x] advance project version from `0.2.253` to `0.2.254` and gameplay
  semantics from `63` to `64` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7dy Historical Railgun ray/piercing traversal target

The bounded implementation target for this revision extends the delivered
Railgun ordinary-fire cost with clear-ray traversal and typed piercing. Its
contract must:

- [x] traverse `line_points` from the player through the requested target,
  retaining every living non-player actor in source-to-target order while
  rejecting blocked rays atomically;
- [x] perform one ordered hit check per encountered actor and share one
  deterministic `8d8` damage roll across all successful impacts in a
  multi-actor shot;
- [x] continue traversal after lethal intermediate hits, preserving existing
  damage/death/drop event ordering and five-cell clip consumption;
- [x] preserve direct-core, replay, generic MCP JSON, and `BrowserSession`
  event, observation, effect, scene, and final-state parity, while retaining
  the existing single-target RNG order when no intermediate actor exists;
- [x] keep knockback, wall/cell destruction, spread/falloff, stray-shot,
  callback state, exact legacy timing, controlled runtime, browser capture, and
  audiovisual parity `NOT_RUN` where evidence is unavailable;
- [x] advance project version from `0.2.254` to `0.2.255` and gameplay
  semantics from `64` to `65` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7dz Current Null Pointer actor-splash target

The bounded implementation target for this revision extends the delivered
Null Pointer score branch and delayed-explosion schedule with actor-only
radius-1 splash damage. Its contract must:

- [x] preserve the `AttackResolved → NullPointerHit →
  NullPointerExplosionScheduled` ordering, score branch, zero direct-hit
  damage, and ten-cell clip cost;
- [x] traverse the center plus clear radius-1 blast cells in stable order,
  apply fixed `10d1` Plasma environment damage once per living actor with
  actor deduplication, and continue after lethal intermediate actors;
- [x] emit ordinary `DamageApplied` events followed by `ActorDied` and
  configured death-drop events while preserving atomic death-drop preflight;
- [x] preserve direct-core, ScenarioRunner/replay, generic MCP JSON, and
  `BrowserSession` event, observation, effect, scene, and final-state parity;
- [x] keep terrain/cell destruction, ground-item destruction, splash immunity,
  knockback, exact delayed timing, callback state, controlled runtime, browser
  capture, and audiovisual parity `NOT_RUN` where evidence is unavailable;
- [x] advance project version from `0.2.255` to `0.2.256` and gameplay
  semantics from `65` to `66` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7ea Historical Chaingun first-level chainfire delivery target

The bounded implementation target for this revision extends the delivered
Chaingun ordinary-fire profile with its first alternate chainfire level. Its
contract must:

- [x] expose a typed `AttackRangedChainfire(Position)` command and ordered
  `AlternateAction::Chainfire { shot_count: 3, ammo_cost: 3 }` profile fragment;
- [x] accept the command only for a Chaingun at warm-up level `0`, preflight
  three loaded 9mm rounds before clip/RNG mutation, and reject under-supplied
  clips atomically;
- [x] emit exactly three ordered ranged outcomes against the requested target,
  fill post-lethal slots with deterministic no-op misses without extra damage
  or RNG, consume three clip rounds, and advance observable chainfire state only
  after acceptance; ordinary fire resets that state to `0`;
- [x] preserve direct-core, replay, MCP JSON/catalog, browser snapshot, and
  `BrowserSession` event, observation, effect, scene, and final-state parity;
- [x] keep higher chainfire levels, legacy target rotation/spread, exact
  callback timing/accuracy, controlled runtime, browser capture, and
  audiovisual parity `NOT_RUN` where evidence is unavailable;
- [x] advance project version from `0.2.256` to `0.2.257` and gameplay
  semantics from `66` to `68` while preserving replay schema, RNG, generator,
  and ruleset identities.

### 2.7eb Historical Minigun first-level chainfire profile target

The bounded implementation target for this revision is an immutable profile
for the pinned Minigun first-level alternate chainfire. Its contract must:

- [x] preserve the ordered ordinary `ProjectileCount(8)` and one-round 9mm
  cost fragments;
- [x] add ordered `AlternateAction::Chainfire { shot_count: 6, ammo_cost: 6 }`
  fragments derived from the pinned first-level `Shots div 3` adjustment;
- [x] assert exact profile declaration order without adding a command,
  dispatcher, callback registry, replay-wire field, or gameplay-state mutation;
- [x] keep chainfire execution, higher levels, target rotation/spread, exact
  legacy timing/accuracy, controlled runtime, browser capture, and audiovisual
  parity `NOT_RUN` where comparison evidence is unavailable;
- [x] advance project version from `0.2.257` to `0.2.258` while preserving
  gameplay semantics `68`, replay schema, RNG, generator, and ruleset
  identities.

### 2.7ec Historical Minigun first-level chainfire execution target

The bounded implementation target for this revision extends the existing typed
Chaingun chainfire command to Minigun. Its contract must:

- [x] accept `AttackRangedChainfire` only for a Minigun at warm-up level `0`
  with at least six loaded 9mm rounds, after validating target, LOS, range, and
  death-drop destinations;
- [x] emit exactly six ordered ranged outcomes against the requested target,
  fill post-lethal slots with deterministic no-op misses without extra damage or
  RNG, consume six clip rounds, and advance warm-up state only after acceptance;
- [x] reset the shared warm-up state on ordinary fire and reject higher levels,
  wrong weapons, and under-supplied clips atomically;
- [x] advertise and execute the same semantic action through replay/MCP JSON,
  fair legal-action probing, physical browser `C` routing, and `BrowserSession`
  parity tests;
- [x] advance project version from `0.2.259` to `0.2.260` and gameplay
  semantics from `68` to `69` while preserving replay schema, RNG, generator,
  and ruleset identities;
- [x] keep higher chainfire levels, target rotation/spread, exact legacy
  timing/accuracy, controlled runtime, browser capture, and audiovisual parity
  `NOT_RUN` where comparison evidence is unavailable.

### 2.7ed Historical Plasma Rifle first-level chainfire execution target

The bounded implementation target for this revision extends the existing typed
chainfire command to the standard Plasma Rifle. Its contract must:

- [x] accept `AttackRangedChainfire` only for the standard Plasma Rifle at
  warm-up level `0` with at least four loaded cells, after validating target,
  LOS, range, and death-drop destinations;
- [x] emit exactly four ordered ranged outcomes against the requested target,
  fill post-lethal slots with deterministic no-op misses without extra damage or
  RNG, consume four clip cells, and advance warm-up state only after acceptance;
- [x] reset the shared warm-up state on ordinary fire and reject higher levels,
  wrong weapons, and under-supplied clips atomically;
- [x] advertise and execute the same semantic action through replay/MCP JSON,
  fair legal-action probing, physical browser `C` routing, and `BrowserSession`
  parity tests;
- [x] advance project version from `0.2.260` to `0.2.261` and gameplay
  semantics from `69` to `70` while preserving replay schema, RNG, generator,
  and ruleset identities;
- [x] keep higher chainfire levels, Nuclear Plasma overload/recharge changes,
  target rotation/spread, exact legacy timing/accuracy, controlled runtime,
  browser capture, and audiovisual parity `NOT_RUN` where comparison evidence
  is unavailable.

### 2.7ee Historical Laser Rifle first-level chainfire execution target

The bounded implementation target for this revision extends the existing typed
chainfire command to the Laser Rifle. Its contract must:

- [x] accept `AttackRangedChainfire` only for the Laser Rifle at warm-up level
  `0` with at least four loaded cells, after validating target, LOS, range, and
  death-drop destinations;
- [x] emit exactly four ordered ranged outcomes against the requested target,
  fill post-lethal slots with deterministic no-op misses without extra damage or
  RNG, consume four clip cells, and advance warm-up state only after acceptance;
- [x] reset the shared warm-up state on ordinary fire and reject higher levels,
  wrong weapons, and under-supplied clips atomically;
- [x] advertise and execute the same semantic action through replay/MCP JSON,
  fair legal-action probing, physical browser `C` routing, and `BrowserSession`
  parity tests;
- [x] advance project version from `0.2.261` to `0.2.262` and gameplay
  semantics from `70` to `71` while preserving replay schema, RNG, generator,
  and ruleset identities;
- [x] keep higher chainfire levels, target rotation/spread, exact legacy
  timing/accuracy, controlled runtime, browser capture, and audiovisual parity
  `NOT_RUN` where comparison evidence is unavailable.

### 2.7ef Historical Nuclear Plasma first-level chainfire execution target

The bounded implementation target for this revision extends the existing typed
chainfire command to the Nuclear Plasma Rifle. Its contract must:

- [x] accept `AttackRangedChainfire` only for the Nuclear Plasma Rifle at
  warm-up level `0` with at least four loaded cells, after validating target,
  LOS, range, and death-drop destinations;
- [x] emit exactly four ordered ranged outcomes against the requested target,
  fill post-lethal slots with deterministic no-op misses without extra damage or
  RNG, consume four clip cells, and advance warm-up state only after acceptance;
- [x] reset the shared warm-up state on ordinary fire and reject higher levels,
  wrong weapons, and under-supplied clips atomically;
- [x] advertise and execute the same semantic action through replay/MCP JSON,
  fair legal-action probing, physical browser `C` routing, and `BrowserSession`
  parity tests without changing overload or recharge semantics;
- [x] advance project version from `0.2.262` to `0.2.263` and gameplay
  semantics from `71` to `72` while preserving replay schema, RNG, generator,
  and ruleset identities;
- [x] keep higher chainfire levels, overload/NukeRun map effects, recharge
  cadence changes, target rotation/spread, exact legacy timing/accuracy,
  controlled runtime, browser capture, and audiovisual parity `NOT_RUN` where
  comparison evidence is unavailable.

### 2.7eg Historical BFG 10K first-level chainfire execution target

The bounded implementation target for this revision extends the existing typed
chainfire command to the BFG 10K. Its contract must:

- [x] accept `AttackRangedChainfire` only for the BFG 10K at warm-up level `0`
  with at least twenty loaded cells, after validating target, LOS, range, and
  death-drop destinations;
- [x] emit exactly four ordered exact-hit outcomes against the requested
  target, fill post-lethal slots with deterministic no-op misses without extra
  damage or RNG, consume twenty clip cells, preserve each successful hit's
  delayed-explosion schedule metadata, and advance warm-up only after
  acceptance;
- [x] reset shared warm-up state on ordinary fire and reject higher levels,
  wrong weapons, and under-supplied clips atomically;
- [x] advertise and execute the same semantic action through replay/MCP JSON,
  fair legal-action probing, physical browser `C` routing, and `BrowserSession`
  parity tests;
- [x] advance project version from `0.2.263` to `0.2.264` and gameplay
  semantics from `72` to `73` while preserving replay schema, RNG, generator,
  and ruleset identities;
- [x] keep higher chainfire levels, scatter/target rotation or spread,
  projectile routing, delayed explosion geometry/damage/knockback, exact legacy
  timing/accuracy, controlled runtime, browser capture, and audiovisual parity
  `NOT_RUN` where comparison evidence is unavailable.

### 2.7eh Historical BFG 10K radius-2 explosion fanout target

The bounded implementation target for this revision extends the existing typed
BFG 10K schedule boundary with immediate deterministic actor-only fanout. Its
contract must:

- [x] resolve each successful direct-target BFG 10K hit over the in-bounds,
  line-of-sight-cleared radius-2 blast cells in a documented stable order;
- [x] roll one `6d4` Plasma environment result per blast cell without distance
  falloff, apply the damage once to each living actor per cell, and preserve
  actor de-duplication when a later geometry extension revisits a cell;
- [x] apply bounded radial knockback before environmental damage using the
  pinned integer `damage / 16` ratio, with blocked and center destinations
  remaining in place;
- [x] emit existing `ActorKnockedBack`, `DamageApplied`, `ActorDied`, and
  `ItemDropped` events in deterministic order, mark the game over when the
  player is killed, and preserve the direct hit's existing schedule event;
- [x] preserve atomic death-drop preflight, RNG determinism, replay
  determinism, and direct-core/MCP/BrowserSession parity for ordinary and
  first-level chainfire BFG 10K hits;
- [x] advance project version from `0.2.264` to `0.2.265` and gameplay
  semantics from `73` to `74` while preserving replay schema, RNG, generator,
  and ruleset identities;
- [x] keep delayed timing/state-machine parity, higher chainfire levels,
  scatter/target routing, terrain/content mutation, ground-item destruction,
  splash-immunity traits, exact callback timing/accuracy, controlled runtime,
  browser capture, and audiovisual parity `NOT_RUN` where comparison evidence
  is unavailable.

### 2.7ei Historical BFG 10K ground-item destruction target

The bounded implementation target for this revision extends the delivered
radius-2 actor fanout with the legacy-pinned ordinary loose-ammo destruction
rule. Its contract must:

- [x] preserve the existing schedule, radius geometry, per-cell `6d4` Plasma
  rolls, actor de-duplication, radial knockback, and death/drop ordering;
- [x] after processing any actor on each clear blast cell, destroy at most one
  lowest-ID ordinary loose-ammo stack when that cell's rolled damage is greater
  than `10`; cells without actors still apply this ground-item rule;
- [x] emit `GroundItemDestroyed { item_id, position }` immediately after any
  actor damage on that cell (or after cell processing when no actor is present)
  and before that victim's lethal death/drop follow-up, without destroying ammo
  packs, equipped/inventory items, or non-ammunition ground items;
- [x] preserve one RNG roll per clear blast cell, deterministic ordinary and
  first-level chainfire behavior, replay determinism, and direct-core/MCP/
  BrowserSession event/state parity;
- [x] preserve atomic death-drop preflight and rejected-command state identity,
  including ground-item state and RNG, for all newly reachable failures;
- [x] advance project version from `0.2.265` to `0.2.266` and gameplay
  semantics from `74` to `75` while preserving replay schema, RNG, generator,
  and ruleset identities;
- [x] keep delayed timing/state-machine parity, higher chainfire levels,
  scatter/target routing, terrain/content mutation, non-ammunition item
  destruction, splash-immunity traits, exact callback timing/accuracy,
  controlled runtime, browser capture, and audiovisual parity `NOT_RUN` where
  comparison evidence is unavailable.

### 2.7ej Historical Standard BFG 9000 radius-8 actor-fanout target

The bounded implementation target for this revision extends the delivered
Standard BFG 9000 schedule boundary with immediate deterministic actor-only
radius-8 fanout. Its contract must:

- [x] preserve the existing direct-hit, delay-33/radius-8/knockback-16 schedule
  event and one accepted shot's clip/action-cost behavior;
- [x] resolve the in-bounds, line-of-sight-cleared radius-8 blast cells in a
  documented stable center-then-ring order, consuming one `10d6` Plasma roll
  per clear cell without distance falloff;
- [x] skip the firing actor as required by the legacy `EFSELFSAFE` flag, process
  each other living actor once, apply radial integer `damage / 16` knockback
  before environmental damage, and preserve lethal death/drop/game-over order;
- [x] preserve atomic death-drop preflight and rejected-command state identity,
  one-roll-per-cell RNG determinism, replay determinism, and direct-core/MCP/
  BrowserSession event/state parity;
- [x] advance project version from `0.2.266` to `0.2.267` and gameplay
  semantics from `75` to `76` while preserving replay schema, RNG, generator,
  and ruleset identities;
- [x] keep Nuclear BFG 9000 fanout, EFCHAIN secondary explosions, higher
  chainfire levels, scatter/target routing, delayed timing/state-machine parity,
  terrain/content and ground-item effects, splash-immunity traits, exact
  callback timing/accuracy, controlled runtime, browser capture, and
  audiovisual parity `NOT_RUN` where comparison evidence is unavailable.

### 2.7ek Current Nuclear BFG 9000 radius-8 actor-fanout target

The bounded implementation target for this revision extends the delivered
Nuclear BFG 9000 schedule boundary with immediate deterministic actor-only
radius-8 fanout. Its contract must:

- [x] preserve the existing direct-hit, delay-33/radius-8/knockback-16
  schedule event and one accepted shot's clip/action-cost behavior;
- [x] resolve the in-bounds, line-of-sight-cleared radius-8 blast cells in a
  documented stable center-then-ring order, consuming one `8d6` Plasma roll
  per clear cell without distance falloff;
- [x] skip the firing actor as required by the legacy `EFSELFSAFE` flag,
  process each other living actor once, apply radial integer `damage / 16`
  knockback before environmental damage, and preserve lethal death/drop/
  game-over order;
- [x] preserve atomic death-drop preflight and rejected-command state identity,
  one-roll-per-cell RNG determinism, replay determinism, and direct-core/MCP/
  BrowserSession event/state parity;
- [x] advance project version from `0.2.267` to `0.2.268` and gameplay
  semantics from `76` to `77` while preserving replay schema, RNG, generator,
  and ruleset identities;
- [x] keep EFCHAIN secondary explosions, higher chainfire levels,
  scatter/target routing, delayed timing/state-machine parity, terrain/content
  and ground-item effects, splash-immunity traits, exact callback timing/
  accuracy, controlled runtime, browser capture, and audiovisual parity
  `NOT_RUN` where comparison evidence is unavailable.

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

The `0.2.200` successor carries the pinned BFG 10K `shots=5` policy through the
existing direct-target ranged path. One accepted command now resolves five
ordered exact-hit damage rolls and charges five cells per projectile (twenty-
five cells from the full clip), with deterministic ScenarioRunner/replay, MCP,
and BrowserSession/direct-core parity. Scatter, projectile routing, explosions,
chainfire, runtime, and audiovisual parity remain open.

The `0.2.201` revision recorded the pinned BFG 10K delayed explosion payload
after each direct-target hit as an ordered `Bfg10kExplosionScheduled` event
(delay `25`, radius `2`, knockback `16`). Explosion geometry, splash damage,
knockback application, projectile routing, runtime, and audiovisual parity
remain open.

The `0.2.202` revision recorded the pinned standard BFG 9000 delayed explosion
payload after its direct-target hit as a `Bfg9000ExplosionScheduled` event
(delay `33`, radius `8`, knockback `16`). Explosion geometry, splash damage,
knockback application, projectile routing, alternate overload, runtime, and
audiovisual parity remain open.

The `0.2.203` revision recorded the pinned Nuclear BFG 9000 delayed explosion
payload after its direct-target hit as a `NuclearBfg9000ExplosionScheduled`
event (delay `33`, radius `8`, knockback `16`). Recharge timing, alternate
overload, NukeRun, explosion geometry, splash damage, knockback application,
projectile routing, runtime, and audiovisual parity remain open.

The `0.2.204` revision recorded the immutable `NUCLEAR_PLASMA_BEHAVIOR`
profile with ordered alternate-overload and delay-40/cadence-2/amount-1
recharge fragments. Dedicated transition modules remain authoritative;
chainfire, runtime, and audiovisual parity remain open.

The `0.2.205` revision recorded the immutable `BLASTER_BEHAVIOR` profile with
the delay-30/cadence-10/amount-1 recharge fragment. The dedicated transition
remains authoritative; aimed fire, runtime, and audiovisual parity remain open.

The `0.2.206` successor records the immutable `MALEK_ARMOR_BEHAVIOR` profile
with the delay-50/cadence-5/amount-1 durability-recharge fragment. The
dedicated transition remains authoritative; armor resistance/degradation,
runtime, and audiovisual parity remain open.

The `0.2.207` successor records the immutable `LAVA_ARMOR_BEHAVIOR` profile
with the TileKind::Lava-gated interval-5/amount-3 durability-recharge
fragment. The dedicated transition remains authoritative; hazard damage,
armor resistance, runtime, and audiovisual parity remain open.

The `0.2.208` successor records the immutable `JACKHAMMER_BEHAVIOR` profile
with ordered Single/Burst mode fragments and the one-point score-count cost.
The dedicated transition remains authoritative; spread/falloff, exact timing,
runtime, and audiovisual parity remain open.

The `0.2.209` successor records the immutable `GRAMMATON_BEHAVIOR` profile
with ordered Single/Burst/Auto mode fragments and the 200-point score-count
cost. The dedicated transition remains authoritative; legacy accuracy
equations, exact timing, runtime, and audiovisual parity remain open.

The `0.2.210` successor records the immutable `ACID_SPITTER_BEHAVIOR` profile
with the Acid-to-Water terrain reload and one-round amount plus the 1,000-point
score-count cost. The dedicated transition remains authoritative; hazard
damage, resistance, runtime, and audiovisual parity remain open.

The `0.2.211` successor records the immutable `MISSILE_LAUNCHER_BEHAVIOR`
profile with ordinary single-rocket reload and capped full-deficit reload
fragments. Dedicated reload/planner paths remain authoritative; rocket-jump,
explosion, runtime, and audiovisual parity remain open.

The `0.2.212` successor records the immutable `COMBAT_SHOTGUN_BEHAVIOR`
profile with ordinary single-shell reload and capped full-deficit reload
fragments. Dedicated reload/planner and pump-action paths remain authoritative;
exact legacy timing, runtime, chamber presentation, and audiovisual parity
remain open.

The `0.2.213` successor records the immutable `REVENANTS_LAUNCHER_BEHAVIOR`
profile with its pinned exact-hit attack fragment. Dedicated combat execution
remains authoritative; homing, projectile routing, delayed explosions, runtime,
and audiovisual parity remain open.

The `0.2.214` successor records the immutable `ASSAULT_SHOTGUN_BEHAVIOR`
profile with ordinary single-shell reload and capped full-deficit reload
fragments. Dedicated reload/planner paths remain authoritative; exact legacy
timing, partial-reserve policy, runtime, and audiovisual parity remain open.

The `0.2.215` successor extends `COMBAT_SHOTGUN_BEHAVIOR` with the typed
pump-only chamber action at cost `200`, while retaining dedicated chamber and
reload execution. Exact legacy timing, partial-reserve policy, chamber
presentation, runtime, and audiovisual parity remain open.

The `0.2.216` successor records the immutable `DOUBLE_SHOTGUN_BEHAVIOR`
profile and deterministic two-projectile/two-shell ranged fire. Spread/falloff,
exact timing, controlled legacy runtime, and audiovisual parity remain open.

The `0.2.217` successor records the immutable `SHOTGUN_BEHAVIOR` profile for
the delivered one-cell knockback hit and one-shell cost. Exact legacy force and
timing, spread/falloff, controlled runtime, and audiovisual parity remain open.

The `0.2.218` successor records the immutable `PISTOL_BEHAVIOR` profile for
the delivered one-projectile ordinary fire and one-9mm-round cost. Aimed-fire
callback semantics, exact legacy timing/accuracy, controlled runtime, and
audiovisual parity remain open.

The `0.2.219` successor records the immutable `ROCKET_LAUNCHER_BEHAVIOR`
profile for the delivered one-projectile ordinary fire and one-rocket cost.
Rocket-jump/explosion callback semantics, exact legacy timing/accuracy,
controlled runtime, and audiovisual parity remain open.

The `0.2.220` successor records the immutable `COMBAT_PISTOL_BEHAVIOR`
profile for the delivered one-projectile ordinary fire and one-9mm-round cost.
Aimed-fire callback semantics, exact legacy timing/accuracy, controlled
runtime, and audiovisual parity remain open.

The `0.2.221` successor records the immutable `PLASMA_SHOTGUN_BEHAVIOR`
profile for the delivered one-projectile ordinary fire and three-cell clip
cost. Generic ranged execution now enforces the cost before mutation; full
spread/falloff/knockback semantics, exact legacy timing/accuracy, controlled
runtime, and audiovisual parity remain open.

The `0.2.222` successor records the immutable `FRAG_SHOTGUN_BEHAVIOR` profile
for the delivered one-projectile ordinary fire and two-round 9mm clip cost.
Generic ranged execution now enforces the cost before mutation; full
spread/falloff/knockback semantics, exact legacy timing/accuracy, controlled
runtime, and audiovisual parity remain open.

The `0.2.223` successor records the immutable `RAILGUN_BEHAVIOR` profile for
the delivered one-projectile ordinary fire and five-cell clip cost. Generic
ranged execution now enforces the cost before mutation; ray/piercing routing,
spread/falloff semantics, exact legacy timing/accuracy, controlled runtime, and
audiovisual parity remain open.

The `0.2.224` successor records the immutable `NULL_POINTER_BEHAVIOR` profile
for the delivered one-projectile ordinary fire and ten-cell clip cost. Generic
ranged execution now enforces the cost before mutation while retaining the
typed target-score branch and deferred explosion schedule; delayed explosion
geometry, full callback parity, controlled runtime, and audiovisual parity
remain open.

The `0.2.225` successor records the immutable `TRISTAR_BLASTER_BEHAVIOR`
profile for the delivered three-projectile ordinary fire and five-cell
per-projectile cost. Generic ranged execution now resolves the ordered volley
and enforces its fifteen-cell total cost before mutation; spread routing,
delayed explosion geometry, callback parity, controlled runtime, and
audiovisual parity remain open.

The `0.2.226` successor records the immutable `ACID_SPITTER_BEHAVIOR` profile
for the delivered one-projectile ordinary fire and ten-rocket clip cost.
Generic ranged execution now enforces the cost before mutation while retaining
the dedicated Acid-to-Water reload transition; explosion geometry/content,
spread/falloff, controlled runtime, and audiovisual parity remain open.

The `0.2.227` successor records the immutable `MEGA_BUSTER_BEHAVIOR` profile
for the delivered three-projectile ordinary fire and three-round
per-projectile cost. Generic ranged execution now resolves the ordered volley
and enforces its nine-round total cost before mutation; the kill callback,
spread/falloff, controlled runtime, and audiovisual parity remain open.

The `0.2.228` successor records the immutable `SUPER_SHOTGUN_BEHAVIOR` profile
for the delivered two-projectile ordinary fire and one-shell per-projectile
cost. Generic ranged execution now resolves the ordered volley and enforces
its two-shell aggregate cost before mutation; spread/falloff, exact timing,
controlled runtime, and audiovisual parity remain open.

The `0.2.229` successor records the immutable `MINIGUN_BEHAVIOR` profile for
the delivered eight-projectile ordinary fire and one-round per-projectile cost.
Generic ranged execution now resolves the ordered volley and enforces its
eight-round aggregate cost before mutation; alternate chainfire, spread/falloff,
exact timing, controlled runtime, and audiovisual parity remain open.

The `0.2.230` successor records the immutable `CHAINGUN_BEHAVIOR` profile for
the delivered four-projectile ordinary fire and one-round per-projectile cost.
Generic ranged execution now resolves the ordered volley and enforces its
four-round aggregate cost before mutation; alternate chainfire, spread/falloff,
exact timing, controlled runtime, and audiovisual parity remain open.

The `0.2.231` successor records the immutable `LASER_RIFLE_BEHAVIOR` profile
for the delivered five-projectile ordinary fire and one-cell per-projectile
cost. Generic ranged execution now resolves the ordered volley and enforces its
five-cell aggregate cost before mutation; alternate chainfire, spread/falloff,
exact timing, controlled runtime, and audiovisual parity remain open.

The `0.2.232` successor verifies the delivered Laser Rifle ordinary-fire
contract through the direct-core and `BrowserSession` boundaries. A fixed
five-projectile command now reproduces events, fair observations, render
effects, scene state, and replay determinism across both paths; controlled
legacy runtime, browser capture, exact timing/accuracy, alternate chainfire,
and audiovisual parity remain open.

The `0.2.233` successor verifies the delivered Minigun ordinary-fire contract
through the direct-core and `BrowserSession` boundaries. A fixed eight-projectile
command now reproduces events, fair observations, render effects, scene state,
and replay determinism across both paths; controlled legacy runtime, browser
capture, exact timing/accuracy, alternate chainfire, and audiovisual parity
remain open.

The `0.2.234` successor verifies the delivered Chaingun ordinary-fire contract
through the direct-core and `BrowserSession` boundaries. A fixed four-projectile
command now reproduces events, fair observations, render effects, scene state,
and replay determinism across both paths; controlled legacy runtime, browser
capture, exact timing/accuracy, alternate chainfire, and audiovisual parity
remain open.

The `0.2.235` successor verifies the delivered Mega Buster ordinary-fire
contract through the direct-core and `BrowserSession` boundaries. A fixed
three-projectile command now reproduces events, fair observations, render
effects, scene state, and replay determinism across both paths; controlled
legacy runtime, browser capture, exact timing/accuracy, kill callback, and
audiovisual parity remain open.

The `0.2.236` successor verifies the delivered Super Shotgun ordinary-fire
contract through the direct-core and `BrowserSession` boundaries. A fixed
two-projectile command now reproduces events, fair observations, render
effects, scene state, and replay determinism across both paths; controlled
legacy runtime, browser capture, exact timing/accuracy, spread/falloff, and
audiovisual parity remain open.

The `0.2.237` successor verifies the delivered Tristar Blaster ordinary-fire
contract through the direct-core and `BrowserSession` boundaries. A fixed
three-projectile command now reproduces events, fair observations, render
effects, scene state, and replay determinism across both paths; controlled
legacy runtime, browser capture, exact timing/accuracy, spread/falloff, delayed
effects, and audiovisual parity remain open.

The `0.2.238` successor verifies the delivered Railgun ordinary-fire contract
through the direct-core and `BrowserSession` boundaries. A fixed one-projectile
command now reproduces events, fair observations, render effects, scene state,
and replay determinism across both paths; controlled legacy runtime, browser
capture, exact timing/accuracy, piercing, spread/falloff, and audiovisual parity
remain open.

The `0.2.239` successor verifies the delivered Frag Shotgun ordinary-fire
contract through the direct-core and `BrowserSession` boundaries. A fixed
one-projectile command now reproduces events, fair observations, render effects,
scene state, and replay determinism across both paths; controlled legacy
runtime, browser capture, exact timing/accuracy, spread/falloff, knockback, and
audiovisual parity remain open.

The `0.2.240` successor verifies the delivered Combat Pistol ordinary-fire
contract through the direct-core and `BrowserSession` boundaries. A fixed
one-projectile command now reproduces events, fair observations, render effects,
scene state, and replay determinism across both paths; controlled legacy
runtime, browser capture, aimed callback, exact timing/accuracy, and audiovisual
parity remain open.

The `0.2.241` successor verifies the delivered Pistol ordinary-fire contract
through the direct-core and `BrowserSession` boundaries. A fixed one-projectile
command now reproduces events, fair observations, render effects, scene state,
and replay determinism across both paths; controlled legacy runtime, browser
capture, aimed behavior, exact timing/accuracy, and audiovisual parity remain
open.

The `0.2.242` successor verifies the delivered Plasma Shotgun ordinary-fire
contract through the direct-core and `BrowserSession` boundaries. A fixed
one-projectile command now reproduces events, fair observations, render
effects, scene state, three-cell clip consumption, and replay determinism
across both paths; controlled legacy runtime, browser capture, spread/falloff,
knockback, exact timing/accuracy, and audiovisual parity remain open.

The `0.2.243` successor verifies the delivered Blaster ordinary-fire contract
through the direct-core and `BrowserSession` boundaries. A fixed one-projectile
command now reproduces events, fair observations, render effects, scene state,
one-cell clip consumption, and replay determinism across both paths; controlled
legacy runtime, browser capture, aimed callback, exact timing/accuracy, and
audiovisual parity remain open.

The `0.2.244` successor verifies the delivered Pistol aimed-fire contract
through typed direct-core, replay/MCP JSON, and `BrowserSession` boundaries. A
fixed aimed command now applies +3 accuracy, pays doubled action cost, consumes
one 9mm round, reproduces events/observations/effects/scene state, and remains
replay-deterministic across both paths; exact legacy callback state/timing,
controlled runtime, browser capture, and audiovisual parity remain open.

The `0.2.245` successor extends that typed aimed-fire contract to Combat Pistol
through direct-core, replay/MCP JSON/catalog, and `BrowserSession` boundaries.
A fixed aimed command now applies +3 accuracy, pays doubled action cost,
consumes one 9mm round, reproduces events/observations/effects/scene state,
and remains replay-deterministic; exact legacy callback state/timing,
controlled runtime, browser capture, and audiovisual parity remain open.

The `0.2.246` successor extends that shared typed aimed-fire contract to Blaster
through direct-core, replay/MCP JSON/catalog, and `BrowserSession` boundaries.
A fixed aimed command now applies +3 accuracy, pays doubled action cost,
consumes one cell, resets the typed recharge timer, reproduces
events/observations/effects/scene state, and remains replay-deterministic;
exact legacy callback state/timing, controlled runtime, browser capture, and
audiovisual parity remain open.

The `0.2.247` successor extends the typed ordinary-fire contract to the Plasma
Rifle through direct-core, replay/MCP JSON/catalog, and `BrowserSession`
boundaries. A fixed ordinary command now resolves six ordered projectiles,
consumes six cells as an aggregate cost, rejects below-six clips atomically,
and remains replay-deterministic; chainfire, overcharge, exact callback timing,
controlled runtime, browser capture, and audiovisual parity remain open.

The `0.2.248` successor extends the typed aimed-fire contract to Trigun through
direct-core, replay/MCP JSON/catalog, and `BrowserSession` boundaries. A fixed
aimed command now applies +3 accuracy, pays doubled action cost, consumes one
9mm round, reproduces events/observations/effects/scene state, and remains
replay-deterministic; exact callback state/timing, alternate-target UI,
controlled runtime, browser capture, and audiovisual parity remain open.

The `0.2.249` successor extends the typed ordinary-fire volley contract to
Nuclear Plasma Rifle through direct-core, ScenarioRunner/replay, replay/MCP
JSON/catalog, and `BrowserSession` boundaries. A fixed ordinary command now
resolves six ordered projectiles, consumes six cells as an aggregate cost,
resets its typed recharge timer, rejects below-six clips atomically, and remains
replay-deterministic; chainfire callback state, exact callback timing,
controlled runtime, browser capture, and audiovisual parity remain open.

The `0.2.250` successor extends the shared typed aimed-fire contract to the
Anti-Freak Jackal through direct-core, replay/MCP JSON/catalog, and
`BrowserSession` boundaries. A fixed aimed command now applies +3 accuracy,
pays doubled action cost, consumes one 9mm round, reproduces
events/observations/effects/scene state, and remains replay-deterministic;
legacy delayed-explosion callback state/timing, controlled runtime, browser
capture, and audiovisual parity remain open.

The `0.2.251` successor extends the Anti-Freak Jackal contract with a typed
delayed-explosion schedule event. Successful hits now project delay `40`,
radius `1`, and default knockback `8` through direct core, MCP JSON, and
`BrowserSession` parity tests; splash geometry, damage fanout, callback state,
controlled runtime, browser capture, and audiovisual parity remain open.

The `0.2.252` successor extends that schedule with a bounded radius-1 splash
fanout. The typed resolver considers the impact center and eight neighboring
cells clockwise from north, applies one deterministic fire-damage result per
eligible actor, and reproduces direct-core, replay/MCP, and `BrowserSession`
events, observations, effects, and scene state. Blast knockback,
terrain/item destruction, callback state, controlled runtime, browser capture,
and audiovisual parity remain open.

The `0.2.253` successor extends that fanout with bounded radial knockback.
Each eligible actor uses the explicit integer `damage / 8` displacement ratio;
non-center actors move along the radial direction until the map, terrain, or a
living actor blocks them, and a successful move is emitted before its damage
event. The existing generic MCP JSON serializer covers the typed schedule,
knockback, and damage events. Terrain/cell destruction, callback state,
controlled runtime, browser capture, and audiovisual parity remain open.

The `0.2.254` successor extends that knockback with bounded ground-item
destruction. Each blast cell rolls deterministic `5d3` damage; when the roll is
strictly greater than `10`, the lowest-ID ordinary loose-ammo stack at that cell
is removed after actor damage and emits `GroundItemDestroyed`. Terrain/cell
destruction, callback state, controlled runtime, browser capture, and
audiovisual parity remain open.

The `0.2.255` successor extends the Railgun ordinary-fire contract with bounded
clear-ray piercing. A multi-actor shot performs ordered hit checks while
sharing one deterministic `8d8` damage roll across successful impacts and
continues after lethal intermediate hits. Knockback, wall/cell destruction,
spread/falloff, callback state, controlled runtime, browser capture, and
audiovisual parity remain open.

The `0.2.256` successor extends the Null Pointer target-score and delayed
explosion contract with bounded actor-only radius-1 splash. A successful hit
applies fixed `10d1` Plasma environment damage once per living actor in stable
blast-cell order and continues after lethal actors, preserving death/drop
follow-up. Terrain/item destruction, splash immunity, knockback, exact delayed
timing, callback state, controlled runtime, browser capture, and audiovisual
parity remain open.

The `0.2.257` successor extends the Chaingun ordinary-fire profile with a typed
first-level chainfire command. A Chaingun at warm-up level `0` now accepts a
three-projectile burst after atomic three-round preflight, always emits three
ordered ranged outcomes (deterministic no-op misses fill slots after a lethal
target without additional damage or RNG sampling), advances observable
warm-up state only after acceptance, and lets ordinary fire reset that state.
Replay/MCP JSON/catalog, browser snapshot, physical `C` key routing, and
`BrowserSession` parity are verified; higher chainfire levels, target
rotation/spread, exact callback timing/accuracy, controlled runtime, browser
capture, and audiovisual parity remain open.

The `0.2.258` successor adds Minigun's immutable first-level chainfire profile.
Pinned legacy `Shots div 3` adjustment yields six projectiles and six 9mm
rounds after the eight-projectile ordinary profile. The `0.2.260` successor
executes that profile through the existing typed chainfire command and browser/
MCP boundaries while leaving higher levels, exact timing/accuracy, controlled
runtime, browser capture, and audiovisual parity open.

The `0.2.261` successor extends the same typed first-level chainfire command to
the standard Plasma Rifle. Pinned legacy `Shots div 3` adjustment yields four
projectiles and four cells after the six-projectile ordinary profile. Direct
core, replay, MCP, physical `C` routing, and BrowserSession parity are verified;
higher levels, Nuclear Plasma behavior, exact timing/accuracy, controlled
runtime, browser capture, and audiovisual parity remain open.

The `0.2.262` successor extends the same typed first-level chainfire command to
the Laser Rifle. Pinned legacy `Shots div 3` adjustment yields four projectiles
and four cells after the five-projectile ordinary profile. Direct core, replay,
MCP, physical `C` routing, and BrowserSession parity are verified; higher
levels, target rotation/spread, exact timing/accuracy, controlled runtime,
browser capture, and audiovisual parity remain open.

The `0.2.263` successor extends the same typed first-level chainfire command to
the Nuclear Plasma Rifle. Pinned legacy `Shots div 3` adjustment yields four
projectiles and four cells after the six-projectile ordinary profile. Direct
core, replay, MCP, physical `C` routing, and BrowserSession parity are verified;
higher levels, overload map effects, exact timing/accuracy, controlled runtime,
browser capture, and audiovisual parity remain open.

The `0.2.264` successor extends the same typed first-level chainfire command to
the BFG 10K. Pinned legacy `Shots div 3` adjustment yields four projectiles and
twenty cells after the five-projectile ordinary profile. Direct core, replay,
MCP, physical `C` routing, and BrowserSession parity are verified; higher
levels, scatter/routing, delayed explosion geometry/damage/knockback, exact
timing/accuracy, controlled runtime, browser capture, and audiovisual parity
remain open.

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
