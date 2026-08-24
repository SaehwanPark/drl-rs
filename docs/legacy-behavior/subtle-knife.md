# Subtle Knife — Behavioral Specification

**Domain:** alternate melee action with status and visible-target effects
**Milestone relevance:** M9 / Gate D behavior stress cases
**Last updated:** 2026-08-24
**Status:** Partial — evidence captured; Rust behavior not implemented

This note is the second selected Gate D stress-case artifact. It isolates the
legacy alternate-fire rule and its action-dispatch boundary; it does not treat
the legacy Lua callback registry as a Rust architecture.

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

- Lua perk and item declarations: `bin/data/drl/items/uitems.lua`,
  `perk_usubtle_altfire` at lines 210–235 and `usubtle` at lines 237–261.
- Alternate-fire command preparation: `src/drlbase.pas`,
  `TDRL.HandleFireCommand` at lines 690–842.
- Action dispatch and melee short-circuit: `src/dfbeing.pas`,
  `TBeing.ActionFire` at lines 892–970 and `TBeing.HandleCommand` at lines
  1589–1617.
- Hook checking semantics: `src/dfthing.pas`, `TThing.CallHookCheck` at lines
  133–139; perk dispatch at `src/drlperk.pas`, `TPerks.CallHookCheck` at lines
  130–145.
- Direct damage and explosion bindings: `src/dfbeing.pas`,
  `lua_being_apply_damage` at lines 2960–2968; `src/dflevel.pas`,
  `lua_level_explosion` at lines 1817–1848 and `TLevel.Explosion` at lines
  991–1095; explosion decoding at `src/dfdata.pas`, lines 896–929.
- Status definition: `bin/data/drl/affects.lua`, `tired` at lines 3–14.

## Verified legacy behaviors (`observed`)

### Item registration and target preparation

- `usubtle` is a unique, modable, single-mod melee weapon with damage `3d5`
  and damage type `DAMAGE_SPLASMA` (`uitems.lua:237–255`). Its `OnCreate`
  callback adds `perk_usubtle_altfire` and the `BLADE` property
  (`uitems.lua:257–260`).
- The perk exposes alternate fire under the short label `invoke`; the item
  does not set `IF_ALTTARGET`. When the player chooses alternate fire without
  a target-selection flag, `HandleFireCommand` supplies the player's current
  position as the target (`drlbase.pas:750–765`, `:811–814`). The perk itself
  ignores that coordinate.
- `ActionFire` marks the action as melee, invokes the weapon's
  `Hook_OnAltFire`, and returns success immediately after the hook for melee
  weapons (`dfbeing.pas:904–914`). It does not enter the ranged-ammo path.

### Successful invocation

- If the actor does not already have the `tired` perk, the callback emits a
  health-drain message, reduces HP by `5` but clamps HP to at least `1`, adds
  `tired`, and subtracts `1000` from `scount` (`uitems.lua:218–225`).
- It then iterates `level:beings()`. Every being that is not the player and is
  currently visible is selected; each receives a blue, range-1, delay-50
  `DAMAGE_SPLASMA` explosion request followed by direct `15` internal
  `DAMAGE_SPLASMA` damage (`uitems.lua:226–230`). No spatial target query or
  random sampling appears in the callback.
- The explosion table supplies color, range, delay, and damage type but no
  `damage` dice string (`uitems.lua:228`; `dfdata.pas:896–905`). The decoded
  explosion therefore contributes a presentation/area request with zero
  explosion damage; the explicit `apply_damage(15, ...)` call is the observed
  per-target damage operation. `TLevel.Explosion` only applies damage when its
  decoded dice roll is non-zero (`dflevel.pas:1039–1050`).

### Tired invocation and dispatch edge

- If the actor already has `tired`, the callback emits “You are too tired to
  invoke the Knife!” and performs none of the HP, status, `scount`, explosion,
  or target-damage mutations (`uitems.lua:218–221`).
