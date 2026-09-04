# Near-Term Development Steering

Last reviewed: 2026-09-04
Baseline branch: `main`
Baseline merge commit: `8e0d5f1`
Latest pull request inspected: `#462`
Baseline project version: `0.2.345`

## Purpose

This document constrains near-term slice selection after the progression audit
recorded in [`audit-2026-08-30-post-0.2.318.md`](audit-2026-08-30-post-0.2.318.md).
It does not replace the roadmap or the one active slice in `SPEC.md`.

The previous `0.2.88` steering wave successfully established command rejection
evidence, unbiased RNG sampling, explicit replay metadata, routine content
catalogs, and typed behavior foundations. Those results remain architecture and
test invariants. The gates below supersede the old slice-selection order because
the remaining risks are semantic micro-slicing and control-plane drift;
persistent-history compatibility and transaction ownership are protected
invariants, not open gates, after Gate A and Gate C closure.

## Current diagnosis

The architecture remains sound. The deterministic core, fair observations,
explicit RNG, replay/scenario tooling, browser/MCP boundaries, and provenance
checks should continue.

Development has nevertheless optimized for locally reviewable increments more
than for closing complete behavior rules. The clearest example is chainfire:
the legacy rule groups all levels `2..255`, while recent Rust delivery repeatedly
adds one plateau value and projects it through many boundaries. In parallel,
pre-M10 browser command-history saves omitted the gameplay identities that
interpret them, and the interim full-state rollback backstop had no measured
exit plan. M10 binds those histories and Gate A is closed; M9 closes the
whole-rule chainfire branch and Gate B; M1/M11 measures transaction cost and
closes Gate C. M0 now records and enforces the control-plane review policy;
persistent-history compatibility, transaction ownership, and review enforcement
are protected invariants rather than open gates.

Near-term work may resume bounded vertical canonical-fidelity slices; broad
scalar-only migration remains ineligible while any applicable evidence or
behavior gate is open.

## Priority order

With the temporary control-plane gates closed, select work in this order:

1. **Reviewable control plane (M0, closed)**
   - keep `SPEC.md` to one active slice;
   - require an attributable independent determinism review for every
   replay-visible or legacy-fidelity slice;
   - retain the required review and branch-protection policy before 1.0.
2. **Vertical canonical fidelity**
   - select bounded end-to-end mechanics that close a complete behavior branch.
3. **Controlled reference captures**
   - run when the required environment exists; unavailable work stays
     `NOT_RUN`.
4. **Resume broad content and platform expansion**
   - only after the applicable gates below close.
   - the platform track follows the order in
     [`audit-2026-09-02.md`](audit-2026-09-02.md) §13: modularize `drl-web`,
     establish Linux CI, define the native frontend boundary, create
     `drl-desktop`, then record Fedora/Wayland/Vulkan acceptance. `drl-web`
     is refactored rather than copied, and no gameplay or presentation-policy
     fork is introduced for Fedora.
   - step 1 delivered in PR #457 (`0.2.341`, merged as `85e50c4`); step 2, Linux
     and Fedora CI coverage, delivered in PR #458 (`0.2.342`, merged as `4aaa010`).
     Step 3, the native frontend boundary, was delivered in PR #460
     (`0.2.343`, merged as `ee38357`); it defines the shared scene/session
     contract and the thin `drl-desktop` scaffold without opening native
     productization. The next platform item is Fedora 43 GNOME/Mutter
     Wayland/Mesa/RADV Vulkan interactive acceptance, which remains `NOT_RUN`.

## Development stop gates

### Gate A — Persistent histories bind their interpreter (closed)

M10 closed this gate in merged PR #428 (`b0a36fa`). Do not merge another
replay-visible gameplay-semantics change that weakens the resulting invariant:
browser snapshots must not silently replay an unversioned command history
under current rules.

The delivered evidence proves that:

- new saves carry the semantic identities required to interpret them;
- mismatches reject before simulation;
- rejected restore preserves the active session and saved token;
- V1/V2 tokens are rejected or migrated only through an evidenced policy that
  does not invent missing provenance;
- direct, browser-storage, and cross-version fixtures cover the contract.

### Gate B — Fidelity slices close semantic branches, not counters

Do not accept a slice whose primary outcome is the next chainfire warm-up level
or another identical plateau constant.

Gate B closes for chainfire when one typed model covers the evidenced initial,
second, sustained, and saturated states plus ammunition shortage, reset, target
continuation/routing, and applicable weapon traits. Canonical differences must
be identified as DRL-Rust decisions. Atomicity describes how an accepted or
rejected result commits; it does not choose which result is canonical.

M9 closes this gate in merged PR #430 (`180f7dd`) with the shared typed model,
six-family saturation vectors, atomic under-supply checks, replay/MCP/
BrowserSession parity, and an independent determinism review. M1/M11 closes
Gate C in merged PR #432; M0 now closes the temporary control-plane gate.

### Gate C — The rollback backstop has an exit budget

Do not add another unconditional outer game clone or materially expand cohort
scale before measuring the cost of the existing rollback path.

