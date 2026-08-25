# Acid and Lava terrain hazard evidence

**Domain:** entered-cell environmental damage

This note records the pinned Lua cell callbacks that motivate the bounded
baseline hazard transition. It does not reproduce the legacy callback system or
claim full hazard parity.

## Source identity

- Legacy checkout: `/Users/saehwan/repos/doom-the-roughlike-original`
- Revision: `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`
- Source: `bin/data/drl/cells.lua`
- Acid callback: lines `411-443`
- Lava callback: lines `445-475`
- Checkout dirty state: unrelated `drlhq`/`drllq` metadata edits and an
  untracked `fpcvalkyrie/` directory were present; inspected source is pinned
  to the revision above.

## Observed behavior

- Acid is a liquid hazard. Its `OnEnter` callback applies base damage `6` with
  `DAMAGE_ACID` unless fluid contact is avoided or acid resistance is complete.
- Acid declares `move_cost = 1.25` for fluid movement.
- Lava is a liquid hazard. Its `OnEnter` callback applies base damage `12` with
  `DAMAGE_FIRE` unless fluid contact is avoided or fire resistance is complete.
- Lava declares `move_cost = 1.25` for fluid movement.
- Water also declares `move_cost = 1.25` in the pinned source (`cells.lua` lines
  387-397), while Mud declares `move_cost = 1.65` (lines 399-409). The current
  Acid/Lava/Water/Mud movement-cost slice covers both ratios; Mud is modeled as
  a neutral walkable terrain with no contact damage in this Rust policy.
- Both callbacks halve damage on Easy difficulty and while the player is using
  the legacy `running` perk. Player-facing messages and periodic hit sounds are
  presentation effects, not part of this slice.
- The inspected hook is tied to entering a cell; it does not establish a
  repeated damage tick for waiting in place.

## Rust boundary

This revision implements only the observed baseline amounts for a normal,
non-running, non-resistant player: Acid `6`, Lava `12`. Damage is applied after
an accepted player move onto the hazard and uses existing
`DamageSource::Environment`, `DamageApplied`, and `ActorDied` contracts.
The core uses its internal-damage path so generic armor protection does not
silently change the selected raw baseline; this is an explicit DRL-Rust policy,
not a claim that legacy resistance is absent.
The typed `DamageApplied` event now carries optional `DamageType`: environment
Acid/Fire contact emits `Some(Acid)`/`Some(Fire)`, while actor and unclassified
environment damage remains explicitly absent. This exposes classification
without claiming resistance or balance parity.
Lethal contact records the accepted action cost and turn end while suppressing
periodic armor and pending-nuke follow-up; scheduled monster turns stop after
the environment death event.

The follow-up movement-cost slice represents the Acid/Lava/Water `1.25` ratio as
integer `ActionCost::new(1250)` and Mud's `1.65` ratio as
`ActionCost::new(1650)` for direct player movement; ordinary walkable movement
remains 1000. Fractional legacy scheduler behavior and fluid flow are not
claimed.

Difficulty, running, resistance, avoidance, monster hazard contact, runtime
comparison, and exact audio/visual parity remain
`NOT_RUN`.
