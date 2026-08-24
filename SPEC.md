# Specification

Last reviewed: 2026-08-24
Current project version: `0.2.109`

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

## 2. Active Implementation Slice: M1/M9 Correctness and Behavior Foundation

### 2.1 Objective

Close correctness and architecture gaps that would otherwise become more
expensive as legacy content migration accelerates.

This slice establishes four foundations before additional broad item-family
migration:

1. rejected player commands are transactionally state-identical;
2. deterministic RNG uses an unbiased bounded sampler with explicit semantics;
3. replay compatibility distinguishes wire schema from gameplay semantics;
4. legacy callback-heavy behavior has a typed Rust representation and a single
   authoritative content-catalog path rather than proliferating manual
   cross-crate registries.

The legacy Pascal/Lua implementation remains the behavioral reference. Its
architecture, global callback machinery, and runtime Lua object model remain
non-goals for reproduction.

### 2.2 Why this slice supersedes content breadth

The preceding M9 work successfully established provenance-aware immutable
content definitions and replay-visible item families. It also intentionally left
many legacy behaviors open: callbacks, resistances, movement modifiers,
alternate actions, recharge, set effects, exact weapon timing, and other
special-case semantics.

At the same time, adding a conventional content family now fans out across
protocol enums, definitions, validation, replay codecs, assets, documentation,
and other exhaustive registries. Continuing scalar-only breadth before a
behavior model is selected would increase both migration debt and change
amplification.

Therefore, additional broad scalar-only family additions are temporarily
blocked by the exit gates in Section 2.7.

### 2.3 Transactional Command Contract

#### Current bounded delivery target: ranged target rejection atomicity

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
- [ ] Cover all current command classes with at least one rejected-command
  scenario where rejection is reachable without malformed construction.
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
- [ ] Fix pickup/use/drop/reload and other multi-step commands so expected
  validation failures cannot lose or partially mutate items.
- [ ] Audit all current `Game::step` command paths for mutation-before-error
  behavior.
- [x] Preserve turn and RNG state exactly on rejection.
- [x] Document the chosen implementation pattern: prepare/commit is preferred;
  a bounded rollback guard is used as an interim correctness backstop.

### 2.4 RNG and Replay Semantics Contract

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

### 2.5 Typed Content and Behavior Architecture Contract

The project must retain compile-time exhaustiveness without requiring every new
ordinary content archetype to be manually synchronized across many independent
registries.

Legacy Lua callbacks are evidence of required behavior, not an architecture to
recreate. The Rust model should use a small number of explicit typed effect and
action concepts, with bespoke state machines only where composition is
insufficient.

#### Acceptance criteria

- [ ] Inventory all current manual fan-out points for adding an item archetype.
- [ ] Establish one authoritative compile-time item/content catalog or
  equivalent source of truth.
- [ ] Generate or mechanically derive routine projections such as stable IDs,
  display strings, validation coverage, replay names, and presentation lookup
  where doing so does not weaken type safety.
- [ ] Keep genuinely behavioral code explicit and reviewable rather than
  embedding arbitrary callbacks in the catalog.
- [ ] Define a typed behavior vocabulary that can represent at least:
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

### 2.6 Behavior Stress Cases

The behavior model is not accepted based on toy examples alone. Demonstrate it
using a small set of deliberately difficult legacy cases selected to exercise
different mechanisms.

The initial target set should include at least three of the following or
similarly difficult equivalents, with source evidence recorded from the pinned
legacy revision:

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

### 2.7 Exit Gates Before Broad Content Migration Resumes

All of the following are required:

- [ ] Rejected commands are state-identical across the audited command surface.
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

### 2.8 Vertical Fidelity Successor Slice

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

### 2.9 Explicit Non-Goals

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