- The Lua callback returns `false` on both branches (`uitems.lua:233`). The
  Pascal `CallHookCheck` path shown here uses protected-call success as its
  boolean and does not read a Lua return value (`dfthing.pas:133–138`,
  `drlperk.pas:130–145`). For this melee weapon, `ActionFire` consequently
  returns after dispatch rather than treating the callback's Lua `false` as a
  typed effect result (`dfbeing.pas:910–914`). Whether the outer scheduler
  spends a turn for a tired no-effect invocation is not established by these
  sources and requires a runtime probe.

## Callback decomposition

| Dimension | Legacy evidence |
| --- | --- |
| Trigger | Player alternate-fire command on equipped Subtle Knife. |
| Preconditions | Weapon has the alt-fire hook; actor lacks `tired` for the effect branch. |
| State read | Actor status/HP; all level beings and each being's player/visibility state. |
| State mutated | Actor HP, `tired`, `scount`; selected targets' HP and damage effects. |
| Cadence | One invocation per accepted alternate-fire command; no internal timer. |
| Cost | 5 HP (clamped to 1), `tired` status, and `1000` `scount` on success. |
| Target selection | Every non-player, currently visible being; callback ignores supplied aim coordinate. |
| Ordering dependency | Hook dispatch precedes the melee `ActionFire` success return; target loop follows actor-cost mutations. |
| Presentation | Two UI messages plus one blue delayed explosion request per selected target. |

## Inferred intent (`inferred-intent`)

- The alternate action is intended as a room-wide visible-enemy attack whose
  power is paid from the actor's health and action budget, with `tired` as a
  one-perk lockout. This follows from the paired actor costs and the all-visible
  target loop; player-facing cadence remains unprobed.
- The explicit explosion request appears intended to show each hit rather than
  add a second damage source, because its decoded damage roll is empty and the
  callback immediately applies fixed damage. This is source-based inference,
  not a runtime visual confirmation.
- A Rust port should define a stable iteration order for replay/event output.
  The legacy `level:beings()` call demonstrates collection traversal but the
  inspected sources do not specify a canonical order.

## Legacy implementation artifacts (`implementation-artifact`)

- The perk ID, string label, dynamic `tired` lookup, and `CallHookCheck` return
  convention are legacy dispatch/storage machinery, not requirements for
  `drl-core`.
- The callback mutates actor state before iterating targets and can apply
  effects to multiple beings. Treating the whole operation as one typed action
  is a deliberate Rust design question; importing a generic callback bus is
  out of scope.

## Candidate DRL-Rust decisions (`drl-rust-decision`, not yet accepted)

- Model `SubtleKnifeInvoke` as a typed alternate action with an explicit
  precondition (`tired` absent), actor-cost transition, stable visible-target
  selection, and typed per-target damage/presentation events.
- Make the successful transition atomic at the simulation boundary: either
  the precondition passes and the actor cost plus selected-target effects are
  emitted deterministically, or the rejected command leaves gameplay state
  unchanged (apart from an explicitly modeled feedback event).
- Keep the target-coordinate parameter absent from the behavior model unless a
  later legacy probe shows it affects this specific perk.

## Proposed acceptance tests

1. A pure transition test proves the successful HP clamp, `tired` addition,
   `scount` cost, fixed per-visible-target damage, and exclusion of the player
   and hidden beings.
2. A rejected `tired` invocation test proves no gameplay-state mutation and a
   stable feedback/event result; it must explicitly decide whether the command
   consumes time in DRL-Rust rather than inheriting the legacy wrapper quirk.
3. A deterministic replay/scenario test fixes target iteration order and proves
   the same visible target set produces the same event sequence.

## Open questions and non-goals

- Does a tired no-effect invocation consume a turn in the canonical runtime?
  Needs: controlled legacy runtime probe.
- Is `level:beings()` order stable across saves/platforms, or is only the set of
  selected beings canonical? Needs: runtime capture or collection evidence.
- Does the delayed blue explosion have any player-visible timing beyond the
  direct damage event? Needs: controlled capture.
- This slice does not implement the Subtle Knife, generic alternate-action
  infrastructure, status effects, room-wide targeting, or Lua compatibility.

## Provenance and rights

This note records numeric mechanics and control-flow evidence only. It adds no
new copied creative description or media to Rust-owned content; existing Rust
content and release-rights inventories remain separately tracked.
