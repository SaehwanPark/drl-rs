# Specification

Last reviewed: 2026-09-07
Current project version: `0.2.346`
Audited starting checkpoint: `main` at `5814e26` (Null Pointer SPLASMA
reconciliation documentation follow-up)
Delivery checkpoint: **merged** in PR #463 as `87683d6`

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

## 2. Active implementation slice: M9 Plasma Shotgun direct Plasma classification

Slice status: **delivered and verified** in PR #463; no subsequent slice is
selected. The temporary branch `feat/plasma-shotgun-direct-plasma` was based
on `main` commit `5814e26` (`0.2.345`).

### 2.1 Objective

Complete one bounded M9 vertical canonical-fidelity branch by carrying the
pinned Plasma Shotgun `DAMAGE_PLASMA` classification into the existing Rust
typed direct-damage path. A successful ordinary direct hit must apply Blue
Armor's catalog-defined Plasma resistance before flat protection while
preserving the existing one-projectile hit roll, three-cell clip cost, event
ordering, replay metadata, and boundary projections.

This is eligible vertical canonical-fidelity work under the current steering
priority. It does not continue a chainfire plateau and does not reopen Gates A,
B, C, or D.

### 2.2 Scope and ownership

- Use the existing `ItemArchetype::PlasmaShotgun` and generic ranged execution;
  add no command, callback registry, content registration path, or public wire
  field.
- Keep `Game`/`World` as the execution authority for direct damage and retain
  the existing generic ranged validation, RNG sampling, and transactional clip
  mutation.
- Add focused direct ordinary, same-seed armored, replay/stale-semantics,
  rejection, MCP JSON, and BrowserSession parity coverage.
- Update the pinned Plasma Shotgun evidence/profile, architecture ownership
  summary, user-facing weapon guide, changelog, roadmap, and replay-semantics
  comments only after verification.
- Transition code version exactly once from `0.2.345` to `0.2.346`.

### 2.3 Observable acceptance criteria

- [x] Successful Plasma Shotgun direct hits emit `DamageApplied` with
  `DamageType::Plasma`; Blue Armor applies its 20% resistance before the
  existing flat protection, while the raw roll and RNG stream match an
  unarmored run.
- [x] A same-seed unarmored/Blue-Armored direct pair preserves one projectile,
  the three-cell clip cost, equal raw damage and final RNG state, and a lower
  typed damage amount for the armored target.
- [x] Existing attack, damage, action-cost, and turn-event ordering remains
  unchanged; spread, falloff, and knockback policy are not silently added.
- [x] Invalid target, blocked line-of-sight, and below-three-cell commands
  reject before clip/RNG mutation and preserve exact pre/post `Game` identity.
- [x] Replay determinism and direct-core/MCP JSON/BrowserSession event, state,
  observation, effect, and scene parity remain valid; stale gameplay-semantics
  `144` metadata is rejected after the semantics advance to `145`.
- [x] `drl-core` remains platform-independent, no hidden world state crosses a
  boundary, and no legacy runtime/audiovisual or human-play parity claim is
  made.
- [x] Focused tests, repository checks, web checks, version/spec checks, and an
  independent determinism review pass; unavailable native, controlled legacy,
  audiovisual/reference-capture, and human surfaces remain explicitly
  `NOT_RUN`.

### 2.4 Semantic and boundary impact

- **Damage policy:** The pinned `upshotgun` definition carries
  `DAMAGE_PLASMA`. The existing typed actor path applies Blue Armor's catalog
  resistance before flat protection; no new resistance family or body-zone
  aggregation is introduced.
- **Command atomicity:** No new rejection branch is intended. Existing generic
  validation and rollback must preserve exact `Game` identity, including RNG,
  for Plasma Shotgun direct commands.
- **RNG/replay:** Only the damage interpretation changes; sampling order and
  the one-projectile/three-cell policy remain unchanged. Advance gameplay
  semantics from `144` to `145`; wire/schema, RNG-sampling, generator, and
  ruleset identities remain unchanged.
- **Content/catalog:** No definition or registration changes; the existing
  Plasma Shotgun and Blue Armor catalog entries remain authoritative.
- **Presentation:** `DamageApplied` retains its existing shape and gains the
  already-supported `Some(Plasma)` classification on this direct path; MCP,
  browser, render, and audio projections remain thin consumers.
- **Rights/evidence:** The pinned source supports the damage-family
  classification. Controlled legacy runtime, balance, audiovisual, browser
  capture, and human acceptance remain `NOT_RUN` unless prerequisites exist.

### 2.5 Non-goals

- No Plasma Shotgun spread/routing, falloff, knockback, exact timing/accuracy,
  alternate fire/reload, callback recreation, terrain/cell mutation, delayed
  queue, or broader resistance aggregation.
- No changes to Plasma Rifle, BFG, Blaster, Null Pointer, or other already
  classified paths beyond regression coverage.
- No new protocol enum, command, snapshot, generator, ruleset, or content
  identity; no claim of controlled legacy runtime, audiovisual, browser-capture,
  balance, or human-play parity.

### 2.6 Delivery evidence

Evidence is bound to the merged candidate:

- focused `drl-core`, `drl-mcp`, and `drl-web` tests pass, including direct
  Plasma Shotgun mitigation, replay, rejection, JSON, and browser parity;
- `cargo fmt --all -- --check`, `cargo test --workspace --locked`,
  `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
  `sh scripts/check-repository.sh`, `sh scripts/check-web.sh`,
  `DRL_VERSION_BASE=5814e26 sh scripts/check-version.sh`,
  `sh scripts/check-spec-structure.sh`, and `git diff --check` pass;
- an attributable independent determinism review returns `PASS` after any
  focused correction pass;
- hosted Repository, Linux, Fedora, and WASM checks: `PASS` in CI run
  `34078165900`;
- hosted Review policy: `FAIL` closed in run `34078165839` because the sole
  maintainer cannot create a non-self approval; the documented live
  `enforce_admins=false` exception was used;
- PR #463 merged as `87683d6` with exact head
  `be0287b60f284d77a6b4006956cf5ed19b6fc896`;
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
