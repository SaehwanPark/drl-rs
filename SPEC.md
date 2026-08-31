# Specification

Last reviewed: 2026-08-31
Current project version: `0.2.319`
Audited checkpoint: `main` at
`3796a2ff50c748c45b50ade1d07d68a3f9c06395` (merged PR #426)

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

## 2. Active implementation slice: M10 — Semantics-bound browser snapshot V3

### 2.1 Objective

Replace the browser's semantics-unbound command-history save token with a V3
token that records the identities required to interpret its commands. Restore
must reject incompatible or provenance-free histories before simulation,
rebuild a compatible session in temporary state, and commit only after every
command succeeds.

This closes steering Gate A. It is a compatibility and transaction slice, not
a gameplay feature.

### 2.2 Audited starting point

At version `0.2.318`:

- `drl-web::persistence` writes strict V2 tokens containing snapshot version,
  fixed-content identity, command count, and encoded commands;
- `BrowserSession::restore_snapshot` decodes the token, constructs the current
  fixed M4 session, and resubmits its commands under current rules;
- V1 tokens are accepted and migrated to V2 after successful restore;
- the token does not record gameplay, RNG-sampling, generator, or ruleset
  identities;
- replay V2 already records and validates those identities;
- current identities are gameplay `127`, RNG-sampling `1`, generator `2`,
  ruleset `drl-rs-ruleset-v1`, and fixed content `fixed-m4-v1`.

Therefore a V1/V2 snapshot can be syntactically valid yet be silently
reinterpreted by a later build. Successful execution under the later build is
not proof of compatibility with the state that originally wrote the token.

### 2.3 Scope and ownership

- **Steering gate:** Gate A — persistent histories bind their interpreter.
- **Primary owner:** `drl-web::persistence` owns the browser token grammar and
  bounds; `BrowserSession` owns temporary reconstruction and final commit.
- **Identity source:** snapshot code imports the canonical current identities
  from `drl-protocol`; it must not duplicate their numeric/string values.
- **Core boundary:** `drl-core` remains the execution authority. This slice
  does not add snapshot policy or browser storage to core.
- **Project version:** implementation advances `VERSION` from `0.2.318` to
  `0.2.319`.
- **Gameplay/replay semantics:** gameplay remains `127`; replay wire schema,
  RNG-sampling, generator, and ruleset identities do not change. Snapshot wire
  format advances from V2 to V3.

### 2.4 V3 token contract

The canonical textual token is:

```text
DRL-RUST-BROWSER-SAVE/3:<content>:<gameplay>:<rng>:<generator>:<ruleset>:<count>:<payload>
```

Where:

- `<content>` is the fixed browser-session content identity;
- `<gameplay>`, `<rng>`, and `<generator>` are canonical unsigned decimal
  integers with no sign, padding, or alternate representation;
- `<ruleset>` is the stable ruleset identity;
- `<count>` is the canonical command count;
- `<payload>` retains the current deterministic command encoding and ordering.

The shipped bounds remain 16 KiB per token and 4,096 commands. Parsing remains
strict and allocation-bounded. Empty histories are valid with count `0` and an
empty payload. Count/payload disagreement, malformed numbers, unknown commands,
extra delimiters, or a token over either bound rejects without execution.

The exact current values are read from their owners when encoding. Tests may
pin the checkpoint values, but production code must not create a second
constant set.

### 2.5 Compatibility policy

V3 restore validates, in this order:

1. overall byte bound and token prefix/version;
2. fixed-content identity;
3. gameplay semantics identity;
4. RNG-sampling identity;
5. generator identity;
6. ruleset identity;
7. command-count and payload structure;
8. command execution in a temporary `BrowserSession`.

No command executes until checks 1–7 succeed. Each semantic mismatch returns a
diagnostic that identifies the mismatched field and the found/expected value.
Unknown future token versions remain distinguishable from malformed tokens.

V1 and V2 tokens are **semantics-unbound**. Because they contain no engine or
release identity, the implementation must not label them as current and replay
them speculatively. In this slice:

- direct restore rejects V1/V2 with a distinct unbound-semantics error;
- browser storage follows the existing bounded quarantine/recovery policy;
- the active session remains playable and unchanged;
- automatic V1-to-V2 migration is retired;
- no V1/V2 token is rewritten as V3 unless a later, separately specified
  migration can prove its source semantics.

This intentionally prefers explicit save incompatibility over silent state
reinterpretation during pre-1.0 development.

### 2.6 Transaction contract

Restore is a two-phase operation:

```text
decode + validate identities
          |
          v
construct temporary fixed session
          |
          v
execute every decoded command
          |
          +---- error: discard temporary state; preserve active session
          |
          v
commit complete restored session
```

On every error, the active session's complete game, observation, successful
command history, error state, and saved token remain unchanged except for the
already-defined bounded quarantine/diagnostic side effect at the browser
storage shell. No failed restore writes a migrated token.

Successful V3 restore produces the same final `Game`, events/observable state,
and command history as direct execution from the fixed session under the exact
recorded identities.

### 2.7 User-visible recovery

- Startup/load diagnostics state that an older or incompatible save cannot be
  safely interpreted by this build.
- Failure does not prevent starting or continuing a playable current session.
- The existing accessible Clear Save confirmation remains the explicit recovery
  action; this slice does not silently delete user storage.
- Storage read, quarantine, cleanup, or rewrite failure remains a warning and
  must not corrupt the active session.

### 2.8 Acceptance criteria

- [ ] New snapshots encode V3 with canonical current content, gameplay,
  RNG-sampling, generator, and ruleset identities.
- [ ] V3 round-trips empty and representative complete command histories with
  exact command order and canonical count encoding.
- [ ] Production snapshot code imports semantic identities from
  `drl-protocol` rather than duplicating checkpoint literals.
- [ ] A mismatch in each identity field rejects before session construction or
  command execution and reports the specific incompatibility.
- [ ] V1 and V2 fixtures reject as semantics-unbound; no automatic migration or
  V3 rewrite occurs.
- [ ] Unsupported future versions, malformed fields, count mismatch, unknown
  commands, oversized counts, and oversized tokens retain distinct bounded
  failure behavior where currently defined.
- [ ] Every failed direct restore preserves exact active `BrowserSession`
  authority, including game and successful command history.
- [ ] Every failed storage restore preserves the active session and original
  saved value except for the existing bounded quarantine policy.
- [ ] Successful restore commits only after all commands execute and matches
  direct fixed-session execution under the recorded identities.
- [ ] Save/load browser controls expose an actionable incompatible-save
  diagnostic and preserve accessible clear/cancel recovery.
- [ ] Replay V2 compatibility validation remains unchanged and no snapshot
  identity policy is moved into `drl-core`.
- [ ] Focused native tests cover codec, compatibility ordering,
  transactionality, storage recovery, and exact checkpoint fixtures.
- [ ] Supported WASM/browser contract tests cover V3 save/load and incompatible
  stored-token recovery; unavailable human/audiovisual comparison stays
  `NOT_RUN`.
- [ ] Independent determinism review returns `pass` after comparing the active
  specification, implementation, tests, persistent-history identity, and
  rejection transaction.
- [ ] `sh scripts/check-repository.sh` and `scripts/check-version.sh` pass; the
  relevant hosted repository and WASM browser checks pass for the reviewed
  revision.
- [ ] On delivery, roadmap, architecture, changelog, browser persistence notes,
  and steering Gate A are reconciled from verified evidence.

### 2.9 Non-goals

- No chainfire, combat, AI, RNG, generator, ruleset, balance, or content-policy
  change.
- No generalized cross-version gameplay migration framework.
- No best-effort replay of provenance-free V1/V2 saves.
- No online account, cloud save, backend, encryption, or compression work.
- No service-worker/cache redesign, offline-lifecycle expansion, or unrelated
  browser UI refactor.
- No broad module split unless a small extraction is required to make the
  compatibility validator pure and independently testable.
- No claim that browser save V3 makes older replay or snapshot formats
  cross-version compatible.

### 2.10 Evidence boundary

This slice is driven by current Rust compatibility contracts rather than a
legacy gameplay rule. Repository/native and supported WASM/browser tests can
prove current V3 behavior. Legacy runtime, audiovisual parity, broad browser
support, assistive-technology acceptance, and long-horizon migration remain
outside the claim and are `NOT_RUN` unless separately executed and recorded.

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
6. presentation timing and storage side effects do not advance gameplay;
7. no runtime Lua or generic callback recreation;
8. current-Rust, cross-version, legacy, browser, audiovisual, and performance
   evidence remain separately labeled.
