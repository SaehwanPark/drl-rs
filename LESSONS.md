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

## Keep legacy damage subtypes inside typed mitigation policy

- **Context:** The protocol exposes a shared Plasma family while legacy combat
  distinguishes `DAMAGE_PLASMA` from `DAMAGE_SPLASMA` for armor protection.
- **Symptom:** Adding a new public damage enum or routing every Plasma event
  through one helper either expands the wire contract or changes already-
  delivered direct and splash behavior.
- **Cause:** The legacy subtype is an internal mitigation rule, not a distinct
  fair-observation or event family for this bounded slice.
- **Resolution:** Keep `DamageType::Plasma` on the public event and add a
  narrowly named core helper for SPLASMA-style fanout: apply catalog resistance
  first, then integer-floor one-third armor protection, then the existing
  minimum-one rule. Route only the Null Pointer splash through it and advance
  gameplay semantics.
- **Prevention:** Before reusing a typed damage path, compare the legacy
  subtype's full mitigation order and divisor with the existing Rust policy;
  preserve the public vocabulary unless the boundary itself observably
  distinguishes the subtype.

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

## Bind slice evidence to the actual merge base

- **Context:** A temporary branch can start after a post-merge documentation
  follow-up, even when the earlier merge revision is still the most prominent
  checkpoint in the roadmap.
- **Symptom:** Scope/evidence artifacts and a version check can cite the older
  merge instead of the commit the branch actually diverged from; the check may
  still pass because the project version did not change in between.
- **Cause:** The branch lineage was recorded from the preceding feature merge
  before checking the repository's current `main` tip.
- **Resolution:** Compare `git merge-base HEAD main` with `git rev-parse main`
  before finalizing artifacts, then use that exact merge base for the
  `DRL_VERSION_BASE` version check and predecessor fields. This slice corrected
  its evidence from the older frontend merge to the actual `main` commit.
- **Prevention:** Treat `git merge-base HEAD main` as authoritative for every
  temporary slice; record the full revision in `00-scope.md`, `01-evidence.md`,
  and later handoff artifacts before claiming the version transition.

## Bind review receipts to the final PR metadata

- **Context:** A required-review policy may inspect API metadata rather than
  the checked-out diff.
- **Symptom:** A superficially valid approval can survive a new commit, a later
  reviewer state, pagination, or a rename and still be counted for protected
  code.
- **Resolution:** Normalize every review page, reduce each reviewer's latest
  state before filtering by the current head, require the receipt's exact
  commit, and match both `filename` and `previous_filename`. Keep the live
  branch-protection settings and their solo-maintainer exception in the same
  auditable evidence chain.
- **Prevention:** Fixture-test stale heads, later changes-requested reviews,
  paginated responses, and renames; run the hosted policy workflow on every
  pull-request revision before merging.

When an aggregate PR check remains `pending`, inspect the workflow run's
step-level state before treating it as stalled; merge only after each required
repository and WASM job has reached a passing terminal state.

## Keep replay file IO at the application boundary

- **Context:** The canonical replay JSON decoder and deterministic engine are
  reusable from native tooling, but the simulation core must remain free of
  filesystem and process concerns.
- **Resolution:** Put path/stdin selection, UTF-8 reads, stable diagnostics, and
  exit-status mapping in `drl-app`; inject a `Read` implementation in unit
  tests, then pass the parsed value through the existing MCP decoder and
  `ReplayEngine::verify_determinism` without duplicating validation.
- **Prevention:** Treat replay migration, network IO, and cross-version
  interchange as separate capabilities; a file-verification command proves
  only current-engine V2 acceptance.

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

## Check every target before letting a fixer prune re-exports

- **Context:** Splitting a large crate root into modules leaves a
  `pub(crate) use` surface whose consumers are spread across `cfg(test)`,
  `cfg(target_arch = "wasm32")`, and normal builds.
- **Symptom:** After a native-only `cargo fix`, WASM-target builds failed with
  "cannot find ... in the crate root" because the fixer deleted re-exports only
  the WASM target resolved, silently narrowing the internal surface.
- **Resolution:** Run `cargo check --all-targets` for every relevant target,
  then re-inspect the re-export block against per-name usage before accepting
  any automatic import cleanup; restore gated names and split them by the
  target that resolves them.
- **Prevention:** Treat `cargo fix`/`cargo clippy --fix` as unsafe on
  target-gated crates: diff the re-export block against the intended module map
  and keep the second target's check in the same verification pass.

## Make mechanical source scanners monotonic and output-bounded

- **Context:** A scripted line-range split of a 14,764-line source file walked
  members to assign them to generated modules.
- **Symptom:** The member scan rewound its own cursor after walking back over
  attributes, so it looped while appending to an unbounded list until the
  process was killed for memory exhaustion.
- **Resolution:** Keep scanner cursors strictly monotonic, add a runaway guard,
  and cap both resources and chatter: `timeout`, `ulimit -v`, and
  `head`/`tail` on command logs so a stuck loop fails fast and legibly.
