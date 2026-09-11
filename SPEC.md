# Specification

Last reviewed: 2026-09-11
Current project version: `0.2.355`
Audited starting checkpoint: `main` at `67d0985` (`0.2.354`)
Delivery checkpoint: delivered and verified locally on branch
`codex/revenants-launcher-radius3`

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

## 2. Active implementation slice: M9 Revenant's Launcher radius-3 Fire fanout

Slice status: **delivered and verified locally** on
`codex/revenants-launcher-radius3`, based on `main` at `67d0985` (`0.2.354`).

### 2.1 Objective

Complete one bounded M9 vertical canonical-fidelity branch by retaining the
existing typed direct Fire hit for Revenant's Launcher and adding its
center-inclusive radius-3 Fire fanout. A successful shot must preserve the
direct exact-hit/one-rocket policy, emit delay-40/radius-3/knockback-8 schedule
metadata, then consume one ordered `7d6` roll per clear blast cell with legacy
distance falloff, actor de-duplication, radial integer `damage / 8` knockback,
ordinary-ground-item thresholding, and normal death/drop ordering.

This is eligible vertical canonical-fidelity work under the current steering
priority. It closes the Revenant's Launcher radius-3 explosion branch without
reopening Gates A, B, C, or D.

### 2.2 Scope and ownership

- Use the existing `ItemArchetype::RevenantsLauncher` and generic ranged
  execution; add no command, callback registry, or second content catalog.
- Keep `Game`/`World` as the execution authority for direct and splash damage,
  preflight, RNG order, event ordering, and transactional clip mutation.
- Add focused radius-3 geometry/roll/falloff tests, direct fanout and item/
  death-drop tests, replay/scenario coverage, MCP JSON projection, and
  BrowserSession/direct-core parity.
- Update the pinned evidence/profile, architecture ownership summary,
  user-facing weapon guide, changelog, roadmap, steering status, and
  replay-semantics comments only after verification.
- Transition code version exactly once from `0.2.354` to `0.2.355` and gameplay
  semantics from `148` to `149`.

### 2.3 Observable acceptance criteria

- [x] Successful direct hits retain typed `Fire` mitigation, then emit a
  distinct `RevenantsLauncherExplosionScheduled` event with delay `40`, radius
  `3`, and knockback `8` before splash events.
- [x] The radius-3 fanout visits deterministic clear cells, consumes one `7d6`
  roll per cell, applies legacy distance falloff, de-duplicates actors,
  applies radial integer `damage / 8` knockback before typed Fire damage, and
  preserves source self-damage and normal death/drop follow-up.
- [x] A post-falloff damage result greater than `10` removes at most the lowest
  ID ordinary ground item in that blast cell after actor processing; terrain,
  feature-item, and chained-explosion behavior remains excluded.
- [x] Every possible splash death-drop destination is preflighted before clip
  or splash RNG mutation; representative invalid commands preserve exact
  pre/post `Game` identity, including RNG.
- [x] Replay determinism plus direct-core/MCP JSON/audio/metrics/render/
  BrowserSession event, state, observation, effect, and scene parity remain
  valid; stale gameplay-semantics `148` metadata is rejected after advancing
  to `149`.
- [x] `drl-core` remains platform-independent, no hidden world state crosses a
  boundary, and no legacy runtime/audiovisual or human-play parity claim is
  made.
- [x] Focused tests, repository checks, version/spec checks, and an independent
  determinism review pass; static/browser-library checks pass, while the WASM
  headless browser phase is `NOT_RUN` because the installed Chrome (152) does
  not match the available ChromeDriver (153). Unavailable native, controlled
  legacy, audiovisual/reference-capture, and human surfaces remain `NOT_RUN`.

### 2.4 Semantic and boundary impact

- **Damage policy:** The pinned `urbazooka` definition carries `DAMAGE_FIRE`.
  Direct and splash actor damage use the existing typed Fire path, applying
  Red Armor's 25% resistance before flat protection (4).
- **Command atomicity:** Splash death-drop destinations are validated in the
  prepare phase before clip mutation or any direct/splash RNG draw; rejection
  preserves exact `Game` identity.
- **RNG/replay:** Accepted exact-hit order is the direct damage roll, schedule
  event, then one ordered `7d6` splash roll per clear cell; the exact-hit
  policy consumes no to-hit roll. Advance gameplay semantics from `148` to
  `149`; wire/schema, RNG-sampling, generator, and ruleset identities remain
  unchanged.
- **Content/catalog:** Existing Revenant's Launcher and armor catalog entries
  remain authoritative; no new registration path is added.
- **Presentation:** The new per-weapon schedule event is a thin protocol/MCP,
  audio, metrics, render, and browser projection. Delay remains metadata; no
  presentation callback mutates simulation state.
- **Rights/evidence:** Static legacy evidence supports payload/radius/falloff
  rules. Controlled legacy runtime, balance, audiovisual, browser capture, and
  human acceptance remain `NOT_RUN` unless prerequisites exist.

### 2.5 Non-goals

- No pending delayed-explosion queue, homing, projectile routing, exact
  missile timing/accuracy, callback recreation, terrain/cell mutation, feature
  semantics, splash immunity, chained explosions, rocket-jump/mod callbacks,
  broader resistance aggregation, or cross-version migration.
- No changes to Rocket Launcher, Missile Launcher, Anti-Freak Jackal, Plasma
  weapons, or other already classified paths beyond regression coverage.
- No claim of controlled legacy runtime, audiovisual, browser-capture, balance,
  or human-play parity.

### 2.6 Delivery evidence

Evidence is bound to the delivery branch; hosted checks and merge revision are
not part of this local handoff:

- focused core, scenario/replay, MCP, audio/metrics/render, and web tests pass
  for the schedule and radius-3 fanout, including rejection atomicity;
- `cargo fmt --all -- --check`, `cargo test --workspace --locked`,
  `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
  `sh scripts/check-repository.sh`, the static/browser-library portions of
  `sh scripts/check-web.sh`,
  `DRL_VERSION_BASE=67d0985 sh scripts/check-version.sh`,
  `sh scripts/check-spec-structure.sh`, and `git diff --check` pass;
- an attributable independent determinism review returns `PASS` after the
  focused MCP fixture correction (`/root/revenants_radius3_review2`);
- hosted checks are not yet available on this branch; Fedora/Wayland/Vulkan,
  macOS/Metal, controlled legacy runtime, audiovisual/reference captures,
  browser capture, and human gameplay acceptance remain `NOT_RUN` or outside
  this slice. The WASM headless browser invocation was attempted but remains
  `NOT_RUN` because ChromeDriver 153 cannot launch installed Chrome 152.

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
