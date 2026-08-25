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

## Advance replay semantics for deterministic policy changes

- **Context:** Replay compatibility covers every deterministic gameplay rule,
  not only random-number streams.
- **Symptom:** A movement-policy correction can make an old command history
  produce different actor positions even when the AI decision consumes no RNG.
- **Cause:** Replay execution reinterprets the command history under the
  current policy unless the gameplay-semantics identity distinguishes the two
  rule sets.
- **Resolution:** Advance `CURRENT_GAMEPLAY_SEMANTICS_VERSION` for the policy
  change and add a cross-version rejection test with the previous value. Keep
  the wire/schema version separate from gameplay semantics.
- **Prevention:** For every deterministic behavior change, inspect replay
  metadata before merging; document the new identity in `crates/drl-protocol`
  and reject older envelopes until an explicit migration exists.

## Do not confuse callback results with dispatch success

- **Context:** Legacy Lua perks can return `true` or `false` from action hooks
  such as alternate fire and alternate reload.
- **Symptom:** A direct port may treat the Lua return value as the command's
  accepted/rejected result and accidentally change action timing or rejection
  atomicity.
- **Cause:** The Pascal `CallHookCheck` boundary uses protected-call success as
  its Boolean and does not necessarily expose the callback's returned Lua
  value. The Subtle Knife and Trigun evidence notes both show this mismatch.
- **Resolution:** Inspect the native wrapper before assigning meaning to a
  callback return. Separate dispatch success, gameplay effect eligibility,
  feedback, and time/action cost in the typed Rust model.
- **Prevention:** Add an explicit accepted/rejected/feedback result to each
  behavior transition and test no-effect branches independently from transport
  or callback invocation success.

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

## Treat each new content archetype as an exhaustive fan-out

- **Context:** Adding an item or armor family crosses the spawn enum, display
  and JSON protocol, validation, factories, definition tables, and asset
  descriptors.
- **Symptom:** The new entry appears in one registry but compilation or asset
  coverage checks fail elsewhere, often because an expected slot or animation
  list was not updated.
- **Cause:** These typed boundaries intentionally repeat the archetype in
  exhaustive matches and coverage contracts; they are separate invariants, not
  incidental duplication.
- **Resolution:** Build an update matrix from the archetype name, then search
  every exhaustive match and descriptor coverage list before reviewing the
  slice.
- **Prevention:** Treat a missing protocol, definition, or asset-coverage
  case as an incomplete slice and keep the focused exhaustive tests alongside
  the content change.

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

When an aggregate PR check remains `pending`, inspect the workflow run's
step-level state before treating it as stalled; merge only after each required
repository and WASM job has reached a passing terminal state.

## Clean generated Python caches after repository gates

- **Context:** The repository gate invokes Python content tooling from the
  tracked `scripts/` tree.
- **Symptom:** A successful `sh scripts/check-repository.sh` can leave an
  untracked `scripts/__pycache__/convert-legacy-content*.pyc` file, making a
  merged branch look dirty.
- **Cause:** Python bytecode caching is emitted beside the helper script and is
  not part of the intended source change.
- **Resolution:** Remove only that known cache artifact after the gate, then
  re-check the worktree before handoff.
- **Prevention:** Always inspect `git status --short` after repository checks;
  distinguish generated caches from source edits before committing or merging.

## Commit replay sessions only after temporary execution

- **Context:** A canonical replay can be syntactically valid but fail during
  simulation, and a caller may already have an active session.
- **Symptom:** Replacing session fields while decoding or replaying can leave
  a partially restored game after a rejected load.
- **Resolution:** Decode and run the replay in temporary core state first;
  commit the game, metrics, events, and imported log only after the complete
  run succeeds. Retain the original imported log as the explicit reset source
  so appended commands have deterministic, documented reset behavior.
- **Prevention:** Test malformed input, simulation failure, active-session
  rollback, append-after-load, terminal loads, session wrapper metadata such as
  turn limits, and repeated reset reruns as separate acceptance cases.

## Make bounded presentation storage fail explicitly

- **Context:** Legacy particle callbacks append accepted decal requests, while
  the renderer-neutral Rust boundary must remain deterministic and caller-owned.
- **Symptom:** A fixed request budget can silently discard later effects or
  accidentally deduplicate repeated sprite/position requests.
- **Cause:** Unbounded vectors hide the caller's policy, while replacement or
  set-like storage changes observable insertion order.
- **Resolution:** Require a caller capacity at construction, retain requests in
  insertion order including duplicates, and return a capacity error without
  mutating existing entries when full.
- **Prevention:** Test empty, append, duplicate, zero-capacity, and full-boundary
  behavior before exposing storage to a renderer or browser lifecycle.
