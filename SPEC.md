# Specification

Last reviewed: 2026-09-11
Current project version: `0.2.356`
Audited starting checkpoint: `codex/revenants-launcher-radius3` at `7f66f08`
(`0.2.355`)
Delivery checkpoint: delivered and verified locally on branch
`codex/missile-launcher-radius3`

The [Roadmap](docs/DRL-RS_Project_Roadmap.md) owns milestone scope, ordering,
and progress. [`docs/steering/current-priorities.md`](docs/steering/current-priorities.md)
constrains slice selection while its stop gates remain open. This file expands
**exactly one active implementation slice**. Delivered history belongs in the
roadmap, changelog, evidence notes, and Git rather than accumulating here.

## 1. Status vocabulary

- `[x]` — **Delivered and verified**: supported by checked implementation and
evidence.
- `[ ]` — **Open**: required by the active slice and not yet delivered.
- `NOT_RUN` — **Environment unavailable**: prerequisites were unavailable; no
pass or failure is inferred.
- `INCONCLUSIVE` — **Evidence unresolved**: available evidence cannot support
the claim.

## 2. Active implementation slice: M9 Missile Launcher radius-3 Fire fanout

Slice status: **delivered and verified locally** on
`codex/missile-launcher-radius3`, based on `codex/revenants-launcher-radius3` at
`7f66f08` (`0.2.355`).

### 2.1 Objective

Complete one bounded M9 vertical canonical-fidelity branch by retaining the
existing normal Missile Launcher direct Fire path and adding its bounded
center-inclusive radius-3 Fire fanout. An accepted shot that resolves against
the validated target emits delay-40/radius-3/knockback-8 schedule metadata and
immediately consumes one ordered `6d6` roll per clear blast cell. A direct hit
keeps its typed Fire damage before the schedule; a miss has no direct damage
but still resolves the legacy-style explosion at the impact coordinate. The
fanout applies the current shared integer distance-falloff policy, actor
de-duplication, radial `damage / 8` knockback, ordinary-ground-item
thresholding, and normal death/drop ordering.

This is eligible vertical canonical-fidelity work under the current steering
priority. It closes the Missile Launcher's radius-3 explosion branch without
reopening Gates A, B, C, or D.

### 2.2 Scope and ownership

- Use the existing `ItemArchetype::MissileLauncher` and generic ranged
  execution; add no command, callback registry, or second content catalog.
- Keep `Game`/`World` as the execution authority for direct and splash damage,
  preflight, RNG order, event ordering, and transactional clip mutation.
- Add typed Missile Launcher geometry/roll/falloff helpers, an explicit
  schedule event and behavior fragments, and focused direct/miss fanout,
  item/death-drop, replay/scenario, MCP JSON, and BrowserSession parity tests.
- Update the pinned evidence/profile, architecture ownership summary,
  user-facing weapon guide, changelog, roadmap, steering status, and
  replay-semantics comments only after verification.
- Transition code version exactly once from `0.2.355` to `0.2.356` and gameplay
  semantics from `149` to `150`.

### 2.3 Observable acceptance criteria

- [x] A normal direct hit retains typed `Fire` mitigation and emits a distinct
  `MissileLauncherExplosionScheduled` event with delay `40`, radius `3`, and
  knockback `8` before splash events; a resolved miss emits the schedule and
  splash without a direct damage event.
- [x] The radius-3 fanout visits the deterministic current Rust clear-cell
  order, consumes one `6d6` roll per cell, applies the shared integer
  distance-falloff policy, de-duplicates actors, applies radial integer
  `damage / 8` knockback before typed Fire damage, and preserves source
  self-damage and normal death/drop follow-up.
- [x] A post-falloff damage result greater than `10` removes at most the lowest
  ID ordinary ground item in that blast cell after actor processing; terrain,
  feature-item, and chained-explosion behavior remains excluded.
- [x] Every possible splash death-drop destination is preflighted before clip
  or splash RNG mutation; representative invalid commands preserve exact
  pre/post `Game` identity, including RNG.
- [x] Replay determinism plus direct-core/MCP JSON/audio/metrics/render/
  BrowserSession event, state, observation, effect, and scene parity remain
  valid; stale gameplay-semantics `149` metadata is rejected after advancing
  to `150`.
- [x] `drl-core` remains platform-independent, no hidden world state crosses a
  boundary, and no legacy runtime/audiovisual or human-play parity claim is
  made.
