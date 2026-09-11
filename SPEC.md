# Specification

Last reviewed: 2026-09-11
Current project version: `0.2.358`
Audited starting checkpoint: `codex/mega-buster-fanout` at `81a37ce`
(`0.2.357`)
Delivery checkpoint: active implementation on branch
`codex/mega-buster-fanout`

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

## 2. Active implementation slice: M9 Mega Buster exact dice and radius-one fanout

Slice status: **delivered and verified** on `codex/mega-buster-fanout`, based on
`81a37ce` (`0.2.357`).

### 2.1 Objective

Complete one bounded M9 vertical canonical-fidelity branch after the delivered
Mega Buster morph state. Execute each current Mega profile with its explicit
`NdM` dice shape (`1d8`, `4d2`, `1d10`) using one deterministic RNG draw per
die. Fire and Acid modes additionally emit a typed radius-one explosion
schedule event and immediately resolve the existing Rust splash helper with
one independent `4d2` roll per eligible cell. Preserve the current Rust
direct-hit-plus-environmental-splash convention, including fanout after a
miss, while keeping the prior direct-only morph trigger and ordinary
three-projectile early-stop behavior.

Preserve the existing three-projectile, nine-ammo, clip, reload, action-cost,
target-equipment, replay, MCP, and browser contracts. Treat explosion delay as
event metadata; do not add a pending scheduler. This bounded slice does not
claim exact legacy radius geometry/traversal, delayed timing, accuracy/miss
rules, same-volley mutation, audiovisual parity, or whole-game completion.

This remains bounded work under the current steering priority. It does not
reopen Gates A, B, C, or D.

### 2.2 Scope and ownership

- Keep the existing `ItemArchetype::MegaBuster` and generic ranged execution;
  add no command, callback registry, or second content catalog.
- Keep `Game`/`World` as the execution authority for lethal/splash resolution,
  RNG order, event ordering, and transactional item mutation. Use a pure total
  morph selector and immutable profile snapshot so post-commit transitions
  cannot fail or reread mutable state unexpectedly.
- Store morph state on `Item`, expose typed immutable profiles, and add an
  optional `MonsterSpawnSpec.equipped_weapon` that is reconstructed by
  Scenario/ReplayEngine and round-tripped by MCP replay JSON.
- Add `GameEvent::MegaBusterExplosionScheduled` with stable mode-independent
  fields and exhaustive MCP, audio, metrics, render, and browser projections;
  retain the delivered `MegaBusterMorphed` event without leaking hidden target
  state.
- Add focused exact-dice, direct hit/miss, radius-one geometry/fanout,
  target-topology, replay/scenario, MCP JSON, rollback, and BrowserSession
  parity tests.
- Update the pinned evidence/profile, architecture ownership summary,
  user-facing weapon guide, changelog, roadmap, steering status, and
  replay-semantics comments only after verification.
- Transition code version exactly once from `0.2.357` to `0.2.358` and gameplay
  semantics from `151` to `152`.

### 2.3 Observable acceptance criteria

- [x] Each Mega profile rolls its explicit dice shape (`1d8`, `4d2`, `4d2`,
  `1d10`) with one deterministic RNG draw per die, and the exposed bounded
  damage range remains consistent with those shapes.
- [x] Fire and Acid projectiles emit a typed delay-40/radius-one/knockback-8
  schedule event and resolve immediate radius-one fanout with one independent
  `4d2` roll per eligible cell; Bullet and Plasma remain direct-only.
- [x] Fire and Acid fanout follows the current Rust direct-plus-environment
  convention on both hits and misses, uses stable center-first geometry, and
  carries typed Fire/Acid damage through direct and splash events.
- [x] The Mega Buster profile is snapshotted per projectile; ordinary fire
  retains three-projectile early-stop and nine-ammo semantics without
  same-volley morph mutation, and clip/reload/action behavior remains intact.
- [x] All possible radius-one death-drop destinations are preflighted before
  clip/RNG mutation; rejected commands preserve exact game state and RNG.
- [x] `MegaBusterExplosionScheduled` is exhaustive across MCP JSON, audio,
  metrics, render, BrowserSession, and fair-observation boundaries, with stable
  owner/item/target/profile fields and no hidden target disclosure.
