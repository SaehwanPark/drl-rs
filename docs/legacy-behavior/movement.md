# Movement — Behavioral Specification

**Domain:** movement
**Milestone relevance:** M1, M4
**Last updated:** 2026-08-20
**Status:** Partial

---

## Evidence Sources

- **Observed game behavior** — DRL-Rust headless simulation, M1 and M4 test
  suites (`crates/drl-core/tests/simulation.rs`, `visibility.rs`,
  `level_progression.rs`).
- **Legacy Pascal source** — available locally for reference but not yet
  exhaustively audited for movement semantics.
- **Game manual** — not systematically reviewed at this time.

---

## Verified Behaviors

These behaviors are confirmed by DRL-Rust implementation and test suites.

- **Grid movement** — movement is tile-by-tile on a 2D integer grid.
  Positions are `(row, col)` values; each step moves exactly one tile in
  one of eight compass directions (`N`, `NE`, `E`, `SE`, `S`, `SW`, `W`, `NW`).
- **Bounds enforcement** — movement outside the map bounds is rejected with
  `CommandError::OutOfBounds`. The player cannot move off the edge of a level.
- **Terrain blocking** — `Tile::Wall` and other non-walkable tiles block
  movement. Attempting to walk into a wall returns
  `CommandError::TileNotWalkable`.
- **Actor occupancy** — two actors cannot occupy the same tile. Moving into
  an occupied tile with an enemy triggers a melee bump-attack rather than
  movement (bump-attack semantics are in `combat.md`).
- **Diagonal movement** — diagonal movement (e.g., `NE`, `SW`) is allowed
  and occupies a single step. No special cost or restriction beyond normal
  action cost applies in the current implementation.
- **Cost of movement** — each movement step costs one unit of action energy
  at normal speed. See `turn-economy.md` for the energy scheduling model.
- **Level exit** — the player moves onto `Tile::StairsDown` by normal
  movement, then uses `Command::Descend` to transition levels. The stairs
  tile itself is walkable.

---

## Inferred Design Intent

- **8-directional movement** — DRL legacy appears to support 8-directional
  movement (numpad-based). DRL-Rust implements this. Confirmation against
  Pascal movement code pending.
- **Diagonal cost parity** — in the legacy game, diagonal movement likely
  costs the same as cardinal movement (no Euclidean distance penalty).
  DRL-Rust treats them equally. This may diverge from strict action-cost
  fidelity if the legacy system uses fractional energy costs for diagonals.
  Uncertainty: medium.

---

## Legacy Implementation Artifacts

- No movement-specific artifacts identified yet. Pending Pascal source review.

---

## Deliberate DRL-Rust Decisions

- **No free diagonal smoothing** — some roguelikes apply terrain-crossing
  rules that prevent "cutting corners" around walls. DRL-Rust does not
  currently apply such restrictions. If legacy DRL does apply corner-cutting
  rules, this will be addressed as a verified behavioral change.

---

## Open Questions

- **Corner-cutting rules** — does legacy DRL prevent diagonal movement through
  a tile pair where both adjacent cardinal tiles are walls? Needs: Pascal
  source inspection of the movement validation function.
- **Diagonal action cost** — does legacy DRL apply a fractional (×1.4) energy
  cost for diagonal steps? Needs: Pascal source inspection of action cost
  assignment.
- **Swim / terrain-type movement modifiers** — does legacy DRL have
  terrain-type movement cost multipliers (e.g., liquid, rubble)? Needs:
  Pascal/Lua map tile type inventory.
- **Push/knockback as forced movement** — knockback is implemented in
  DRL-Rust as a separate resolution path. Confirmation that knockback
  bypasses normal movement validation (e.g., no action cost, no bump-attack
  trigger) is needed from legacy source.

---

## Non-Goals

- Pathfinding AI movement — covered in `ai.md` (not yet created).
- Targeting and line-of-sight — see `targeting` module documentation and
  planned `targeting.md`.
- Level generation and room/corridor layout — deferred to generation domain
  documentation (Milestone 9).
