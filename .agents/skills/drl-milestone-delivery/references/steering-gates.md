# DRL Delivery Steering Gates

Last reviewed: 2026-08-23

## Purpose

This reference operationalizes the temporary near-term gates in
`docs/steering/current-priorities.md` for milestone delivery. It does
not create a second source of truth: the roadmap remains canonical and
`SPEC.md` still defines exactly one active slice.

The milestone owner applies these gates when selecting or accepting slices.
If this reference and `docs/steering/current-priorities.md` diverge, the steering
document is authoritative and this reference must be reconciled.

## Gate 1 — Atomic rejection

A code-path slice that adds or changes simulation commands cannot pass final
verification unless representative rejected paths prove exact pre/post `Game`
equality.

Known mutation-before-error behavior blocks feature breadth in the affected
subsystem until repaired.

## Gate 2 — Deterministic semantics

A slice that changes RNG sampling, combat probability, generation randomness,
item defaults used by replay reconstruction, or other replay-visible policy
must state whether gameplay semantics are preserved or advanced.

Cross-version replay compatibility must not be inferred from a stable wire
schema alone.

## Gate 3 — Content fan-out

Before accepting another broad batch of scalar-only content families, the
milestone owner must verify that routine content identity has a single
authoritative registration path or record why the proposed slice cannot wait
for that foundation.

A one-off content addition needed to exercise the behavior model is allowed.
Mass breadth is not.

## Gate 4 — Behavior evidence

For legacy items/traits/levels with callbacks, a completed scalar definition is
reported as definition coverage only.

Behavior-complete claims require:

- pinned legacy evidence;
- an explicit Rust behavioral representation;
- deterministic tests or scenarios;
- unresolved ordering/interaction questions recorded as gaps.

## Gate 5 — Vertical fidelity before infrastructure expansion

New MCP, release, platform, browser-target, or generalized framework work should
be selected only when one of these is true:

- it is required to close the active correctness/fidelity slice;
- it is required for a named 1.0 acceptance criterion;
- delaying it would create a concrete migration or security risk.

Otherwise prefer canonical gameplay fidelity work.

## Gate 6 — Protocol ownership

When a new stable semantic type is needed across boundaries, the milestone owner
must decide separately:

1. what belongs in the stable protocol contract; and
2. what gameplay policy belongs in the core/domain implementation.

Do not place balance values in `drl-protocol` solely for convenience.

## Gate 7 — Rights/provenance

When importing legacy creative text, graphics, audio, or other expression, the
slice must identify the source revision and the redistribution/evidence status.
Unknown status remains explicit. Numeric/factual mechanics and copied creative
expression should not be treated as the same rights category by default.

## Slice-selection checklist

Before implementation, the milestone owner records:

- which gate the slice closes or why it is exempt;
- observable success criteria;
- legacy evidence required;
- replay/RNG semantic impact;
- content-catalog impact;
- protocol/domain ownership impact;
- explicit non-goals.

During review, a determinism reviewer treats any undocumented gate impact as a
reason for `fix` or `blocked`, depending on whether the evidence can be repaired
within the bounded slice.
