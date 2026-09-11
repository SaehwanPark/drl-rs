# Specification

Last reviewed: 2026-09-11
Current project version: `0.2.357`
Audited starting checkpoint: `codex/mega-buster-kill-morph` at `264d4b4`
(`0.2.356`)
Delivery checkpoint: active implementation on branch
`codex/mega-buster-kill-morph`

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

## 2. Active implementation slice: M9 Mega Buster post-kill typed morph

Slice status: **delivered and verified locally** on `codex/mega-buster-kill-morph`,
based on `264d4b4` (`0.2.356`).

### 2.1 Objective

Complete one bounded M9 vertical canonical-fidelity branch for the Mega
Buster's post-kill morph. After a confirmed post-mitigation lethal direct hit
from a Mega Buster, read the defeated target's optional equipped weapon
metadata, select one total typed profile (Bullet, Fire, Acid, or Plasma), and
store that profile on the killer's Mega Buster for future commands. Emit a
stable morph event only when the selected mode differs from the current mode.
Preserve the existing three-projectile, nine-ammo, clip, reload, and action
cost rules. This slice records Fire/Acid radius-one and legacy `4d2` metadata,
but does not execute their explosion fanout, delayed queue, or presentation.

This is bounded state/protocol work under the current steering priority. It
does not reopen Gates A, B, C, or D and does not claim whole-game completion.

### 2.2 Scope and ownership

- Keep the existing `ItemArchetype::MegaBuster` and generic ranged execution;
  add no command, callback registry, or second content catalog.
- Keep `Game`/`World` as the execution authority for lethal resolution, RNG
  order, event ordering, and transactional item mutation. Use a pure total
  morph selector so the post-commit transition cannot fail.
- Store morph state on `Item`, expose typed immutable profiles, and add an
  optional `MonsterSpawnSpec.equipped_weapon` that is reconstructed by
  Scenario/ReplayEngine and round-tripped by MCP replay JSON.
- Add `GameEvent::MegaBusterMorphed` and exhaustive MCP, audio, metrics, render,
  and browser projections without leaking hidden target state.
- Add focused pure-profile, direct lethal/nonlethal/miss, target-topology,
  replay/scenario, MCP JSON, rollback, and BrowserSession parity tests.
- Update the pinned evidence/profile, architecture ownership summary,
  user-facing weapon guide, changelog, roadmap, steering status, and
  replay-semantics comments only after verification.
- Transition code version exactly once from `0.2.356` to `0.2.357` and gameplay
  semantics from `150` to `151`.

### 2.3 Observable acceptance criteria

- [x] A confirmed post-mitigation lethal direct Mega Buster hit selects the
  defeated target's optional equipped weapon damage family; missing,
  unsupported, and Physical families fall back to Bullet.
- [x] The selected profile is stored on the Mega Buster without changing its
  archetype, clip, reload, action cost, three-projectile count, or nine-ammo
  cost. Future direct shots observe the new bounded damage range and typed
  damage family.
- [x] `MegaBusterMorphed` is emitted at most once per lethal direct hit and only
  when the mode changes; misses, nonlethal hits, non-Mega weapons, splash-only
  deaths, and same-mode kills emit no morph event.
- [x] Event ordering is pinned as `AttackResolved < DamageApplied < ActorDied <
  MegaBusterMorphed < ItemDropped`; rejected commands preserve exact state and
  RNG because morph selection and application are total and post-commit.
- [x] Optional target equipment is represented in Scenario, ReplayEngine, and
  MCP replay JSON with deterministic item IDs; direct and replay executions
  agree for Bullet, Fire, Acid, Plasma, and fallback branches, and stale
  gameplay-semantics `150` metadata is rejected after advancing to `151`.
- [x] MCP JSON, audio, metrics, render, and BrowserSession projections are
  exhaustive and fair-observation-safe; no hidden target equipment crosses a
  boundary.
- [x] `drl-core` remains platform-independent and no legacy runtime,
  audiovisual, browser-capture, balance, or human-play parity claim is made.
- [x] Focused tests, repository checks, version/spec checks, and an independent
  determinism review pass; unavailable native, controlled legacy,
  audiovisual/reference-capture, and human surfaces remain `NOT_RUN`.

### 2.4 Semantic and boundary impact

- **Damage policy:** The pinned legacy profiles are Bullet `1d8`, Fire `4d2`,
  Acid `4d2`, and Plasma `1d10`; current Rust stores bounded inclusive ranges
  `(1,8)`, `(4,8)`, `(4,8)`, and `(1,10)` with typed Physical/Fire/Acid/Plasma
  mitigation. Exact dice distribution remains a follow-up.
- **Morph trigger/order:** Only a direct Mega Buster hit whose final damage is
  lethal can morph. The transition is inserted after `ActorDied` marks the
  victim dead and before inventory/death-drop processing, matching the legacy
  `OnKill` boundary; splash/environment deaths never infer a Mega source.
- **State preservation:** Morph changes only the Mega Buster's bounded damage
  profile and typed mode. Clip, reload, action, projectile count, and ammo cost
  remain the existing catalog values.
- **RNG/replay:** Morph selection is pure and consumes no RNG. Advance gameplay
  semantics from `150` to `151`; wire/schema, generator, and ruleset identities
  remain unchanged. Replays carry target equipment explicitly rather than
  inferring it from monster kind or scalar damage.
- **Content/catalog:** Existing weapon catalog entries remain authoritative;
  target equipment is an optional replay/scenario topology field and allocates
  deterministic item IDs.
- **Presentation:** `MegaBusterMorphed` is a thin protocol/MCP, audio, metrics,
  render, and browser projection. No presentation callback mutates simulation
  state.
- **Rights/evidence:** Static legacy evidence supports target-weapon mapping,
  profile values, and post-lethal/pre-drop callback order. Controlled legacy
  runtime, balance, audiovisual, browser capture, and human acceptance remain
  `NOT_RUN` unless prerequisites exist.

### 2.5 Non-goals

- No Fire/Acid radius-one splash execution, delayed-explosion queue, projectile
  routing, exact `4d2` distribution, legacy accuracy/miss timing, colors,
  sprites, callback registry, same-volley mutation, or audiovisual parity.
- No inference from `MonsterKind`, scalar ranged damage, or hidden target state;
  unsupported target families remain the explicit Bullet fallback.
- No changes to other weapon behavior beyond regression coverage, and no claim
  of controlled legacy runtime, browser capture, balance, or human-play parity.

### 2.6 Delivery evidence

Evidence will be bound to the delivered branch; hosted checks and merge
revision remain outside this local handoff:

- focused core, scenario/replay, MCP, audio/metrics/render, and web tests cover
  all morph branches, event ordering, rollback atomicity, and topology parity;
- `cargo fmt --all -- --check`, `cargo test --workspace --locked`,
  `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
  `sh scripts/check-repository.sh`, the static/browser-library portions of
  `sh scripts/check-web.sh`,
  `DRL_VERSION_BASE=264d4b4 sh scripts/check-version.sh`,
  `sh scripts/check-spec-structure.sh`, and `git diff --check` pass;
- an attributable independent determinism review returns `PASS` for this
  bounded scope;
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