Gate C closes when a reproducible representative benchmark records accepted
and rejected command throughput/allocation behavior, transaction ownership is
explicit at core/browser/MCP boundaries, redundant cloning is removed where
safe, and any retained full-state clone has a documented reason and budget.

Exact `Game` equality on rejection remains an enduring invariant after this
temporary gate closes.

M1/M11 closed this gate in merged PR #432 (`1cd4233`) at version `0.2.322`.
The five-case contract benchmark records accepted and rejected throughput and
allocation behavior; the redundant BrowserSession outer snapshot is removed;
the core one-snapshot backstop has an explicit prepare/commit exit condition;
MCP candidate clones remain fair-observation admission probes; and inventory
staging remains local atomicity. Local and hosted checks plus independent
determinism and code reviews were reconciled against the merged revision.

### Gate D — Canonical scope and review are independently auditable (closed)

Do not append delivered slice history to `SPEC.md` or accept a replay-visible
or legacy-fidelity slice without an attributable independent determinism-review
result.

Gate D closes as a temporary stop gate when structural checks prevent multiple
active specifications and the repository records how required review is
enforced. The enduring rule remains: `SPEC.md` expands one active slice, while
delivered history belongs in the roadmap, changelog, evidence notes, and Git.

M0 delivered the structural SPEC checker and deterministic fixture contract in
merged PR #434 (`e598ae2`), then delivered the independent-review receipt and
branch-protection policy in merged PR #436 (`49add3a`). The exact current-head
receipt is independently reviewed, deterministic fixtures pass, and the live
`main` API reports one approving review, stale-review dismissal, strict status
updates, the `Repository checks`, `WASM browser checks`, and `Review policy`
contexts, with `enforce_admins=false` recorded as the temporary solo-maintainer
exception.

### Gate E — Claims remain evidence-bounded

Source similarity, a matching name, a copied scalar, or a current-Rust test is
not controlled legacy parity. Runtime, audiovisual, balance, browser, and
performance claims require the relevant environment and recorded evidence or
remain `NOT_RUN`/`INCONCLUSIVE`.

This gate is a permanent evidence discipline rather than a task that broad
content work can “complete.”

## Slice-selection rules

A candidate slice is eligible only when it:

- closes one named gate or is required by a named 1.0 acceptance criterion;
- states observable outcomes and explicit non-goals before implementation;
- identifies replay, persistent-history, RNG, generator, and ruleset impact;
- distinguishes legacy observations from DRL-Rust policy decisions;
- uses typed, core-owned gameplay policy and thin boundary projections;
- includes rejection/rollback behavior and cross-version fixtures where
  applicable;
- has a bounded independent review surface.

Reject or rescope a candidate that:

- adds the next value in an already-understood plateau;
- treats full-volley rejection as a consequence of transaction atomicity;
- replays an old command history under current rules without matching semantic
  identities;
- adds a wrapper clone without establishing why the core transaction is
  insufficient;
- expands scalar content or tooling while a relevant stop gate is open;
- claims parity or performance from unavailable evidence.

## Preferred architecture shape

```text
persistent Command history + semantics identity
                    |
                    v
          compatibility validation -----> mismatch: no execution
                    |
                    v
semantic Command -> validate / prepare -----> rejected: no state change
                    |
                    v
              PreparedAction
                    |
                    v
          commit deterministic mutation
                    |
                    +--> GameEvent stream
                    +--> next Game state

pinned legacy evidence
        |
        v
typed state classes + explicit DRL-Rust decisions
        |
        +--> boundary projections (replay / MCP / browser)
```

## Progress language

Keep these claims separate:

- **definition-covered** — scalar/static metadata migrated;
- **behavior-covered** — relevant Rust runtime mechanics implemented and
  tested;
- **legacy-compared** — controlled comparison with canonical runtime/evidence;
- **presentation-compared** — controlled visual/audio comparison;
- **repeatable-current** — the same declared semantics repeat on the current
  implementation;
- **cross-version-compatible** — explicitly supported identities or migration
  prove compatibility.

## Retirement and re-audit

Retire each temporary gate when its acceptance evidence exists, promote durable
invariants into accepted architecture/ADRs as appropriate, and update the
roadmap and active specification from verified evidence.

Re-audit before broad M9 migration resumes, or when a new stop-gate candidate
is selected. The current audited tree is `main` at `8e0d5f1` (PR #462 merge,
version `0.2.345`); local workspace/Clippy/repository/web/version/SPEC checks
pass, the independent determinism review is PASS, and hosted Repository,
Linux, Fedora, and WASM checks pass in run `33921025782`. The protected-path
Review policy check `33921025763` failed closed because the sole maintainer
cannot create a non-self approval and was merged under the live
`enforce_admins=false` exception. The Null Pointer SPLASMA armor divisor is
delivered; no subsequent bounded candidate is selected. Platform interactive
acceptance, controlled legacy runtime, human, audiovisual, performance, and
reference-capture surfaces remain `NOT_RUN` where prerequisites are
unavailable.
