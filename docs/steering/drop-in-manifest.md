# Steering Overlay Manifest

Prepared: 2026-08-23
Reviewed repository baseline: `6783f87439ee2708c7d46e2d15d899e7f4b8d9f8`
Baseline project version: `0.2.88`

This file records the files intentionally supplied by the steering overlay.
Extract the archive at the repository root.

## New steering documents

- `docs/steering/README.md`
- `docs/steering/current-priorities.md`
- `docs/steering/audit-2026-08-23.md`
- `docs/steering/decisions/atomic-command-transactions.md`
- `docs/steering/decisions/content-catalog-and-typed-behavior-model.md`
- `docs/steering/decisions/replay-semantics-and-rng-stability.md`

## Agent harness

- `.agents/skills/drl-milestone-delivery/SKILL.md`
- `.agents/skills/drl-milestone-delivery/references/steering-gates.md`
- `.agents/skills/drl-determinism-review/SKILL.md`
- `.agents/skills/drl-legacy-archaeology/SKILL.md`
- `.agents/skills/drl-test-play/SKILL.md`
- `.agents/skills/drl-test-play/references/test-play-modes.md`

## Existing files replaced with reconciled versions

- `AGENTS.md`
- `CONTRIBUTING.md`
- `SPEC.md`
- `ARCHITECTURE.md`
- `docs/adr/0001-project-architecture-principles.md`
- `docs/adr/0003-semantic-command-model.md`
- `docs/adr/0004-explicit-deterministic-rng.md`
- `docs/adr/0008-build-time-legacy-content-migration.md`
- `docs/harness/drl-delivery/team-spec.md`
- `docs/release-rights.md`
- `scripts/check-agent-harness.sh`

## Compatibility replacement

- `docs/audit-feedback-20260823.md` is replaced by a short pointer to
  `docs/steering/audit-2026-08-23.md` so the historical path cannot become a
  competing steering authority.

## Intentionally unchanged

The project roadmap remains the canonical milestone scope/progress history and
is not rewritten by this overlay. Its relationship to temporary steering is
made explicit in `AGENTS.md`, `CONTRIBUTING.md`, `SPEC.md`, and
`docs/steering/README.md`. `CHANGELOG.md` is also unchanged because this overlay
prepares steering/project documentation rather than claiming delivered runtime
behavior.

The repository's normal version policy remains in force. This overlay updates
harness validation to require the new steering gate reference; integration
should follow the repository's existing version check without special casing.
