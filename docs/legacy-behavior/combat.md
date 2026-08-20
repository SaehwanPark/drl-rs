# Combat — Behavioral Specification

**Domain:** combat
**Milestone relevance:** M2, M4, M9
**Last updated:** 2026-08-20
**Status:** Partial

---

## Evidence Sources

- **Observed game behavior** — DRL-Rust headless simulation, M2/M4 test
  suites (`crates/drl-core/tests/combat.rs`, `stochastic_combat.rs`,
  `monsters_ai.rs`, `targeting.rs`).
- **Legacy Pascal source** — available locally for reference; combat
  resolution formulas and damage tables not yet exhaustively audited.
- **Legacy Lua source** — actor and weapon stat definitions likely in Lua;
  not yet reviewed.

---

## Verified Behaviors

These behaviors are confirmed by DRL-Rust implementation and test suites.

### Hit resolution

- **Accuracy roll** — each attack uses a uniform random roll in [0, 100).
  If the roll is less than the attacker's effective accuracy, the attack
  hits. Otherwise, it misses.
- **Range penalty** — ranged weapon accuracy degrades with distance. The
  current implementation applies a linear falloff based on `(range - distance)
  / range`. This is an approximation pending legacy formula verification.
- **Melee always in range** — melee attacks require the target to be in an
  adjacent (including diagonal) tile. Range validation is strict; off-by-one
  is rejected.
- **Line-of-sight requirement for ranged attacks** — `Command::AttackRanged`
  requires an unobstructed line of fire. Blocked shots return
  `CommandError::LineOfSightBlocked`.

### Damage calculation

- **Uniform damage roll** — on a successful hit, damage is drawn uniformly
  from `[weapon.min_damage, weapon.max_damage]` (inclusive). The statistical
  test suite in `stochastic_combat.rs` verifies the distribution falls within
  3-sigma bounds over large sample sizes.
- **Armor mitigation** — equipped body armor reduces incoming damage by its
  `protection` value. Damage is clamped to a minimum of 1 (or 0 if armor
  exceeds raw damage — exact floor pending legacy verification).
- **HP clamping** — HP cannot go below 0 or above `max_hp`. Both bounds are
  enforced at the `HitPoints` type level.
- **Death threshold** — an actor dies when HP reaches 0. Death is immediate
  and does not require an additional step.

### Death

- **Dead actors cannot act** — a dead actor is excluded from scheduling
  immediately after the lethal damage event.
- **Occupancy cleared on death** — a dead actor's tile is no longer treated
  as occupied; other actors may move through it on the next turn.
- **Loot drops on death** — monsters have a configurable loot table. On
  lethal hit, floor items are spawned at the monster's position and
  `GameEvent::ItemDropped` is emitted.

### Knockback

- **Kinetic knockback** — Shotgun and Former Sergeant attacks apply knockback
  of 1 tile along the shot vector if the target survives.
- **Knockback collision** — knockback stops if the destination is out of
  bounds, a wall, or occupied by another actor. In that case the target
  remains in place.
- **Player knockback** — if the player is knocked back, FOV and fog-of-war
  are updated immediately.

---

## Inferred Design Intent

- **Accuracy values are percentage-like** — legacy DRL accuracy is expressed
  as an integer in approximately [0, 100]. DRL-Rust treats it as a percent
  chance to hit. This matches observed game feel. Uncertainty: low.
- **No critical hits in base combat** — legacy DRL does not appear to have
  a critical hit mechanic in standard melee/ranged combat. Uncertainty: medium
  (some traits or weapons may modify this).
- **Armor does not absorb all damage** — armor mitigation is additive
  reduction, not a percentage. Very high armor values could reduce most
  physical damage but do not provide full immunity. Uncertainty: medium.

---

## Legacy Implementation Artifacts

- **Accuracy falloff formula** — DRL-Rust uses a simplified linear falloff
  for ranged accuracy. The legacy Pascal formula may use a different curve
  (step function, quadratic, table lookup). This is a known approximation.
- **Minimum damage floor** — the exact minimum damage after armor mitigation
  (0 vs. 1) is not confirmed from legacy source. DRL-Rust currently allows 0.

---

## Deliberate DRL-Rust Decisions

- **Explicit damage type enum** — DRL-Rust models damage types (`Physical`,
  `Fire`, `Plasma`, etc.) as an enum variant rather than integer flags. This
  is a type-safety improvement that does not change observable behavior.
- **Range as u32, not float** — distances are computed on integer grid
  coordinates. No floating-point intermediate is used in range checking.

---

## Open Questions

- **Exact ranged accuracy formula** — what is the precise legacy formula for
  accuracy degradation with distance? Needs: Pascal combat resolution source.
- **Melee hit formula** — is melee accuracy always 100%, or does it use a
  skill-based roll? Needs: Pascal melee attack source.
- **Armor damage type interaction** — does armor protection apply equally to
  all damage types (physical, fire, plasma)? Or does each armor type have
  per-type resistances? Needs: Pascal/Lua armor stat tables.
- **Explosion / AoE damage** — legacy DRL has rocket launchers and other
  area-effect weapons. How does AoE damage resolve against multiple targets?
  Needs: Pascal explosion resolution source.
- **Damage upon contact / thorns** — do any legacy enemies deal damage on
  contact or on being hit? Needs: legacy special ability inventory.
- **Death animation vs. simulation** — does death in legacy DRL have a
  multi-frame linger state that affects gameplay (e.g., can a dying actor
  complete its current action)? DRL-Rust treats death as immediate.

---

## Non-Goals

- Trait and class modifiers to combat stats — deferred to M9.
- Special weapon modes (e.g., burst fire, alt-fire) — deferred to M9.
- Boss combat mechanics — deferred to M9.
- Lua-driven special abilities — deferred to M3/M9.
