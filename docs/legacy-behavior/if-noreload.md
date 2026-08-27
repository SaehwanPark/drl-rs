# `IF_NORELOAD` manual-reload evidence

Status: source-backed behavior evidence; controlled legacy runtime comparison
is `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. It has unrelated local
audio metadata changes and an untracked `fpcvalkyrie/` directory; those files
are outside this evidence.

- `src/dfbeing.pas:801-846` checks `IF_NORELOAD` at the start of the ordinary
  `ActionReload` path and rejects with “The weapon cannot be manually
  reloaded!” before clip or reserve mutation.
- `bin/data/drl/items/eitems.lua:135-169` marks `ublaster` (`Blaster`) with
  `IF_NORELOAD` and `IF_NOUNLOAD`.
- `bin/data/drl/items/eitems.lua:436-472` marks `unplasma` (Nuclear Plasma
  Rifle) with both flags.
- `bin/data/drl/items/eitems.lua:474-518` marks `unbfg9000` (Nuclear BFG 9000)
  with both flags.

## Bounded Rust contract

DRL-Rust exposes an explicit `Item::allows_manual_reload` policy for exactly
those three `ItemArchetype` values. `Command::Reload` returns
`CommandError::CannotReload(item_id)` before any pump, clip, reserve,
recharge-timer, turn, or RNG mutation. The existing full-game transaction guard
also preserves the complete state on the rejected command. All other ranged
weapons retain their ordinary reload rules; alternate actions and automatic
recharge remain separate behavior slices.

MCP's cloned legal-action probe and BrowserSession both use the same core
command result, so a denied reload is not advertised or committed at either
boundary. No replay wire field or event is added; incompatible gameplay
semantics are rejected through the existing replay metadata contract.

## Boundaries and open questions

This slice does not implement `IF_NOUNLOAD`, alternate reload/nuke behavior,
chainfire, exact-hit/explosion behavior, other weapon families/mods, or
controlled legacy runtime/audio comparison.
