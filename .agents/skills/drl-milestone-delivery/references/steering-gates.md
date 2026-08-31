# DRL Delivery Steering Gates

Last reviewed: 2026-08-31
Baseline: `main` at `180f7dd2d350b11c114ae4f5fdbc27ba12d32829`

## Purpose

This reference operationalizes the current near-term gates in
`docs/steering/current-priorities.md`. It does not create a second source of
truth: the roadmap remains canonical and `SPEC.md` defines exactly one active
slice.

If this file and the steering document diverge, the steering document is
authoritative and delivery pauses until this reference is reconciled.

## Gate A — Persistent histories bind their interpreter (closed by M10)

A slice that writes or restores a command history must bind the history to the
gameplay, RNG-sampling, generator, ruleset/content, and fixed-session identities
needed to interpret it. Validate compatibility before execution.

M10 delivered this invariant in merged PR #428 (`b0a36fa`) with direct,
browser-storage, and cross-version fixtures. Future replay-visible
gameplay-semantics changes must preserve it. Legacy tokens without semantic
provenance must be rejected or covered by an explicit evidenced migration; do
not assign them a current identity by assumption.

## Gate B — Fidelity work closes a semantic branch

Do not select the next value in a known plateau as a milestone slice. Group
behavior by evidenced state classes, transitions, saturation, resource policy,
reset rules, target behavior, and trait interactions.

For chainfire, per-level constants beyond the audited `0.2.318` boundary are
blocked. M9 merged as PR #430 (`180f7dd`) and covers the whole evidenced rule;
future work must name every intentional DRL-Rust difference.

Atomicity does not decide whether a partial action is accepted. It requires the
chosen accepted or rejected result to commit as one transaction.

M9 closes this gate for chainfire with six-family saturation and under-supply
vectors, replay/MCP/BrowserSession parity, and independent review. Gate C is
the next active stop gate.

## Gate C — Rollback has an exit budget

Rejected commands still require exact pre/post `Game` equality. Before adding
another unconditional state clone or scaling cohort workloads, record a
representative throughput/allocation baseline and assign transaction ownership
at core and outer boundaries.

Retain safety backstops until equivalent tests exist. Remove redundant outer
rollback or migrate hot paths toward validate/prepare/commit only from measured
evidence.

## Gate D — Scope and review are auditable

`SPEC.md` contains one active slice and no historical delivery ledger. A
replay-visible or legacy-fidelity slice requires an attributable independent
determinism-review result before acceptance.

The milestone owner records branch/input revision, gate, evidence, semantic
impact, checks, review disposition, and unavailable surfaces in the handoff or
PR. Hosted checks do not substitute for the review edge.

## Gate E — Claims remain evidence-bounded

Current-Rust tests prove current Rust behavior. Source inspection supports only
the attributable rule it shows. Runtime, balance, audiovisual, browser, and
performance claims require the relevant recorded evidence or remain
`NOT_RUN`/`INCONCLUSIVE`.

## Enduring architecture checks

The earlier steering wave delivered foundations that remain mandatory even
though they no longer define the priority order:

- rejected commands are exact state identity and consume no RNG;
- replay-visible changes make an explicit semantics decision;
- routine content identity is single-sourced where implemented;
- callback-heavy legacy behavior uses typed Rust policy, not runtime Lua or a
  generic callback bus;
- stable protocol contracts and core-owned gameplay policy remain separate;
- creative-expression provenance and redistribution status remain explicit.

## Slice-selection checklist

Before implementation, record:

- which current gate the slice closes or which named 1.0 criterion requires it;
- observable success criteria and explicit non-goals;
- branch and input revision;
- legacy evidence and classification required;
- replay, snapshot, RNG, generator, and ruleset impact;
- content-catalog and protocol/domain ownership impact;
- accepted and rejected transaction behavior;
- independent-review owner and expected artifact;
- local, hosted, runtime, browser, and capture checks that can actually run.

During review, an undocumented gate impact, invented migration provenance, or
another counter-only chainfire continuation is a `fix` disposition unless the
slice must be re-scoped entirely.
