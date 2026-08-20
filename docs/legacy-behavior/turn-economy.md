# Turn Economy — Behavioral Specification

**Domain:** action economy / scheduling
**Milestone relevance:** M1, M2
**Last updated:** 2026-08-20
**Status:** Partial

---

## Evidence Sources

- **Observed game behavior** — DRL-Rust headless simulation, M2 test suites
  (`crates/drl-core/tests/simulation.rs`, `combat.rs`, `monsters_ai.rs`).
- **Legacy Pascal source** — available locally for reference; action cost and
  scheduling internals not yet exhaustively reviewed.

---

## Verified Behaviors

These behaviors are confirmed by DRL-Rust implementation and test suites.

- **Energy-based scheduling** — actors accumulate energy each global tick
  proportional to their `Speed` value. An actor acts when its energy meets or
  exceeds a threshold. Higher-speed actors act more frequently than
  lower-speed actors.
- **Player turn first** — when multiple actors are eligible to act in the same
  tick, the player acts before monsters. Monster ordering within a tick is
  deterministic (BTreeMap key order by `EntityId`).
- **Action costs are fixed per command type** — each command type (`Move`,
  `Wait`, `AttackMelee`, `AttackRanged`, `Reload`, `Pickup`, etc.) has an
  associated `ActionCost` that is deducted from the actor's energy when the
  command is executed. Current costs are uniform (1 unit) across most
  commands in the current implementation.
- **Dead actors do not act** — an actor with `is_alive == false` is never
  scheduled. Dead actor entries remain in the world until explicitly removed
  (currently they persist until end of level).
- **Speed affects turn frequency, not cost** — faster actors take more turns
  per time unit; they do not pay less energy per action. The ratio of actions
  between actors of different speeds is proportional to their `Speed` values.

### Speed reference (current implementation)

| Actor | Speed |
|---|---|
| Player | 10 |
| FormerHuman | 10 |
| FormerSergeant | 10 |
| Imp | 10 |
| Demon | 12 |

These values match DRL-Rust's current implementation and are subject to
revision against legacy Pascal source evidence.

---

## Inferred Design Intent

- **Speed ratios drive relative action frequency** — legacy DRL uses a similar
  energy-based tick system. The specific constants (threshold, accumulation
  rate) may differ. DRL-Rust's model is functionally equivalent if the ratio
  of speeds is preserved.
- **Wait has a normal action cost** — waiting consumes an action slot;
  it is not free. This is consistent with the legacy roguelike convention.
  Uncertainty: low.
- **Weapon reload has a full action cost** — reloading consumes a turn.
  Uncertainty: low (matches observed game feel).

---

## Legacy Implementation Artifacts

- **Fixed-threshold energy model** — DRL-Rust uses a fixed energy threshold
  for action eligibility. The legacy Pascal model may use a different
  accumulation formula (e.g., fractional ticks, speed-scaled costs).
  Pending Pascal source review to determine whether the current model is
  behaviorally equivalent.

---

## Deliberate DRL-Rust Decisions

- **Uniform action costs** — all command types currently cost 1 energy unit.
  If the legacy game applies different costs per action type (e.g., moving
  diagonally costs more, reloading costs more than firing), this must be
  updated from legacy evidence before M9 gameplay breadth work.

---

## Open Questions

- **Exact legacy cost constants** — what energy threshold and per-tick
  accumulation values does legacy DRL use? Needs: Pascal scheduler source
  inspection.
- **Per-command action cost differentiation** — does legacy DRL assign
  different action costs to different command types (move vs. attack vs.
  reload vs. use)? Needs: Pascal cost table inspection.
- **Diagonal movement cost** — does diagonal movement consume the same energy
  as cardinal movement? See also `movement.md`.
- **Speed values for major monster types** — are the current DRL-Rust speed
  constants (all at 10 except Demon at 12) accurate to legacy behavior?
  Needs: Pascal/Lua monster stat tables.
- **Boss/special actor speed** — do any legacy actors have speed above 12 or
  below 10? Needs: legacy actor roster review.

---

## Non-Goals

- Monster AI decision logic — covered in the planned `ai.md`.
- Trait and class effects on speed — deferred to M9 player progression work.
