# Reusable development lessons

These notes capture patterns verified while delivering the bounded M8–M10 and
M9 slices in this repository. They are intentionally short; the roadmap,
`SPEC.md`, `ARCHITECTURE.md`, and slice evidence remain the detailed sources of
truth.

## Preserve source-side payload assignments

- **Context:** Legacy item data can provide part of an effect payload while
  the native ranged path supplies the rest.
- **Symptom:** Copying only the serialized explosion fields leaves a schedule
  with the default range instead of the weapon's evidenced radius.
- **Resolution:** Trace both the item declaration and the native assignment
  site; record the resulting typed event only after the complete payload is
  established. For standard and Nuclear BFG 9000, the ranged path copies item
  radius `8` into the explosion range alongside delay `33` and knockback `16`.
- **Prevention:** Keep source provenance for each payload field in the slice
  evidence and test the complete event tuple at every boundary.

## Keep direct and splash damage sampling distinct

- **Context:** Typed weapon definitions expose compact minimum/maximum damage
  ranges, while legacy explosion payloads roll their dice independently for
  every clear blast cell.
- **Symptom:** Reusing a per-die splash helper for the direct hit consumes the
  wrong RNG sequence even when both paths have the same numeric bounds.
- **Resolution:** Preserve the direct combat resolver's one bounded range
  sample, and use an explicit dice helper only for the per-cell splash policy;
  lock both sequences with a replay/RNG regression test.
- **Prevention:** Trace the legacy call site that creates each `TDiceRoll` and
  document whether the Rust boundary models it as one range sample or multiple
  die samples before writing expected-state assertions.

## Keep behavior profiles descriptive and execution-owned

- **Context:** A typed profile can describe a callback-derived transition while
  the dedicated Rust state machine remains its execution authority.
- **Symptom:** Moving profile fragments into a generic dispatcher would blur
  ownership and make replay or rejection guarantees harder to audit.
- **Resolution:** Add only evidence-backed `BehaviorSpec` fragments to the
  immutable profile; keep command validation, state mutation, and rollback in
  the focused transition module.
- **Prevention:** Test exact profile declaration order and document deferred
  callbacks such as Nuclear Plasma chainfire separately from delivered
  recharge/overload behavior.

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

## Preflight aggregate resource requirements before consuming stacks

- **Context:** An alternate/full reload can consume several inventory stacks in
  one accepted command while rejected commands must remain state-identical.
- **Symptom:** Calling a mutating `take_ammo` helper before checking the total
  reserve can partially deplete inventory when the complete clip deficit is not
  available.
- **Cause:** Stack-wise inventory helpers naturally return the amount available,
  but a full-reload callback has an all-or-nothing requirement.
- **Resolution:** Plan the complete deficit from immutable state first, reject
  under-supplied clips before mutation, then consume exactly the planned amount
  and commit one aggregate reload event.
- **Prevention:** Keep aggregate resource planning in a pure typed transition
  helper and cover both multi-stack sufficiency and under-supply with exact
  `Game` equality tests.

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

## Replay the command under test, not only the setup

- **Context:** A boundary test may build its initial `Game` through a replay
  and then submit the command directly through a browser or core wrapper.
- **Symptom:** `ReplayEngine::verify_determinism` passes, but the test has only
  proved deterministic setup reconstruction; it says nothing about whether the
  command payload, events, or post-command state survive replay.
- **Cause:** The replay log was never given the command being compared, so the
  determinism assertion exercised an empty or setup-only history.
- **Resolution:** Clone the setup replay, record the exact command, run that
  replay, and compare replayed events and authoritative state with direct
  execution. Keep setup construction and command replay as distinct assertions.
- **Prevention:** Every scenario/browser parity test must verify the command
  list contains the exercised command before claiming replay preservation.

## Compare generated scenario replays with their runner result

- **Context:** `ScenarioRunner` returns a replay log alongside the final game
  and event stream for a declarative fixture.
- **Symptom:** A test can prove that the generated log is repeatable while a
  missing tile, spawn field, or command serialization still differs from the
  runner's authoritative result.
- **Cause:** Replay determinism compares repeated executions of the log but does
  not by itself compare that log with the original scenario execution.
- **Resolution:** Run the generated replay once more and assert both its final
  `Game` and complete events equal the `ScenarioRunner` outputs before claiming
  scenario/replay parity.
- **Prevention:** Keep generated-log parity and hand-authored boundary-replay
  checks as separate assertions in every vertical scenario test.

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

## Prove fixture identity before boundary parity

- **Context:** A vertical slice often reconstructs one encounter through both
  a declarative scenario and a replay log before comparing browser output.
- **Symptom:** Direct and browser commands can agree while the two setup paths
  quietly differ in monster metadata, terrain, seed, or item identity.
- **Cause:** Event/effect assertions begin after setup and do not establish
  that both boundaries started from the same authoritative `Game`.
- **Resolution:** Assert `Scenario::instantiate() == ReplayEngine::run(setup)`
  before submitting the command, then compare replayed events and final state
  with direct execution. Require literal effect spans for newly exercised
  presentation behavior instead of only comparing a mapper to itself.