- [x] Direct and replay executions agree for exact Bullet/Fire/Acid/Plasma
  profiles, target topology, and deterministic event/RNG/state sequences;
  semantics `152` is accepted and stale `151` metadata is rejected.
- [x] The delivered direct-only post-kill morph contract remains intact:
  splash-only deaths do not morph, and `MegaBusterMorphed` stays ordered after
  `ActorDied` and before a configured drop.
- [x] `drl-core` remains platform-independent and no legacy runtime,
  audiovisual, browser-capture, balance, or human-play parity claim is made;
  exact legacy timing/accuracy/traversal and same-volley mutation remain
  explicitly deferred.
- [x] Focused tests, repository checks, version/spec checks, and a fresh
  independent determinism review pass; unavailable native, controlled legacy,
  audiovisual/reference-capture, and human surfaces remain `NOT_RUN`.

### 2.4 Semantic and boundary impact

- **Damage policy:** The pinned legacy profiles are Bullet `1d8`, Fire `4d2`,
  Acid `4d2`, and Plasma `1d10`; current Rust now consumes one deterministic
  draw per die and exposes bounded inclusive ranges `(1,8)`, `(4,8)`, `(4,8)`,
  and `(1,10)` with typed Physical/Fire/Acid/Plasma mitigation.
- **Morph trigger/order:** Only a direct Mega Buster hit whose final damage is
  lethal can morph. The delivered transition remains after `ActorDied` marks
  the victim dead and before inventory/death-drop processing; splash/environment
  deaths never infer a Mega source.
- **Explosion policy:** Fire/Acid emit delay-40/radius-one/knockback-8 metadata
  and apply immediate current-Rust fanout using the shared center-first helper.
  Direct damage and splash damage are both retained by this bounded convention;
  the direct target is already dead before fanout when lethal. There is no
  pending explosion queue.
- **State preservation:** Morph changes only the Mega Buster's bounded damage
  profile and typed mode. Clip, reload, action, projectile count, and ammo cost
  remain the existing catalog values.
- **RNG/replay:** Morph selection is pure and consumes no RNG; exact Mega rolls
  consume one sample per die in deterministic event order. Advance gameplay
  semantics from `151` to `152`; wire/schema, generator, and ruleset identities
  remain unchanged. Replays carry target equipment explicitly rather than
  inferring it from monster kind or scalar damage.
- **Content/catalog:** Existing weapon catalog entries remain authoritative;
  target equipment is an optional replay/scenario topology field and allocates
  deterministic item IDs.
- **Presentation:** `MegaBusterMorphed` and
  `MegaBusterExplosionScheduled` are thin protocol/MCP, audio, metrics, render,
  and browser projections. Delay remains metadata; no presentation callback
  mutates simulation state.
- **Rights/evidence:** Static legacy evidence supports target-weapon mapping,
  profile values, and post-lethal/pre-drop callback order. Controlled legacy
  runtime, balance, audiovisual, browser capture, and human acceptance remain
  `NOT_RUN` unless prerequisites exist.

### 2.5 Non-goals

- No delayed-explosion queue, projectile routing, legacy accuracy/miss timing,
  colors, sprites, callback registry, same-volley mutation, or audiovisual
  parity. The immediate current-Rust Fire/Acid fanout is bounded and does not
  claim exact legacy radius traversal or delayed execution.
- No inference from `MonsterKind`, scalar ranged damage, or hidden target state;
  unsupported target families remain the explicit Bullet fallback.
- No changes to other weapon behavior beyond regression coverage, and no claim
  of controlled legacy runtime, browser capture, balance, or human-play parity.

### 2.6 Delivery evidence

Evidence will be bound to the delivered branch; hosted checks and merge
revision remain outside this local handoff:

- focused core, scenario/replay, MCP, audio/metrics/render, and web tests cover
  exact profile rolls, Fire/Acid fanout, morph ordering, rollback atomicity,
  and topology parity;
- `cargo fmt --all -- --check`, `cargo test --workspace --locked`,
  `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
  `sh scripts/check-repository.sh`, the static/browser-library portions of
  `sh scripts/check-web.sh`,
  `DRL_VERSION_BASE=81a37ce sh scripts/check-version.sh`,
  `sh scripts/check-spec-structure.sh`, and `git diff --check` pass;
- an attributable fresh independent determinism review returns `PASS` for this
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
