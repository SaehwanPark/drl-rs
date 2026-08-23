I audited the accessible rewrite and compared it with the current upstream DRL source.

## Bottom line

I think the rewrite is **very feasible and architecturally promising**, and I strongly prefer its overall direction to a close Pascal→Rust translation.

But I would describe its present state as:

> **a sophisticated deterministic DRL engine/reimplementation framework with partial DRL gameplay fidelity**, rather than yet a faithful Rust port of DRL.

That distinction matters. The infrastructure is unusually mature relative to the amount of canonical game behavior that has actually crossed the migration boundary.

The architecture itself is one of the strongest parts. `drl-core` as deterministic simulation authority, semantic `Command` input, `GameEvent` output, fair `PlayerObservation`, and presentation as an outer shell is a very good decomposition.  It is substantially more testable and comprehensible than upstream's Pascal + runtime-Lua architecture.

At the same time, I found some concrete correctness and maintainability issues that I would address **before continuing the current rapid content-import sequence**.

---

# 1. First priority: the advertised transactional invariant is currently false

This is the most important finding.

The architecture states:

> Illegal or rejected commands roll back ... without ... modifying world state.

But `Game::step()` does not actually establish a checkpoint or transaction before dispatching the command.

I found at least two concrete ways this breaks.

### Out-of-range firing consumes ammunition

In ranged attack handling:

1. weapon is found;
2. `current_clip` is checked;
3. **`current_clip -= 1` happens**;
4. then range is checked;
5. `TargetOutOfRange` can be returned.

So:

```text
invalid ranged command
→ returns Err(TargetOutOfRange)
→ turn does not advance
→ but ammunition has disappeared
```

That is a real state-corruption bug relative to the documented contract.

### Equipping a non-equippable item can remove it

`execute_player_equip()` first removes the item from inventory and only afterwards asks for `item.equipment_slot()`:

```rust
let item = player.inventory_mut().remove_item(item_id)?;
let slot = item
  .equipment_slot()
  .ok_or(CommandError::CannotEquip(item_id))?;
```

If `equipment_slot()` is `None`, the command returns `CannotEquip`, but the removed item is never restored.

This is exactly the class of problem the transactional architecture was supposed to eliminate.

### What I would change

I would establish a very strong property:

```text
For every command C:

if game.step(C) == Err(_)
then
  game_after == game_before
```

including:

* world
* inventory/equipment
* RNG state
* item/entity counters
* turn
* scheduler energy
* visibility
* game-over status

Then test this generically across command classes.

Longer-term, I prefer:

```text
Command
   ↓
validate / prepare
   ↓
PreparedAction
   ↓
commit
```

where preparation is non-mutating and all expected user errors occur there. Execution of a `PreparedAction` should be close to infallible.

A full `GameState` clone-and-rollback guard would be an acceptable short-term correctness backstop, but I would not make cloning the permanent implementation because headless cohort/agent workloads make per-command cloning unnecessarily expensive.

---

# 2. The rewrite is solving the right architectural problem in upstream

Looking through upstream explains why rewriting rather than translating is the right choice.

The old architecture is not simply "a large Pascal game." It is a Pascal execution engine coupled to an extensive dynamic Lua behavior system. The README explicitly describes Lua 5.1 as compulsory and notes that DRL uses "sophisticated Lua tricks."

More revealingly, upstream has **72 hook types**:

* `OnUse`
* `OnEquip`
* `OnUnequip`
* `OnAltFire`
* `OnAltReload`
* `OnHitBeing`
* `OnKill`
* `OnTick`
* `OnDamage`
* `OnReceiveDamage`
* `getDamageBonus`
* `getFireCostMul`
* `getResistBonus`
* etc.

And the actual item definitions demonstrate that these are central gameplay mechanisms rather than incidental scripting. For example, `uitems.lua` contains equipment-set bonuses, resistance mutation, custom explosions, energy manipulation, phase effects, kill hooks, alt-fire behavior, health costs and bespoke weapon behavior.

So the hard part of this port is not:

> "How do we translate Lua tables into Rust structs?"

It is:

> **"What is the smallest clean Rust behavioral vocabulary capable of expressing DRL's 72-hook ecosystem without recreating a dynamic callback soup?"**

