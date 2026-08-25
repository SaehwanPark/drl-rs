# Movement — Behavioral Specification

**Domain:** movement
**Milestone relevance:** M1, M4
**Last updated:** 2026-08-25
**Status:** Source-informed, runtime comparison pending

---

## Evidence Sources

- **Observed game behavior** — DRL-Rust headless simulation, M1 and M4 test
  suites (`crates/drl-core/tests/simulation.rs`, `visibility.rs`,
  `level_progression.rs`).
- **Legacy Pascal source** — pinned revision
  `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`, specifically
  `TDRL.HandleMoveCommand`, `TBeing.TryMove`, `TBeing.MoveTowards`, and
  `TLevel.isEmpty`/`isPassableExt`. The inspected legacy checkout was dirty;
  immutable `git show` output was used, as recorded in
  `_workspace/drl/movement-corner-cutting/01-evidence.md`.
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
  `CommandError::BlockedByTerrain`.
- **Actor occupancy** — two actors cannot occupy the same tile. Moving into
  an occupied tile with an enemy triggers a melee bump-attack rather than
  movement (bump-attack semantics are in `combat.md`).
- **Diagonal movement** — diagonal movement (e.g., `NE`, `SW`) is allowed
  and occupies a single step. No special cost or restriction beyond normal
  action cost applies in the current implementation. A focused test also
  records that a walkable diagonal destination remains enterable when both
  adjacent cardinal tiles are walls (destination-only validation).
- **Cost of movement** — each movement step costs one unit of action energy
  at normal speed. See `turn-economy.md` for the energy scheduling model.
- **Level exit** — the player moves onto `Tile::StairsDown` by normal
  movement, then uses `Command::Descend` to transition levels. The stairs
  tile itself is walkable.

---

## Legacy Source Findings

- **Direct player movement is destination-only** — `HandleMoveCommand`
  computes one target and delegates to `Player.TryMove`; `TryMove` checks that
  target's bounds, blocking flags/items, hazards, and occupancy. It does not
  inspect the two cardinal neighbors of a diagonal target. The pinned source
  therefore permits corner cutting when the diagonal destination is valid.
- **AI movement is separate** — `MoveTowards` tries a smoothed preferred step,
  retries the raw direction after a block, then tries horizontal and vertical
  cardinal candidates in that order. It does not search every remaining
  neighbor, and the fallback is not a direct-player diagonal restriction. The
  bounded AI behavior is documented in `ai.md`.
- **8-directional movement** — the source accepts a direction computed from
  input, and DRL-Rust's eight-direction command contract is consistent with
  the inspected movement paths. Controlled runtime confirmation is pending.
- **Diagonal cost parity** — source inspection of `TryMove` and
  `HandleMoveCommand` does not establish the final action-cost rule; the
  legacy `getMoveCost` implementation and runtime timing remain open.

---

## Legacy Implementation Artifacts

- `MoveTowards`'s bounded candidate order is an AI/pathing implementation
  detail, not evidence for adding corner-cutting restrictions to direct player
  input.

---

## Deliberate DRL-Rust Decisions

- **Destination-only direct movement** — DRL-Rust intentionally keeps the
  direct player path free of adjacent-cardinal corner checks. This matches the
  pinned source shape and is protected by
  `diagonal_movement_allows_destination_only_corner_cutting`. AI fallback
  behavior remains a separate policy and is not generalized into player
  movement.

---

## Open Questions

- **Controlled corner-cutting comparison** — source evidence supports
  destination-only validation, but a canonical runtime/capture probe is still
  `NOT_RUN` because the legacy executable environment is unavailable.
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

- Monster AI movement policy — covered in `ai.md`; broad pathfinding remains
  out of scope.
- Targeting and line-of-sight — see `targeting` module documentation and
  planned `targeting.md`.
- Level generation and room/corridor layout — deferred to generation domain
  documentation (Milestone 9).
