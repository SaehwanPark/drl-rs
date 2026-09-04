# Specification

Last reviewed: 2026-09-04
Current project version: `0.2.345`
Audited starting checkpoint: `main` at `32f54e5` (Anti-Freak delivery reconciliation)
Delivery checkpoint: **open** on the candidate branch; merge evidence is pending

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

## 2. Active implementation slice: M9 Null Pointer SPLASMA armor divisor

Slice status: **candidate verified locally; hosted checks and merge pending** on
temporary branch `feat/null-pointer-splasma-divisor`, based on `main` commit
`32f54e5` (`0.2.344`).

### 2.1 Objective

Close one bounded M9 canonical-fidelity branch by carrying the pinned legacy
`DAMAGE_SPLASMA` armor-value divisor into the existing Null Pointer radius-1
actor splash. For that splash only, apply the catalog-defined Plasma resistance
first, then subtract the current equipped armor protection divided by three
using integer floor arithmetic, with the existing minimum-one rule. Preserve
the fixed `10d1` roll, geometry, actor de-duplication, event ordering,
death/drop behavior, replay identities, and boundary projections.

### 2.2 Scope and ownership

- Keep `NullPointer` target-score handling and the existing generic radius-1
  splash resolver authoritative; add only the explicit SPLASMA mitigation
  policy needed by its actor damage call.
- Reuse `ArmorProperties` and the typed resistance helper. The divisor applies
  to the current Rust armor protection value after resistance; no new armor
  slots, body zones, callbacks, or mutable registries are introduced.
- Keep the public `DamageType::Plasma` event classification and wire shape;
  the legacy Plasma/SPLASMA distinction is an internal mitigation policy for
  this bounded path, not a new protocol enum.
- Add focused unarmored/Blue-Armored, same-seed, replay, rejection, and
  direct-core/BrowserSession parity coverage, plus a regression proving direct
  Plasma and other splash paths are unchanged.
- Update the Null Pointer evidence/profile, architecture ownership summary,
  weapon/item guide if needed, changelog, roadmap, and replay semantics
  comments only after verification.
- Transition code version exactly once from `0.2.344` to `0.2.345`.

### 2.3 Observable acceptance criteria

- [x] Successful Null Pointer splash actor damage remains a typed Plasma event
  and applies Plasma resistance before `floor(armor_protection / 3)` flat
  protection, with minimum-one behavior and no-armor preservation.
- [x] A same-seed Blue-Armored/unarmored Null Pointer splash pair has equal
  fixed raw damage and final RNG state; with current Blue Armor (`20%`, `2`
  protection), a positive `10d1` splash applies `8` damage rather than the
  ordinary typed-Plasma flat-protection result.
- [x] Radius-1 center/neighbor order, actor de-duplication, death/drop and
  game-over follow-up, schedule/event ordering, score transition, and fixed
  `10d1` no-RNG behavior remain unchanged.
- [x] Direct Plasma mitigation, Anti-Freak/Rocket Fire mitigation, and other
  splash policies remain unchanged; the change does not add a public wire,
  command, snapshot, generator, ruleset, or content-registration identity.
- [x] Rejected Null Pointer invocations remain exact-state atomic, and
  direct-core, replay/MCP, and BrowserSession event/state/effect/scene parity
  remains valid.
- [x] `drl-core` remains platform-independent and the implementation/review
  distinguish current-Rust behavior from legacy runtime, audiovisual, balance,
  and human-play claims.
- [x] Focused tests, repository/web/version/spec checks, and an independent
  determinism review pass; unavailable native/legacy/capture surfaces remain
  explicitly `NOT_RUN`.

### 2.4 Semantic and boundary impact

- **Damage policy:** This is the first bounded use of the legacy SPLASMA
  armor-value divisor. Resistance uses the existing deterministic rounded
  percentage helper; the effective flat protection is
  `armor.protection / 3` after resistance, and minimum-one damage remains in
  force.
- **Command atomicity:** No new rejection branch is intended. Existing
  validation and rollback must preserve exact `Game` identity, including RNG,
  for Null Pointer invoke/attack paths.
- **RNG/replay:** The fixed `10d1` splash roll and all sampling order remain
  unchanged. Advance gameplay semantics from `143` to `144`; wire/schema,
  RNG-sampling, generator, and ruleset identities remain unchanged.
- **Content/catalog:** No content definition or registration changes. The
  existing Null Pointer and Blue Armor catalog entries remain authoritative.
- **Presentation:** `DamageApplied` retains the existing `Some(Plasma)` event
  shape; only the authoritative amount for the bounded Null Pointer splash can
  change when armor is equipped.
- **Rights/evidence:** The pinned Pascal source supports the family mapping and
  divisor. Controlled legacy runtime, audiovisual, browser-capture, balance,
  and human acceptance remain `NOT_RUN` unless their prerequisites exist.

### 2.5 Non-goals

- No direct Plasma divisor-2 migration, broader resistance aggregation,
  innate/weapon/boots/body-zone bonuses, armor durability/degradation, shield
  callbacks, or resistance families not represented by the current catalog.
- No changes to BFG, Anti-Freak, Rocket, or other explosion paths in this
  slice; their existing typed behavior remains regression coverage.
- No delayed queue, terrain/cell or ground-item effects, splash immunity,
  projectile routing, runtime Lua/callback recreation, asset/audio work,
  balance validation, or human/browser audiovisual parity claim.

### 2.6 Delivery evidence

Evidence bound to the current candidate:

- focused Null Pointer SPLASMA-divisor, actor minimum-one, core replay,
  rejection, MCP JSON, and BrowserSession parity tests: PASS;
- `cargo fmt --all -- --check`: PASS;
- `cargo test --workspace --locked`: PASS;
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`:
  PASS;
- `sh scripts/check-repository.sh`: PASS;
- `sh scripts/check-web.sh`: PASS, including the local headless Chromium/WASM
  contract tests;
- `DRL_VERSION_BASE=32f54e5 sh scripts/check-version.sh`: PASS (`0.2.345`);
- `sh scripts/check-spec-structure.sh` and `git diff --check`: PASS;
- fresh independent determinism review: PASS, reviewer mission
  `b46272de-d36c-499f-9c27-1146bf0f4db6`, against the current review packet;
- hosted Repository, Linux, Fedora, WASM, Review-policy, and merge results:
  pending for the single PR;
- explicit `NOT_RUN` records remain for controlled legacy runtime,
  native interactive Wayland/Vulkan or Metal acceptance, audiovisual/reference
  captures, browser capture, and human gameplay.

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