That should probably become one of the project's central design questions.

---

# 3. I agree with dropping runtime Lua

ADR 0008 is a good decision.

The rewrite now treats Lua as migration evidence only and explicitly avoids bundling a Lua VM into WASM.  The converter intentionally extracts shallow scalar fields while recording functions/nested structures as migration gaps rather than pretending they have been converted.

That is exactly the truthful behavior I would want.

However, this means the enormous set of callback gaps must eventually become explicit Rust behavior.

I would **not** respond by adding increasingly large `match ItemArchetype` blocks.

Instead I would start building a typed composition vocabulary along approximately these lines:

```text
ItemDefinition
├── static properties
├── PassiveModifiers
├── EquipEffects
├── AttackEffects
├── OnKillEffects
├── AlternateAction
├── RechargePolicy
├── SetMembership
└── special behavior ID/state
```

with explicit effect enums/systems rather than a generic event bus.

For special levels, similarly, a small typed state-machine/trigger abstraction will probably be cleaner than trying to reproduce arbitrary Lua.

A good stress test would be to port a handful of deliberately difficult upstream items end-to-end—not another 20 scalar-only weapons. For example:

* Inquisitor/Angelic set: set membership + equip/unequip modifiers
* Subtle Knife: alternative action + HP cost + visible-target iteration
* Trigun: alternative reload + confirmation semantics + nuke
* Null Pointer: hit callback + boss-dependent behavior

If the Rust model expresses those cleanly, you are probably approaching the right abstraction.

---

# 4. Content migration currently has too much change amplification

This deserves attention fairly soon.