- **Prevention:** Print counts instead of collected contents, bound every
  generated collection, and prefer several small verified passes over one
  pass that rewrites a whole file.

## Keep source-grep boundary contracts pointed at a module set

- **Context:** Browser boundary scripts asserted contracts by grepping the
  single `drl-web` crate root for specific strings.
- **Symptom:** Moving behavior between modules silently drops such coverage,
  because the grep now matches nothing and a naive rewrite can pass by
  accident while asserting less.
- **Resolution:** Declare the owning module set in the script, `test -s` each
  file, and grep the contract string across the set so a string may move but
  must remain owned by the shell.
- **Prevention:** When code moves, update the assertion script in the same
  commit and confirm each individual contract string still resolves to exactly
  one intended file.

## Separate pre-existing lint debt from newly introduced warnings

- **Context:** Modularized code inherits the lints of the code it moves, and a
  reviewer cannot tell new warnings from old ones by count alone.
- **Symptom:** WASM-target Clippy reported seven warnings after the split,
  which looked like a regression.
- **Resolution:** Re-run the exact lint command at the audited baseline
  revision in a throwaway worktree, and record that the same seven warnings
  reproduce there, so the slice claims no fix and hides no regression.
- **Prevention:** State the lint command, target, and baseline revision in the
  slice evidence, and keep tolerated lint debt out of unrelated slices.

## Compute mechanical fidelity before delegating a large-move review

- **Context:** A module split moved ~14,700 lines, and the first independent
  review was one read-only pass asked to reconcile the whole before/after pair.
- **Symptom:** The reviewer exhausted a 30-minute budget without a verdict,
  because it re-derived counts by reading huge files instead of checking claims.
- **Resolution:** Compute the mechanical invariants first (shader-digest and
  length match, export-signature census, item-name census, test-name diff,
  platform-API import counts), write them to one evidence file, then delegate
  several narrow review lanes with tool budgets and a short per-lane timeout.
- **Prevention:** Never delegate "verify this whole diff" for a bulk move; give
  each lane a bounded file list, a reuse-instead-of-recompute evidence path,
  a per-read line cap, and `toolBudget`/`timeoutMs`. Validate child options
  early: `toolBudget.block` must be `"*"` or an array of tool names, not a
  single bare name.

## Match the reviewer to a context budget you actually have

- **Context:** Subagent children inherit the parent's configured model, and this
  workspace was running a local Vulkan-served model with a small context window.
- **Symptom:** Bulk-review children failed twice for different-looking reasons:
  one 30-minute timeout, then three lanes each ending `Context size has been
  exceeded` after ~13 tool calls, with `Saved output: unavailable`.
- **Resolution:** Read the run metadata (`attemptedModels`, `error`, `toolCount`)
  before re-launching; either hand children pre-cut excerpts with a per-read
  line cap, or delegate bulk reconciliation to a runner with its own larger
  context (for example the read-only Codex CLI agent) and tell it to bound its
  own diff reads.
- **Prevention:** Before delegating a review, state the child model's context
  budget, cap what each read may pull, and check `status view='transcript'` plus
  the run's `meta.json` instead of guessing from a generic failure line.

## Never pre-author an independent review verdict

- **Context:** The slice template had a sentence asserting the independent review
  "returned PASS", and it went into `SPEC.md` while that review was still
  running, next to checked hosted-check boxes.
- **Symptom:** The review came back `fix`, so the canonical slice claimed an
  acceptance verdict and hosted-check status that did not exist, pinned to an
  implementation head that later commits had already superseded.
- **Resolution:** Replace the verdict sentence with an evidence ledger: one
  bullet per gate naming the exact revision, command, host, and result; an
  explicit `pending` for anything unfinished; and the first review's disposition
  and findings kept as history alongside the corrections they caused.
- **Prevention:** Write acceptance claims only after the run finishes, bind each
  claim to a named commit, and re-run the gate whenever a later commit touches a
  build input. A green run on an older commit is evidence about that commit only.

## Launch the distro container from the runner, not as the job container

- **Context:** Adding a Fedora 43 job to the Ubuntu-hosted CI, the obvious shape is
  `runs-on: ubuntu-latest` with `container: image: fedora:43`.
- **Symptom:** `actions/checkout` needs a Node runtime inside the job image, and a
  bare distro image does not guarantee one; the failure appears as a checkout error
  in the new job, not as a repository problem.
- **Resolution:** Check out on the runner, then start the distro container yourself
  (`docker run --rm --volume "$GITHUB_WORKSPACE":/src --workdir /src
  --env CARGO_TARGET_DIR=/tmp/drl-target fedora:43 sh -c '... && sh
  scripts/check-fedora-dev.sh'`). The runner keeps its own Node runtime and the
  crate build writes outside the mounted worktree.
- **Prevention:** Prove a container job by running the exact invocation locally
  (podman) before committing the workflow, keep `CARGO_TARGET_DIR` inside the
  container, and let the check script install nothing so missing prerequisites stay
  visible as workflow provisioning facts.