- [x] Focused tests, repository checks, version/spec checks, and an independent
  determinism review pass; static/browser-library checks must pass while the
  WASM headless browser phase and unavailable native, controlled legacy,
  audiovisual/reference-capture, and human surfaces remain `NOT_RUN`.

### 2.4 Semantic and boundary impact

- **Damage policy:** The pinned `umbazooka` definition carries `DAMAGE_FIRE`.
  Direct hits and splash actor damage use the existing typed Fire path, applying
  Red Armor's 25% resistance before flat protection (4); the source actor is
  not self-safe for splash.
- **Command atomicity:** Splash death-drop destinations are validated in the
  prepare phase before clip mutation or any direct/splash RNG draw; rejection
  preserves exact `Game` identity.
- **RNG/replay:** A hit consumes the normal hit roll and direct damage roll,
  then emits schedule metadata and ordered `6d6` splash rolls. A miss consumes
  its normal hit roll, then schedule metadata and splash rolls without direct
  damage. Advance gameplay semantics from `149` to `150`; wire/schema,
  generator, and ruleset identities remain unchanged.
- **Geometry decision:** The bounded slice reuses the current Rust center-first
  clockwise clear-cell helper and Chebyshev distance, preserving established
  splash behavior. Legacy `Distance` metric/cell traversal (37 radius-3 cells
  and x/y iteration) is observed evidence but remains explicit follow-up work
  rather than an exact-parity claim.
- **Content/catalog:** Existing Missile Launcher, rocket-ammo, and armor
  catalog entries remain authoritative; no new registration path is added.
- **Presentation:** The new per-weapon schedule event is a thin protocol/MCP,
  audio, metrics, render, and browser projection. Delay remains metadata; no
  presentation callback mutates simulation state.
- **Rights/evidence:** Static legacy evidence supports payload/radius/falloff,
  default knockback, and unconditional explosion rules. Controlled legacy
  runtime, balance, audiovisual, browser capture, and human acceptance remain
  `NOT_RUN` unless prerequisites exist.

### 2.5 Non-goals

- No pending delayed-explosion queue, homing, projectile routing, exact legacy
  missile timing/accuracy, callback recreation, terrain/cell mutation, feature
  semantics, splash immunity, chained explosions, rocket-jump/mod callbacks,
  broader resistance aggregation, exact legacy radius metric/order, or
  cross-version migration.
- No changes to Rocket Launcher, Revenant's Launcher, Anti-Freak Jackal, Plasma
  weapons, or other already classified paths beyond regression coverage.
- No claim of controlled legacy runtime, audiovisual, browser-capture, balance,
  or human-play parity.

### 2.6 Delivery evidence

Evidence is bound to the delivered branch; hosted checks and merge revision
remain outside this local handoff:

- focused core, scenario/replay, MCP, audio/metrics/render, and web tests cover
  hit/miss scheduling, radius-3 fanout, and rejection atomicity;
- `cargo fmt --all -- --check`, `cargo test --workspace --locked`,
  `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
  `sh scripts/check-repository.sh`, the static/browser-library portions of
  `sh scripts/check-web.sh`,
  `DRL_VERSION_BASE=7f66f08 sh scripts/check-version.sh`,
  `sh scripts/check-spec-structure.sh`, and `git diff --check` pass;
- an attributable independent determinism review by
  `/root/missile_determinism_review` returns `PASS` after the SPEC wording
  correction;
- hosted checks are not yet available on this branch; Fedora/Wayland/Vulkan,
  macOS/Metal, controlled legacy runtime, audiovisual/reference captures,
  browser capture, and human gameplay acceptance remain `NOT_RUN` or outside
  this slice. The WASM headless browser phase remains `NOT_RUN` if the
  ChromeDriver/Chrome prerequisites do not match.

## 3. Enduring invariants

The active slice must preserve:

1. no ambient state, platform APIs, filesystem, browser, or presentation policy
   in `drl-core`;
2. identical declared seed, commands, and semantics produce identical current
   simulation results;
3. incompatible histories fail explicitly before simulation;
4. rejected commands and rejected restores do not partially mutate authoritative
   simulation state;
5. renderers, browser code, MCP, and bots consume fair observations/events and
   do not inspect hidden core state;
6. presentation timing, resize, scale factor, surface loss, and storage side
   effects do not advance gameplay;
7. no runtime Lua or generic callback recreation;
8. current-Rust, cross-version, legacy, browser, audiovisual, and performance
   evidence remain separately labeled.
