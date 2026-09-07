# Specification

Last reviewed: 2026-09-07
Current project version: `0.2.347`
Audited starting checkpoint: `main` at `9449e78` (Plasma Shotgun fidelity
checklist reconciliation)
Delivery checkpoint: **merged** in PR #464 as `347109c`

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

## 2. Active implementation slice: M9 Tristar Blaster direct Plasma classification

Slice status: **delivered and verified** in PR #464; no subsequent slice is
selected. The temporary branch `feat/tristar-blaster-direct-plasma` was based
on `main` commit `9449e78` (`0.2.346`).

### 2.1 Objective

Complete one bounded M9 vertical canonical-fidelity branch by carrying the
pinned Tristar Blaster `DAMAGE_PLASMA` classification into the existing Rust
typed direct-damage path. A successful ordinary direct-target volley must apply
Blue Armor's catalog-defined Plasma resistance before flat protection while
preserving the existing three-projectile volley, five-cell-per-projectile cost,
RNG sampling, event ordering, replay metadata, and boundary projections.

This is eligible vertical canonical-fidelity work under the current steering
priority. It closes the direct Plasma family branch for the Tristar Blaster
without reopening Gates A, B, C, or D.

### 2.2 Scope and ownership

- Use the existing `ItemArchetype::TristarBlaster` and generic ranged
  execution; add no command, callback registry, content-registration path, or
  public wire field.
- Keep `Game`/`World` as the execution authority for direct damage and retain
  the existing generic ranged validation, RNG sampling, and transactional
  fifteen-cell clip mutation.
- Add focused direct ordinary, same-seed armored, replay/stale-semantics,
  rejection, MCP JSON, and BrowserSession parity coverage for all three
  ordered projectiles.
- Update the pinned Tristar Blaster evidence/profile, architecture ownership
  summary, user-facing weapon guide, changelog, roadmap, and replay-semantics
  comments only after verification.
- Transition code version exactly once from `0.2.346` to `0.2.347`.

### 2.3 Observable acceptance criteria

- [x] Successful Tristar Blaster direct hits emit three ordered
  `DamageApplied` events with `DamageType::Plasma`; Blue Armor applies its 20%
  resistance before the existing flat protection, while raw rolls and the RNG
  stream match an unarmored run.
- [x] A same-seed unarmored/Blue-Armored direct pair preserves three
  projectiles, the fifteen-cell clip cost, equal raw damage and final RNG state,
  and lower typed damage amounts for the armored target.
- [x] Existing attack, damage, action-cost, and turn-event ordering remains
  unchanged; spread, routing, delayed explosion, and knockback policy are not
  silently added.
- [x] Invalid target, blocked line-of-sight, and below-fifteen-cell commands
  reject before clip/RNG mutation and preserve exact pre/post `Game` identity.
- [x] Replay determinism and direct-core/MCP JSON/BrowserSession event, state,
  observation, effect, and scene parity remain valid; stale gameplay-semantics
  `145` metadata is rejected after the semantics advance to `146`.
- [x] `drl-core` remains platform-independent, no hidden world state crosses a
  boundary, and no legacy runtime/audiovisual or human-play parity claim is
  made.
- [x] Focused tests, repository checks, web checks, version/spec checks, and an
  independent determinism review pass; unavailable native, controlled legacy,
  audiovisual/reference-capture, and human surfaces remain explicitly
  `NOT_RUN`.

### 2.4 Semantic and boundary impact

- **Damage policy:** The pinned `utristar` definition carries
  `DAMAGE_PLASMA`. The existing typed actor path applies Blue Armor's catalog
  resistance before flat protection; no new resistance family or body-zone
  aggregation is introduced.
- **Command atomicity:** No new rejection branch is intended. Existing generic
  validation and rollback must preserve exact `Game` identity, including RNG,
  for Tristar Blaster direct commands.
- **RNG/replay:** Only the damage interpretation changes; sampling order and
  the three-projectile/fifteen-cell policy remain unchanged. Advance gameplay
  semantics from `145` to `146`; wire/schema, RNG-sampling, generator, and
  ruleset identities remain unchanged.
- **Content/catalog:** No definition or registration changes; the existing
  Tristar Blaster and Blue Armor catalog entries remain authoritative.
- **Presentation:** `DamageApplied` retains its existing shape and gains the
  already-supported `Some(Plasma)` classification on this direct path; MCP,
  browser, render, and audio projections remain thin consumers.
- **Rights/evidence:** The pinned source supports the damage-family
  classification. Controlled legacy runtime, balance, audiovisual, browser
  capture, and human acceptance remain `NOT_RUN` unless prerequisites exist.

### 2.5 Non-goals

- No Tristar Blaster spread/routing, falloff, delayed explosion, knockback,
  exact timing/accuracy, alternate fire/reload, callback recreation,
  terrain/cell mutation, or broader resistance aggregation.
- No changes to Plasma Shotgun, Plasma Rifle, BFG, Blaster, Null Pointer, or
  other already classified paths beyond regression coverage.
- No new protocol enum, command, snapshot, generator, ruleset, or content
  identity; no claim of controlled legacy runtime, audiovisual, browser-capture,
  balance, or human-play parity.

### 2.6 Delivery evidence

Evidence is bound to the merged candidate:

- focused `drl-core`, `drl-mcp`, and `drl-web` tests pass, including three-hit
  direct Plasma mitigation, replay, rejection, JSON, and browser parity;
- `cargo fmt --all -- --check`, `cargo test --locked --workspace --jobs 1
  -- --test-threads=1`, `cargo clippy --workspace --all-targets --all-features
  -- -D warnings`, `sh scripts/check-repository.sh`, `sh scripts/check-web.sh`,
  `DRL_VERSION_BASE=9449e78 sh scripts/check-version.sh`,
  `sh scripts/check-spec-structure.sh`, and `git diff --check` pass;
- an attributable independent determinism review returns `PASS` after the
  focused Clippy correction pass;
- hosted Repository, Linux, Fedora, and WASM browser checks: `PASS` in CI run
  `34093450402`;
- hosted Review policy: `FAIL` closed in run `34093448757` because the sole
  maintainer cannot create a non-self approval; the documented live
  `enforce_admins=false` exception was used;
- PR #464 merged as `347109c` with exact head
  `62b2023e263882c0741936408437ea415f005a51`;
- explicit `NOT_RUN` records remain for Fedora/Wayland/Vulkan interactive
  acceptance, macOS/Metal native interactive acceptance, controlled legacy
  runtime, audiovisual/reference captures, browser capture, and human
  gameplay acceptance.

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
