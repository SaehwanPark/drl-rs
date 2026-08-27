# Missile Launcher single-shell reload evidence

Status: source-backed behavior evidence; controlled legacy runtime comparison
is `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. It has unrelated local
audio metadata changes and an untracked `fpcvalkyrie/` directory; those files
are outside this evidence.

- `bin/data/drl/items/eitems.lua:397-434` defines `umbazooka` (Missile
  Launcher) with a four-rocket clip, `IF_SINGLERELOAD`, and reload time `12`.
- `src/dfbeing.pas:1407-1457` loads `Min(iCount, 1)` when the single-shell
  flag is active; the normal reload path passes that flag from the item
  definition before mutating reserve ammunition.

## Bounded Rust contract

DRL-Rust keeps the single-shell rule as an explicit archetype policy in the
ordinary reload transition. An equipped Missile Launcher with a clip deficit
loads exactly one loose rocket per accepted `Reload` and pays the existing
standard reload cost. Full clips and empty reserve reject before mutation;
the existing whole-game transaction restores clip, inventory, turn, and RNG
state on every rejection.

The shared `WeaponReloaded` event reports the resulting clip, and existing MCP
legal-action filtering plus BrowserSession/direct-core tests observe the same
command and event sequence. Alternate/full reload, rocket-jump, explosion,
controlled runtime, and audiovisual parity remain open.
