# Repository Agents Guide

## What

- DRL-Rust is a ground-up Rust reimplementation of Doom the Roguelike.
- The project is currently a Rust 2024 binary scaffold, not a playable game.
- `docs/DRL-Rust_Project_Roadmap.md` is the canonical project plan and progress
  tracker. `SPEC.md` expands only the active milestone slice.
- `ARCHITECTURE.md` records verified current structure and invariants.
  `CHANGELOG.md` records delivered contributor- or user-visible changes.
- The legacy Pascal and Lua implementation is a behavioral reference, not an
  architecture to translate mechanically.

## Why

- Preserve game semantics while replacing legacy implementation machinery.
- Keep the future simulation deterministic, headless, and independent from
  graphics, audio, operating-system, filesystem, and MCP concerns.
- Distinguish implemented facts from the target design in the project proposal.

## How

- Before changing a milestone item, read the roadmap, the active `SPEC.md`
  slice, applicable architecture constraints, and relevant implementation or
  legacy evidence.
- Use `.agents/skills/drl-milestone-delivery/SKILL.md` for milestone work.
- Use spaces with an indentation and tab width of 2. Run:

  ```sh
  sh scripts/check-repository.sh
  ```

- Update the roadmap only from verified evidence. Keep incomplete work active
  and do not claim remote CI success until the remote check has passed.
