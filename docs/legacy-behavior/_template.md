# Legacy Behavior Specification Template

Use this template to create a new behavior-spec document. Fill in sections
based on available evidence. Replace `[FEATURE]` with the feature name.

---

## `[FEATURE]` — Behavioral Specification

**Domain:** `[e.g., combat / movement / inventory / AI]`
**Milestone relevance:** `[e.g., M2, M4]`
**Last updated:** YYYY-MM-DD
**Status:** `Shell` | `Partial` | `Complete`

---

## Evidence Sources

List what has been examined. Be specific about file names, function names, or
line ranges where useful. Distinguish source types:

- **Pascal source** — `[filename.pas, function/unit name]`
- **Lua source** — `[filename.lua, function name]`
- **Game manual / in-game text** — `[quote or reference]`
- **Observed game behavior** — `[tested scenario description]`

---

## Verified Behaviors

List what is known with high confidence. Each item should be traceable to at
least one evidence source above.

- `[Behavior A]` — source: `[Pascal unit / Lua file / observed]`
- `[Behavior B]` — source: `[...]`

---

## Inferred Design Intent

List what is believed to be the intended design but is not fully confirmed.
Distinguish from verified facts.

- `[Probable behavior X]` — inferred from `[evidence]`, uncertainty: medium
- `[Probable behavior Y]` — inferred from `[evidence]`, uncertainty: high

---

## Legacy Implementation Artifacts

List behaviors that appear to be incidental to the Pascal implementation
rather than intentional game design. These are candidates for deliberate
deviation in DRL-Rust.

- `[Quirk A]` — likely artifact of `[reason]`; DRL-Rust may diverge

---

## Deliberate DRL-Rust Decisions

List where DRL-Rust intentionally deviates from the legacy behavior and why.

- `[Decision A]` — rationale: `[reason]`

---

## Open Questions

List what remains unknown or unresolved. These block full specification.

- `[Question A]` — needs: `[evidence type]`
- `[Question B]` — depends on: `[another domain decision]`

---

## Non-Goals

List related behaviors explicitly excluded from this document's scope.

- `[Excluded topic A]` — covered in `[other-doc.md]` or deferred to `[Milestone N]`
