# Grammaton Cleric Beretta — Behavioral Specification

**Domain:** alternate reload and ranged-fire mode
**Milestone relevance:** M9 / Gate D behavior stress cases
**Last updated:** 2026-08-25
**Status:** Typed behavior-covered in DRL-Rust `0.2.128`; controlled legacy
runtime parity remains `NOT_RUN`

## Evidence identity and scope

- **Legacy repository:** `/Users/saehwan/repos/doom-the-roughlike-original`
- **Revision inspected:** `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`
- **Legacy working-tree state:** dirty at inspection time (`audio.lua` and
  `meta.lua` edits plus an untracked `fpcvalkyrie/` directory). Findings below
  come from immutable `git show <revision>:<path>` content.
- **Runtime probe:** `NOT_RUN`; no controlled Linux reference session was
  available for this slice.

## Sources inspected

- Lua perk declaration and callback: `bin/data/drl/items/uitems.lua`,
  `perk_uberetta_altreload`, lines 478–508.
- Lua item declaration: `bin/data/drl/items/uitems.lua`, `uberetta`, lines
  510–541.
- Existing DRL-Rust ranged command boundary:
  `crates/drl-core/src/game.rs`, `execute_player_ranged_attack`.

## Verified legacy behaviors (`observed`)

- The equipped Grammaton alternate-reload callback cycles modes in the order
  single → burst → full-auto → single based on its current accuracy marker.
- Single mode uses `2d6`, accuracy marker `5`, and no burst count; burst uses
  `1d8`, marker `3`, and three shots; full-auto uses `1d7`, marker `1`, and six
  shots (`uitems.lua:485–503`).
- Each mode change subtracts `200` from the actor's `scount` and returns an
  error-free callback result (`uitems.lua:505–506`).
- The item is a unique ranged 9mm weapon with an 18-round magazine and a
  `2d6` base damage declaration (`uitems.lua:510–529`).

## DRL-Rust decisions and boundaries

- `WeaponFireMode` is a typed protocol value and the core transition is a pure
  mode-cycle operation; no runtime Lua or generic callback registry is added.
- DRL-Rust's existing combat policy expresses accuracy as a percentage rather
  than the legacy raw marker. The current typed mapping is `80`/`75`/`70` for
  single/burst/auto until the legacy accuracy equation is audited.
- A ranged command consumes the selected mode's full shot count only after
  bounds, target, line-of-sight, weapon, range, and clip-capacity checks pass.
  Ordered shot events stop at the first lethal result and keep death-drop
  insertion bounded to one drop.
- The V1 replay wire format is unchanged. Gameplay-semantics identity advances
  to `7` because a mode transition changes subsequent command outcomes.

## Open questions and non-goals

- The legacy accuracy marker's exact hit equation and whether a partially
  loaded burst is permitted require a controlled runtime/source audit.
- Legacy mode-switch messages, sounds, and action-time accounting remain
  presentation/runtime evidence gaps.
- Other alternate reloads, generic fire-mode infrastructure, runtime Lua, and
  broad scalar content migration remain outside this slice.

## Rights/provenance

This note records numeric mechanics and control flow only. It adds no copied
creative text or media to Rust-owned content; existing release-rights records
remain authoritative.
