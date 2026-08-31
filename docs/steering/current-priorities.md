# Near-Term Development Steering

Last reviewed: 2026-08-31
Baseline branch: `main`
Baseline merge commit: `262957e0d0471022a2128f8acba3120dd946f6e9`
Latest pull request inspected: `#426`
Baseline project version: `0.2.319`

## Purpose

This document constrains near-term slice selection after the progression audit
recorded in [`audit-2026-08-30-post-0.2.318.md`](audit-2026-08-30-post-0.2.318.md).
It does not replace the roadmap or the one active slice in `SPEC.md`.

The previous `0.2.88` steering wave successfully established command rejection
evidence, unbiased RNG sampling, explicit replay metadata, routine content
catalogs, and typed behavior foundations. Those results remain architecture and
test invariants. The gates below supersede the old slice-selection order because
the current risks are now persistent-history compatibility, semantic
micro-slicing, transaction cost, and control-plane drift.

## Current diagnosis

The architecture remains sound. The deterministic core, fair observations,
explicit RNG, replay/scenario tooling, browser/MCP boundaries, and provenance
checks should continue.

Development has nevertheless optimized for locally reviewable increments more
than for closing complete behavior rules. The clearest example is chainfire:
the legacy rule groups all levels `2..255`, while recent Rust delivery repeatedly
adds one plateau value and projects it through many boundaries. In parallel,
baseline browser command-history saves omit the gameplay identities that
interpret them, and the interim full-state rollback backstop has no measured
exit plan. The current M10 candidate binds those histories and is pending its
hosted PR/merge handoff before Gate A is retired.

Near-term work must reduce those risks before more scalar breadth or another
counter-level continuation becomes eligible.

## Priority order

Until the gates below close, select work in this order:

1. **Semantics-bound browser persistence (M10)**
   - ship a snapshot format that binds command history to gameplay,
     RNG-sampling, generator, ruleset, and fixed-content identities;
   - reject unsupported identities before executing commands;
   - handle legacy unbound tokens explicitly and transactionally.
2. **Whole-rule chainfire fidelity (M9)**
   - replace per-level plateaus with evidenced state classes and saturation;
   - decide ammunition shortage, target continuation/routing, reset, and trait
     interactions from pinned evidence or an explicit DRL-Rust policy;
   - verify equivalence classes and boundaries across core, replay, MCP, and
     browser without duplicating gameplay policy.
3. **Measured transaction ownership (M1/M11)**
   - establish command throughput/allocation baselines;
   - remove redundant boundary clones where core atomicity is sufficient;
   - move hot accepted paths toward validate/prepare/commit without weakening
     exact rejection identity.
4. **Reviewable control plane (M0)**
   - keep `SPEC.md` to one active slice;
   - require an attributable independent determinism review for every
     replay-visible or legacy-fidelity slice;
   - decide and enforce review/branch-protection policy before 1.0.
5. **Vertical canonical fidelity**
   - select bounded end-to-end mechanics that close a complete behavior branch.
6. **Controlled reference captures**
   - run when the required environment exists; unavailable work stays
     `NOT_RUN`.
7. **Resume broad content and platform expansion**
   - only after the applicable gates below close.

## Development stop gates

### Gate A — Persistent histories bind their interpreter

Do not merge another replay-visible gameplay-semantics change while browser
snapshots can silently replay an unversioned command history under current
rules.

Gate A closes when the active M10 slice proves that:

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

### Gate C — The rollback backstop has an exit budget

Do not add another unconditional outer game clone or materially expand cohort
scale before measuring the cost of the existing rollback path.

Gate C closes when a reproducible representative benchmark records accepted
and rejected command throughput/allocation behavior, transaction ownership is
explicit at core/browser/MCP boundaries, redundant cloning is removed where
safe, and any retained full-state clone has a documented reason and budget.

Exact `Game` equality on rejection remains an enduring invariant after this
temporary gate closes.

### Gate D — Canonical scope and review are independently auditable

Do not append delivered slice history to `SPEC.md` or accept a replay-visible
or legacy-fidelity slice without an attributable independent determinism-review
result.

Gate D closes as a temporary stop gate when structural checks prevent multiple
active specifications and the repository records how required review is
enforced. The enduring rule remains: `SPEC.md` expands one active slice, while
delivered history belongs in the roadmap, changelog, evidence notes, and Git.

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

Re-audit after snapshot V3 and chainfire semantic consolidation merge, or
before broad M9 migration resumes, whichever happens first. Name the branch,
latest inspected PR, merge commit, project version, audited tree, local checks,
hosted checks, and unavailable evidence.
