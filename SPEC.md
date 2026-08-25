# Specification

Last reviewed: 2026-08-25
Current project version: `0.2.129`

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

## 2. Active Implementation Slice: M9/Gate D Jackhammer Fire-Mode Behavior

### 2.1 Objective

Close one callback-heavy legacy behavior gap by pinning the Jackhammer's
alternate-reload mode cycle and making it an explicit Rust state transition.
The equipped shotgun toggles between its default three-shot burst and a
single-shot mode, preserving the existing shotgun damage profile and charging
one score count for the cycle.

The legacy Pascal/Lua implementation remains the behavioral reference. Its
architecture, global callback machinery, and runtime Lua object model remain
non-goals for reproduction.

### 2.1a Scope and steering gate

- **Steering priority:** Typed legacy behavior (Gate D), with Gate E's
  evidence-bounded claims applied to the source comparison.
- **Observable outcome:** `Command::AltReload` on an equipped Jackhammer
  toggles `Burst -> Single -> Burst`, emits a typed
  `JackhammerFireModeChanged` event, and spends one score count. Ranged
  attacks consume the selected one- or three-shell count atomically and retain
  the pinned `8d3` shotgun damage profile.
- **Replay/RNG impact:** The V1 wire format remains unchanged, but gameplay
  semantics advance from `7` to `8` because mode state changes future command
  outcomes. The transition and shot-count validation consume no RNG until all
  command validation succeeds; rejected commands preserve exact `Game` state.
- **Content-catalog impact:** No new item family or registration path is added.
  The existing Jackhammer definition remains the authoritative identity.
- **Protocol/domain ownership:** `WeaponFireMode` and the mode-change event are
  stable semantic protocol contracts; Jackhammer cycle policy, score cost, and
  shot resolution remain core-owned.
- **Evidence boundary:** The pinned Lua source supports the 3/1 cycle, the
  default three-shot profile, and the one-score-count cost. Controlled legacy
  runtime/capture comparison is `NOT_RUN`.
- **Non-goals:** Generic callback or burst registries, other weapon
  alt-reloads, spread/falloff parity, legacy UI text, runtime Lua, and broad
  content migration.

### 2.2 Why this slice supersedes content breadth

The preceding M9 work successfully established provenance-aware immutable
content definitions, replay-visible item families, and typed Gate D stress
cases. It also intentionally left many legacy behaviors open: callbacks,
resistances, movement modifiers, alternate actions, recharge, set effects,
exact weapon timing, and other special-case semantics. This slice takes the
next eligible vertical-fidelity step without broadening those behavior claims.

At the same time, adding a conventional content family now fans out across
protocol enums, definitions, validation, replay codecs, assets, documentation,
and other exhaustive registries. Continuing scalar-only breadth before a
behavior model is selected would increase both migration debt and change
amplification. The Jackhammer source audit gives this slice a narrow typed state
machine, a focused end-to-end test, and an explicit runtime evidence boundary.

Therefore, additional broad scalar-only family additions are temporarily
blocked by the exit gates in Section 2.8.

### 2.3 Jackhammer fire-mode contracts

For an equipped Jackhammer with a loaded clip, each accepted alternate reload
must toggle the mode and subtract exactly one score count using saturating core
policy. Burst mode resolves three shells and single mode resolves one shell;
both use the existing `8d3` weapon profile. The mode transition itself does
not consume RNG. A ranged command must reject before clip/RNG mutation when
the current clip contains fewer rounds than the selected shot count. Successful
shots resolve in deterministic order against the originally selected target;
lethal resolution stops the sequence and commits one death drop at most.

#### Legacy evidence boundary

Pinned source evidence is recorded in
[`docs/legacy-behavior/jackhammer.md`](docs/legacy-behavior/jackhammer.md).
The source callback's UI text and exact shot timing remain evidence-only.
Runtime parity, spread/falloff, and presentation feedback remain `NOT_RUN`.

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

**Current bounded delivery target (`0.2.127`):** The pinned legacy AI movement
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
version and ruleset/content identity independently from wire schema V1. Core and
MCP replay validation reject unsupported values before simulation, avoiding
silent reinterpretation through current rules.

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
- [ ] Establish one authoritative compile-time item/content catalog or
  equivalent source of truth. **Partial in `0.2.118`:** stable spawn-family
  identity, definition coverage, replay JSON encode/decode and completeness
  fixtures, stable-name parsing/display, and routine descriptor coverage tests
  now derive from typed catalogs/projections; core structural validation uses
  the protocol catalog through `CURRENT_ITEM_SPAWN_KINDS`; behavioral and
  presentation mappings remain explicit.
- [ ] Generate or mechanically derive routine projections such as stable IDs,
  display strings, validation coverage, replay names, and presentation lookup
  where doing so does not weaken type safety.
- [x] Derive the replay loose-ammo count requirement from the protocol
  archetype catalog and remove the duplicate MCP decoder variant list.
- [x] Resolve core item definitions through a single catalog ordered to the
  protocol spawn-family catalog, with length, uniqueness, and order coverage.
- [x] Derive ordinary inverse spawn lookup from `ItemSpawnKind::ALL` while
  retaining explicit loose-ammo count reconstruction and rejection coverage.
- [ ] Keep genuinely behavioral code explicit and reviewable rather than
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
- [ ] Avoid a generic string-keyed event bus or unconstrained dynamic callback
  registry in `drl-core`.
- [ ] Keep runtime Lua absent from the shipped game.

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

- [ ] Each selected stress case has a legacy evidence note identifying scalar
  fields, callback behavior, ordering assumptions, and unresolved ambiguity.
- [ ] Each behavior is represented using the typed model without adding a
  one-off global hook framework.
- [ ] Deterministic scenario tests exercise successful behavior and at least one
  rejected/edge path.
- [ ] Behavior-complete migration is distinguished from scalar-definition
  coverage in roadmap/status language.

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

### 2.7c Current Trigun delivery target

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

### 2.7e Current Jackhammer delivery target

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

### 2.8 Exit Gates Before Broad Content Migration Resumes

All of the following are required:

- [x] Rejected commands are state-identical across the audited command surface,
  including late death-drop terrain failures and the typed alternate commands.
- [ ] RNG sampling semantics are unbiased, golden-tested, and versioned.
- [ ] Replays declare gameplay semantics compatibility and reject incompatible
  interpretation.
- [ ] Routine content registration has one authoritative catalog path with
  materially reduced manual fan-out.
- [ ] The typed behavior model passes the selected legacy stress cases.
- [ ] `drl-protocol` contains stable semantic contracts but no longer owns
  mutable gameplay balance merely because a type crosses a boundary.
- [ ] Large implementation modules touched by this work are split only where
  there are clear independent reasons to change; no new crate is introduced
  solely to reduce file size.
- [ ] Repository, deterministic scenario, replay, and supported browser checks
  pass for the resulting revision.

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
