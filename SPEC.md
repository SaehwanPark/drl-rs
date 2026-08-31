# Specification

Last reviewed: 2026-08-31
Current project version: `0.2.325`
Audited starting checkpoint: `main` at
`4ec6561` (merged PR #438; Gate D policy and steering reconciled)
Delivery checkpoint: `main` at
`8e86f26` (merged PR #439; JSON compatibility delivered)

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

## 2. Active implementation slice: M13 — MCP JSON compatibility

Slice status: **delivered and verified** at the delivery checkpoint above.

### 2.1 Objective

Make the zero-dependency MCP JSON boundary accept valid JSON strings used by
external clients, including UTF-16 surrogate-pair escapes, while rejecting
ill-formed surrogate sequences and unescaped control characters. Prove the
decoded value through the MCP `initialize` client-info path.

This is a bounded M13 compatibility slice. It changes no game behavior,
replay identity, RNG sampling, content catalog, protocol envelope, transport
reconnect, or presentation boundary.

### 2.2 Audited starting point

At audited starting revision `4ec6561` (version `0.2.324`):

- `crates/drl-mcp/src/json.rs` decodes each `\\uXXXX` escape directly through
  `char::from_u32`, so valid UTF-16 surrogate pairs such as
  `\\ud83d\\ude80` are rejected before MCP dispatch.
- The same parser accepts raw `U+0000..U+001F` control characters inside JSON
  strings, which is outside the JSON string grammar.
- Existing MCP lifecycle and tool tests cover ordinary ASCII client metadata,
  but no escaped-Unicode initialize fixture protects this compatibility edge.

### 2.3 Scope and ownership

- **Roadmap:** M13 — Browser-First 1.0 Release, complete deterministic
  headless/MCP agent tooling and external-client compatibility.
- **Primary owner:** `crates/drl-mcp/src/json.rs` owns JSON string decoding;
  parser unit tests and `crates/drl-mcp/tests/protocol_jsonrpc.rs` own the
  compatibility fixtures.
- **Project version:** implementation advances `VERSION` from `0.2.324` to
  `0.2.325`.
- **Gameplay/replay semantics:** no gameplay, replay, RNG-sampling, generator,
  ruleset, snapshot, protocol envelope, or content identity changes.

### 2.4 Review and branch contract

- JSON `\\uXXXX` escapes are decoded as UTF-16 code units. A high surrogate
  must be immediately followed by a low-surrogate escape and the pair is
  combined into one Unicode scalar; either lone surrogate is rejected.
- Raw control characters from `U+0000` through `U+001F` are rejected inside
  strings; their escaped forms remain valid.
- Existing string serialization, numeric safety, notification, batch, and
  lifecycle behavior remain unchanged.
- The focused initialize fixture uses an escaped Unicode `clientInfo.name` and
  proves the decoded request reaches the normal lifecycle validation.

### 2.5 Acceptance criteria

- [x] The parser decodes a valid UTF-16 surrogate pair and preserves the
  resulting scalar in a `JsonValue::String`.
- [x] Lone high, lone low, and mismatched surrogate escapes are rejected.
- [x] Unescaped `U+0000..U+001F` control characters are rejected while escaped
  control characters remain supported.
- [x] MCP `initialize` accepts an escaped-Unicode `clientInfo.name` through the
  normal JSON-RPC path.
- [x] The focused parser/protocol tests, formatting, clippy, repository gate,
  web contracts, and version transition pass on the final revision.
- [x] An attributable independent determinism-review receipt is recorded for
  the exact final head; hosted Repository/WASM checks pass. The hosted Review
  policy check fails closed only because the sole maintainer cannot create a
  non-self approval, and the documented `enforce_admins=false` exception was
  used without weakening the policy for external contributors.

### 2.6 Non-goals

- No full MCP schema or external-client compatibility claim.
- No transport reconnect/session persistence, replay-file migration, or
  deployment work.
- No changes to gameplay semantics, replay formats, RNG, content, or browser
  presentation.
- No changes to JSON number handling beyond the existing safety contract.

### 2.7 Evidence boundary

The parser and in-process protocol fixtures prove only current-Rust JSON
decoding and initialize acceptance. They do not prove full external-client
compatibility, transport behavior, or runtime/browser acceptance; those
surfaces remain `NOT_RUN` or open in the roadmap. Review policy enforcement is
audited separately; this sole-maintainer PR used the documented admin
exception after the hosted policy check failed closed for lack of a second
GitHub reviewer.

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
