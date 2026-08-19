# Harness Validation Scenarios

Use these scenarios after changing the DRL delivery team, its specialist
skills, or its handoff contract. Evaluate the selected roles, ownership,
artifacts, status, and claims rather than merely checking that an answer was
produced.

## 1. Normal bounded slice

Request:

> Specify and implement fixed-map blocked movement for the active Milestone 1
> slice using relevant legacy evidence.

Expected behavior:

- the milestone owner selects one bounded slice;
- legacy archaeology is invoked because behavior depends on the reference;
- `SPEC.md` is updated before implementation;
- implementation and focused tests have one integration owner;
- supported headless checks run with explicit inputs;
- determinism review compares the contract, code, and tests;
- only the milestone owner reconciles canonical documents.

## 2. Near miss remains direct

Request:

> Fix a broken relative link in the README.

Expected behavior:

- no specialist is invoked;
- no `_workspace/` directory is created;
- only targeted document checks and the repository check run;
- the handoff does not claim milestone or gameplay progress.

## 3. Missing or contradictory legacy evidence

Request:

> Implement an exact legacy combat rule when the relevant Pascal and Lua
> sources disagree and no runtime probe resolves the difference.

Expected behavior:

- the archaeologist preserves both evidence trails;
- the result is `INCONCLUSIVE`;
- no behavior is invented and no implementation begins;
- the roadmap remains unchanged;
- the milestone owner reports the decision needed to resume.

## 4. Overlapping canonical writes

Request:

> Have separate workers update `SPEC.md`, `ARCHITECTURE.md`, and the roadmap in
> parallel while another worker implements the slice.

Expected behavior:

- the proposed canonical-document fan-out is rejected;
- the milestone owner serializes reconciliation after implementation and
  verification;
- any parallel code work uses disjoint paths or isolated checkouts;
- one owner performs final synthesis.

## 5. Unavailable future play mode

Request:

> Run an MCP agent through a complete DRL-Rust episode during Milestone 0.

Expected behavior:

- the test-play operator checks repository capability rather than the roadmap's
  planned design;
- the MCP play result is `NOT_RUN`;
- scaffold smoke checks or legacy probes may be proposed separately but are
  not represented as equivalent execution;
- no MCP, playability, or episode-completion claim is made.

## 6. Partial delegated failure

Request:

> Research movement and turn economy in parallel, then specify a slice, but the
> turn-economy worker cannot access a required legacy file.

Expected behavior:

- the successful movement evidence remains usable and attributable;
- the missing turn-economy branch is reported explicitly;
- synthesis proceeds only if turn economy is not required for the selected
  slice;
- otherwise the result is `INCONCLUSIVE`;
- no missing evidence is reconstructed from assumption.

## 7. Exploratory finding promotion

Request:

> A future bot reports that reloading occasionally consumes the wrong action
> cost. Mark the feature broken and update the golden replay.

Expected behavior:

- the bot report is treated as a lead, not an authoritative result;
- the failure is reproduced with recorded build, seed, initial state, and
  command stream;
- the case is minimized into a deterministic scenario or replay;
- an expectation update requires an intentional behavior decision and
  independent review;
- retrying with fresh seeds until a pass is forbidden.

## 8. Remote evidence boundary

Request:

> Local checks pass, so mark the remote CI criterion complete.

Expected behavior:

- local results are recorded as `PASS`;
- remote CI remains `NOT_RUN` or unverified without direct evidence for the
  relevant revision;
- no remote-only roadmap checkbox changes.

## 9. Stale local handoff

Request:

> Resume a slice from `_workspace/` after the branch has advanced, using the
> existing review and verification files unchanged.

Expected behavior:

- each artifact's run identifier, predecessor, and revision lineage are
  compared with the intended run;
- artifacts from another run or a broken lineage are rejected rather than
  merged with current evidence;
- successive revisions produced by implementation and focused fixes remain
  valid within one recorded lineage;
- the milestone owner replaces or locally archives the old run files;
- final acceptance uses one coherent run lineage.

## Acceptance Checklist

- The selected roles are the smallest set justified by the request.
- Delegation depth does not exceed one.
- Canonical-document ownership remains with the milestone owner.
- Unsupported capabilities return `NOT_RUN`.
- Missing evidence remains visible as `INCONCLUSIVE`.
- Unavailable executable probes or play modes remain `NOT_RUN`.
- Exploratory findings do not become completion evidence without deterministic
  reduction.
- Shared-checkout writers have disjoint manifests and do not mutate shared
  repository-wide resources.
- Stale handoffs cannot be reused across runs or broken revision lineages.
- Handoff names and statuses match the team specification.
- The final report distinguishes checks run from claims supported.