The most recent three-armor PR (#181) touched **17 files**, including:

* core definition
* item implementation
* validation
* protocol enums
* replay structures/codecs
* assets
* MCP
* Cargo/version files
* multiple canonical docs

The core definition itself is straightforward.  But adding an archetype also requires another enum variant and another manual `Display` arm in protocol, for example. The latest PR shows exactly that. This is deliberate exhaustiveness, and `LESSONS.md` explicitly recognizes the fan-out.

Compiler-enforced exhaustiveness is valuable.

**Manual synchronization across half a dozen registries is not.**

This is starting to exhibit the "shotgun surgery" smell.

I would move toward **one authoritative compile-time catalog with generated projections**:

```text
canonical item catalog
      │
      ├── stable archetype identifiers
      ├── spawn definitions
      ├── replay names
      ├── presentation identifiers
      └── validation coverage
```

The projections can still be strongly typed and compile-time checked. A simple declarative `macro_rules!` catalog may be sufficient; I would avoid introducing a sophisticated procedural-macro framework unless necessary.

The key objective is:

> Adding a conventional item should mean defining the item once and implementing genuinely unique behavior—not manually teaching seven subsystems that the identifier exists.

---

# 5. `drl-protocol` is carrying too much game-domain policy

The semantic boundary idea is excellent, but I think the crate boundary needs refinement.

For example, `drl-protocol::types` currently owns `MonsterDefinition`, including:

* HP
* speed
* melee damage
* ranged damage
* accuracy
* knockback
* death drop

and `MonsterKind::definition()` contains actual gameplay balance values.

That means the "protocol" crate is partly a gameplay/content-definition crate.

Likewise replay owns the ever-growing `ItemSpawnKind` enum.

I would aim for a cleaner division:

```text
drl-protocol
  stable commands
  observations
  events
  stable semantic IDs
  replay wire structures

drl-core / perhaps drl-domain
  gameplay definitions
  rules
  balance
  behavior
  content resolution
```

`ItemArchetypeId` or `MonsterArchetypeId` can cross the protocol boundary without the protocol crate knowing that an Imp has a particular damage range.

This will become increasingly important once traits, exotic behaviors, resistances, challenges and special levels arrive.

---

# 6. Replay determinism is excellent locally, but not yet archival determinism

This is another subtle but important distinction.

The deterministic machinery itself is strong:

* RNG state is explicitly part of `Game`;
* Xoshiro256++ is owned by `GameRng`;
* no ambient RNG is intended;
* `Game` equality includes RNG state.

But `ReplayEngine::verify_determinism()` currently verifies determinism by running the **same replay through the same current implementation twice** and comparing results.

That proves:

> same implementation + same replay → same result

which is valuable.

It does not yet prove:

> a replay produced by 0.2.88 will produce the same run on 0.4.0.

The replay contains `engine_version`, but `ReplayEngine::validate()` currently performs structural/spatial checks and does not enforce the recorded engine version. Moreover, item spawns are reconstructed through the **current** `Item::from_spawn_kind()` implementation.

So changing an item definition can alter an old replay without changing its V1 envelope.

Before calling the format stable, I would add something equivalent to:

```text
ReplayMetadata
├── replay_schema_version
├── engine_semantics_version
├── content/ruleset hash
└── perhaps generator version
```

Then either:

* reject mismatched semantics explicitly, or
* run an explicit replay migration.

The roadmap already acknowledges that cross-version replay schemas/migrations remain open, so this is not a criticism of an unclaimed feature.  I would simply address it **before** too much ecosystem starts depending on V1.

---

# 7. Fix the RNG sampling implementation before freezing replays

`GameRng::gen_range()` currently does:

```rust
range.start + (self.next_u32() % span)
```

This introduces modulo bias whenever `span` does not divide $2^{32}$.

For a roguelike the practical effect will generally be tiny, but this project is explicitly building statistical cohort/evaluation infrastructure. There is little reason to retain a known biased range sampler.

`gen_bool()` similarly converts floating-point probability through a threshold based on `u32::MAX`; I would tighten that implementation too.

I would implement unbiased rejection/Lemire-style bounded sampling now.

The reason to do it **now** is not that this is an urgent gameplay bug. It is that correcting it later intentionally changes every downstream deterministic RNG sequence.

---

# 8. Some files are already becoming new monoliths

The crate decomposition is good, but module-level decomposition has not kept pace.

For example:

* `drl-render/src/lib.rs`: ~96 KB
* `drl-web/src/lib.rs`: ~88 KB
* `drl-core/src/item_definition.rs`: ~54 KB
* `drl-core/src/game.rs`: ~41 KB
* `drl-core/src/item.rs`: ~33 KB

I would not create more crates. The workspace already has nine.

I would instead split modules by reasons to change.

For example:

```text
drl-web/
  session.rs
  input.rs
  dom.rs
  animation_loop.rs
  gpu/
  persistence.rs
  texture.rs

drl-render/
  scene.rs
  layout.rs
  sprites.rs
  effects.rs
  lighting.rs
  animation.rs
  minimap.rs

drl-core/
  game/
    mod.rs
    movement.rs
    combat.rs
    inventory.rs
    levels.rs
```

The goal isn't small files for their own sake. It is keeping unrelated reasons for change apart.

---

# 9. `drl-script` is now misleading

ADR 0008 superseded the runtime-Lua strategy and says `drl-script` is merely a future conversion/content boundary.

Its implementation is currently essentially a placeholder returning the string `"drl-script"`.

I would either:

* remove the crate until it has a responsibility, or
* rename it to something like `drl-content-import` when the Rust-side conversion boundary actually exists.

A crate whose name implies runtime scripting while the architecture explicitly prohibits runtime scripting creates unnecessary conceptual noise.

---

# 10. There is a licensing/provenance question worth resolving

The rights machinery itself is unusually careful. The repository explicitly declares:

* project code: MIT
* imported graphics: CC BY-SA 4.0
* legacy Pascal/Lua: excluded / not cleared
* legacy audio/music/fonts: excluded / not cleared

Upstream explicitly licenses its code under GPL 2.0 and graphics under CC BY-SA 4.0.

However, the Rust source contains verbatim legacy item descriptions such as the distinctive prose attached to weapons and armor.

Numeric mechanics and factual identifiers are one question; copied creative textual descriptions are another.

I would therefore have the release-rights analysis explicitly cover:

> **legacy textual/game-content expression embedded in Rust definitions**

rather than treating the relevant boundary only as "legacy Pascal/Lua code."

I am not making a legal conclusion here. This is exactly the sort of ambiguity your otherwise careful provenance system should surface for an appropriate rights review.

---

# 11. The biggest strategic risk is infrastructure outrunning game fidelity

This was perhaps the strongest overall impression.

The project already has:

* deterministic replay
* MCP/JSON-RPC
* bots
* cohort evaluation
* release manifests
* detached signing
* PWA/offline behavior
* accessibility contracts
* WebGPU rendering
* extensive CI
* content evidence pipelines
* release-rights verification

The CI itself is quite good: repository checks run formatting, Clippy with `-D warnings`, the whole workspace test suite, content-conversion tests, MCP integration tests, rights/signing checks, plus a separate WASM/browser job.

Meanwhile, canonical gameplay still explicitly lacks things such as:

* item callbacks/effects
* exact weapon timing/accuracy
* resistances
* movement modifiers
* dynamic healing
* prepared-slot behavior
* much broader item behavior

And the actual legacy audiovisual fidelity matrix is still `NOT_RUN`.

That doesn't mean the infrastructure was wasted—the deterministic architecture is excellent.

But I think the project has now reached the point where I would deliberately reverse the emphasis:

> **less platform/tooling expansion, more end-to-end DRL semantic parity.**

In other words, don't measure progress by how many item definitions exist. Measure it by how many difficult slices are behaviorally complete.

---

# 12. Upstream itself validates this strategy

The original repository contains decades of history—the README explicitly says parts date to 2002.  Its Pascal/Lua architecture accumulated exactly the kind of implicit cross-cutting mechanisms that a clean rewrite can eliminate.

The rewrite should therefore **preserve behavior, not architecture**.

I think your present project largely understands this. Its explicit statement that source similarity is not proof of parity is particularly good.

The mistake to avoid now is accidentally constructing a new equivalent of the old hook architecture out of:

```text
ItemArchetype match
→ another match
→ another registry
→ callback special case
→ another special case
→ ...
```

The next few architectural decisions around behavior composition will determine whether this remains clean when it contains 100% of DRL.

---

## What I would do next

I would pause broad content addition for about five architectural slices—not pause development, but make the next work correctness/fidelity enabling:

1. **Repair transactional command semantics.** Add a generic invariant test asserting `Err => exact Game equality`; fix ranged ammo and equip first.

2. **Fix RNG bounded sampling and define replay semantics versioning.** Do both before deterministic histories become more expensive to invalidate.

3. **Collapse item/content registry fan-out.** Establish one authoritative catalog and generated/compiler-checked projections.

4. **Clean the domain boundary.** Keep semantic IDs/views in `drl-protocol`; move gameplay definitions and balance out of it.

5. **Build the typed behavior model using 3–5 nasty upstream examples.** Only after that resume mass migration of uniques/exotics/armor/special levels.

Then I would make the next major milestone a **vertical canonical-fidelity slice**, something like:

```text
one canonical early-game progression
+ canonical monsters
+ canonical weapons
+ armor/resistance
+ traits
+ representative exotic callback behavior
+ exact turn economy
+ deterministic behavioral comparison against upstream
+ browser presentation
```

That would tell you considerably more about feasibility than migrating another fifty scalar definitions.

### Overall assessment

| Area                                  | Assessment                               |
| ------------------------------------- | ---------------------------------------- |
| Rust rewrite feasibility              | **High**                                 |
| Core architectural direction          | **Very strong**                          |
| Determinism/testability               | **Strong, with replay-version caveat**   |
| Rust domain modeling                  | **Good, but boundaries need refinement** |
| CI/tooling                            | **Excellent for this stage**             |
| Current gameplay fidelity             | **Still partial**                        |
| Content architecture scalability      | **Needs intervention soon**              |
| Legacy-behavior migration difficulty  | **High but manageable**                  |
| Browser/WASM feasibility              | **High**                                 |
| Risk of reproducing legacy complexity | **Moderate and increasing**              |

So I would absolutely continue the rewrite. I would **not** translate upstream more literally. But I would change the project's center of gravity now: the engine/harness foundation is mature enough. The difficult and valuable work is increasingly the behavioral model and verified DRL fidelity, and the rollback bug shows that strengthening core invariants before adding further breadth will pay off immediately.
