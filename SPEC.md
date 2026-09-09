# Specification

Last reviewed: 2026-09-08
Current project version: `0.2.354`
Audited starting checkpoint: `main` at `3127602` (bound solo review
exception, `0.2.353`)
Delivery checkpoint: active candidate on temporary branch
`feat/revenants-launcher-direct-fire`

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

## 2. Active implementation slice: M9 Revenant's Launcher direct Fire classification

Slice status: **delivered and verified locally** on temporary branch
`feat/revenants-launcher-direct-fire` based on `main` at `3127602`
(`0.2.353`).

### 2.1 Objective

Complete one bounded M9 vertical canonical-fidelity branch by carrying the
pinned Revenant's Launcher `DAMAGE_FIRE` classification into the existing Rust
typed direct-damage path. A successful direct-target hit must apply Red Armor's
catalog-defined 25% Fire resistance before flat protection (4) while preserving
the existing exact-hit policy, one-rocket-per-shot cost from its one-rocket
clip, RNG sampling, event ordering, replay metadata, and boundary projections.
It also verifies integration with ordinary single-rocket `Reload` and
empty-clip atomic rejection.

This is eligible vertical canonical-fidelity work under the current steering
priority. It closes the direct Fire family branch for the Revenant's Launcher
without reopening Gates A, B, C, or D.

### 2.2 Scope and ownership

- Use the existing `ItemArchetype::RevenantsLauncher` and generic ranged
  execution; add no command, callback registry, content-registration path, or
  public wire field.
- Keep `Game`/`World` as the execution authority for direct damage and retain
  the existing generic ranged validation, exact-hit policy, RNG sampling, and
  transactional clip mutation.
- Add focused direct ordinary, same-seed armored, single-shot clip depletion,
  single-rocket reload integration, replay/stale-semantics, rejection, MCP JSON,
  and BrowserSession parity coverage.
- Update the pinned Revenant's Launcher evidence/profile, architecture
  ownership summary, user-facing weapon guide, changelog, roadmap, and
  replay-semantics comments only after verification.
- Transition code version exactly once from `0.2.353` to `0.2.354`.

### 2.3 Observable acceptance criteria

- [x] Successful Revenant's Launcher direct hits emit `DamageApplied` with
  `DamageType::Fire`; Red Armor applies its 25% resistance before flat
  protection (4), while the raw roll and RNG stream match an unarmored run.
- [x] A same-seed unarmored/Red-Armored/Blue-Armored direct triple preserves
  one projectile, the one-rocket clip cost, equal raw damage and final RNG
  state, and a lower typed damage amount for the armored targets; Blue Armor
  (0% fire resistance) applies only flat protection (2).
- [x] The one-rocket clip allows exactly one Fire attack (clip 1 -> 0); a 2nd
  shot rejects atomically with `NoAmmoInClip` before clip/RNG mutation;
  ordinary single-rocket `Reload` restores 1 rocket and allows 1 follow-up
  Fire shot.
- [x] Invalid target, blocked line-of-sight, out-of-range, and empty-clip
  commands reject before clip/RNG mutation and preserve exact pre/post `Game`
  identity.
- [x] Replay determinism and direct-core/MCP JSON/BrowserSession event, state,
  observation, effect, and scene parity remain valid; stale gameplay-semantics
  `147` metadata is rejected after the semantics advance to `148`.
- [x] `drl-core` remains platform-independent, no hidden world state crosses a
  boundary, and no legacy runtime/audiovisual or human-play parity claim is
  made.
- [x] Focused tests, repository checks, web checks, version/spec checks, and an
  independent determinism review pass; unavailable native, controlled legacy,
  audiovisual/reference-capture, and human surfaces remain explicitly
  `NOT_RUN`.

### 2.4 Semantic and boundary impact

- **Damage policy:** The pinned `urbazooka` definition carries
  `DAMAGE_FIRE`. The existing typed actor path applies Red Armor's catalog
  resistance (25%) before flat protection (4); no new resistance family or
  body-zone aggregation is introduced.
- **Command atomicity:** No new rejection branch is intended. Existing generic
  validation and rollback must preserve exact `Game` identity, including RNG,
  for Revenant's Launcher direct commands.
- **RNG/replay:** Only the damage interpretation changes; sampling order and
  the one-rocket exact-hit policy remain unchanged. Advance gameplay semantics
  from `147` to `148`; wire/schema, RNG-sampling, generator, and ruleset
  identities remain unchanged.
- **Content/catalog:** No definition or registration changes; the existing
  Revenant's Launcher and Red Armor catalog entries remain authoritative.
- **Presentation:** `DamageApplied` retains its existing shape and gains the
  already-supported `Some(Fire)` classification on this direct path; MCP,
  browser, render, and audio projections remain thin consumers.
- **Rights/evidence:** The pinned source supports the damage-family
  classification. Controlled legacy runtime, balance, audiovisual, browser
  capture, and human acceptance remain `NOT_RUN` unless prerequisites exist.

### 2.5 Non-goals

- No Revenant's Launcher homing, projectile routing, radius-3 delayed
  explosion queue, exact timing/accuracy, callback recreation, terrain/cell
  mutation, or broader resistance aggregation.
- No changes to Rocket Launcher, Missile Launcher, Anti-Freak Jackal, Plasma
  weapons, or other already classified paths beyond regression coverage.
- No new protocol enum, command, snapshot, generator, ruleset, or content
  identity; no claim of controlled legacy runtime, audiovisual, browser-capture,
  balance, or human-play parity.

### 2.6 Delivery evidence

Evidence is bound to the active candidate branch; the commit, hosted checks,
and merge revision will be reconciled at handoff:

- focused `drl-core`, `drl-mcp`, and `drl-web` tests pass, including direct
  Revenant's Launcher Fire mitigation, clip depletion, single-rocket reload,
  replay, rejection, JSON, and browser parity;
- `cargo fmt --all -- --check`, `cargo test --workspace --locked`,
  `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
  `sh scripts/check-repository.sh`, `sh scripts/check-web.sh`,
  `DRL_VERSION_BASE=3127602 sh scripts/check-version.sh`,
  `sh scripts/check-spec-structure.sh`, and `git diff --check` pass;
- an attributable independent determinism review returns `PASS` after any
  focused correction pass;
- hosted PR checks and the eventual merge revision are not yet available on
  this active temporary branch; Fedora/Wayland/Vulkan, macOS/Metal, controlled
  legacy runtime, audiovisual/reference captures, browser capture, and human
  gameplay acceptance remain `NOT_RUN` or outside this slice.

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
