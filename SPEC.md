# Specification

Last reviewed: 2026-09-04
Current project version: `0.2.344`
Audited starting checkpoint: `main` at `5b9a037` (codexbar documentation)
Delivery checkpoint: **active implementation on temporary branch**

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

## 2. Active implementation slice: Anti-Freak Jackal direct Fire classification

Slice status: **active** on temporary branch `feat/anti-freak-direct-fire`,
based on `main` commit `5b9a037` (`0.2.343`).

### 2.1 Objective

Complete one bounded M9 canonical-fidelity branch by carrying the pinned
Anti-Freak Jackal direct-hit `DAMAGE_FIRE` classification into the existing
Rust typed damage path. A successful ordinary or aimed direct hit must apply
Red Armor's catalog-defined Fire resistance before flat protection while
preserving the existing hit roll, raw damage, clip cost, action cost, splash
fanout, event ordering, replay metadata, and boundary projections.

### 2.2 Scope and ownership

- Use the existing `AntiFreakJackal` archetype and generic ranged execution;
  add no new command, callback registry, content registration path, or public
  wire field.
- Keep `Game`/`World` as the execution authority for direct damage and retain
  the existing typed splash resolver for the delayed-explosion metadata path.
- Add focused direct ordinary, aimed, armored, replay, and stale-semantics
  coverage in `crates/drl-core/tests`.
- Update the existing pinned evidence note, architecture ownership summary,
  user-facing weapon guide, changelog, roadmap, and replay semantics comments
  only after verification.
- Transition code version exactly once from `0.2.343` to `0.2.344`.

### 2.3 Observable acceptance criteria

- [x] Successful Anti-Freak ordinary direct hits emit `DamageApplied` with
  `DamageType::Fire`; Red Armor mitigates the typed amount before flat
  protection, while the raw roll and RNG stream match an unarmored run.
- [x] Successful Anti-Freak aimed direct hits use the same typed Fire path and
  retain the existing +3 accuracy / doubled action-cost behavior.
- [x] Empty-clip, invalid-target, and blocked direct commands remain rejected
  before clip/RNG mutation; the slice adds exact pre/post `Game` equality for
  the changed weapon path.
- [x] Replay determinism and direct-core/BrowserSession event/state parity
  remain valid; stale gameplay-semantics `142` metadata is rejected after the
  semantics advance to `143`.
- [x] `drl-core` remains platform-independent, no hidden world state crosses
  a boundary, and no legacy runtime/audiovisual or human-play parity claim is
  made.
- [x] Focused tests, repository checks, web checks, version checks, and an
  independent determinism review pass; unavailable native/legacy runtime
  surfaces remain explicitly `NOT_RUN` (the local headless Chromium contract
  is available and passes).

### 2.4 Semantic and boundary impact

- **Command atomicity:** no new rejection branch is intended. Existing generic
  validation and rollback must preserve exact `Game` identity, including RNG,
  for Anti-Freak direct commands.
- **RNG/replay:** direct damage classification changes mitigation and therefore
  gameplay semantics; raw sampling order is unchanged. Advance gameplay
  semantics from `142` to `143`; wire/schema, RNG, generator, and ruleset
  identities remain unchanged.
- **Content/catalog:** no definitions or registration paths change; the
  existing Anti-Freak Jackal catalog entry remains authoritative for Fire
  damage and Red Armor remains authoritative for its resistance.
- **Presentation:** only the typed `DamageApplied` event payload changes for
  direct Anti-Freak hits; existing browser/MCP projections consume the same
  event shape.
- **Rights/evidence:** source inspection supports the legacy damage-family
  classification only; controlled legacy runtime, audiovisual, and human
  acceptance remain `NOT_RUN`.

### 2.5 Non-goals

- No Anti-Freak terrain/cell mutation, delayed queue, alternate callback state,
  new spread/routing, splash immunity, resistance aggregation, accuracy/timing
  correction, asset/audio work, or balance validation.
- No changes to the already delivered radius-1 splash behavior beyond keeping
  its typed Fire path intact.
- No claim of legacy runtime, browser capture, audiovisual parity, or human
  gameplay acceptance without the required controlled evidence.

### 2.6 Delivery evidence

Evidence is bound to the current candidate branch; the commit, hosted checks,
and merge revision will be appended at PR handoff:

- `cargo test --locked -p drl-core` passes, including the three focused
  `anti_freak_jackal_direct_fire` tests; `cargo test --locked -p drl-mcp`
  passes (83 library, 26 protocol JSON-RPC, 3 security/fairness, 7 gameplay,
  and 1 virtual-agent test); and `cargo test --locked -p drl-web --lib` passes
  all 100 native browser-boundary tests.
- `sh scripts/check-repository.sh` passes. `sh scripts/check-web.sh` passes,
  including 11 asset, 1 audio, 82 render, 100 native web, and 2 local headless
  Chromium tests. `DRL_VERSION_BASE=5b9a037 sh scripts/check-version.sh`
  passes for `0.2.344`.
- The independent determinism re-review returns `PASS` after adding invalid-
  target exact-state coverage and reconciling current replay-semantics
  documentation. Direct source review confirms typed Fire, resistance ordering,
  raw RNG preservation, replay rejection, and boundary parity.
- Hosted PR checks and the eventual merge revision are not yet available on
  this active temporary branch. Fedora/Wayland/Vulkan, macOS/Metal, controlled
  legacy runtime, audiovisual/reference captures, and human gameplay acceptance
  remain `NOT_RUN` or outside this slice.

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
