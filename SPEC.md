# Specification

## Document Contract

The [project roadmap](docs/DRL-Rust_Project_Roadmap.md) is the canonical plan
for milestone scope, order, status, and exit criteria. This file expands only
the active implementation slice into observable outcomes and verification. It
does not replace or duplicate the full roadmap.

## Past

- The repository, Rust 2024 binary scaffold, license, proposal, roadmap, and
  local legacy-asset research location were established before this
  specification workflow was adopted.

## Present

### Milestone 0: Documentation and Harness Foundation

Status: Active

This slice establishes the operating contract for later milestone work.

Observable outcomes:

- repository-wide agent guidance identifies canonical documents and checks;
- one repo-local skill drives a bounded roadmap slice through specification,
  implementation, verification, and documentation reconciliation;
- a staged delivery team contract keeps one milestone owner while selectively
  routing legacy archaeology, test play, and determinism review;
- test-play modes are activated only by implemented repository capability, with
  unsupported modes reported as `NOT_RUN` and unresolved evidence as
  `INCONCLUSIVE`;
- optional cross-agent handoffs use deterministic ignored paths and statuses
  without duplicating canonical project state;
- architecture, specification, and changelog state distinguish verified facts
  from planned design;
- repository-controlled text uses spaces with an indentation and tab width of
  2, with local and macOS CI checks;
- the README accurately describes the current scaffold, future direction,
  developer workflow, optional legacy research setup, and licensing boundary.

Verification:

- `sh scripts/check-repository.sh` succeeds locally;
- `sh scripts/check-agent-harness.sh` validates skill frontmatter, required
  contracts and harness paths, plus required handoff and status vocabulary;
- `git diff --check` succeeds;
- document links and source-of-truth relationships agree across `AGENTS.md`,
  this file, `ARCHITECTURE.md`, `CHANGELOG.md`, and the roadmap;
- remote CI remains unverified until a GitHub Actions run passes.

Out of scope:

- converting the package into the planned multi-crate workspace;
- implementing gameplay or legacy behavior specifications;
- running DRL-Rust gameplay, replay, bot, MCP, statistical, or human play
  modes that have not been implemented;
- creating ADR, contribution-policy, or provenance inventories;
- selecting Lua, rendering, audio, ECS, or MCP dependencies.

## Future

Select the next bounded Milestone 0 slice from the canonical roadmap after this
foundation is verified. Load only that slice into `Present`; do not move work
from the roadmap merely to populate this document.