- **Prevention:** Keep exact dimensions, ASCII rows, spawn fields, command
  payloads, and derived IDs in the slice plan; treat any fixture drift as a
  scope failure rather than adjusting an expected output in isolation. For
  burst weapons, choose seeds against the interleaved hit and damage draws,
  not only the first hit roll.

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

## Replay mode transitions before mode-dependent effects

- **Context:** A multi-mode weapon can expose a typed fire-mode transition before
  its attack produces mode-specific damage or presentation effects.
- **Symptom:** A boundary test can appear to cover the weapon while its replay
  history still uses the default mode, leaving the transition and its score cost
  outside the parity claim.
- **Cause:** Setup and attack are often treated as separate fixtures, so the
  mode-changing command is omitted from the replayed command stream.
- **Resolution:** Record the mode transition and the mode-dependent attack in
  one command history; assert the transition event and cost before checking
  attack, hit, knockback, and browser effect spans.
- **Prevention:** For every fire-mode vertical slice, compare direct and replayed
  events/final state only after the exact toggle-then-attack sequence is present.

## Atomicity does not choose the canonical outcome

- **Context:** A legacy action may accept a reduced result when a complete
  resource requirement is unavailable, while Rust requires accepted and
  rejected commands to be transactional.
- **Symptom:** The implementation rejects an under-supplied multi-shot action
  and describes that policy as necessary for atomicity even though the legacy
  path fires the affordable subset.
- **Cause:** Transaction safety and gameplay policy were collapsed into one
  decision. Atomicity says that the selected outcome commits completely; it
  does not require every incomplete request to reject.
- **Resolution:** Decide accepted, reduced, or rejected behavior from pinned
  evidence or an explicit DRL-Rust policy, then prepare the complete chosen
  result before mutation. The post-`0.2.318` chainfire audit records this
  distinction for partial volleys.
- **Prevention:** Every resource-sensitive specification states both the
  canonical outcome policy and the transaction boundary, with separate tests
  for each.

## Version the interpreter, not only the save grammar

- **Context:** Browser saves and replays persist command histories that are
  interpreted by gameplay, RNG, generator, content, and ruleset policy.
- **Symptom:** An old token decodes and executes successfully but reconstructs
  a different state after a deterministic rule changes.
- **Cause:** The token version describes its syntax, while the semantic
  identities that give the commands meaning are absent.
- **Resolution:** Persist and validate every relevant interpreter identity
  before executing commands, restore in temporary state, and reject histories
  with missing or incompatible provenance unless an evidenced migration exists.
- **Prevention:** Treat every persistent command history as replay metadata:
  schema compatibility and gameplay compatibility are separate acceptance
  questions.

## Close behavior classes instead of enumerating plateaus

- **Context:** Legacy code often expresses a large state range with a small
  formula or state machine.
- **Symptom:** A sequence of releases adds the next identical counter value,
  duplicating constants, tests, boundary fixtures, documentation, and semantics
  bumps without closing the actual rule.
- **Cause:** The delivery unit followed the observable counter rather than the
  equivalence classes in the source behavior.
- **Resolution:** Specify initial, transition, sustained, and saturation classes
  once; include reset, resource shortage, targeting, and trait interactions in
  the same bounded semantic branch.
- **Prevention:** A milestone slice must reduce an unresolved behavior branch.
  A new counter-only slice is rejected when the value is already covered by an
  evidenced formula or plateau.

## Give rollback backstops an owner and an exit budget

- **Context:** Full-state cloning is a useful interim guard while mutation-
  before-error paths are being converted to validate/prepare/commit.
- **Symptom:** Core and outer wrappers both clone authoritative state on every
  command, and batch/cohort workloads grow without a throughput or allocation
  baseline.
- **Cause:** The safety net became invisible infrastructure after correctness
  tests passed; no boundary owned its cost or retirement criteria.
- **Resolution:** Measure representative accepted/rejected workloads, identify
  which layer owns transactionality, remove redundant outer rollback when the
  core contract is sufficient, and refactor hot paths only with equivalent
  rejection evidence.
- **Prevention:** Every temporary rollback guard records its owner, benchmark,
  retained-risk rationale, and retirement trigger when introduced.

## Keep the active specification replaceable

- **Context:** The roadmap and changelog own delivered progress, while
  `SPEC.md` is supposed to expand one active implementation slice.
- **Symptom:** Historical targets accumulate under labels such as `Current`,
  `Previous`, and `Historical` until reviewers cannot distinguish the contract
  being implemented.
- **Cause:** Each delivery appended its proof to the active control document
  instead of replacing the completed slice and preserving history in its
  designated records.
- **Resolution:** Replace `SPEC.md` when the next slice is selected; retain
  delivered outcomes in the roadmap, changelog, evidence notes, and Git.
- **Prevention:** Add a structural check if the file again contains multiple
  active-slice headings or historical delivery ledgers.
