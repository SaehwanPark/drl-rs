# Medical Powerarmor — Behavioral Specification

**Domain:** equipped-item periodic behavior
**Milestone relevance:** M9 / Gate D behavior stress cases
**Last updated:** 2026-08-25
**Status:** Partial — legacy evidence captured; typed Rust behavior covered in
`0.2.119`; legacy runtime cadence and presentation parity remain `NOT_RUN`

This note is an evidence artifact for the first selected callback-heavy stress
case. It describes the legacy rule and the current Rust gap without treating
the legacy callback machinery as a Rust architecture.

## Evidence identity and scope

- **Legacy repository:** `/Users/saehwan/repos/doom-the-roughlike-original`
- **Revision inspected:** `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`
- **Legacy working-tree state:** dirty at inspection time (`audio.lua` and
  `meta.lua` modifications in the high/low-quality data trees plus an
  untracked `fpcvalkyrie/` directory). All findings below come from immutable
  `git show <revision>:<path>` content, not those working-tree edits.
- **Runtime probe:** `NOT_RUN`; no controlled legacy capture or Linux runtime
  session was available for this slice.

## Sources inspected

- Lua item declaration: `bin/data/drl/items/uitems.lua`,
  `umedparmor` at lines 903–922.
- Lua perk declaration: `bin/data/drl/items/uitems.lua`,
  `perk_umedparmor` at lines 880–901.
- Pascal perk dispatch: `src/dfthing.pas`, `TThing.CallHook` at lines 117–131;
  `src/drlinventory.pas`, `TInventory.CallHook` at lines 307–323 and
  `TInventory.Tick` at lines 238–244; `src/drlperk.pas`, `TPerks.CallHook`
  at lines 120–126.
- Pascal per-being scheduling: `src/dfbeing.pas`, `TBeing.Tick` at lines
  1619–1629; `src/dflevel.pas`, `TLevel.Tick` at lines 1355–1392.
- Hook taxonomy: `src/drlhooks.pas`, `Hook_OnTick` and related hook names at
  lines 24–45 and 101–118.

## Verified legacy behaviors (`observed`)

### Static item registration

- `umedparmor` is an armor item with protection `6`, movement modifier `-15`,
  unique/non-destroyable flags, and an `OnCreate` callback that adds the
  `perk_umedparmor` perk (`uitems.lua:903–921`).
- Adding that perk initializes an integer `pp_med_timer` property to `0`
  (`uitems.lua:880–884`).

### Periodic healing rule

- On each item `OnTick`, the perk first requires an owning being and armor
  durability greater than `20` (`uitems.lua:886–889`).
- While the owner is below half of maximum HP, the timer increments by one.
  When it reaches `30`, the callback sets the timer to `20`, heals one HP, and
  removes one durability (`uitems.lua:889–895`).
- When the owner is at or above half maximum HP, the timer is reset to `0`
  (`uitems.lua:896–899`). Healing therefore requires both a health threshold
  and sufficient armor durability.

### Trigger and ordering

- The player/being tick calls inventory ticking before the being's HP-decay,
  speed-counter, and general `OnTick` work (`dfbeing.pas:1619–1629`).
- Inventory ticking visits equipped slots and calls each equipped item's tick
  (`drlinventory.pas:238–244`). The broader `Hook_OnTick` dispatch can include
  equipped inventory slots (`drlinventory.pas:307–323`) and routes through the
  item's perk list (`dfthing.pas:124–131`, `drlperk.pas:120–126`).
- The level loop increments level time, dispatches level/perk work, and then
  ticks active beings before allowing ready non-player actors to act
  (`dflevel.pas:1355–1414`). This establishes ordering, but not a DRL-Rust
  turn-equivalence claim.

## Callback decomposition

