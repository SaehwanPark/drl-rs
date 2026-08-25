# Trigun — Behavioral Specification

**Domain:** alternate reload with destructive level action
**Milestone relevance:** M9 / Gate D behavior stress cases
**Last updated:** 2026-08-25
**Status:** Behavior-covered in DRL-Rust `0.2.121`; controlled legacy runtime
parity remains `NOT_RUN`

This note is the third selected Gate D stress-case artifact. It records the
Trigun's alternate reload boundary and the nuke transition it requests without
turning the legacy Lua callback mechanism into a Rust design requirement.

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
  `perk_uni_trigun_altreload` at lines 263–284 and `utrigun` at lines 286–319.
- Alternate-reload dispatch and destruction check: `src/dfbeing.pas`,
  `TBeing.ActionAltReload` at lines 871–890.
- Hook checking semantics: `src/dfthing.pas`, `TThing.CallHookCheck` at lines
  133–139; perk dispatch at `src/drlperk.pas`, `TPerks.CallHookCheck` at lines
  130–145.
- Command/scheduler boundary: `src/drlbase.pas`, `TDRL.HandleCommand` at
  lines 920–956; level scheduling at `src/dflevel.pas`, `TLevel.Tick` at lines
  1355–1392.
- Nuke state and resolution: `src/dfplayer.pas`, `TPlayer.NukeActivated` and
  `NukeTime` at lines 22–24 and 89–95; `src/dflevel.pas`, `NukeTick` at lines
  1422–1456 and `NukeRun` at lines 1458–1481.
- Comparable nuke item path: `bin/data/drl/items/items.lua`, `nuke` at lines
  910–964, used to interpret the `being:nuke(1)` and `being:nuke(100)` timer
  arguments.

## Verified legacy behaviors (`observed`)

### Item registration

- `utrigun` is a unique, modable, single-mod ranged pistol with ammo capacity
  `6`, damage `3d6` of type `DAMAGE_BULLET`, use time `7`, and reload time `20`
  (`uitems.lua:286–313`).
- Its `OnCreate` callback adds the aimed alternate-fire perk and
  `perk_uni_trigun_altreload` (`uitems.lua:315–318`). The item does not set an
  `IF_DESTROY` flag in its declaration.

### Alternate-reload success path

- The perk is reachable through the `Hook_OnAltReload` path. It only proceeds
  for a player whose `hpmax` is greater than `10`; it then asks for explicit
  confirmation with a danger warning (`uitems.lua:263–273`).
- On confirmation, it emits a message and history entry, reduces `hpmax` by
  `5` but clamps it to at least `10`, reduces current HP by `5` but clamps it
  to at least `1`, subtracts `1000` from `scount`, and calls `being:nuke(1)`
  (`uitems.lua:273–279`).
- The callback returns `true` on this branch, but the observed gameplay effects
  are the explicit state mutations and the nuke request. `ActionAltReload`
  checks `IF_DESTROY` after dispatch and removes the equipped weapon only when
  that flag is set (`dfbeing.pas:876–884`). Since Trigun never sets the flag in
  this callback or declaration, the destructive consequence is the requested
  level nuke, not weapon deletion.

### Rejected or cancelled paths

- If the actor is not the player, `hpmax` is `10` or lower, or confirmation is
  declined, the callback makes no HP, max-HP, `scount`, history, or nuke
  mutation and returns `false` (`uitems.lua:270–282`). The inspected callback
  does not emit a separate cancellation message.
- `TThing.CallHookCheck` and `TPerks.CallHookCheck` use protected-call success
  for their Boolean result; they do not expose the Lua callback's returned
  value as a typed action result (`dfthing.pas:133–138`, `drlperk.pas:130–145`).
  Therefore, an error-free callback can make `ActionAltReload` return success
  even when this Lua function returns `false` (`dfbeing.pas:876–884`).

### Nuke transition and ordering

- The player stores the countdown in `NukeActivated`/`NukeTime`
  (`dfplayer.pas:22–24`, `:89–95`). The `being:nuke(1)` call therefore requests
  the shortest timer value represented by the legacy API; the exact binding
  implementation is not present in the inspected Pascal units.
- On each `TLevel.Tick`, level time and level/perk tick hooks run before
  `NukeTick` (`dflevel.pas:1355–1375`). `NukeTick` decrements a non-zero timer;
  when it reaches zero it marks the level nuked, runs `NukeRun`, clears the
  timer, applies `6000` internal plasma damage to the player, and dispatches
  `Hook_OnNuked` (`dflevel.pas:1422–1455`).
