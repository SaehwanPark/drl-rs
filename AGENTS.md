# Repository Agents Guide

## What

- DRL-Rust is a ground-up Rust reimplementation of Doom the Roguelike.
- The project is a Rust 2024 workspace with a deterministic headless kernel
  and a bounded browser/WASM playable slice; full audiovisual parity remains
  staged.
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
- For coordinated work, follow `docs/harness/drl-delivery/team-spec.md`; keep
  one milestone owner and serialize canonical-document writes.
- Use `.agents/skills/drl-test-play/SKILL.md` for test play. Run only modes
  enabled by implemented repository capabilities.
- Use spaces with an indentation and tab width of 2. Run:

  ```sh
  sh scripts/check-repository.sh
  ```

- Update the roadmap only from verified evidence. Keep incomplete work active
  and do not claim remote CI success until the remote check has passed.
- Browser acceptance records browser/version, OS, GPU backend, viewport, DPR,
  build revision, and audio state. Unsupported WebGPU/audio or unavailable
  Linux legacy captures are `NOT_RUN`, not inferred passes.
- Project versioning is canonical in `VERSION` and follows
  [docs/VERSIONING.md](docs/VERSIONING.md): code changes require one valid
  `x.y.z` transition, while documentation-only and setting-only changes do
  not bump the version. Run `scripts/check-version.sh`; CI supplies
  `DRL_VERSION_BASE` so the agent harness can enforce the transition.