| Dimension | Legacy evidence |
| --- | --- |
| Trigger | Equipped item's periodic `OnTick` path. |
| Preconditions | Owner exists; durability `> 20`; owner HP `< hpmax / 2`. |
| State read | Owner HP/max HP; armor durability; `pp_med_timer`. |
| State mutated | `pp_med_timer`; owner HP; armor durability. |
| Cadence | Increment every item tick; first heal at timer `30`; timer becomes `20` after healing. |
| Cost | One armor durability per one HP healed. |
| Target selection | The perk's owning being (`self.parent`); no spatial target query. |
| Ordering dependency | Equipped inventory tick occurs before the rest of `TBeing.Tick`. |
| Presentation | No item-specific visual/audio callback was established by the inspected sources. |

## Inferred intent (`inferred-intent`)

- The item is intended to provide slow, conditional self-healing while worn,
  with durability as the explicit resource cost. This follows directly from
  the perk name, threshold, and paired HP/durability mutation, but no runtime
  probe was available to confirm player-facing timing.
- Setting the timer to `20` after a `30`-tick heal appears intended to shorten
  the interval between subsequent heals to ten ticks rather than restart a
  full thirty-tick wait. This is a reasoned interpretation of the assignment,
  not a separately observed runtime result.

## Legacy implementation artifacts (`implementation-artifact`)

- The string-keyed `pp_med_timer` property, `OnCreate` callback, and perk
  registry are legacy storage/dispatch machinery. They are not requirements
  for DRL-Rust.
- The source leaves the timer untouched when durability is `<= 20` because the
  reset is nested under the health branch. Whether that is deliberate gameplay
  or an incidental quirk is unresolved; a Rust model must choose explicitly.

## Candidate DRL-Rust decisions (`drl-rust-decision`, not yet accepted)

- Model this as a dedicated typed armor behavior (for example,
  `MedicalRepair`) with explicit timer state and a pure tick transition, not a
  generic callback registry.
- Apply the effect only to the equipped armor and emit a typed healing/resource
  event through the existing deterministic simulation boundary.
- Keep the `durability > 20`, half-health, one-HP/one-durability policy as
  evidence-backed parameters until a runtime probe or a steering decision
  resolves the timer-edge ambiguity.

## Proposed acceptance tests

1. A pure transition test proves no effect at/above half HP, no effect at
   durability `<= 20`, timer progression below half HP, and the heal/cost
   mutation at the threshold.
2. An integration scenario proves only equipped Medical Powerarmor ticks,
   repeated heals are deterministic, and the event/replay stream is stable.
3. Edge tests decide and document whether crossing the durability guard resets
   or preserves the timer, rather than inheriting the Lua nesting accidentally.

## Open questions and non-goals

- Is one item tick exactly one player turn in the canonical runtime, or can
  scheduler speed cause a different observable cadence? Needs: controlled
  legacy runtime probe.
- Should the durability guard reset the timer when durability reaches `20` or
  below? Source nesting is clear; intended behavior is not.
- Does the canonical runtime expose a distinct healing message/effect for this
  perk? Needs: runtime/capture evidence.
- This slice does not implement generic periodic effects, other recharge perks,
  item sets, or any runtime Lua compatibility layer.

## Current DRL-Rust delivery evidence

`drl-core::behavior::MedicalRepairState` is the explicit Rust transition for
this first Gate D case. An equipped `ItemArchetype::MedicalPowerarmor` advances
its armor-owned timer once per accepted player command; the transition preserves
the strict durability guard, heals one HP at timer `30`, spends one durability
point, retains timer `20`, and emits `GameEvent::MedicalPowerarmorRepaired` on
the repair boundary. Pure transition tests and headless integration/replay
event determinism tests pass in the `0.2.119` revision. This is behavior
coverage for the current Rust policy, not a controlled legacy parity claim.

## Provenance and rights

This note records numeric mechanics and control-flow evidence only. It adds no
new copied creative description or media to Rust-owned content; the existing
Rust item name/description remain separately tracked by the content evidence
and release-rights inventories.
