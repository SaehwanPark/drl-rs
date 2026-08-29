# Trigun typed aimed-fire evidence

Status: delivered typed Trigun aimed-fire support in `0.2.248`; alternate
reload/nuke behavior remains covered by `trigun.md`, while exact callback state,
timing, controlled runtime comparison, and audiovisual parity remain `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. Its working tree has
unrelated audio/meta edits and an untracked `fpcvalkyrie/` directory; all
claims below come from the pinned revision.

- `bin/data/drl/items/uitems.lua:286-318` declares Trigun as a six-round
  9mm ranged pistol and attaches both the aimed-fire and alternate-reload
  perks.
- `bin/data/drl/perks.lua:128-169` defines aimed fire as a +3 accuracy bonus
  and doubled fire cost; the callback-owned armed state remains outside this
  typed boundary.
- `src/dfbeing.pas:1477-1515` resolves an absent `shots` value to one
  projectile and charges the default one-round cost before the fire loop.

## DRL-Rust boundary

`TRIGUN_BEHAVIOR` now records one ordered projectile, one 9mm round, and the
shared typed aimed-fire policy. Generic ranged execution remains authoritative
for target/LOS/range validation, damage RNG, event ordering, and transactional
clip consumption; `Command::AttackRangedAimed` applies +3 accuracy and
`ActionCost(2_000)` for Trigun. Direct-core, replay/MCP JSON and catalog, and
`BrowserSession` tests verify deterministic parity and atomic empty-clip
rejection. The existing typed alternate reload and nuke transition are
unchanged.

Legacy callback state (`pp_aimed`), alternate-target UI flags, exact timing,
controlled runtime, browser capture, and audiovisual parity remain deferred;
source similarity alone is not parity proof.