- `NukeRun` schedules ten large, always-visible fire explosions with delays
  from `200` through `2000`, followed by a full-area nuke-cell pass
  (`dflevel.pas:1458–1481`). The callback's `nuke(1)` request is thus a
  destructive level transition at the next timer resolution, not a direct
  weapon attack; precise player-visible timing remains unprobed.
- The outer command dispatcher calls `Player.HandleCommand` but does not use
  its Boolean result to gate `Player.PostAction` and subsequent level ticks
  (`drlbase.pas:920–956`). Whether cancelled/low-HP alternate reload should
  consume a turn in a typed Rust model is therefore an explicit semantic
  decision, not something to infer from the callback's `return false`.

## Callback decomposition

| Dimension | Legacy evidence |
| --- | --- |
| Trigger | Player alternate-reload command with Trigun equipped. |
| Preconditions | Equipped ranged weapon has alt-reload hook; player `hpmax > 10`; confirmation accepted. |
| State read | Player identity, `hpmax`, confirmation result, nuke timer/level state. |
| State mutated | Player HP/max HP, `scount`, history, nuke timer; later level flag/cells/player HP. |
| Cadence | One confirmation-gated request per alt-reload command; nuke resolves on level ticks. |
| Cost | 5 max HP, 5 current HP (both clamped), and 1000 `scount` on success. |
| Target selection | The current level; no aim coordinate or enemy selection. |
| Ordering dependency | Callback costs and timer request precede `TLevel.Tick`'s nuke resolution; `IF_DESTROY` is checked after hook dispatch. |
| Presentation | Confirmation prompt, activation message/history, delayed global explosions, and nuke hooks. |

## Inferred intent (`inferred-intent`)

- The alternate reload is intended as a voluntary emergency weapon: it trades
  permanent maximum-health pressure and current health for an immediate-level
  destructive reset. This follows from the name, confirmation warning, HP-max
  mutation, and nuke request; the exact risk/reward timing is unprobed.
- The `hpmax > 10` guard and `max(hpmax - 5, 10)` clamp imply a minimum maximum
  health floor, while the current-HP clamp prevents the callback itself from
  reducing HP below `1`. The subsequent nuke can still kill the actor through
  the level's `6000` internal damage path.
- A Rust model should represent the nuke as a typed scheduled transition with
  explicit countdown semantics rather than as an untyped callback side effect.

## Legacy implementation artifacts (`implementation-artifact`)

- The perk ID, `ui.confirm`, string history template, `NukeActivated` storage,
  and Boolean `CallHookCheck` convention are legacy presentation/dispatch
  mechanisms, not required Rust contracts.
- `return false` is overloaded: it denotes the Lua function's branch result,
  but the Pascal protected-call wrapper does not use it as the action result.
  A Rust command should not inherit that ambiguity.

## Accepted DRL-Rust decision (`drl-rust-decision`)

- Model the behavior as `Command::AltReload { item_id, confirmed }` with a
  typed confirmation decision, HP/max-HP/score costs, a scheduled `NukeState`,
  and a separate weapon-destruction policy (none for the observed Trigun).
- Require confirmation and `hpmax > 10` before mutation; rejected commands are
  exact-state atomic and do not spend a turn or RNG.
- Resolve the one-tick nuke at the accepted command boundary, preserving
  explicit replay semantics for activation, level-nuked state, raw player
  damage, and terminal death ordering.
- Keep explosion geometry, map-cell destruction, animation/audio, confirmation
  UI, and legacy runtime capture outside this bounded slice.

## Delivered acceptance tests

1. `crates/drl-core/src/trigun.rs` covers the max-HP/current-HP clamps,
   signed score cost, and low-health rejection without mutation.
2. `crates/drl-core/tests/special_items.rs` covers missing-item,
   low-max-HP, and declined-confirmation rollback, weapon preservation, exact
   nuke event ordering, terminal rejection, and replay determinism.
3. The typed nuke state is deterministic and intentionally bounded to the
   level-nuked event plus internal player damage; explosion/map effects remain
   open rather than inferred.

## Open questions and non-goals

- Does `being:nuke(1)` resolve before or after other same-tick effects in the
  canonical runtime? Needs: controlled legacy runtime probe.
- Does a cancelled or low-max-HP alt-reload consume a player turn despite the
  callback returning `false`? Needs: controlled runtime capture.
- Are the ten nuke explosion positions and exact animation timings part of
  canonical gameplay or presentation-only evidence? Needs: capture and replay
  policy.
- This slice does not implement generic confirmation UI, explosion/map effects,
  weapon destruction, or Lua compatibility.

## Provenance and rights

This note records numeric mechanics and control-flow evidence only. It adds no
new copied creative description or media to Rust-owned content; existing Rust
content and release-rights inventories remain separately tracked.
