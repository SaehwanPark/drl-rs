---
name: drl-legacy-archaeology
description: Extract attributable DRL behavior evidence from legacy Pascal, Lua, data, manuals, and focused probes without copying legacy architecture or inventing certainty.
---

# DRL Legacy Archaeology

## When to Use

- Use this skill when a bounded roadmap slice depends on legacy gameplay,
  content, interface, or provenance evidence.
- Use it to locate rules, compare Pascal and Lua behavior, design a focused
  legacy probe, or review a proposed behavioral note.
- Do not use it for Rust architecture design that does not depend on legacy
  behavior, or to translate legacy modules mechanically.

## Required Inputs

- the selected roadmap slice plus either the active `SPEC.md` contract or the
  milestone owner's bounded phase-one scope;
- the question to resolve and why it affects observable behavior;
- the available legacy checkout or other attributable source;
- the relevant DRL-Rust implementation and tests, when they exist.

If the legacy checkout is local, record its revision and dirty state before
using it as evidence. Missing source context is uncertainty, not permission to
infer a rule.

## Evidence Classes

Classify each material finding as one of:

- `observed`: directly supported by source, data, manual text, or a recorded
  runtime probe;
- `inferred-intent`: a reasoned interpretation supported by evidence but not
  directly demonstrated;
- `implementation-artifact`: legacy machinery that need not be preserved;
- `ambiguous`: missing, contradictory, version-dependent, or underdetermined;
- `drl-rust-decision`: an explicit new-project choice, not a legacy fact.

Keep source evidence and interpretation separate. One source location may
support more than one interpretation; record that disagreement rather than
collapsing it.

## Workflow

1. Restate the smallest behavioral question and the evidence needed to answer
   it.
2. Record repository, revision, dirty state, build or frontend variant, and
   configuration relevant to the evidence.
3. Search the narrowest likely Pascal, Lua, data, manual, and test surfaces.
   Expand only when references or conflicting behavior require it.
4. Trace both definition and use. A constant, command, or field declaration
   alone does not prove runtime behavior.
5. For a runtime probe, write the setup, ordered actions, observations, and
   uncertainty before interpreting the result. Use god or developer controls
   only to establish setup, and disclose their use.
6. For stochastic behavior, characterize rules or distributions with repeated
   probes. Do not use legacy random output as a DRL-Rust golden trace.
7. Classify every finding and cite its source path, symbol or section, and
   revision context.
8. Compare the evidence with the active specification and current tests.
   Identify missing decisions instead of making them for the milestone owner.
9. Return the evidence artifact to the milestone owner for synthesis.

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
- candidate DRL-Rust decisions;
- associated or proposed tests;
- provenance concerns;
- result status and unresolved questions.

Use `PASS` when the requested evidence was found and is coherent. Use
`INCONCLUSIVE` when accessible evidence is missing, contradictory, or unable
to resolve the question. Use `FAIL` when an attempted probe or evidence process
failed unexpectedly. Use `NOT_RUN` when a requested runtime probe could not
start because its executable, tool, or setup was unavailable.

The milestone owner, not the archaeologist, decides whether findings update
`docs/legacy-behavior/`, `SPEC.md`, implementation, or roadmap status.

## Stop Conditions

- Required legacy source or version is unavailable.
- A requested runtime probe cannot start because its executable, tool, or
  setup is unavailable.
- Pascal, Lua, data, manual, or observed behavior conflicts materially.
- A conclusion would depend on an unrecorded local modification.
- The requested work would import legacy architecture, assets, or code across
  an unresolved licensing boundary.
- The question expands beyond the selected milestone slice.

Stop with `NOT_RUN` for an unavailable runtime probe. Stop with `INCONCLUSIVE`
for missing or contradictory source evidence. Preserve the evidence trail
rather than choosing a convenient interpretation.

## Validation

- Every behavioral claim has attributable evidence.
- Revision and dirty state are recorded when local legacy sources were used.
- Declarations are distinguished from demonstrated runtime behavior.
- Observed behavior, inferred intent, artifacts, ambiguities, and new decisions
  are visibly separate.
- Stochastic evidence does not claim exact legacy RNG compatibility.
- No milestone, implementation, or completion claim exceeds the evidence.

## Browser and Asset Evidence

- When researching presentation, classify atlas layers, emissive/mask/shadow
  behavior, LUTs, particles, animation, HUD, and audio cues separately from
  gameplay rules.
- Record browser-reference capture setup as part of the evidence: legacy
  revision and dirty state, executable hash, frontend/configuration, viewport,
  DPR, capture tools, ordered actions, and whether media is committed or kept
  in ignored `_workspace/` storage.
- Never use a dirty legacy checkout for imported assets or reference captures;
  read Git objects at the named revision. Track each asset group independently
  so uncertain fonts/audio/music do not contaminate a cleared graphics import.
