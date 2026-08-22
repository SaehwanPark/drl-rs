# Reusable development lessons

These notes capture patterns verified while delivering the bounded M8–M10 and
M9 slices in this repository. They are intentionally short; the roadmap,
`SPEC.md`, `ARCHITECTURE.md`, and slice evidence remain the detailed sources of
truth.

## Separate current-Rust evidence from legacy parity

- **Context:** Many slices use the pinned Pascal/Lua project as behavioral
  reference.
- **Symptom:** A matching name or a similar rule can look like proof that the
  Rust implementation is legacy-equivalent.
- **Cause:** The legacy checkout contains broader/different content, may be
  dirty, and its available x86-64 Linux executable cannot run in the arm64
  macOS development environment.
- **Resolution:** Read attributable source with `git show` at the pinned
  revision; label current-Rust behavior `PASS`, legacy comparison
  `INCONCLUSIVE`, and unavailable runtime/capture work `NOT_RUN`.
- **Prevention:** Every slice evidence/review artifact must state provenance,
  environment, and the narrow claim it supports. Never turn source similarity
  into a numeric, visual, or balance-parity claim without a controlled probe.

## Prefer immutable tables with compatibility accessors

- **Context:** Item, monster, loot, tile, and level metadata was repeated in
  factories, generators, and protocol boundaries.
- **Symptom:** A later edit can update one match while leaving another with
  different values or RNG behavior.
- **Cause:** Content ownership is implicit in ordinary constructors and
  branch-heavy selection code.
- **Resolution:** Add a compile-time immutable table and a pure lookup;
  preserve existing constructors/accessors as wrappers. For generated content,
  test every threshold and compare fixed-seed output, RNG state, item counters,
  ordering, and payloads against the equivalent pre-table path.
- **Prevention:** Keep tables small and protocol-safe, avoid mutable registries
  or runtime loading, and make custom caller configuration an explicit escape
  hatch rather than silently replacing it with a default profile.

## Let the active scope supersede exploratory design

- **Context:** Discovery can identify several related policies at once; Loop 56
  evidence found both core-default and MCP room profiles.
- **Symptom:** Implementing every suggested row expands a bounded slice and may
  silently change a caller's behavior.
- **Cause:** Evidence describes opportunities, while the owner must choose one
  acceptance boundary.
- **Resolution:** Record the selected scope and test plan before coding. Loop
  56 delivered only `standard-procedural`; the MCP five-room literal stayed
  explicit, and the reviewer recorded the broader two-profile design as
  superseded planning rather than a missing implementation.
- **Prevention:** Keep one active slice in `SPEC.md`, list deferred profiles as
  non-goals, and require review to compare code against the active scope.

## Keep canonical documents synchronized and readable

- **Context:** Each merged slice changes implementation ownership and the
  roadmap's progress vocabulary.
- **Symptom:** A technically correct change becomes hard to trust when
  `SPEC.md`, architecture, changelog, roadmap, and README disagree.
- **Cause:** These documents serve different audiences and are easy to update
  partially.
- **Resolution:** Update them together from verified evidence: `SPEC.md` for
  the active contract, `ARCHITECTURE.md` for ownership/invariants,
  `CHANGELOG.md` for delivered behavior, and the roadmap for progress/open
  work. Keep README capability summaries as concise nested bullets instead of
  long list paragraphs.
- **Prevention:** Make canonical-document writes part of the slice checklist;
  keep README additions to the smallest useful bullet and preserve explicit
  open/NOT_RUN boundaries.

## Distinguish local browser limits from hosted acceptance

- **Context:** `sh scripts/check-web.sh` is runnable locally, but a local WASM
  browser runner is not always installed.
- **Symptom:** Native web contracts pass while browser tests cannot execute.
- **Cause:** Browser, GPU, audio, viewport, and OS capabilities are environment
  dependent; remote CI has a different controlled setup.
- **Resolution:** Report native contracts as `PASS` and the unavailable local
  browser portion as `NOT_RUN`; then wait for the hosted repository and
  WASM/browser jobs before merging. Do not infer human-play or capture parity
  from native tests.
- **Prevention:** Record browser/version/OS/GPU/DPR/audio details for real
  acceptance, and keep local `NOT_RUN` and hosted `PASS` as separate evidence.

## Use a repeatable PR handoff and clean branch state

- **Context:** The preferred loop uses temporary branches, independent review,
  hosted checks, and ignored evidence artifacts.
- **Symptom:** A merged change can leave stale remote branches or lose the
  exact verification record.
- **Cause:** `_workspace/drl/...` is intentionally ignored, and GitHub may
  delete a branch remotely while a local remote-tracking ref remains stale.
- **Resolution:** Keep scope/design/evidence/review/verification artifacts in
  the ignored workspace, put a concise summary and checks in the PR body, wait
  for every required hosted job, merge, then run `git fetch origin --prune` and
  verify `git status --short --branch` on `main`.
- **Prevention:** End every slice with a handoff containing source/merge
  revisions, local and hosted results, explicit NOT_RUN surfaces, and a clean
  branch check before measuring usage or selecting more work.
