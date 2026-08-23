---
name: drl-legacy-archaeology
description: Extract attributable DRL behavior and provenance evidence from legacy Pascal, Lua, data, manuals, and focused probes without copying legacy architecture or inventing certainty.
---

# DRL Legacy Archaeology

## When to Use

- Use this skill when a bounded roadmap slice depends on legacy gameplay,
  content, interface, or provenance evidence.
- Use it to locate rules, compare Pascal and Lua behavior, design a focused
  legacy probe, or characterize callback-heavy stress cases for the typed Rust
  behavior model.
- Do not use it for Rust architecture design unrelated to legacy behavior or to
  translate legacy modules mechanically.

## Required Inputs

- the selected roadmap slice plus the active `SPEC.md` contract or milestone
  owner's bounded scope;
- `docs/steering/current-priorities.md` when the work concerns behavior,
  content migration, or rights/provenance;
- the question to resolve and why it affects observable behavior;
- the available legacy checkout or other attributable source;
- the relevant DRL-Rust implementation and tests, when they exist.

If the legacy checkout is local, record its revision and dirty state before
using it as evidence. Missing source context is uncertainty, not permission to
infer a rule.

## Evidence Classes

Classify each material finding as one of:

- `observed`: directly supported by source, data, manual text, or recorded
  runtime probe;
- `inferred-intent`: reasoned interpretation supported by evidence but not
  directly demonstrated;
- `implementation-artifact`: legacy machinery that need not be preserved;
- `ambiguous`: missing, contradictory, version-dependent, or underdetermined;
- `drl-rust-decision`: an explicit new-project choice, not a legacy fact.

For callback-heavy content, separately record static fields, trigger/hook,
preconditions, state read, state mutated, ordering dependencies, costs, target
selection, emitted presentation effects, and unresolved ambiguity. This is the
input to a typed Rust behavior design; do not convert hook names directly into
a generic callback registry.

## Workflow

1. Restate the smallest behavioral/provenance question and evidence needed.
2. Record repository, revision, dirty state, build/frontend variant, and
   configuration relevant to the evidence.
3. Search the narrowest likely Pascal, Lua, data, manual, and test surfaces.
4. Trace both definition and use. A constant, callback declaration, or field
   alone does not prove runtime behavior.
5. For a runtime probe, write setup, ordered actions, observations, and
   uncertainty before interpreting the result.
6. For stochastic behavior, characterize rules/distributions with repeated
   probes. Do not use legacy random output as a DRL-Rust golden trace.
7. Classify every finding and cite source path, symbol/section, and revision.
8. When creative text or media is copied, record provenance and redistribution
   status separately from numeric/factual mechanics.
9. Compare evidence with the active specification and current tests. Identify
   missing decisions rather than making them for the milestone owner.
10. Return the evidence artifact to the milestone owner for synthesis.

## Outputs

For delegated work, write or return content shaped as
`_workspace/drl/{milestone}-{slice}/01-evidence.md` with:

- run identifier, owner and role;
- input and output repository state;
- predecessor artifact and revision, normally `00-scope.md`;
- question and scope;
- legacy source identity and repository state;
- sources inspected;
- observed rules;
- inferred intent;
- implementation artifacts;
- ambiguities or contradictions;
- callback/trigger behavior decomposition when relevant;
- candidate DRL-Rust decisions;
- associated or proposed tests;
- provenance/rights concerns;
- result status and unresolved questions.

Use `PASS`, `FAIL`, `INCONCLUSIVE`, or `NOT_RUN` with the existing team-spec
meanings. The milestone owner decides whether findings update
`docs/legacy-behavior/`, `docs/steering/`, `SPEC.md`, implementation, or roadmap
status.

## Stop Conditions

- Required legacy source/version is unavailable.
- A requested runtime probe cannot start.
- Pascal, Lua, data, manual, or observed behavior conflicts materially.
- A conclusion depends on an unrecorded local modification.
- Work would import legacy architecture, assets, code, or creative expression
  across an unresolved rights boundary.
- The question expands beyond the selected milestone slice.

## Validation

- Every behavioral claim has attributable evidence.
- Revision and dirty state are recorded when local legacy sources were used.
- Declarations are distinguished from demonstrated runtime behavior.
- Static fields are distinguished from callback behavior.
- Observed behavior, inferred intent, artifacts, ambiguities, and new decisions
  are visibly separate.
- Stochastic evidence does not claim exact legacy RNG compatibility.
- No completion/parity claim exceeds the evidence.
