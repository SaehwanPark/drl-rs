# Missile Launcher reload evidence

Status: source-backed behavior evidence; controlled legacy runtime comparison
is `NOT_RUN`.

## Pinned source

The cited legacy checkout is `/Users/saehwan/repos/doom-the-roughlike-original`
at revision `17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. It has unrelated local
audio metadata changes and an untracked `fpcvalkyrie/` directory; those files
are outside this evidence.

- `bin/data/drl/items/eitems.lua:397-434` defines `umbazooka` (Missile
  Launcher) with a four-rocket clip, `IF_SINGLERELOAD`, reload time `12`, and
  an `OnCreate` hook that adds `perk_altreload_full`.
- `bin/data/drl/perks.lua:204-220` implements `perk_altreload_full` by calling
  `being:full_reload(self)` and capping the aggregate score-count cost at
  `2500`.
- `bin/data/core/being.lua:446-478` rejects an already full clip or missing
  reserve, then loops shell reloads until the clip reaches capacity.
- `src/dfbeing.pas:1407-1457` loads `Min(iCount, 1)` when the single-shell
  flag is active; the normal reload path passes that flag from the item
  definition before mutating reserve ammunition.

## Bounded Rust contract

DRL-Rust keeps both rules as explicit archetype policies rather than recreating
the legacy callback system. An equipped Missile Launcher with a clip deficit
loads exactly one loose rocket per accepted ordinary `Reload`, or the complete
deficit on one accepted `AltReload` using `confirmed: false`. Full clips and
insufficient reserve reject before mutation. Alternate reload consumes exactly
the deficit and pays `min(deficit * reload_cost, 2500)`; the existing whole-game
transaction restores clip, inventory, turn, and RNG state on every rejection.

The shared `WeaponReloaded` event reports the resulting clip, and existing MCP
legal-action filtering plus BrowserSession/direct-core tests observe the same
command and event sequence. Rocket-jump, explosion, controlled runtime, and
audiovisual parity remain open. Rust intentionally preflights total reserve,
so it does not reproduce the legacy helper's possible partial mutation before
an under-supplied reload returns false.
